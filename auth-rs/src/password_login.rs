use std::sync::Arc;

use anyhow::anyhow;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use http::StatusCode;
use hxi2_proto::proto::auth::v2::PasswordLoginResponse;
use serde::Serialize;
use tracing::{error, instrument, trace, warn};
use zeroize::Zeroizing;

use crate::login_manager::{LoginManager, LoginResponse};

#[derive(Clone)]
pub struct PasswordLoginManager {
    login_manager: Arc<LoginManager>,
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub redirect_to: Option<String>,
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

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn remove_password(&self, user_id: i64) -> anyhow::Result<()> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        db.remove_user_password(user_id)
            .await
            .map_err(|e| anyhow!("Failed to remove password: {e}"))
    }

    #[cfg_attr(
        debug_assertions,
        instrument(skip(self, password), level = "trace", ret)
    )]
    pub async fn try_login(&self, username: &str, password: &str) -> anyhow::Result<i64> {
        // make sure to always check a hash, to avoid timing attacks which could reveal whether a user exists or not.
        const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$VVdiljEvbLBmvl8ztJXUWg$YPEyPpwKESeb72CDEfZd2fMkT1mwUReqKiVkhkgDe/U";

        let password_input = Zeroizing::new(password.to_string());

        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        let (user_id, hash_to_verify) = db
            .get_user_password_hash(username)
            .await
            .map_err(|e| anyhow!("Failed to get user password hash: {e}"))
            .unwrap_or((0, DUMMY_HASH.to_string()));
        let user_exists = user_id != 0;

        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&hash_to_verify)?;

        let is_password_valid = argon2
            .verify_password(password_input.as_bytes(), &parsed_hash)
            .is_ok();

        if user_exists && is_password_valid {
            Ok(user_id)
        } else {
            Err(anyhow!("Invalid username or password"))
        }
    }

    // POST /api/password_login with form data: username, password, remember_me
    // Returns: 200 OK with cookies set on success, 401 Unauthorized on failure
    pub async fn login_handler(
        State(manager): State<Arc<Self>>,
        jar: CookieJar,
        form: axum::extract::Form<LoginForm>,
    ) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
        let user_id = manager
            .try_login(&form.username, &form.password)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username or password"))?;

        let login_response = manager.login_manager.login_as(
                &crate::login_manager::LoginID::UserID(user_id)
            )
            .await
            .map_err(|err| {
                if matches!(
                    err.downcast_ref::<crate::database::DbError>(),
                    Some(crate::database::DbError::NotFound)
                ) {
                    warn!("User {} (id={}) not found in database, yet tried to login with UserID.", form.username, user_id);
                    return (
                        StatusCode::UNAUTHORIZED,
                        "No user found with the given UserID. Ask in the Discord to have an account created.",
                    );
                }
                error!(error = %err, "Failed to login user by UserID.");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to login user by UserID.",
                )
            })?;
        let mut jar = login_response.make_cookies(jar)?;

        // get redirect_to from form data, then from cookie, then default to "/"
        let redirect_to = {
            if form.redirect_to.is_some() {
                form.redirect_to.clone()
            } else if let Some(cookie) = jar.get("redirect_to") {
                let redirect_to = cookie.value().to_string();
                jar = jar.remove(Cookie::from("redirect_to"));
                Some(redirect_to)
            } else {
                None
            }
        }
        .unwrap_or_else(|| "/".to_string());

        trace!(
            "User {} (id={}) logged in via password, redirecting to {}",
            form.username, user_id, redirect_to
        );

        let res = PasswordLoginResponse {
            redirect_to,
            ..Default::default()
        };
        let res_json = serde_json::to_string(&res).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize response",
            )
        })?;

        Ok((jar, res_json))
    }
}
