use std::sync::LazyLock;

use hxi2_proto::proto::auth::v2::JwtClaims;
use jsonwebtoken::{DecodingKey, Validation, decode};
use tracing::instrument;

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
    pub fn verify_token(
        &self,
        token: &str,
    ) -> std::result::Result<JwtClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<JwtClaims>(token, &self.public_key, &self.validation_settings)?;
        Ok(token_data.claims)
    }

    #[instrument(skip(self), err)]
    pub fn verify_token_ignore_expiry(
        &self,
        token: &str,
    ) -> std::result::Result<JwtClaims, jsonwebtoken::errors::Error> {
        let mut validation = self.validation_settings.clone();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        let token_data = decode::<JwtClaims>(token, &self.public_key, &validation)?;
        Ok(token_data.claims)
    }
}
