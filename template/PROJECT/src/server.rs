use crate::{
    middleware::{
        trusted_forwarded_for::TrustedForwardedForConfig, trusted_header_auth::AuthConfig,
    },
    prelude::*,
    routes::router,
    tls::{ensure_rustls_crypto_provider, generate_self_signed_with_validity},
};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use futures_util::StreamExt;
use rustls::ServerConfig as RustlsServerConfig;
use sqlx::{ConnectOptions, SqlitePool, sqlite::SqliteConnectOptions};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::fs;
use tokio::task::AbortHandle;
use tokio_rustls_acme::{AcmeConfig, caches::DirCache};
use tower_sessions::{
    Expiry, SessionManagerLayer, cookie::time::Duration as CookieDuration,
    session_store::ExpiredDeletion,
};
use tower_sessions_sqlx_store::SqliteStore;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[derive(Clone, Debug)]
pub enum TlsConfig {
    /// Plain HTTP, no TLS.
    Http,
    /// Rustls with certificate and key loaded from PEM files.
    RustlsFiles {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
    /// Self-signed TLS, generated at startup.
    ///
    /// If `cache_dir` is Some, certificates are stored/reused there.
    /// If `cache_dir` is None, certificates are ephemeral (in-memory only).
    SelfSigned {
        cache_dir: Option<PathBuf>,
        sans: Vec<String>,
        valid_days: u32,
    },
    /// ACME (Let's Encrypt or other CA) via TLS-ALPN-01.
    ///
    /// Certificates and account data are stored in `cache_dir`.
    Acme {
        directory_url: String,
        cache_dir: PathBuf,
        domains: Vec<String>,
        contact_email: Option<String>,
    },
}

/// Run the HTTP server until shutdown.
pub async fn run(
    addr: SocketAddr,
    user_cfg: AuthConfig,
    fwd_cfg: TrustedForwardedForConfig,
    db_url: String,
    session_secure: bool,
    session_expiry_secs: u64,
    session_check_secs: u64,
    tls_config: TlsConfig,
) -> anyhow::Result<()> {
    // Database pool and migration
    let connect_opts = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .log_statements(tracing::log::LevelFilter::Trace)
        .log_slow_statements(
            tracing::log::LevelFilter::Warn,
            std::time::Duration::from_millis(100),
        );
    let db: SqlitePool = SqlitePool::connect_with(connect_opts).await?;
    info!("Loaded database connection pool. DATABASE_URL={db_url}");
    sqlx::migrate!().run(&db.clone()).await?;

    // Session store
    let session_store = SqliteStore::new(db.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(core::time::Duration::from_secs(session_check_secs)),
    );

    // Convert the CLI/env-specified seconds into a cookie::time::Duration
    let session_expiry = CookieDuration::seconds(session_expiry_secs as i64);

    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(session_secure)
        .with_expiry(Expiry::OnInactivity(session_expiry));

    // Shared state
    let state = AppState { db };

    let app = router(user_cfg, fwd_cfg, state.clone())
        .layer(session_layer)
        .with_state(state);

    ensure_rustls_crypto_provider();

    match tls_config {
        TlsConfig::Http => {
            let listener = tokio::net::TcpListener::bind(addr).await?;
            let bound_addr = listener.local_addr()?;
            info!("listening on http://{bound_addr}");

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle(), None))
            .await?;
        }
        TlsConfig::RustlsFiles {
            cert_path,
            key_path,
        } => {
            info!(
                "loading TLS certificate from '{}' and key from '{}'",
                cert_path.display(),
                key_path.display()
            );

            let rustls_config =
                axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path).await?;

            // Create a handle for graceful shutdown
            let handle = axum_server::Handle::new();

            // Spawn the shutdown handler that will:
            //  - abort the deletion task
            //  - call handle.graceful_shutdown(...)
            let shutdown_task = tokio::spawn(shutdown_signal(
                deletion_task.abort_handle(),
                Some(handle.clone()),
            ));

            info!("listening on https://{addr}");

            axum_server::bind_rustls(addr, rustls_config)
                .handle(handle)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;

            // Make sure the shutdown task has finished (and bubble up any errors)
            shutdown_task.await?;
        }
        TlsConfig::SelfSigned {
            cache_dir,
            mut sans,
            valid_days, // still used for generation, see note below
        } => {
            if sans.is_empty() {
                sans.push("localhost".to_string());
            }

            let (cert_pem, key_pem) = if let Some(dir) = cache_dir {
                fs::create_dir_all(&dir).await?;

                let cert_path = dir.join("self_signed_cert.pem");
                let key_path = dir.join("self_signed_key.pem");

                // Try to load & validate the cached cert using x509-parser
                use rustls_pemfile::certs as load_pem_certs;
                use x509_parser::prelude::*;

                let use_cached = if cert_path.exists() && key_path.exists() {
                    match fs::read(&cert_path).await {
                        Ok(pem_bytes) => {
                            let mut slice: &[u8] = &pem_bytes;
                            let mut iter = load_pem_certs(&mut slice);

                            // Option<Result<CertificateDer<'static>, io::Error>>
                            match iter.next().transpose() {
                                Ok(Some(der)) => match parse_x509_certificate(der.as_ref()) {
                                    Ok((_rem, x509)) => {
                                        let validity = x509.validity();
                                        let now = ASN1Time::now();
                                        if validity.is_valid_at(now) {
                                            true
                                        } else {
                                            info!(
                                                "Cached self-signed cert at '{}' is expired/invalid; regenerating",
                                                cert_path.display()
                                            );
                                            false
                                        }
                                    }
                                    Err(err) => {
                                        info!(
                                            "Failed to parse cached self-signed cert '{}': {err}; regenerating",
                                            cert_path.display()
                                        );
                                        false
                                    }
                                },
                                Ok(None) => {
                                    info!(
                                        "Cached self-signed cert '{}' has no certificates; regenerating",
                                        cert_path.display()
                                    );
                                    false
                                }
                                Err(err) => {
                                    info!(
                                        "Failed to decode PEM for cached self-signed cert '{}': {err}; regenerating",
                                        cert_path.display()
                                    );
                                    false
                                }
                            }
                        }
                        Err(err) => {
                            info!(
                                "Failed to read cached self-signed cert '{}': {err}; regenerating",
                                cert_path.display()
                            );
                            false
                        }
                    }
                } else {
                    false
                };

                if use_cached {
                    // unchanged
                    let cert = fs::read(&cert_path).await?;
                    let key = fs::read(&key_path).await?;
                    info!(
                        "Loading cached self-signed TLS certificate from '{}' and key from '{}'",
                        cert_path.display(),
                        key_path.display()
                    );
                    (cert, key)
                } else {
                    info!(
                        "Generating new cached self-signed TLS certificate \
         (valid_days={}, sans={:?}) in '{}'",
                        valid_days,
                        sans,
                        dir.display()
                    );

                    let (cert_pem, key_pem) = generate_self_signed_with_validity(sans, valid_days)?;

                    fs::write(&cert_path, &cert_pem).await?;
                    fs::write(&key_path, &key_pem).await?;
                    (cert_pem, key_pem)
                }
            } else {
                info!(
                    "Generating ephemeral self-signed TLS certificate \
         (valid_days={}, sans={:?}); not cached",
                    valid_days, sans
                );

                generate_self_signed_with_validity(sans, valid_days)?
            };

            let rustls_config = RustlsConfig::from_pem(cert_pem, key_pem).await?;

            let handle = Handle::new();
            let shutdown_task = tokio::spawn(shutdown_signal(
                deletion_task.abort_handle(),
                Some(handle.clone()),
            ));

            info!("listening on https://{addr} (self-signed)");

            axum_server::bind_rustls(addr, rustls_config)
                .handle(handle)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;

            shutdown_task.await?;
        }
        TlsConfig::Acme {
            directory_url,
            cache_dir,
            domains,
            contact_email,
        } => {
            // Ensure cache dir exists
            fs::create_dir_all(&cache_dir).await?;

            info!(
                "Starting ACME TLS (tls-alpn-01) – directory_url='{}', cache_dir='{}', domains={:?}, contact_email={:?}",
                directory_url,
                cache_dir.display(),
                domains,
                contact_email,
            );

            // Build ACME configuration
            let mut state = {
                let mut cfg = AcmeConfig::new(domains.clone())
                    .cache(DirCache::new(cache_dir.clone()))
                    .directory(directory_url.clone());

                if let Some(ref email) = contact_email {
                    if !email.is_empty() {
                        cfg = cfg.contact([format!("mailto:{email}")]);
                    }
                }

                cfg.state()
            };

            // Hook ACME into rustls
            let rustls_config = RustlsServerConfig::builder()
                .with_no_client_auth()
                .with_cert_resolver(state.resolver());

            let acceptor = state.axum_acceptor(Arc::new(rustls_config));

            // Drive ACME state + log events
            tokio::spawn(async move {
                while let Some(res) = state.next().await {
                    match res {
                        Ok(ev) => tracing::info!("acme event: {:?}", ev),
                        Err(err) => tracing::error!("acme error: {:?}", err),
                    }
                }
            });

            // axum_server + graceful shutdown, same pattern as self-signed
            let handle = Handle::new();
            let shutdown_task = tokio::spawn(shutdown_signal(
                deletion_task.abort_handle(),
                Some(handle.clone()),
            ));

            info!("listening on https://{addr} (ACME)");

            axum_server::bind(addr)
                .handle(handle)
                .acceptor(acceptor)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;

            shutdown_task.await?;
        }
    }

    // Make sure the background deletion task finishes cleanly.
    deletion_task.await??;

    Ok(())
}

/// Shutdown signal for graceful shutdown on Ctrl+C / SIGTERM.
async fn shutdown_signal(deletion_task_abort_handle: AbortHandle, handle: Option<Handle>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
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

    // Stop the background deletion task
    deletion_task_abort_handle.abort();

    // If we are running behind axum_server, trigger graceful shutdown there too
    if let Some(handle) = handle {
        // You can tune the timeout; 10 seconds is a typical choice.
        handle.graceful_shutdown(Some(Duration::from_secs(10)));
    }

    info!("shutdown signal received; starting graceful shutdown");
}
