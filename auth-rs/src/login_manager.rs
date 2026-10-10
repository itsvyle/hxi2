use std::any::type_name;
use std::sync::Arc;

use anyhow::Context as _;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use base64::prelude::*;
use buffa::MessageField;
use buffa_types::Timestamp;
use http::StatusCode;
use hxi2_proto::proto::auth::v2::{Attribute, Permission, SmallData};
use rand::RngExt;
use tracing::{error, instrument, trace};

use crate::{
    app_config::AppConfiguration,
    database::{DbError, DbUser, DbUserIdentifier},
    jwt_signer::JWTSignerOptions,
};

pub struct LoginManager {
    pub signer: &'static crate::jwt_signer::JWTSigner,
    #[allow(unused)]
    pub verifier: &'static crate::jwt_verifier::JWTVerifier,
}

#[derive(Debug, Clone)]
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

        // JWT Cookie: must be alive at least as long as refresh token, to provide adequate refreshing
        let mut jwt_cookie = Cookie::build((&cfg.COOKIE_JWT_TOKEN_NAME, self.token.to_owned()))
            .max_age(time::Duration::seconds(self.refresh_token_max_age))
            .build();
        LoginManager::apply_secure_cookie_options(&mut jwt_cookie);

        // Refresh Token Cookie
        let mut refresh_token_cookie = Cookie::build((
            &cfg.COOKIE_REFRESH_TOKEN_NAME,
            self.refresh_token.to_owned(),
        ))
        .max_age(time::Duration::seconds(self.refresh_token_max_age))
        .build();
        LoginManager::apply_secure_cookie_options(&mut refresh_token_cookie);

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

        let mut small_data_cookie = Cookie::build((&cfg.COOKIE_SMALL_DATA_NAME, small_data_b64))
            .max_age(time::Duration::seconds(self.refresh_token_max_age))
            .build();
        LoginManager::apply_secure_cookie_options(&mut small_data_cookie);
        small_data_cookie.set_http_only(false);

        Ok(jar
            .add(jwt_cookie)
            .add(refresh_token_cookie)
            .add(small_data_cookie))
    }
}

impl LoginManager {
    pub fn new(
        signer: &'static crate::jwt_signer::JWTSigner,
        verifier: &'static crate::jwt_verifier::JWTVerifier,
    ) -> Self {
        Self { signer, verifier }
    }

    /// To move somewhere else, to be used more globally
    #[instrument(level = "trace", fields(enum_type = %type_name::<T>()))]
    pub fn retrieve_active_bitfields<T: buffa::Enumeration>(
        bitfield: i64,
    ) -> impl Iterator<Item = T> {
        let mut copy = bitfield;

        std::iter::from_fn(move || {
            while copy != 0 {
                let index = copy.trailing_zeros();
                copy &= copy - 1;

                if let Some(role) = T::from_i32(index as i32) {
                    return Some(role);
                } else {
                    trace!("Unknown index {} in bitfield {}", index, bitfield);
                }
            }
            None
        })
    }

    fn small_data_from_user(&self, user: &DbUser) -> SmallData {
        SmallData {
            user_id: user.id,
            username: user.username.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            permissions: user.permissions,
            promotion: user.promotion,
            expiration: MessageField::none(),
            roles: Self::retrieve_active_bitfields::<Permission>(user.permissions)
                .map(Into::into)
                .collect(),
            is_api_token: user.is_api,
            attributes: user.attributes,
            attribute_list: Self::retrieve_active_bitfields::<Attribute>(user.attributes)
                .map(Into::into)
                .collect(),
            __buffa_unknown_fields: Default::default(),
        }
    }

    fn generate_refresh_token() -> String {
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 16] = rng.random();
        hex::encode(jti)
    }

    pub fn apply_secure_cookie_options(cookie: &mut Cookie) {
        let cfg = AppConfiguration::INSTANCE();
        cookie.set_domain(&cfg.cookies_domain);
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_secure(match cfg.environment {
            crate::app_config::Environnement::Development => false,
            crate::app_config::Environnement::Production => true,
        });
    }

    // This creates a token for the user
    // Authorization to login as such a user must have been checked PRIOR
    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn login_as(&self, identifier: &DbUserIdentifier) -> anyhow::Result<LoginResponse> {
        let cfg = crate::app_config::AppConfiguration::INSTANCE();
        let user = cfg
            .db()
            .await
            .get_db_user(identifier)
            .await
            .context("getting user from database")?;

        let mut small_data = self.small_data_from_user(&user);
        small_data.expiration =
            Timestamp::from_unix_secs((chrono::Utc::now() + cfg.JWT_TOKEN_VALIDITY).timestamp())
                .into();

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

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
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

        let l = self.login_as(&DbUserIdentifier::Id(uid)).await?;
        Ok(l)
    }

    #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    pub async fn logout(&self, refresh_token: &str) -> anyhow::Result<()> {
        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        if let Err(e) = db.delete_refresh_token(refresh_token).await {
            if matches!(e, DbError::NotFound) {
                trace!("Refresh token not found during logout, ignoring.");
            } else {
                error!(error = ?e, "Failed to delete refresh token during logout");
                return Err(anyhow::anyhow!(
                    "Failed to delete refresh token during logout"
                ));
            }
        }
        Ok(())
    }

    pub fn router(self: &Arc<Self>) -> axum::Router {
        let state = Arc::clone(self);
        axum::Router::new()
            .route("/logout", axum::routing::get(Self::logout_route))
            .with_state(state)
    }

    pub async fn logout_route(
        State(manager): State<Arc<Self>>,
        jar: CookieJar,
    ) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
        let cfg = AppConfiguration::INSTANCE();
        let refresh_token_cookie = jar
            .get(&cfg.COOKIE_REFRESH_TOKEN_NAME)
            .ok_or((StatusCode::BAD_REQUEST, "No refresh token cookie found"))?;
        let refresh_token = refresh_token_cookie.value();

        if let Err(e) = manager.logout(refresh_token).await {
            error!(error = ?e, "Failed to logout user");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to logout user"));
        }

        // Remove cookies
        let mut jwt_cookie = Cookie::build((&cfg.COOKIE_JWT_TOKEN_NAME, ""))
            .max_age(time::Duration::seconds(0))
            .build();
        Self::apply_secure_cookie_options(&mut jwt_cookie);

        let mut refresh_token_cookie = Cookie::build((&cfg.COOKIE_REFRESH_TOKEN_NAME, ""))
            .max_age(time::Duration::seconds(0))
            .build();
        Self::apply_secure_cookie_options(&mut refresh_token_cookie);

        let mut small_data_cookie = Cookie::build((&cfg.COOKIE_SMALL_DATA_NAME, ""))
            .max_age(time::Duration::seconds(0))
            .build();
        Self::apply_secure_cookie_options(&mut small_data_cookie);

        // redirect to the default redirect URL after logout
        let redirect_url = cfg.default_redirect_url.clone();

        Ok((
            jar.remove(jwt_cookie)
                .remove(refresh_token_cookie)
                .remove(small_data_cookie),
            Redirect::to(&redirect_url),
        ))
    }
}
