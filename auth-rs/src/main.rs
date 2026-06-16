mod app_config;
mod auth_service;
mod jwt_signer;
mod permissions_checking;

use anyhow::{Context as _, Result};
use app_config::AppConfiguration;
use auth_service::{AuthServiceExt, AuthServiceImpl};
use axum::routing::get;
use connectrpc::{ConnectError, Router as ConnectRouter};
use hxi2_proto::proto::auth::v2::SmallData;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;

#[derive(Debug, Clone)]
pub struct UserId(String);

struct RequireAuth;

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
        else {
            return Err(ConnectError::permission_denied("missing Bearer token")
                .into_http_response(&parts.headers)
                .into_response());
        };

        let route =
            permissions_checking::get_route_from_public_url(parts.uri.path()).unwrap_or_default();

        parts.extensions.insert(UserId(token.to_string()));

        println!(
            "Received request for route: {}, with token: {}",
            route, token
        );

        Ok(Self)
    }
}

fn encode_jwt() -> Result<String> {
    let signer = jwt_signer::JWTSigner::new()?;
    let (token, _) = signer.new_token(
        Duration::from_hours(2),
        "me",
        SmallData {
            first_name: "ur mom".to_string(),
            ..Default::default()
        },
    )?;

    Ok(token)
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(AuthServiceImpl);
    let connect = service.register(ConnectRouter::new());

    let cfg = AppConfiguration::INSTANCE();
    // Force initialization of the JWT public key at startup, so we fail fast if the private key is invalid.
    let _jwt_public = cfg.jwt_public_key();

    println!("Encoded JWT: {}", encode_jwt()?);

    let app = axum::Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service())
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_extractor::<RequireAuth>())
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
