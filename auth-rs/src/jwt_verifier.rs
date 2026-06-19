use std::sync::LazyLock;

use anyhow::{Context as _, Result};
use hxi2_proto::proto::auth::v2::JwtClaims;
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::app_config::AppConfiguration;

pub struct JWTVerifier {
    pub public_key: DecodingKey,
    pub validation_settings: Validation,
}

pub static GLOBAL_JWT_VERIFIER: LazyLock<JWTVerifier> = LazyLock::new(|| {
    AppConfiguration::INSTANCE()
        .new_jwt_verifier()
        .expect("failed to initialize JWTVerifier")
});

impl JWTVerifier {
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(token, &self.public_key, &self.validation_settings)
            .context("verifying token")?;
        Ok(token_data.claims)
    }
}
