use axum::{Json, extract::State};
use axum_anyhow::{ApiResult, OptionExt, ResultExt, unauthorized};
use axum_valid::Valid;
use validify::Modify;

use crate::auth::authn_backend::AuthSession;
use crate::auth::password::{hash_password, verify_password_hash};
use crate::mailer::MailerContext;
use crate::models::users::User;
use crate::views::auth::{ChangePasswordParams, UserCredentials};

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

    let password_hash = hash_password(params.password)
        .await
        .context_internal("Hashing failed", "Internal Server Error.")?;

    User::create_user(&mut db, &params.email, &password_hash)
        .await
        .context_bad_request("User creation failed", "Could not register user.")?;

    if let Err(err) = mailer.send_welcome(&params.email, "").await {
        tracing::error!(
            "Failed to send welcome email to {}: {:?}",
            params.email,
            err
        );
    }

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
        .authenticate(params)
        .await
        .context_internal("Auth backend failed", "Internal Server Error.")?
        .context_unauthorized("Invalid credentials", "Invalid email or password.")?;

    auth_session
        .logout()
        .await
        .context_internal("Session purge failed", "Internal Server Error.")?;

    auth_session
        .login(&user)
        .await
        .context_internal("Session write failed", "Internal Server Error.")?;

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
        .context_internal("Logout error", "Internal Server Error.")?;

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
        .context_unauthorized("Auth required", "You are not logged in.")?;

    let expected_hash = user.password_hash.clone();

    let is_valid = verify_password_hash(params.current_password, &expected_hash)
        .await
        .context_internal("Task error", "Internal Server Error.")?;

    if !is_valid {
        return Err(unauthorized(
            "Invalid password",
            "Failed to verify current password.",
        ));
    }

    let new_password_hash = hash_password(params.new_password)
        .await
        .context_internal("Task error", "Internal Server Error.")?;

    User::update_user_password(&mut db, user.id, &new_password_hash)
        .await
        .context_internal("DB error", "Failed to update password.")?;

    auth_session
        .logout()
        .await
        .context_internal("Session error", "Internal Server Error.")?;

    Ok(())
}
