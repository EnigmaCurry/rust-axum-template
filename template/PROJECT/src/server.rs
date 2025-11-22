use axum::{
    body::Body,
    extract::{ConnectInfo, Extension, Path, State},
    http::{HeaderMap, HeaderName, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tower_http::trace::TraceLayer;

use crate::prelude::*;

/// Config for trusting an auth header from a forward-auth proxy (user/email).
#[derive(Clone, Debug)]
pub struct TrustedHeaderAuthConfig {
    pub enabled: bool,
    pub header_name: HeaderName,
    pub trusted_proxy: IpAddr,
}

impl TrustedHeaderAuthConfig {
    /// Reasonable disabled default.
    #[allow(dead_code)]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            header_name: HeaderName::from_static("x-forwarded-user"),
            trusted_proxy: IpAddr::from([127, 0, 0, 1]),
        }
    }
}

/// Config for trusting a forwarded-for header from a proxy (client IP).
#[derive(Clone, Debug)]
pub struct TrustedForwardedForConfig {
    pub enabled: bool,
    pub header_name: HeaderName,
    pub trusted_proxy: IpAddr,
}

impl TrustedForwardedForConfig {
    /// Reasonable disabled default.
    #[allow(dead_code)]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            header_name: HeaderName::from_static("x-forwarded-for"),
            trusted_proxy: IpAddr::from([127, 0, 0, 1]),
        }
    }
}

/// Authenticated user email extracted from a trusted header.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser(#[allow(dead_code)] pub String);

/// Client IP extracted from trusted forwarded-for header.
#[derive(Clone, Debug)]
pub struct ClientIp(#[allow(dead_code)] pub IpAddr);

/// Build your Axum router. Keep this as a separate function so it’s testable.
pub fn router(user_cfg: TrustedHeaderAuthConfig, fwd_cfg: TrustedForwardedForConfig) -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/hello/{name}", get(hello))
        .route("/whoami", get(whoami))
        .fallback(fallback_404)
        .layer(TraceLayer::new_for_http());

    // Always install both middlewares; they self-disable and
    // reject spoofing when disabled.
    app.layer(middleware::from_fn_with_state(
        fwd_cfg,
        trusted_forwarded_for,
    ))
    .layer(middleware::from_fn_with_state(
        user_cfg,
        trusted_header_auth,
    ))
}

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

/// Middleware that enforces trusted-header auth for user/email.
///
/// Rules:
/// - If disabled: 403 if header present.
/// - If enabled: only trusted proxy may send it (403 otherwise).
/// - Header must be present and non-empty.
/// - First comma-separated token treated as email.
async fn trusted_header_auth(
    State(cfg): State<TrustedHeaderAuthConfig>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    if !cfg.enabled {
        if req.headers().contains_key(&cfg.header_name) {
            warn!(
                "trusted user header auth disabled, but header '{}' was present from peer {}",
                cfg.header_name,
                peer.ip()
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        return next.run(req).await;
    }

    if peer.ip() != cfg.trusted_proxy {
        if req.headers().contains_key(&cfg.header_name) {
            warn!(
                "trusted user header auth: rejecting spoofed header '{}' from untrusted peer {} (expected {})",
                cfg.header_name,
                peer.ip(),
                cfg.trusted_proxy
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        // If no spoofed header, allow request through.
        return next.run(req).await;
    }

    let email: String = {
        let raw = req
            .headers()
            .get(&cfg.header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let first = match raw {
            Some(v) => v.split(',').next().unwrap().trim(),
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        first.to_string()
    };

    req.extensions_mut().insert(AuthenticatedUser(email));
    next.run(req).await
}

/// Middleware that enforces trusted forwarded-for header for client IP.
///
/// Rules:
/// - If disabled: 403 if header present.
/// - If enabled: only trusted proxy may send it (403 otherwise).
/// - If trusted proxy sends it, parse first IP and store as ClientIp.
async fn trusted_forwarded_for(
    State(cfg): State<TrustedForwardedForConfig>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    if !cfg.enabled {
        if req.headers().contains_key(&cfg.header_name) {
            warn!(
                "trusted forwarded-for disabled, but header '{}' was present from peer {}",
                cfg.header_name,
                peer.ip()
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        return next.run(req).await;
    }

    // Enabled mode: if header is present from untrusted peer, reject.
    if peer.ip() != cfg.trusted_proxy {
        if req.headers().contains_key(&cfg.header_name) {
            warn!(
                "trusted forwarded-for: rejecting spoofed header '{}' from untrusted peer {} (expected {})",
                cfg.header_name,
                peer.ip(),
                cfg.trusted_proxy
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        return next.run(req).await;
    }

    // Trusted proxy: if header present, parse it.
    let client_ip: Option<IpAddr> = {
        let raw = req
            .headers()
            .get(&cfg.header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        raw.and_then(|v| {
            let first = v.split(',').next().unwrap().trim();
            IpAddr::from_str(first).ok()
        })
    };

    if let Some(ip) = client_ip {
        req.extensions_mut().insert(ClientIp(ip));
    } else if req.headers().contains_key(&cfg.header_name) {
        // Header existed but was unparsable.
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
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

async fn whoami(
    user: Option<Extension<AuthenticatedUser>>,
    client_ip: Option<Extension<ClientIp>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let email = match user {
        Some(Extension(AuthenticatedUser(email))) => email,
        None => "<unauthenticated>".to_string(),
    };

    let peer_ip = peer.ip();
    let client_ip = client_ip
        .map(|Extension(ClientIp(ip))| ip)
        .unwrap_or(peer_ip);

    let mut hdr_lines = String::new();
    for (name, value) in headers.iter() {
        let val_str = value.to_str().unwrap_or("<non-utf8>");
        hdr_lines.push_str(&format!("{name}: {val_str}\n"));
    }

    let body =
        format!("identity:\nuser={email}\npeer_ip={peer_ip}\nclient_ip={client_ip}\n\nheaders:\n{hdr_lines}");

    (StatusCode::OK, body)
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
