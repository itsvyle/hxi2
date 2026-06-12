mod auth_service;
mod permissions_checking;

use anyhow::{Context as _, Result};
use auth_service::{AuthServiceExt, AuthServiceImpl};
use axum::{Router, routing::get};
use connectrpc::Router as ConnectRouter;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{:#?}", permissions_checking::get_compiled_permissions());

    let service = Arc::new(AuthServiceImpl);
    let connect = service.register(ConnectRouter::new());

    // Plain HTTP liveness probe for `kubectl`'s httpGet style. For the
    // standard gRPC Health protocol (grpc_health_probe, kubelet `grpc:`
    // probes), mount `connectrpc_health::HealthService` on the Connect
    // router instead — see docs/guide.md#health-checking.
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .context("bind TCP listener")?;
    axum::serve(listener, app).await?;
    Ok(())
}
