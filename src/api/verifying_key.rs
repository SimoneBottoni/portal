use axum::{Json, extract::State};

use crate::key_pairs::ApplicationKeyPairs;

#[utoipa::path(
    get,
    path = "/verifying_key",
    responses(
        (status = 200, description = "Public verifying key in PEM format", body = String)
    ),
    security(("bearer_auth" = [])),
    tag = "VerifyingKey"
)]
pub async fn verifying_key(State(key_pairs): State<ApplicationKeyPairs>) -> Json<String> {
    Json(key_pairs.ed25519_keypair.verifying_key_pem)
}
