use axum::{Json, extract::State, response::IntoResponse};
use axum_anyhow::ResultExt;
use axum_valid::Valid;
use serde::Deserialize;
use validator::Validate;
use validify::Modify;
use validify::Validify;

use crate::models::users::User;

#[derive(Debug, Clone, Deserialize, Validate, Validify, utoipa::ToSchema)]
pub struct RegistrationParam {
    #[modify(lowercase)]
    #[validate(email(message = "Please provide a valid email address"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password_hash: String,
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegistrationParam,
    responses(
        (status = 200, description = "User registered successfully"),
        (status = 401, description = "Registration failed")
    ),
    tag = "Auth"
)]
#[axum::debug_handler]
pub async fn register(
    State(mut db): State<toasty::Db>,
    Valid(Json(mut params)): Valid<Json<RegistrationParam>>,
) -> impl IntoResponse {
    params.modify();

    User::create_user(&mut db, &params.email, &params.password_hash)
        .await
        .context_unauthorized("Error", "Could not register user.")
}
