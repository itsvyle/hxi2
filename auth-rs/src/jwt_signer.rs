use std::time;

use anyhow::{Context as _, Result};
use hxi2_proto::proto::auth::v2::{JwtClaims, SmallData};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngExt;

use crate::app_config::AppConfiguration;

pub struct JWTSigner {
    header: Header,
    encoding_key: EncodingKey,
}

impl JWTSigner {
    pub fn new() -> Result<Self> {
        let cfg = AppConfiguration::INSTANCE();
        Ok(JWTSigner {
            header: Header::new(Algorithm::EdDSA),
            encoding_key: EncodingKey::from_ed_pem(cfg.jwt_private_key.as_bytes())
                .context("encoding private key")?,
        })
    }

    pub fn new_token(
        &self,
        validity: time::Duration,
        sub: &str,
        data: SmallData,
    ) -> Result<(String, JwtClaims)> {
        let cfg = AppConfiguration::INSTANCE();

        // calculate iat, exp, and nbf based on current time and validity
        let now = chrono::Utc::now();
        let iat = now.timestamp();
        let exp = (now + chrono::Duration::from_std(validity)?).timestamp();
        let nbf = iat;

        let claims = JwtClaims {
            iss: format!("https://auth.{}", cfg.tld),
            sub: sub.to_string(),
            aud: format!("https://{}", cfg.tld),
            exp,
            nbf,
            iat,
            jti: self.generate_jti(),
            temporary: false,
            temporary_recheck_after: None,
            data: data.into(),
            ..Default::default()
        };

        let token = encode(&self.header, &claims, &self.encoding_key).context("encoding token")?;
        Ok((token, claims))
    }

    fn generate_jti(&self) -> String {
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 16] = rng.random();
        hex::encode(jti)
    }
}
