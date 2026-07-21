use utoipa::OpenApi;

// struct SecurityAddon;

// impl Modify for SecurityAddon {
//     fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
//         let mut components: Components = openapi.components.clone().unwrap_or_default();
//         components.add_security_scheme(
//             "bearer_auth",
//             SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
//         );
//         openapi.components = Some(components);
//     }
// }

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::version::version,
        // crate::api::public_key::public_key,
        // crate::api::verifying_key::verifying_key,
    ),
    servers(
        (url = "/api/v1", description = "Api v1")
    ),
    components(
        schemas()
    ),
    // modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "Version", description = "Application version endpoint"),
    )
)]
pub struct ApiDoc;
