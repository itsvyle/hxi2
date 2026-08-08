mod app_config;
mod auth_middleware;
mod auth_service;
mod connect_result;
mod csrf_handler;
mod database;
mod jwt_signer;
mod jwt_verifier;
mod permissions_checking;

use anyhow::{Context as _, Result};
use app_config::AppConfiguration;
use auth_middleware::RequireAuthMiddleware;
use auth_service::{AuthServiceExt, AuthServiceImpl};
use axum::routing::get;
use connectrpc::Router as ConnectRouter;
use std::sync::Arc;

use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;

use crate::{
    csrf_handler::{CsrfProtection, GLOBAL_CSRF_PROTECTION},
    jwt_signer::GLOBAL_JWT_SIGNER,
    jwt_verifier::GLOBAL_JWT_VERIFIER,
};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

// 1. Route Handler to serve the static HTML file
async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("src/index.html").await {
        Ok(html_content) => Html(html_content).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not load index.html. Ensure the file is in your running directory.",
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = AppConfiguration::INSTANCE();
    // Force initialization of the JWT public key at startup, so we fail fast if the private key is invalid.
    let _jwt_public = cfg.jwt_public_key();
    let _csrf_prot = GLOBAL_CSRF_PROTECTION.generate_token();
    let _db = cfg.db().await;

    let service = Arc::new(AuthServiceImpl {
        subdomain: format!("auth.{}", cfg.tld),
        signer: &GLOBAL_JWT_SIGNER,
        verifier: &GLOBAL_JWT_VERIFIER,
    });
    let connect = service.register(ConnectRouter::new());

    let app = axum::Router::new()
        .route("/", get(serve_index))
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service())
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_extractor::<RequireAuthMiddleware>())
                .layer(axum::middleware::from_fn(CsrfProtection::middleware))
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
