use axum::{Json, extract::State};

use crate::key_pairs::ApplicationKeyPairs;

#[utoipa::path(
    get,
    path = "/public_key",
    responses(
        (status = 200, description = "Public verification key in PEM format", body = String)
    ),
    security(("session_auth" = [])),
    tag = "Keys"
)]
pub async fn public_key(State(key_pairs): State<ApplicationKeyPairs>) -> Json<String> {
    Json(key_pairs.x25519_keypair.public_key_pem)
}

#[utoipa::path(
    get,
    path = "/verifying_key",
    responses(
        (status = 200, description = "Public verifying key in PEM format", body = String)
    ),
    security(("session_auth" = [])),
    tag = "Keys"
)]
pub async fn verifying_key(State(key_pairs): State<ApplicationKeyPairs>) -> Json<String> {
    Json(key_pairs.ed25519_keypair.verifying_key_pem)
}
