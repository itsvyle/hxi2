use std::sync::Arc;

use anyhow::anyhow;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use tracing::{instrument, trace};
use zeroize::Zeroizing;

use crate::{database::DbUserIdentifier, login_manager::LoginManager};

#[derive(Clone)]
pub struct PasswordLoginManager {
    _login_manager: Arc<LoginManager>,
}

impl PasswordLoginManager {
    pub fn new(_login_manager: Arc<LoginManager>) -> Self {
        Self { _login_manager }
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn add_password(
        &self,
        user_identifier: &DbUserIdentifier,
        password: &str,
    ) -> anyhow::Result<()> {
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
        db.add_user_password(user_identifier, &hash).await?;

        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn remove_password(&self, user_identifier: &DbUserIdentifier) -> anyhow::Result<()> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        db.remove_user_password(user_identifier)
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
            trace!(
                "Login failed for username '{}': user_exists={}, is_password_valid={}",
                username, user_exists, is_password_valid
            );
            Err(anyhow!("Invalid username or password"))
        }
    }
}
