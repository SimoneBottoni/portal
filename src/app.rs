use std::time::Duration;

use anyhow::Context;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::{Router, routing::get};

use axum_health::service::Health;
use axum_login::{
    AuthManagerLayerBuilder,
    tower_sessions::{Expiry, SessionManagerLayer},
};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_sessions::MemoryStore;
use tower_sessions::cookie::time::Duration as TowerDuration;
use tower_sessions::cookie::{Key, SameSite};
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::api::health::health_check;
use crate::api::version::version;
use crate::api::{auth, keys};
use crate::auth::authn_backend::Backend;
use crate::auth::oauth_providers::OAuthProviders;
use crate::key_pairs::ApplicationKeyPairs;
use crate::mailer::MailerContext;
use crate::openapi::ApiDoc;
use crate::{config::Configuration, logs::http_trace_layer};

#[derive(Clone)]
pub struct AppState {
    pub application_key_pairs: ApplicationKeyPairs,
    pub database: toasty::Db,
    pub mailer: MailerContext,
}

impl FromRef<AppState> for ApplicationKeyPairs {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.application_key_pairs.clone()
    }
}

impl FromRef<AppState> for toasty::Db {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.database.clone()
    }
}

impl FromRef<AppState> for MailerContext {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.mailer.clone()
    }
}

pub struct Application {
    app: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build(
        config: Configuration,
        application_key_pairs: ApplicationKeyPairs,
        database: toasty::Db,
        mailer: MailerContext,
        oauth_providers: OAuthProviders,
    ) -> anyhow::Result<Self> {
        let address = config.application.build_address();

        let state = AppState {
            application_key_pairs,
            database: database.clone(),
            mailer,
        };

        let session_store = MemoryStore::default();

        // ---------------------
        // let session_store = ToastyStore::new(database);
        // session_store.migrate().await?;

        // let deletion_task = tokio::task::spawn(
        //     session_store
        //         .clone()
        //         .continuously_delete_expired(Duration::from_secs(60)),
        // );

        // deletion_task.await??;
        // ---------------------

        // Generate a cryptographic key to sign the session cookie.
        let key = Key::generate();

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(TowerDuration::days(1)))
            .with_same_site(SameSite::Lax)
            .with_signed(key);

        let backend = Backend::new(database, oauth_providers);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let app = Router::<AppState>::new()
            .nest("/api/v1", api_v1_routes())
            // Routes
            .layer(Health::builder().with_ping().build())
            // App Layers
            .layer(auth_layer)
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(10),
            ))
            // Trace Layers
            .layer(http_trace_layer())
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            // States
            .with_state(state);

        let listener = TcpListener::bind(&address)
            .await
            .context(format!("Failed to bind TcpListener on {}", address))?;

        Ok(Self { app, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("Failed to run the application.")?;

        Ok(())
    }

    pub fn address(&self) -> anyhow::Result<String> {
        Ok(format!(
            "{}",
            self.listener
                .local_addr()
                .context("Failed to get address.")?
        ))
    }

    pub fn port(&self) -> anyhow::Result<u16> {
        Ok(self
            .listener
            .local_addr()
            .context("Failed to get port.")?
            .port())
    }
}

fn api_v1_routes() -> Router<AppState> {
    let config = Config::from("/api/v1/api-docs/openapi.json");

    Router::<AppState>::new()
        .route("/version", get(version))
        .route("/health", get(health_check))
        .merge(keys::routes::routes())
        .nest("/auth", auth::routes::routes())
        .merge(
            SwaggerUi::new("/swagger")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(config),
        )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
