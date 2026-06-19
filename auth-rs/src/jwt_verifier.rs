use std::sync::LazyLock;

use anyhow::{Context as _, Result};
use hxi2_proto::proto::auth::v2::JwtClaims;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

pub struct JWTVerifier {
    public_key: DecodingKey,
    validation_settings: Validation,
}

pub static GLOBAL_JWT_VERIFIER: LazyLock<JWTVerifier> =
    LazyLock::new(|| JWTVerifier::new_from_cfg().expect("failed to initialize JWTVerifier"));

impl JWTVerifier {
    pub fn new_from_cfg() -> Result<Self> {
        let cfg = crate::app_config::AppConfiguration::INSTANCE();

        let key = cfg.jwt_public_key();

        let public_key = DecodingKey::from_ed_pem(key.as_bytes()).context("decoding public key")?;

        let mut v = Validation::new(Algorithm::EdDSA);
        v.set_audience(&[format!("https://{}", cfg.tld)]);
        v.leeway = 1;

        Ok(JWTVerifier {
            public_key,
            validation_settings: v,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(token, &self.public_key, &self.validation_settings)
            .context("verifying token")?;
        Ok(token_data.claims)
    }
}
