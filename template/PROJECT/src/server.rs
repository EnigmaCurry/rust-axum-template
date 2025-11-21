use axum::{extract::Path, http::StatusCode, routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use crate::prelude::*;

/// Build your Axum router. Keep this as a separate function so it’s testable.
pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/hello/{name}", get(hello))
        .fallback(fallback_404)
        // Basic request logging / tracing
        .layer(TraceLayer::new_for_http())
}

/// Run the HTTP server until shutdown.
pub async fn run(addr: SocketAddr) -> anyhow::Result<()> {
    let app = router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;
    info!("listening on http://{bound_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "OK"
}

async fn healthz() -> &'static str {
    "healthy"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}

async fn fallback_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Shutdown signal for graceful shutdown on Ctrl+C / SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received; starting graceful shutdown");
}
