use std::sync::Arc;

use anyhow::anyhow;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use http::StatusCode;
use tracing::instrument;
use zeroize::Zeroizing;

use crate::login_manager::LoginManager;

#[derive(Clone)]
pub struct PasswordLoginManager {
    login_manager: Arc<LoginManager>,
}

impl PasswordLoginManager {
    pub fn new(login_manager: Arc<LoginManager>) -> Self {
        Self { login_manager }
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn add_password(&self, user_id: i64, password: &str) -> anyhow::Result<()> {
        let password = Zeroizing::new(password.to_string());

        let hash = tokio::task::spawn_blocking(move || {
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
                .map_err(|e| anyhow!("Failed to hash password: {e}"))
        })
        .await
        .map_err(|e| anyhow!("Join error: {e}"))??;

        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        db.add_user_password(user_id, &hash).await?;

        Ok(())
    }

    // POST /api/password_login with form data: username, password, remember_me
    // Returns: 200 OK with cookies set on success, 401 Unauthorized on failure
    /* pub async fn login_handler(
        State(manager): State<Arc<Self>>,
        jar: CookieJar,
    ) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to logout user"));
    } */
}
