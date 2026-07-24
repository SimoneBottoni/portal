use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use axum_anyhow::{ApiResult, OptionExt, ResultExt, internal_error, unauthorized};
use axum_valid::Valid;
use oauth2::CsrfToken;
use tower_sessions::Session;
use validify::Modify;

use crate::auth::authn_backend::AuthSession;
use crate::mailer::MailerContext;
use crate::services::auth::{AuthService, AuthServiceError};
use crate::views::auth::{
    AuthzResp, ChangePasswordParams, Credentials, OAuthCredentials, OAuthLoginQuery,
    OAuthLoginResponse, OAuthProviderKind, UserCredentials,
};

pub const CSRF_STATE_KEY: &str = "oauth.csrf_state";
pub const PROVIDER_KEY: &str = "oauth.provider";

#[utoipa::path(
    post,
    path = "/register",
    request_body = UserCredentials,
    responses(
        (status = 200, description = "User registered successfully"),
        (status = 500, description = "Registration failed")
    ),
    tag = "Auth"
)]
pub async fn register(
    State(mut db): State<toasty::Db>,
    State(mailer): State<MailerContext>,
    Valid(Json(mut params)): Valid<Json<UserCredentials>>,
) -> ApiResult<()> {
    params.modify();

    AuthService::register_user(&mut db, &mailer, &params.email, params.password)
        .await
        .context_internal(("User creation failed", "Could not register user."))?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = UserCredentials,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "Auth"
)]
pub async fn login(
    mut auth_session: AuthSession,
    Valid(Json(mut params)): Valid<Json<UserCredentials>>,
) -> ApiResult<()> {
    params.modify();

    let user = auth_session
        .authenticate(Credentials::Password(params))
        .await
        .context_internal(("Auth backend failed", "Internal Server Error."))?
        .context_unauthorized(("Invalid credentials", "Invalid email or password."))?;

    auth_session
        .logout()
        .await
        .context_internal(("Session purge failed", "Internal Server Error."))?;

    auth_session
        .login(&user)
        .await
        .context_internal(("Session write failed", "Internal Server Error."))?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/oauth",
    params(("provider" = OAuthProviderKind, Query, description = "OAuth provider")),
    responses(
        (status = 200, description = "OAuth login URL generated", body = OAuthLoginResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn oauth_login(
    auth_session: AuthSession,
    session: Session,
    Query(query): Query<OAuthLoginQuery>,
) -> ApiResult<impl IntoResponse> {
    let (auth_url, csrf_state) = auth_session.backend.authorize_url(query.provider.clone());

    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(PROVIDER_KEY, query.provider).await?;

    Ok(Json(OAuthLoginResponse {
        url: auth_url.to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/oauth/callback",
    params(
        ("code" = String, Query, description = "Authorization code"),
        ("state" = String, Query, description = "CSRF state token")
    ),
    responses(
        (status = 200, description = "OAuth login successful"),
        (status = 400, description = "Invalid OAuth state or code"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn oauth_callback(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthzResp {
        code,
        state: new_state,
    }): Query<AuthzResp>,
) -> ApiResult<impl IntoResponse> {
    let old_state: CsrfToken = session
        .remove(CSRF_STATE_KEY)
        .await
        .context_internal(("Session error reading CSRF state", "Internal Server Error."))?
        .context_bad_request((
            "Missing CSRF state in session",
            "OAuth session expired or invalid.",
        ))?;

    let provider: OAuthProviderKind = session
        .remove(PROVIDER_KEY)
        .await
        .context_internal((
            "Session error reading OAuth provider",
            "Internal Server Error.",
        ))?
        .context_bad_request((
            "Missing OAuth provider in session",
            "OAuth session expired or invalid.",
        ))?;

    let creds = Credentials::OAuth(OAuthCredentials {
        provider,
        code,
        old_state,
        new_state,
    });

    let user = auth_session
        .authenticate(creds)
        .await
        .context_internal(("Auth backend failed", "Internal Server Error."))?
        .context_unauthorized((
            "OAuth verification failed",
            "Invalid OAuth state or authorization code.",
        ))?;

    auth_session
        .login(&user)
        .await
        .context_internal(("Session write failed", "Internal Server Error."))?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn logout(mut auth_session: AuthSession) -> ApiResult<()> {
    auth_session
        .logout()
        .await
        .context_internal(("Logout error", "Internal Server Error."))?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/change_password",
    request_body = ChangePasswordParams,
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 401, description = "Invalid current password")
    ),
    tag = "Auth"
)]
pub async fn change_password(
    mut auth_session: AuthSession,
    State(mut db): State<toasty::Db>,
    Valid(Json(params)): Valid<Json<ChangePasswordParams>>,
) -> ApiResult<()> {
    let user = auth_session
        .user
        .clone()
        .context_unauthorized(("Auth required", "You are not logged in."))?;

    match AuthService::change_password(
        &mut db,
        user.id,
        &user.password_hash,
        params.current_password,
        params.new_password,
    )
    .await
    {
        Ok(()) => {}
        Err(AuthServiceError::InvalidCurrentPassword) => {
            return Err(unauthorized(
                "Invalid password",
                "Failed to verify current password.",
            ));
        }
        Err(_) => {
            return Err(internal_error(
                "Password change failed",
                "Internal Server Error.",
            ));
        }
    }

    auth_session
        .logout()
        .await
        .context_internal(("Session error", "Internal Server Error."))?;

    Ok(())
}
