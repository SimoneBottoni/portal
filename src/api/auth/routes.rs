use axum::{Router, routing::post};

use crate::{api::auth::handlers::register, app::AppState};

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/register", post(register))
}
