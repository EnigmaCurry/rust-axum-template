use std::net::SocketAddr;

use crate::{
    middleware::{TrustedForwardedForConfig, TrustedHeaderAuthConfig},
    prelude::*,
    routes::router,
};

/// Run the HTTP server until shutdown.
pub async fn run(
    addr: SocketAddr,
    user_cfg: TrustedHeaderAuthConfig,
    fwd_cfg: TrustedForwardedForConfig,
) -> anyhow::Result<()> {
    let app = router(user_cfg, fwd_cfg);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;
    info!("listening on http://{bound_addr}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
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
