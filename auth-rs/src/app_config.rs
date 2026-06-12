use anyhow::Result;
use once_cell::sync::Lazy;
use utils::cfg_from_env_or;

#[allow(unused)]
pub struct AppConfiguration {
    #[doc = "Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in)"]
    pub auth_url: String,
    #[doc = "Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url"]
    pub auth_endpoint: String,
    #[doc = "Domain of the global cookies, most importantly token/refreshtoken/smalldata"]
    pub cookies_domain: String,
    #[doc = "Domain name"]
    pub tld: String,
    #[doc = "Default redirect url for the login page after the user has logged in"]
    pub default_redirect_url: String,
    #[doc = "Port on which the server will run"]
    pub running_port: u16,
    #[doc = "Private key for the JWT token"]
    pub jwt_private_key: String,
    #[doc = "Path to the sqlite database file"]
    pub db_path: String,
    #[doc = "Discord application id"]
    pub discord_application_id: String,
    #[doc = "Discord client id"]
    pub discord_client_id: String,
    #[doc = "Discord client secret"]
    pub discord_client_secret: String,
}

impl AppConfiguration {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            auth_url: cfg_from_env_or("HXI2_AUTH_URL", None)?,
            auth_endpoint: cfg_from_env_or("HXI2_AUTH_ENDPOINT", None)?,
            cookies_domain: cfg_from_env_or("HXI2_COOKIES_DOMAIN", None)?,
            tld: cfg_from_env_or("HXI2_TLD", None)?,
            default_redirect_url: cfg_from_env_or(
                "CONFIG_DEFAULT_REDIRECT_URL",
                Some("/".to_string()),
            )?,
            running_port: cfg_from_env_or("CONFIG_RUNNING_PORT", Some(8080))?,
            jwt_private_key: cfg_from_env_or("CONFIG_JWT_PRIVATE_KEY", None)?,
            db_path: cfg_from_env_or("CONFIG_DB_PATH", Some("./auth.db".to_string()))?,
            discord_application_id: cfg_from_env_or("CONFIG_DISCORD_APPLICATION_ID", None)?,
            discord_client_id: cfg_from_env_or("CONFIG_DISCORD_CLIENT_ID", None)?,
            discord_client_secret: cfg_from_env_or("CONFIG_DISCORD_CLIENT_SECRET", None)?,
        })
    }

    fn jwt_public_key_(&self) -> Result<String> {
        use ed25519_dalek::SigningKey;
        use ed25519_dalek::pkcs8::DecodePrivateKey;

        let signing_key = SigningKey::from_pkcs8_pem(&self.jwt_private_key)
            .map_err(|e| anyhow::anyhow!("Invalid PKCS8 PEM private key: {}", e))?;

        let verifying_key = signing_key.verifying_key();

        let public_key_hex = hex::encode(verifying_key.to_bytes());

        Ok(public_key_hex)
    }

    pub fn jwt_public_key(&self) -> &'static str {
        static JWT_PUBLIC_KEY: Lazy<String> = Lazy::new(|| {
            let cfg = AppConfiguration::INSTANCE();
            cfg.jwt_public_key_().expect("Failed to get JWT public key")
        });
        &JWT_PUBLIC_KEY
    }

    #[allow(non_snake_case)]
    pub fn INSTANCE() -> &'static Self {
        static INSTANCE: Lazy<AppConfiguration> = Lazy::new(|| {
            AppConfiguration::from_env().expect("Failed to load configuration from environment")
        });
        &INSTANCE
    }
}
