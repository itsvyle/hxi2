use anyhow::Context as _;
use hxi2_proto::{
    proto::auth::v2::{Attribute, Permission, small_data},
    utils::compile_bitfield,
};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::Instant,
};
use tracing::instrument;

use crate::{
    database::DbUserIdentifier,
    login_manager::{LoginManager, LoginResponse},
};

struct TokenCache {
    valid_ids: HashSet<i64>,
    last_update: Instant,
}

pub struct APILoginManager {
    login_manager: Arc<LoginManager>,
    cache: std::sync::RwLock<TokenCache>,
}
const VALID_IDS_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 5); // 5 minutes

impl APILoginManager {
    pub async fn new(login_manager: Arc<LoginManager>) -> Self {
        let r = Self {
            login_manager,
            cache: RwLock::new(TokenCache {
                valid_ids: HashSet::new(),
                last_update: Instant::now() - VALID_IDS_REFRESH_INTERVAL,
            }),
        };
        r.refresh_valid_ids()
            .await
            .expect("Failed to refresh valid API token IDs");
        r
    }

    #[instrument(skip(self), err)]
    async fn refresh_valid_ids(&self) -> anyhow::Result<()> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        let valid_ids = db
            .get_valid_api_token_ids()
            .await
            .context("Failed to fetch API user IDs")?;

        let mut cache = self.cache.write().unwrap();
        cache.valid_ids = valid_ids.into_iter().collect();
        cache.last_update = Instant::now();

        Ok(())
    }

    pub async fn check_valid_api_token_id(&self, id: i64) -> anyhow::Result<bool> {
        {
            let cache = self.cache.read().unwrap();
            if cache.last_update.elapsed() <= VALID_IDS_REFRESH_INTERVAL {
                return Ok(cache.valid_ids.contains(&id));
            }
        }

        self.refresh_valid_ids().await?;

        let cache = self.cache.read().unwrap();
        Ok(cache.valid_ids.contains(&id))
    }

    pub async fn new_api_user(
        &self,
        username: &str,
        permissions: &[Permission],
        attributes: &[Attribute],
    ) -> anyhow::Result<(i64, String)> {
        if username.is_empty() {
            anyhow::bail!("Username cannot be empty");
        }
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        let compiled_permissions = compile_bitfield(permissions.iter().copied());
        let compiled_attributes = compile_bitfield(attributes.iter().copied());

        // 1: Create new API User in the database
        let mut new_db_user = crate::database::DbUser {
            id: 0,
            username: username.to_string(),
            permissions: compiled_permissions,
            is_api: true,
            discord_id: None,
            first_name: format!("API User {}", username),
            last_name: None,
            promotion: 0,
            attributes: compiled_attributes,
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

        self.refresh_valid_ids()
            .await
            .context("Failed to refresh valid API token IDs")?;

        Ok(new_token)
    }

    pub async fn login_with_api_token(
        &self,
        user_identifier: &DbUserIdentifier,
        token: &str,
    ) -> anyhow::Result<LoginResponse> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        let db_token = db
            .get_potential_api_token(user_identifier)
            .await
            .context("Failed to retrieve API token from database")?;

        if db_token.is_none() {
            anyhow::bail!("No API token found for user");
        }

        let db_token = db_token.unwrap();
        let token_hash = db.hash_token(token);
        if db_token.token_hash != token_hash {
            anyhow::bail!("Invalid API token");
        }

        let response = self
            .login_manager
            .login_as(
                user_identifier,
                Some(small_data::APITokenData {
                    token_id: db_token.id,
                    __buffa_unknown_fields: Default::default(),
                }),
            )
            .await?;
        Ok(response)
    }

    pub async fn delete_api_token(&self, user_identifier: &DbUserIdentifier) -> anyhow::Result<()> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;

        db.delete_api_token_by_user(user_identifier)
            .await
            .context("Failed to remove API token")?;

        self.refresh_valid_ids()
            .await
            .context("Failed to refresh valid API token IDs")?;

        Ok(())
    }
}
