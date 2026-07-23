use axum_login::AuthnBackend;
use oauth2::{
    AuthorizationCode, CsrfToken, EndpointNotSet, EndpointSet, Scope, TokenResponse,
    basic::BasicClient,
};
use reqwest::{
    Client, Url,
    header::{AUTHORIZATION, USER_AGENT},
};

use crate::{
    auth::{
        oauth_providers::{GitHubUserInfo, GoogleUserInfo, NormalizedProfile, OAuthProviders},
        password::{DUMMY_HASH, verify_password_hash},
    },
    models::user::User,
    services::auth::{AuthService, AuthServiceError},
    views::auth::{Credentials, OAuthProviderKind},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication failed.")]
    AuthFailed,

    #[error(transparent)]
    Database(#[from] toasty::Error),
}

pub type BasicClientSet =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Debug, Clone)]
pub struct Backend {
    db: toasty::Db,
    oauth_providers: OAuthProviders,
    http_client: reqwest::Client,
}

impl Backend {
    pub fn new(db: toasty::Db, oauth_providers: OAuthProviders) -> Self {
        let http_client: reqwest::Client = reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");

        Self {
            db,
            oauth_providers,
            http_client,
        }
    }

    pub fn authorize_url(&self, provider_kind: OAuthProviderKind) -> (Url, CsrfToken) {
        let provider = match provider_kind {
            OAuthProviderKind::GitHub => &self.oauth_providers.github,
            OAuthProviderKind::Google => &self.oauth_providers.google,
        };

        let mut builder = provider.client.authorize_url(CsrfToken::new_random);

        for scope in &provider.scopes {
            builder = builder.add_scope(Scope::new(scope.clone()));
        }

        builder.url()
    }
}

impl AuthnBackend for Backend {
    type User = User;

    type Credentials = Credentials;

    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut db = self.db.clone();

        match creds {
            Credentials::Password(user_credentials) => {
                let (user, expected_hash) =
                    match User::read_user_by_email(&mut db, &user_credentials.email).await {
                        Ok(user) => {
                            let hash = user.password_hash.clone();
                            (Some(user), hash)
                        }
                        Err(err) if err.is_record_not_found() => (None, DUMMY_HASH.to_string()),
                        Err(err) => return Err(AuthError::Database(err)),
                    };

                let password_hash = user_credentials.password;

                let is_valid = verify_password_hash(password_hash, &expected_hash)
                    .await
                    .map_err(|_| AuthError::AuthFailed)?;

                Ok(user.filter(|_| is_valid))
            }
            Credentials::OAuth(oauth_credentials) => {
                if oauth_credentials.old_state.secret() != oauth_credentials.new_state.secret() {
                    return Ok(None);
                };

                let provider_config = match oauth_credentials.provider {
                    OAuthProviderKind::GitHub => &self.oauth_providers.github,
                    OAuthProviderKind::Google => &self.oauth_providers.google,
                };

                let token = provider_config
                    .client
                    .exchange_code(AuthorizationCode::new(oauth_credentials.code))
                    .request_async(&self.http_client)
                    .await
                    .map_err(|_| AuthError::AuthFailed)?;

                let res = Client::new()
                    .get(provider_config.user_api.clone())
                    .header(USER_AGENT.as_str(), "axum-login")
                    .header(
                        AUTHORIZATION.as_str(),
                        format!("Bearer {}", token.access_token().secret()),
                    )
                    .send()
                    .await
                    .map_err(|_| AuthError::AuthFailed)?;

                let profile = match oauth_credentials.provider {
                    OAuthProviderKind::GitHub => {
                        let github_user = res
                            .json::<GitHubUserInfo>()
                            .await
                            .map_err(|_| AuthError::AuthFailed)?;

                        let email =
                            github_user
                                .email
                                .filter(|e| !e.is_empty())
                                .unwrap_or_else(|| {
                                    format!(
                                        "{}+{}@users.noreply.github.com",
                                        github_user.id, github_user.login
                                    )
                                });

                        NormalizedProfile {
                            provider_user_id: github_user.id.to_string(),
                            email,
                        }
                    }

                    OAuthProviderKind::Google => {
                        let google_user = res
                            .json::<GoogleUserInfo>()
                            .await
                            .map_err(|_| AuthError::AuthFailed)?;

                        NormalizedProfile {
                            provider_user_id: google_user.id,
                            email: google_user.email,
                        }
                    }
                };

                let user = AuthService::find_or_create_oauth_user(
                    &mut db,
                    oauth_credentials.provider,
                    &profile,
                )
                .await
                .map_err(|err| match err {
                    AuthServiceError::Database(db_err) => AuthError::Database(db_err),
                    _ => AuthError::AuthFailed,
                })?;

                Ok(Some(user))
            }
        }
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut db = self.db.clone();

        match User::read_user_by_id(&mut db, user_id).await {
            Ok(user) => Ok(Some(user)),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(_) => Err(AuthError::AuthFailed),
        }
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
