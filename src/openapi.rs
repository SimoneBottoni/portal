use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let mut components = openapi.components.clone().unwrap_or_default();
        components.add_security_scheme(
            "session_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
        );
        openapi.components = Some(components);
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::health::health_check,
        crate::api::version::version,
        crate::api::keys::handlers::public_key,
        crate::api::keys::handlers::verifying_key,
    ),
    nest(
        (path = "/auth", api = crate::api::auth::openapi::ApiDoc),
    ),
    servers(
        (url = "/api/v1", description = "Api v1")
    ),
    components(
        schemas()
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Health", description = "Health check endpoint"),
        (name = "Keys", description = "Cryptographic key endpoints"),
        (name = "Version", description = "Application version endpoint"),
    )
)]
pub struct ApiDoc;
