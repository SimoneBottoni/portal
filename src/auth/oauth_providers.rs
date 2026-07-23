use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl, basic::BasicClient};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::auth::authn_backend::BasicClientSet;
use crate::config::OAuthProvidersConfig;
use crate::views::auth::OAuthProviderKind;

// GitHub
pub const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
pub const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
pub const GITHUB_USER_API: &str = "https://api.github.com/user";

// Google
pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_USER_API: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

// GitHub Response Model
#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

// Google Response Model
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: Option<bool>,
    pub name: Option<String>,
}

pub struct NormalizedProfile {
    pub provider_user_id: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct OAuthProviders {
    pub github: OAuthProvider,
    pub google: OAuthProvider,
}

impl OAuthProviders {
    pub fn build_all(config: &OAuthProvidersConfig) -> Self {
        let github = OAuthProvider::build(
            OAuthProviderKind::GitHub,
            config.github_id.clone(),
            config.github_secret.expose_secret().to_string(),
        );
        let google = OAuthProvider::build(
            OAuthProviderKind::Google,
            config.google_id.clone(),
            config.google_secret.expose_secret().to_string(),
        );

        Self { github, google }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub user_api: String,
    pub client: BasicClientSet,
    pub scopes: Vec<String>,
}

impl OAuthProvider {
    pub fn build(provider: OAuthProviderKind, id: String, secret: String) -> Self {
        let id = ClientId::new(id);
        let secret = ClientSecret::new(secret);

        let (auth_url, token_url, user_api, scopes) = match provider {
            OAuthProviderKind::GitHub => (
                GITHUB_AUTH_URL.to_string(),
                GITHUB_TOKEN_URL.to_string(),
                GITHUB_USER_API.to_string(),
                vec!["user:email".to_string(), "read:user".to_string()],
            ),
            OAuthProviderKind::Google => (
                GOOGLE_AUTH_URL.to_string(),
                GOOGLE_TOKEN_URL.to_string(),
                GOOGLE_USER_API.to_string(),
                vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            ),
        };

        let auth_url = AuthUrl::new(auth_url).expect("Failed to build AuthUrl.");
        let token_url = TokenUrl::new(token_url).expect("Failed to build TokenUrl.");

        let client = BasicClient::new(id)
            .set_client_secret(secret)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url);

        Self {
            user_api,
            client,
            scopes,
        }
    }
}
