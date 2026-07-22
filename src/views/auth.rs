use serde::Deserialize;
use validator::Validate;
use validify::Validify;

#[derive(Debug, Clone, Deserialize, Validate, Validify, utoipa::ToSchema)]
pub struct RegistrationParam {
    #[modify(lowercase)]
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password_hash: String,
}
