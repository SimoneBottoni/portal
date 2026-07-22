use axum::{Router, routing::get};

use crate::{
    api::keys::handlers::{public_key, verifying_key},
    app::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/public_key", get(public_key))
        .route("/verifying_key", get(verifying_key))
}
