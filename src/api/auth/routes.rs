use axum::{Router, routing::post};
use axum_login::login_required;

use crate::{
    api::auth::handlers::{change_password, login, logout, register},
    app::AppState,
    auth::authn_backend::Backend,
};

pub fn routes() -> Router<AppState> {
    let public_routes = Router::<AppState>::new()
        .route("/register", post(register))
        .route("/login", post(login));

    let protected_routes = Router::<AppState>::new()
        .route("/logout", post(logout))
        .route("/change_password", post(change_password))
        .route_layer(login_required!(Backend));

    public_routes.merge(protected_routes)
}
