use axum::{Router, routing::post};

use crate::{
    api::auth::handlers::{change_password, login, logout, register},
    app::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/change_password", post(change_password))
}
