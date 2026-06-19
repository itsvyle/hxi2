mod app_config;
mod auth_middleware;
mod auth_service;
mod connect_result;
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

use crate::{jwt_signer::GLOBAL_JWT_SIGNER, jwt_verifier::GLOBAL_JWT_VERIFIER};

#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(AuthServiceImpl {
        signer: &GLOBAL_JWT_SIGNER,
        verifier: &GLOBAL_JWT_VERIFIER,
    });
    let connect = service.register(ConnectRouter::new());

    let cfg = AppConfiguration::INSTANCE();
    // Force initialization of the JWT public key at startup, so we fail fast if the private key is invalid.
    let _jwt_public = cfg.jwt_public_key();

    // println!("Encoded JWT: {}", encode_jwt()?);

    let app = axum::Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service())
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_extractor::<RequireAuthMiddleware>())
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
