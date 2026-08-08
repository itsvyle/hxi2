use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, sqlite::SqlitePool};

const ONE_TIME_CODE_VALIDITY_DURATION_MINS: i64 = 10;

#[derive(Debug)]
pub enum DbError {
    Sqlx(sqlx::Error),
    Validation(String),
    NotFound,
    Expired,
    Internal(String),
}

// This standard library trait makes the "?" operator work on your SQLx queries!
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        DbError::Sqlx(err)
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
        }
    }
}

impl std::error::Error for DbError {}

// -----------------------------------------------------------------------------
// Mocked external dependencies (equivalent to your `ggu` and `jwt` packages)
// -----------------------------------------------------------------------------
pub struct Hxi2JwtClaims {/* ... */}
pub struct SmallData {/* ... */}
fn generate_32bits_number() -> Result<i64, DbError> {
    Ok(1)
    // Ok(rand::thread_rng().gen_range(1..i32::MAX as i64))
}
fn generate_6_digit_number() -> Result<String, DbError> {
    Ok("123456".into())
    // Ok(format!("{:06}", rand::thread_rng().gen_range(0..999999)))
}
fn jwt_generate_refresh_token() -> Result<String, DbError> {
    Ok("mock_refresh_token".into())
}
fn jwt_generate_jti_token() -> Result<String, DbError> {
    Ok("mock_jti".into())
}
const REFRESH_TOKEN_VALIDITY_DAYS: i64 = 7;

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

    fn hash_token(&self, token: &str) -> String {
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
    pub id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: Option<String>, // Replaces sql.NullString
    pub discord_id: String,
    #[serde(skip)]
    pub account_created_date: DateTime<Utc>,
    #[serde(skip)]
    pub account_modified_date: DateTime<Utc>,
    pub promotion: i32,
    pub permissions: i64,
}

impl DbUser {
    pub fn check_schema(&self) -> Result<(), DbError> {
        if self.id == 0 {
            return Err(DbError::Validation("ID is missing".into()));
        }
        if self.first_name.is_empty() {
            return Err(DbError::Validation("first_name is missing".into()));
        }
        if self.discord_id.is_empty() {
            return Err(DbError::Validation("discord_id is missing".into()));
        }
        // Note: DateTime<Utc> in Rust cannot inherently be "zero" like time.IsZero() in Go.
        Ok(())
    }

    pub fn get_new_jwt_claims(&self) -> Hxi2JwtClaims {
        Hxi2JwtClaims { /* Map fields here */ }
    }

    pub fn get_small_data(&self) -> SmallData {
        SmallData { /* Map fields here */ }
    }
}

impl DatabaseManager {
    pub async fn add_new_db_user(&self, mut user: DbUser) -> Result<(), DbError> {
        if user.id == 0 {
            user.id = generate_32bits_number()?;
        }

        let now = Utc::now();
        user.account_created_date = now;
        user.account_modified_date = now;
        user.check_schema()?;

        sqlx::query(
            r#"
            INSERT INTO users (id, first_name, last_name, discord_id, account_created_date, account_modified_date, promotion, permissions)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.id)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.discord_id)
        .bind(user.account_created_date)
        .bind(user.account_modified_date)
        .bind(user.promotion)
        .bind(user.permissions)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<DbUser>, DbError> {
        let users = sqlx::query_as::<_, DbUser>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    pub async fn update_user(&self, mut user: DbUser) -> Result<(), DbError> {
        if user.id == 0 {
            return Err(DbError::Validation("no ID field on the user object".into()));
        }
        user.account_modified_date = Utc::now();
        user.check_schema()?;

        sqlx::query(
            r#"
            UPDATE users
            SET first_name = ?, last_name = ?, discord_id = ?, account_modified_date = ?, promotion = ?, permissions = ?, username = ?
            WHERE id = ?
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_db_user_by_discord_id(&self, discord_id: &str) -> Result<DbUser, DbError> {
        let user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE discord_id = ?")
            .bind(discord_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn get_db_user_by_id(&self, user_id: i64) -> Result<DbUser, DbError> {
        let user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }
}

// -----------------------------------------------------------------------------
// Refresh Tokens
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow)]
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

    pub async fn delete_refresh_token_hash(&self, hash: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM refresh_tokens WHERE refresh_token_hash = ?")
            .bind(hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn renew_refresh_token(
        &self,
        refresh_token: &str,
        jti: &str,
    ) -> Result<(i64, String, String), DbError> {
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

        rt.check_schema()?;

        // Token expiry check
        if Utc::now() - rt.created_at > Duration::days(REFRESH_TOKEN_VALIDITY_DAYS) {
            return Err(DbError::Expired);
        }

        let new_refresh_token = jwt_generate_refresh_token()?;
        let new_jti = jwt_generate_jti_token()?;

        if let Err(e) = self
            .add_refresh_token_pair(rt.associated_user_id, &new_refresh_token, &new_jti)
            .await
        {
            return Err(DbError::Internal(
                "failed to add new refresh token pair".into(),
            ));
        }

        if let Err(e) = self.delete_refresh_token_hash(&refresh_token_hash).await {
            return Err(DbError::Internal(
                "failed to delete old refresh token".into(),
            ));
        }

        Ok((rt.associated_user_id, new_refresh_token, new_jti))
    }

    pub async fn delete_refresh_token(&self, refresh_token: &str) -> Result<(), DbError> {
        let hash = self.hash_token(refresh_token);
        self.delete_refresh_token_hash(&hash).await
    }
}

// -----------------------------------------------------------------------------
// API Users
// -----------------------------------------------------------------------------
#[derive(Debug, FromRow, Serialize)]
pub struct DbApiUser {
    pub id: i64,
    pub username: String,
    pub token: String,
    pub permissions: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl DbApiUser {
    pub fn has_permission(&self, permission: i32) -> bool {
        (self.permissions & permission) == permission
    }
}

impl DatabaseManager {
    pub async fn list_api_users(&self) -> Result<Vec<DbApiUser>, DbError> {
        let users = sqlx::query_as::<_, DbApiUser>("SELECT * FROM API_TOKENS")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Sqlx(e))?;
        Ok(users)
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

        // This takes the place of your goroutine
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60));
            loop {
                interval.tick().await; // Waits for the next tick
                let now = Utc::now();

                let res = sqlx::query("DELETE FROM ONE_TIME_CODES WHERE expires_at < ?")
                    .bind(now)
                    .execute(&pool)
                    .await;

                // if let Err(e) = res {
                //     error!(error = ?e, "Failed to clean up expired one-time codes");
                // }
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

        if let Err(e) = sqlx::query("DELETE FROM ONE_TIME_CODES WHERE id = ?")
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
            .map_err(|e| DbError::Sqlx(e))?;

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

impl DbTemporaryCode {
    pub fn get_new_claims(&self) -> Result<Hxi2JwtClaims, DbError> {
        let _jti = jwt_generate_jti_token()?;
        // Implement claims mapping similar to your Go code
        Ok(Hxi2JwtClaims { /* ... */ })
    }
}

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
