use axum::{Json, extract::State};

use crate::key_pairs::ApplicationKeyPairs;

#[utoipa::path(
    get,
    path = "/public_key",
    responses(
        (status = 200, description = "Public verification key in PEM format", body = String)
    ),
    security(("bearer_auth" = [])),
    tag = "PublicKey"
)]
pub async fn public_key(State(key_pairs): State<ApplicationKeyPairs>) -> Json<String> {
    Json(key_pairs.x25519_keypair.public_key_pem)
}
