mod app_config;
mod auth_middleware;
mod auth_service;
mod connect_result;
mod csrf_handler;
mod database;
mod discord_login;
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
    discord_login::DiscordLoginManager,
    jwt_signer::GLOBAL_JWT_SIGNER,
    jwt_verifier::GLOBAL_JWT_VERIFIER,
};

use axum::{http::StatusCode, response::Html};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("sqlx=warn".parse().unwrap());

    tracing_subscriber::fmt().with_env_filter(filter).init();
    let cfg = AppConfiguration::INSTANCE();
    debug!(cfg = ?cfg,"Loaded configuration");
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

    let discord_manager = DiscordLoginManager::new(cfg)?;

    let mut app = axum::Router::new().merge(discord_manager.router());
    for (_, method_perms) in permissions_checking::get_compiled_permissions().permissions {
        if method_perms.is_frontend
            && method_perms.is_public
            && let Some(file_path) = method_perms.frontend_static_file
        {
            for url in method_perms.public_url.unwrap_or(&[]) {
                app = app.route(
                    url,
                    get(move || async move {
                        let file_content = tokio::fs::read(format!("./src/{file_path}"))
                            .await
                            .map_err(|err| {
                                error!(
                                    error = %err,
                                    file_path = "./src/{file_path}",
                                    "Failed to read file from disk"
                                );
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")
                            })?;

                        Ok::<_, (StatusCode, &'static str)>(Html(file_content))
                    }),
                );
                debug!("Registered static file route: {} -> {}", url, file_path);
            }
        }
    }

    app = app
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

    info!(port = ?listener.local_addr()?, "Auth service listening");

    axum::serve(listener, app).await?;
    Ok(())
}
