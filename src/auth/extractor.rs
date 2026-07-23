use axum::{extract::FromRequestParts, http::request::Parts};
use axum_anyhow::{ApiError, OptionExt, unauthorized};
use axum_login::AuthSession;

use crate::auth::authn_backend::Backend;
use crate::models::user::User;

pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::<Backend>::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized("Auth required", "You are not logged in."))?;

        let user = auth_session
            .user
            .context_unauthorized("Auth required", "You are not logged in.")?;

        Ok(Self(user))
    }
}
