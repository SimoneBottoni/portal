use axum::Extension;
use axum::response::IntoResponse;
use axum_health::service::Health;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check")
    ),
    tag = "Health"
)]
pub async fn health_check(Extension(health): Extension<Health>) -> impl IntoResponse {
    health.details().await
}
