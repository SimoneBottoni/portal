use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use validify::Validify;

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
