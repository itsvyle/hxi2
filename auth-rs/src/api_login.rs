use anyhow::Context as _;
use hxi2_proto::proto::auth::v2::Permission;
use std::sync::Arc;

use crate::{database::DbUserIdentifier, login_manager::LoginManager};

#[derive(Clone)]
pub struct APILoginManager {
    _login_manager: Arc<LoginManager>,
}

impl APILoginManager {
    pub fn new(_login_manager: Arc<LoginManager>) -> Self {
        Self { _login_manager }
    }

    pub async fn new_api_user(
        &self,
        username: &str,
        permissions: &[Permission],
    ) -> anyhow::Result<(i64, String)> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        let compiled_permissions = permissions.iter().fold(0, |acc, p| acc | (1 << *p as i64));

        // 1: Create new API User in the database
        let mut new_db_user = crate::database::DbUser {
            id: 0,
            username: username.to_string(),
            permissions: compiled_permissions,
            is_api: true,
            discord_id: "".to_string(),
            first_name: format!("API User {}", username),
            last_name: None,
            promotion: 0,
            account_created_date: chrono::Utc::now(),
            account_modified_date: chrono::Utc::now(),
        };
        db.add_new_db_user(&mut new_db_user)
            .await
            .context("Failed to add API user")?;

        let token = self
            .renew_api_token(&DbUserIdentifier::Id(new_db_user.id))
            .await
            .context("Failed to generate API token")?;

        Ok((new_db_user.id, token))
    }

    fn generate_api_token(&self) -> String {
        use rand::RngExt;
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 32] = rng.random();
        hex::encode(jti)
    }

    pub async fn renew_api_token(
        &self,
        user_identifier: &DbUserIdentifier,
    ) -> anyhow::Result<String> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        let new_token = self.generate_api_token();
        let new_token_hash = db.hash_token(&new_token);
        let expires_at =
            chrono::Utc::now() + crate::app_config::AppConfiguration::INSTANCE().API_TOKEN_VALIDITY;
        db.insert_update_api_token(user_identifier, &new_token_hash, expires_at)
            .await
            .context("Failed to insert/update API token")?;

        Ok(new_token)
    }
}
