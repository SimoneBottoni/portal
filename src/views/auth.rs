use std::fmt::Display;

use oauth2::CsrfToken;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use validify::Validify;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    Password(UserCredentials),
    OAuth(OAuthCredentials),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProviderKind {
    GitHub,
    Google,
}

impl Display for OAuthProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::Google => write!(f, "google"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub provider: OAuthProviderKind,
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct OAuthLoginQuery {
    pub provider: OAuthProviderKind,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OAuthLoginResponse {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    pub code: String,
    pub state: CsrfToken,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Validify, ToSchema)]
pub struct UserCredentials {
    #[modify(lowercase)]
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Validify, ToSchema)]
pub struct ChangePasswordParams {
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub new_password: String,
}
