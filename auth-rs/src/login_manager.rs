use anyhow::Context as _;
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use base64::prelude::*;
use http::StatusCode;
use hxi2_proto::proto::auth::v2::{DBUser, SmallData};
use rand::RngExt;
use tracing::error;

use crate::{app_config::AppConfiguration, database::DbUser, jwt_signer::JWTSignerOptions};

pub struct LoginManager {
    pub signer: &'static crate::jwt_signer::JWTSigner,
    #[allow(unused)]
    pub verifier: &'static crate::jwt_verifier::JWTVerifier,
}

pub struct LoginResponse {
    pub token: String,
    pub token_max_age: i64,
    pub refresh_token: String,
    pub refresh_token_max_age: i64,
    pub small_data: SmallData,
}

impl LoginResponse {
    /// Returns: jwt_cookie, refresh_token_cookie, small_data_cookie
    pub fn make_cookies(&self, jar: CookieJar) -> Result<CookieJar, (StatusCode, &'static str)> {
        let cfg = AppConfiguration::INSTANCE();
        let apply_base_cookie_options = |cookie: &mut Cookie| {
            cookie.set_domain(&cfg.cookies_domain);
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Lax);
            cookie.set_secure(match cfg.environment {
                crate::app_config::Environnement::Development => false,
                crate::app_config::Environnement::Production => true,
            });
        };

        // JWT Cookie: must be alive at least as long as refresh token, to provide adequate refreshing
        let mut jwt_cookie = Cookie::build((cfg.COOKIE_JWT_TOKEN_NAME, self.token.to_owned()))
            .max_age(time::Duration::seconds(self.refresh_token_max_age))
            .build();
        apply_base_cookie_options(&mut jwt_cookie);

        // Refresh Token Cookie
        let mut refresh_token_cookie =
            Cookie::build((cfg.COOKIE_REFRESH_TOKEN_NAME, self.refresh_token.to_owned()))
                .max_age(time::Duration::seconds(self.refresh_token_max_age))
                .build();
        apply_base_cookie_options(&mut refresh_token_cookie);

        // Small Data Cookie
        let small_data_json = serde_json::to_string(&self.small_data).map_err(|e| {
            error!(error = %e, "Failed to serialize small_data to JSON.");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize small_data to JSON.",
            )
        })?;
        let small_data_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(small_data_json);

        let mut small_data_cookie = Cookie::build((cfg.COOKIE_SMALL_DATA_NAME, small_data_b64))
            .max_age(time::Duration::seconds(self.refresh_token_max_age))
            .build();
        apply_base_cookie_options(&mut small_data_cookie);
        small_data_cookie.set_http_only(false);

        Ok(jar
            .add(jwt_cookie)
            .add(refresh_token_cookie)
            .add(small_data_cookie))
    }
}

pub enum LoginID {
    UserID(i64),
    DiscordID(String),
}

impl LoginManager {
    pub fn new(
        signer: &'static crate::jwt_signer::JWTSigner,
        verifier: &'static crate::jwt_verifier::JWTVerifier,
    ) -> Self {
        Self { signer, verifier }
    }

    fn small_data_from_user(&self, user: &DbUser) -> SmallData {
        SmallData {
            user_id: user.id,
            username: user.username.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            permissions: user.permissions,
            promotion: user.promotion,
            __buffa_unknown_fields: Default::default(),
        }
    }

    fn generate_refresh_token() -> String {
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 16] = rng.random();
        hex::encode(jti)
    }

    // This creates a token for the user
    // Authorization to login as such a user must have been checked PRIOR
    pub async fn login_as(&self, user_id: &LoginID) -> anyhow::Result<LoginResponse> {
        let cfg = crate::app_config::AppConfiguration::INSTANCE();
        let user = match user_id {
            LoginID::UserID(uid) => cfg.db().await.get_db_user_by_id(uid).await,
            LoginID::DiscordID(did) => cfg.db().await.get_db_user_by_discord_id(did).await,
        }?;

        let small_data = self.small_data_from_user(&user);
        let opts = JWTSignerOptions::default();
        let (token, claims) = self
            .signer
            .new_token(&format!("{}", user.id), &small_data, &opts)?;

        let refresh_token = Self::generate_refresh_token();

        cfg.db()
            .await
            .add_refresh_token_pair(user.id, &refresh_token, &claims.jti)
            .await
            .context("inserting refresh token into database")?;

        Ok(LoginResponse {
            token,
            token_max_age: opts.validity.num_seconds(),
            refresh_token,
            refresh_token_max_age: cfg.JWT_REFRESH_TOKEN_VALIDITY.num_seconds(),
            small_data: claims
                .data
                .ok_or_else(|| anyhow::anyhow!("claims.data should be present"))?,
        })
    }

    pub async fn renew_authentication(
        &self,
        old_token: &str,
        refresh_token: &str,
    ) -> anyhow::Result<LoginResponse> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        let claims = self
            .verifier
            .verify_token_ignore_expiry(old_token)
            .context("verifying old token")?;

        let uid = claims
            .sub
            .parse::<i64>()
            .context("parsing user ID from claims.sub")?;

        let old_jti = &claims.jti;

        db.check_refresh_token(refresh_token, old_jti).await?;

        if let Err(e) = db.delete_refresh_token(refresh_token).await {
            error!(error = ?e, "Failed to delete old refresh token");
        }

        let l = self.login_as(&LoginID::UserID(uid)).await?;
        Ok(l)
    }
}
