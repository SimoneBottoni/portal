use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::auth::handlers::register,
    ),
    components(
        schemas(crate::api::auth::handlers::RegistrationParam)
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;
