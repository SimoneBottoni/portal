use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::keys::handlers::public_key,
        crate::api::keys::handlers::verifying_key,
    ),
    tags(
        (name = "Keys", description = "Cryptographic key endpoints")
    )
)]
pub struct ApiDoc;
