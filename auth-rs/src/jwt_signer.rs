use std::time;

use anyhow::{Context as _, Result};
use hxi2_proto::proto::auth::v2::{JwtClaims, SmallData};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rand::RngExt;

use crate::app_config::AppConfiguration;

pub struct JWTSigner {
    header: Header,
    encoding_key: EncodingKey,
}

pub struct JWTSignerOptions {
    pub validity: time::Duration,
}
impl Default for JWTSignerOptions {
    fn default() -> Self {
        JWTSignerOptions {
            validity: time::Duration::from_secs(15 * 60),
        }
    }
}

impl JWTSignerOptions {
    pub fn with_validity(mut self, validity: time::Duration) -> Self {
        self.validity = validity;
        self
    }
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
        sub: &str,
        data: SmallData,
        options: &JWTSignerOptions,
    ) -> Result<(String, JwtClaims)> {
        let cfg = AppConfiguration::INSTANCE();

        // calculate iat, exp, and nbf based on current time and validity
        let now = chrono::Utc::now();
        let iat = now.timestamp();
        let exp = (now + chrono::Duration::from_std(options.validity)?).timestamp();
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
        let claims_json = self
            .claims_to_json(&claims)
            .context("converting claims to JSON")?;

        let token =
            encode(&self.header, &claims_json, &self.encoding_key).context("encoding token")?;
        Ok((token, claims))
    }

    fn generate_jti(&self) -> String {
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 16] = rng.random();
        hex::encode(jti)
    }

    // needs to put the exp as a number, and not a string with a number in it
    pub fn claims_to_json(&self, claims: &JwtClaims) -> Result<serde_json::Value> {
        use serde_json::Value;

        let mut val =
            serde_json::to_value(claims).context("converting claims to serde JSON values")?;

        // Intercept and convert stringified numbers back to true JSON numbers
        // if let Some(obj) = val.as_object_mut() {
        //     for field in ["exp", "nbf", "iat"] {
        //         if let Some(Value::String(s)) = obj.get(field)
        //             && let Ok(parsed_num) = s.parse::<i64>()
        //         {
        //             obj.insert(field.to_string(), Value::Number(parsed_num.into()));
        //         }
        //     }
        // }

        Ok(val)
    }
}
