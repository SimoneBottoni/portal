use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::auth::handlers::register,
        crate::api::auth::handlers::login,
        crate::api::auth::handlers::oauth_login,
        crate::api::auth::handlers::oauth_callback,
        crate::api::auth::handlers::logout,
        crate::api::auth::handlers::change_password,
    ),
    components(
        schemas(
            crate::views::auth::UserCredentials,
            crate::views::auth::ChangePasswordParams,
            crate::views::auth::OAuthProviderKind,
            crate::views::auth::OAuthLoginQuery,
            crate::views::auth::OAuthLoginResponse,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;
