use axum::response::IntoResponse;

#[utoipa::path(
    get,
    path = "/version",
    responses(
        (status = 200, description = "Version of the application.", body = String)
    ),
    tag = "Version"
)]
pub async fn version() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    version
}
