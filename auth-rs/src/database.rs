#![allow(dead_code)]
use buffa_types::Timestamp;
use chrono::{DateTime, Duration, Utc};
use hxi2_proto::{
    proto::auth::v2::{Attribute, Permission, list_api_users_response},
    utils::retrieve_active_bitfields,
};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Row, Sqlite, sqlite::SqlitePool};
use tracing::{error, instrument};

use crate::app_config;

const ONE_TIME_CODE_VALIDITY_DURATION_MINS: i64 = 10;

#[derive(Debug)]
pub enum DbError {
    Sqlx(sqlx::Error),
    Validation(String),
    NotFound,
    Expired,
    Internal(String),
    Duplicate(String),
}

// This standard library trait makes the "?" operator work on your SQLx queries!
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        if matches!(err, sqlx::Error::RowNotFound) {
            DbError::NotFound
        } else if matches!(err, sqlx::Error::Database(ref db_err) if db_err.code().map(|c| c == "2067").unwrap_or(false))
        {
            DbError::Duplicate(format!("Duplicate entry: {}", err))
        } else {
            DbError::Sqlx(err)
        }
    }
}

// Implement standard display formatting for your error
impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Sqlx(e) => write!(f, "Database error: {}", e),
            DbError::Validation(s) => write!(f, "Validation error: {}", s),
            DbError::NotFound => write!(f, "Not found or expired"),
            DbError::Expired => write!(f, "Expired"),
            DbError::Internal(s) => write!(f, "Internal error: {}", s),
            DbError::Duplicate(s) => write!(f, "Duplicate entry: {}", s),
        }
    }
}

impl std::error::Error for DbError {}

fn generate_32bits_number() -> Result<i64, DbError> {
    use rand::rngs::ThreadRng;

    let mut rng = ThreadRng::default();
    Ok(rng.random_range(1..i32::MAX as i64))
}
fn generate_6_digit_number() -> Result<String, DbError> {
    use rand::rngs::ThreadRng;

    let mut rng = ThreadRng::default();
    Ok(rng.random_range(100_000..1_000_000).to_string())
}
fn jwt_generate_refresh_token() -> Result<String, DbError> {
    unimplemented!("Implement a secure random refresh token generator here");
}
fn jwt_generate_jti_token() -> Result<String, DbError> {
    unimplemented!("Implement a secure random JTI token generator here");
}

// -----------------------------------------------------------------------------
// Database Manager
// -----------------------------------------------------------------------------
#[derive(Clone)]
pub struct DatabaseManager {
    pub pool: SqlitePool,
}

impl DatabaseManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

// -----------------------------------------------------------------------------
// DBUser
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DbUser {
    #[sqlx(rename = "ID")]
    pub id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>, // Replaces sql.NullString
    pub discord_id: Option<String>,
    #[serde(skip)]
    pub account_created_date: DateTime<Utc>,
    #[serde(skip)]
    pub account_modified_date: DateTime<Utc>,
    pub promotion: i32,
    pub permissions: i64,
    pub attributes: i64,
    pub is_api: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DbUserIdentifier {
    Id(i64),
    Username(String),
    DiscordId(String),
}

impl DbUserIdentifier {
    pub async fn to_user_id(&self, db: &DatabaseManager) -> Result<i64, DbError> {
        match self {
            DbUserIdentifier::Id(id) => Ok(*id),
            _ => db.resolve_user_identifier(self).await,
        }
    }
}

impl DbUser {
    pub fn check_schema(&self) -> Result<(), DbError> {
        if self.id == 0 {
            return Err(DbError::Validation("ID is missing".into()));
        }
        if self.first_name.is_empty() {
            return Err(DbError::Validation("first_name is missing".into()));
        }
        // Note: DateTime<Utc> in Rust cannot inherently be "zero" like time.IsZero() in Go.
        Ok(())
    }
}

impl From<DbUser> for hxi2_proto::proto::auth::v2::DBUser {
    fn from(user: DbUser) -> Self {
        use buffa_types::Timestamp;
        hxi2_proto::proto::auth::v2::DBUser {
            id: user.id,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name.clone(),
            discord_id: user.discord_id.clone(),
            account_created_date: Timestamp::from_unix_secs(user.account_created_date.timestamp())
                .into(),
            account_modified_date: Timestamp::from_unix_secs(
                user.account_modified_date.timestamp(),
            )
            .into(),
            promotion: user.promotion,
            permissions: user.permissions,
            is_api: user.is_api,
            attributes: user.attributes,
            __buffa_unknown_fields: Default::default(),
        }
    }
}

impl From<hxi2_proto::proto::auth::v2::DBUser> for DbUser {
    fn from(proto_user: hxi2_proto::proto::auth::v2::DBUser) -> Self {
        DbUser {
            id: proto_user.id,
            username: proto_user.username,
            first_name: proto_user.first_name,
            last_name: proto_user.last_name.clone(),
            discord_id: proto_user.discord_id,
            account_created_date: proto_user
                .account_created_date
                .map(|ts| chrono::DateTime::<chrono::Utc>::from_timestamp_secs(ts.seconds))
                .flatten()
                .unwrap_or_default(),
            account_modified_date: proto_user
                .account_modified_date
                .map(|ts| chrono::DateTime::<chrono::Utc>::from_timestamp_secs(ts.seconds))
                .flatten()
                .unwrap_or_default(),
            promotion: proto_user.promotion,
            permissions: proto_user.permissions,
            attributes: proto_user.attributes,
            is_api: proto_user.is_api,
        }
    }
}

impl DatabaseManager {
    #[instrument(skip(self), err)]
    pub async fn add_new_db_user(&self, user: &mut DbUser) -> Result<(), DbError> {
        if user.id == 0 {
            user.id = generate_32bits_number()?;
        }

        let now = Utc::now();
        user.account_created_date = now;
        user.account_modified_date = now;
        user.check_schema()?;

        sqlx::query(
            r#"
            INSERT INTO users (ID, username, first_name, last_name, discord_id, account_created_date, account_modified_date, promotion, permissions, is_api, attributes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.discord_id)
        .bind(user.account_created_date)
        .bind(user.account_modified_date)
        .bind(user.promotion)
        .bind(user.permissions)
        .bind(user.is_api)
        .bind(user.attributes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    pub async fn list_users(&self) -> Result<Vec<DbUser>, DbError> {
        let users = sqlx::query_as::<_, DbUser>("SELECT * FROM USERS WHERE is_api = 0")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    #[instrument(skip(self), err)]
    pub async fn update_user(&self, user: &mut DbUser) -> Result<(), DbError> {
        if user.id == 0 {
            return Err(DbError::Validation("no ID field on the user object".into()));
        }
        user.account_modified_date = Utc::now();
        user.check_schema()?;

        sqlx::query(
            r#"
            UPDATE users
            SET first_name = ?, last_name = ?, discord_id = ?, account_modified_date = ?, promotion = ?, permissions = ?, username = ?, is_api = ?, attributes = ?
            WHERE ID = ?
            "#,
        )
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.discord_id)
        .bind(user.account_modified_date)
        .bind(user.promotion)
        .bind(user.permissions)
        .bind(&user.username)
        .bind(user.id)
        .bind(user.is_api)
        .bind(user.attributes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_db_user(&self, identifier: &DbUserIdentifier) -> Result<DbUser, DbError> {
        let query = match identifier {
            DbUserIdentifier::Id(_) => "SELECT * FROM users WHERE ID = ?",
            DbUserIdentifier::Username(_) => "SELECT * FROM users WHERE username = ?",
            DbUserIdentifier::DiscordId(_) => "SELECT * FROM users WHERE discord_id = ?",
        };

        let user = sqlx::query_as::<_, DbUser>(query);
        let user = match identifier {
            DbUserIdentifier::Id(id) => user.bind(id),
            DbUserIdentifier::Username(username) => user.bind(username),
            DbUserIdentifier::DiscordId(discord_id) => user.bind(discord_id),
        };
        let user = user.fetch_one(&self.pool).await.map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                DbError::NotFound
            } else {
                DbError::Sqlx(e)
            }
        })?;
        Ok(user)
    }

    pub async fn resolve_user_identifier(
        &self,
        identifier: &DbUserIdentifier,
    ) -> Result<i64, DbError> {
        if let DbUserIdentifier::Id(id) = identifier {
            return Ok(*id);
        };
        let query = match identifier {
            DbUserIdentifier::Username(_) => "SELECT ID FROM users WHERE username = ?",
            DbUserIdentifier::DiscordId(_) => "SELECT ID FROM users WHERE discord_id = ?",
            _ => unreachable!(),
        };

        let user_id = sqlx::query_scalar::<_, i64>(query)
            .bind(match identifier {
                DbUserIdentifier::Username(username) => username,
                DbUserIdentifier::DiscordId(discord_id) => discord_id,
                _ => unreachable!(),
            })
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    DbError::NotFound
                } else {
                    DbError::Sqlx(e)
                }
            })?;
        Ok(user_id)
    }
}

// -----------------------------------------------------------------------------
// Refresh Tokens
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow)]
#[allow(unused)]
pub struct DbRefreshToken {
    pub associated_user_id: i64,
    pub refresh_token_hash: String,
    pub jti_hash: String,
    pub created_at: DateTime<Utc>,
}

impl DbRefreshToken {
    pub fn check_schema(&self) -> Result<(), DbError> {
        if self.associated_user_id == 0 {
            return Err(DbError::Validation("associated_user_id is missing".into()));
        }
        if self.refresh_token_hash.is_empty() {
            return Err(DbError::Validation("refresh_token_hash is missing".into()));
        }
        if self.jti_hash.is_empty() {
            return Err(DbError::Validation("jti_hash is missing".into()));
        }
        Ok(())
    }
}

impl DatabaseManager {
    #[instrument(skip(self), err)]
    pub async fn add_refresh_token_pair(
        &self,
        user_id: i64,
        refresh_token: &str,
        jti: &str,
    ) -> Result<(), DbError> {
        let refresh_token_hash = self.hash_token(refresh_token);
        let jti_hash = self.hash_token(jti);

        sqlx::query(
            "INSERT INTO refresh_tokens (associated_user_id, refresh_token_hash, jti_hash, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(refresh_token_hash)
        .bind(jti_hash)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), err))]
    pub async fn check_refresh_token(
        &self,
        refresh_token: &str,
        jti: &str,
    ) -> Result<DbRefreshToken, DbError> {
        let refresh_token_hash = self.hash_token(refresh_token);
        let jti_hash = self.hash_token(jti);

        let rt = sqlx::query_as::<_, DbRefreshToken>(
            "SELECT * FROM refresh_tokens WHERE refresh_token_hash = ? AND jti_hash = ?",
        )
        .bind(&refresh_token_hash)
        .bind(&jti_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DbError::NotFound,
            _ => DbError::Sqlx(e),
        })?;

        if Utc::now() - rt.created_at
            > app_config::AppConfiguration::INSTANCE().JWT_REFRESH_TOKEN_VALIDITY
        {
            return Err(DbError::Expired);
        }

        Ok(rt)
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), err))]
    pub async fn delete_refresh_token(&self, refresh_token: &str) -> Result<(), DbError> {
        let refresh_token_hash = self.hash_token(refresh_token);

        sqlx::query("DELETE FROM refresh_tokens WHERE refresh_token_hash = ?")
            .bind(&refresh_token_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// API Users
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow, Serialize)]
pub struct DbApiToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl DatabaseManager {
    pub async fn list_api_tokens(&self) -> Result<Vec<DbApiToken>, DbError> {
        let users = sqlx::query_as::<_, DbApiToken>("SELECT * FROM API_TOKENS")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;
        Ok(users)
    }

    pub async fn insert_update_api_token(
        &self,
        user_identifier: &DbUserIdentifier,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let user_id = user_identifier.to_user_id(self).await?;
        sqlx::query("REPLACE INTO API_TOKENS (user_id, token_hash, expires_at) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(token_hash)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(())
    }

    pub async fn get_potential_api_token(
        &self,
        user_identifier: &DbUserIdentifier,
    ) -> Result<Option<DbApiToken>, DbError> {
        let user_id = user_identifier.to_user_id(self).await?;
        let token = sqlx::query_as::<_, DbApiToken>("SELECT * FROM API_TOKENS WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map(Some)
            .or_else(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    Ok(None)
                } else {
                    Err(DbError::Sqlx(e))
                }
            })?;
        Ok(token)
    }

    pub async fn delete_api_token_by_hash(&self, token_hash: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM API_TOKENS WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(())
    }

    pub async fn delete_api_token_by_user(
        &self,
        user_identifier: &DbUserIdentifier,
    ) -> Result<(), DbError> {
        let user_id = user_identifier.to_user_id(self).await?;
        sqlx::query("DELETE FROM API_TOKENS WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    pub async fn list_api_users(&self) -> Result<Vec<list_api_users_response::APIUser>, DbError> {
        let api_users = sqlx::query::<Sqlite>(
            "SELECT 
        u.id, u.username, u.permissions, u.attributes,
        t.id as token_id, 
        t.created_at as token_created_at,
        t.expires_at as token_expires_at
     FROM users u
     LEFT JOIN API_TOKENS t ON u.id = t.user_id
     WHERE u.is_api = 1",
        )
        .map(|row| list_api_users_response::APIUser {
            username: row.get("username"),
            user_id: row.get("ID"),
            roles: retrieve_active_bitfields::<Permission>(row.get::<i64, _>("permissions"))
                .map(Into::into)
                .collect(),
            attributes: retrieve_active_bitfields::<Attribute>(row.get::<i64, _>("attributes"))
                .map(Into::into)
                .collect(),
            token_created_at: row
                .get::<Option<DateTime<Utc>>, _>("token_created_at")
                .map(|dt| Timestamp::from_unix_secs(dt.timestamp()))
                .into(),
            token_expires_at: row
                .get::<Option<DateTime<Utc>>, _>("token_expires_at")
                .map(|dt| Timestamp::from_unix_secs(dt.timestamp()))
                .into(),
            has_api_token: row.get::<Option<i64>, _>("token_id").is_some(),
            token_id: row.get::<Option<i64>, _>("token_id"),
            __buffa_unknown_fields: Default::default(),
        })
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sqlx)?;

        Ok(api_users)
    }
}

// -----------------------------------------------------------------------------
// One Time Codes & Background Timer
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow)]
pub struct DbOneTimeCode {
    pub id: i64,
    pub user_id: i64,
    pub code_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl DatabaseManager {
    pub fn start_one_time_code_cleanup_timer(&self) {
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60));
            loop {
                interval.tick().await; // Waits for the next tick
                let now = Utc::now();

                let res = sqlx::query("DELETE FROM ONE_TIME_CODES WHERE expires_at < ?")
                    .bind(now)
                    .execute(&pool)
                    .await;

                if let Err(e) = res {
                    error!(error = ?e, "Failed to clean up expired one-time codes");
                }
            }
        });
    }

    pub async fn check_one_time_code(&self, code: &str) -> Result<i64, DbError> {
        if code.is_empty() {
            return Err(DbError::Validation("code is empty".into()));
        }

        let code_hash = self.hash_token(code);

        let otc =
            sqlx::query_as::<_, DbOneTimeCode>("SELECT * FROM ONE_TIME_CODES WHERE code_hash = ?")
                .bind(&code_hash)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => DbError::NotFound,
                    _ => DbError::Sqlx(e),
                })?;

        if Utc::now() > otc.expires_at {
            return Err(DbError::Expired);
        }

        if let Err(_e) = sqlx::query("DELETE FROM ONE_TIME_CODES WHERE id = ?")
            .bind(otc.id)
            .execute(&self.pool)
            .await
        {
            return Err(DbError::Internal("failed to delete one-time code".into()));
        }

        Ok(otc.user_id)
    }

    async fn create_one_time_code_internal(
        &self,
        user_id: i64,
        expires_in: Duration,
    ) -> Result<String, DbError> {
        let code = generate_6_digit_number()?;
        let code_hash = self.hash_token(&code);
        let expires_at = Utc::now() + expires_in;

        sqlx::query("INSERT INTO ONE_TIME_CODES (user_id, code_hash, expires_at) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(&code_hash)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(DbError::Sqlx)?;

        Ok(code)
    }

    pub async fn create_one_time_code(&self, user_id: i64) -> Result<String, DbError> {
        if user_id <= 0 {
            return Err(DbError::Validation("invalid user ID".into()));
        }

        self.create_one_time_code_internal(
            user_id,
            Duration::minutes(ONE_TIME_CODE_VALIDITY_DURATION_MINS),
        )
        .await
    }
}

// -----------------------------------------------------------------------------
// Temporary Codes
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow)]
pub struct DbTemporaryCode {
    pub id: i64,
    pub username: String,
    pub code_hash: String,
    pub recheck_after: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl DbTemporaryCode {}

impl DatabaseManager {
    pub async fn check_temp_code(
        &self,
        username: &str,
        code: &str,
    ) -> Result<DbTemporaryCode, DbError> {
        if username.is_empty() || code.is_empty() {
            return Err(DbError::Validation("username or code is empty".into()));
        }

        let code_hash = self.hash_token(code);

        let temp_code = sqlx::query_as::<_, DbTemporaryCode>(
            "SELECT * FROM TEMPORARY_CODES WHERE username = ? AND code_hash = ?",
        )
        .bind(username)
        .bind(&code_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DbError::NotFound,
            _ => DbError::Sqlx(e),
        })?;

        if Utc::now() > temp_code.expires_at {
            return Err(DbError::Expired);
        }

        Ok(temp_code)
    }

    pub async fn get_temp_from_username(&self, username: &str) -> Result<DbTemporaryCode, DbError> {
        if username.is_empty() {
            return Err(DbError::Validation("username is empty".into()));
        }

        let temp_code = sqlx::query_as::<_, DbTemporaryCode>(
            "SELECT * FROM TEMPORARY_CODES WHERE username = ?",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        Ok(temp_code)
    }
}

// ==============================================================================
// USERS_PASSWORDS
// ==============================================================================
#[derive(Debug, FromRow)]
pub struct DbUserPassword {
    pub user_id: i64,
    pub password_hash: String,
    pub updated_at: DateTime<Utc>,
}

impl DatabaseManager {
    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn add_user_password(
        &self,
        user_identifier: &DbUserIdentifier,
        password_hash: &str,
    ) -> Result<(), DbError> {
        let user_id = user_identifier.to_user_id(self).await?;

        if user_id <= 0 {
            return Err(DbError::Validation("invalid user ID".into()));
        }
        if password_hash.is_empty() {
            return Err(DbError::Validation("password hash is empty".into()));
        }

        sqlx::query(
            "INSERT INTO USERS_PASSWORDS (user_id, password_hash, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(user_id) DO UPDATE SET password_hash = excluded.password_hash, updated_at = excluded.updated_at",
        )
        .bind(user_id)
        .bind(password_hash)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_user_password(
        &self,
        user_identifier: &DbUserIdentifier,
    ) -> Result<(), DbError> {
        let user_id = user_identifier.to_user_id(self).await?;

        if user_id <= 0 {
            return Err(DbError::Validation("invalid user ID".into()));
        }

        sqlx::query("DELETE FROM USERS_PASSWORDS WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn get_user_password_hash(&self, username: &str) -> Result<(i64, String), DbError> {
        if username.is_empty() {
            return Err(DbError::Validation("username is empty".into()));
        }

        let password_hash = sqlx::query_as::<_, DbUserPassword>(
            "SELECT user_id, * FROM USERS_PASSWORDS
            JOIN USERS ON USERS_PASSWORDS.user_id = users.ID
            WHERE USERS.username = ?",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DbError::NotFound,
            _ => DbError::Sqlx(e),
        })?;
        Ok((password_hash.user_id, password_hash.password_hash))
    }
}
