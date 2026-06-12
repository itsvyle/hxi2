mod app_config;
mod auth_service;
mod permissions_checking;

use anyhow::{Context as _, Result};
use app_config::AppConfiguration;
use auth_service::{AuthServiceExt, AuthServiceImpl};
use axum::{Router, routing::get};
use connectrpc::{ConnectError, ErrorCode, Router as ConnectRouter};
use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use hxi2_proto::proto::auth::v2::JwtClaims;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
async fn auth_middleware(req: Request, next: Next) -> Response {
    let Some(token) = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
    else {
        return unauthorized("missing Bearer token");
    };

    let route =
        permissions_checking::get_route_from_public_url(req.uri().path()).unwrap_or_default();

    println!(
        "Received request for route: {}, with token: {}",
        route, token
    );

    next.run(req).await
}

/// Build a 401 response in the Connect-protocol JSON error shape.
/// Returning a structured Connect error keeps clients on the same
/// error-handling path they use for handler-side `ConnectError`s.
/// source: https://github.com/anthropics/connect-rust/blob/main/examples/middleware/src/server.rs
fn unauthorized(message: &'static str) -> Response {
    let err = ConnectError::new(ErrorCode::Unauthenticated, message);
    let body = http_body_util::Full::new(err.to_json())
        .map_err(|never| match never {})
        .boxed_unsync();
    http::Response::builder()
        .status(http::StatusCode::UNAUTHORIZED)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::new(body))
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(AuthServiceImpl);
    let connect = service.register(ConnectRouter::new());

    let cfg = AppConfiguration::INSTANCE();
    // Force initialization of the JWT public key at startup, so we fail fast if the private key is invalid.
    let _jwt_public = cfg.jwt_public_key();

    let app = axum::Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service())
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(auth_middleware))
                .layer(TimeoutLayer::with_status_code(
                    http::StatusCode::REQUEST_TIMEOUT,
                    std::time::Duration::from_secs(5),
                )),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.running_port))
        .await
        .context("bind TCP listener")?;

    println!("Auth service listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}
