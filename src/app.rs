use std::time::Duration;

use anyhow::Context;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::{Router, routing::get};

use axum_health::health_check;
use axum_health::service::Health;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::api::public_key::public_key;
use crate::api::verifying_key::verifying_key;
use crate::api::version::version;
use crate::key_pairs::ApplicationKeyPairs;
use crate::swagger_docs::ApiDoc;
use crate::{config::Configuration, logs::http_trace_layer};

#[derive(Clone)]
pub struct AppState {
    application_key_pairs: ApplicationKeyPairs,
    database: toasty::Db,
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

pub struct Application {
    app: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build(
        config: Configuration,
        application_key_pairs: ApplicationKeyPairs,
        database: toasty::Db,
    ) -> anyhow::Result<Self> {
        let address = config.application.build_address();

        let state = AppState {
            application_key_pairs,
            database,
        };

        let app = Router::<AppState>::new()
            .nest("/api/v1", api_v1_routes())
            // Routes
            .layer(Health::builder().with_ping().build())
            // App Layers
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
        .route("/public_key", get(public_key))
        .route("/verifying_key", get(verifying_key))
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
