use anyhow::Context;
use axum::{Router, routing::get};

use tokio::net::TcpListener;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

use crate::{config::Configuration, logs::http_trace_layer};

pub struct Application {
    app: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build(config: Configuration) -> anyhow::Result<Self> {
        let address = config.application.build_address();

        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .layer(http_trace_layer())
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

        let listener = TcpListener::bind(&address)
            .await
            .context(format!("Failed to bind TcpListener on {}", address))?;

        Ok(Application { app, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to install Ctrl+C handler");
            })
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
