use anyhow::{Context, Result};
use chrono::Duration;
use derivative::Derivative;
use ed25519_dalek::pkcs8::EncodePublicKey;
use once_cell::sync::Lazy;
use sqlx::AssertSqlSafe;
use tokio::sync::OnceCell;

use crate::{database::DatabaseManager, jwt_verifier::JWTVerifier};

pub use config_parsing::{AuthConfig, Environnement};

fn redact_string_fmt(s: &str, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    fn redact_string(s: &str) -> String {
        // if the length is above 10, show the first 5 characters, and the last 2, and replace the rest with "..."; otherwise return ...
        if s.len() > 10 {
            format!("{}...{}", &s[..5], &s[s.len() - 2..])
        } else {
            s.to_string()
        }
    }
    write!(f, "{}", redact_string(s))
}

#[allow(unused, non_snake_case)]
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct AppConfiguration {
    #[doc = "Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in)"]
    pub auth_url: String,
    #[doc = "Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url"]
    pub auth_endpoint: String,
    #[doc = "Domain of the global cookies, most importantly token/refreshtoken/smalldata"]
    pub cookies_domain: String,
    #[doc = "Domain name"]
    pub tld: String,
    #[doc = "Default redirect url for the login page after the user has logged in, or logged out."]
    pub default_redirect_url: String,
    #[doc = "Port on which the server will run"]
    pub running_port: u16,
    #[doc = "Private key for the JWT token"]
    pub jwt_private_key: String,
    #[doc = "Private key for the CSRF token signing"]
    pub csrf_private_key: String,
    #[doc = "Path to the sqlite database file"]
    pub db_path: String,
    #[doc = "Discord application id"]
    pub discord_application_id: String,
    #[doc = "Discord client id"]
    #[derivative(Debug(format_with = "redact_string_fmt"))]
    pub discord_client_id: String,
    #[doc = "Discord client secret"]
    #[derivative(Debug(format_with = "redact_string_fmt"))]
    pub discord_client_secret: String,
    #[doc = "Environment (development or production)"]
    #[derivative(Default(value = "Environnement::Production"))]
    pub environment: Environnement,

    #[doc = "Name of the cookie for the JWT token"]
    pub COOKIE_JWT_TOKEN_NAME: String,
    #[doc = "Name of the cookie for the refresh token"]
    pub COOKIE_REFRESH_TOKEN_NAME: String,
    #[doc = "Name of the cookie for the small data"]
    pub COOKIE_SMALL_DATA_NAME: String,
    #[doc = "Validity of the refresh token"]
    pub JWT_REFRESH_TOKEN_VALIDITY: Duration,
    #[doc = "Validity of the JWT token"]
    pub JWT_TOKEN_VALIDITY: Duration,
    #[doc = "Validity of a API token"]
    pub API_TOKEN_VALIDITY: Duration,

    #[derivative(Debug = "ignore")]
    db_manager: OnceCell<DatabaseManager>,
}

impl AppConfiguration {
    fn config_path() -> String {
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "devconfig.json".to_string()
            } else {
                "config.json".to_string()
            }
        })
    }

    pub fn from_env() -> Result<Self> {
        let config_path = Self::config_path();
        let raw_config = std::fs::read_to_string(&config_path).context(format!(
            "Failed to read config file at {}. Please make sure the file exists and is readable.",
            config_path
        ))?;
        let config: AuthConfig = serde_json::from_str(&raw_config).context(format!(
            "Failed to parse config file at {}. Please make sure the file is valid JSON.",
            config_path
        ))?;

        if config.environment == Environnement::Production && config_path.contains("devconfig.json")
        {
            panic!(
                "You are running in development mode with the default devconfig.json.
                Please create a copy of devconfig.json, rename it to config.json, and set the environment variable CONFIG_PATH to point to your new config file.
                This is to prevent accidentally running in development mode in production."
            );
        }

        Ok(Self {
            auth_url: config.auth_url,
            auth_endpoint: config.auth_endpoint,
            cookies_domain: config.cookie_domain,
            tld: config.tld,
            default_redirect_url: config.default_redirect_url,
            running_port: config.running_port as u16,
            jwt_private_key: config.jwt_private_key,
            db_path: config.db_path,
            discord_application_id: config.discord_application_id,
            discord_client_id: config.discord_client_id,
            discord_client_secret: config.discord_client_secret,
            environment: config.environment,
            db_manager: OnceCell::new(),
            csrf_private_key: config.csrf_private_key,
            COOKIE_JWT_TOKEN_NAME: config.cookies.jwt_token_cookie_name,
            COOKIE_REFRESH_TOKEN_NAME: config.cookies.refresh_token_cookie_name,
            COOKIE_SMALL_DATA_NAME: config.cookies.small_data_cookie_name,
            JWT_REFRESH_TOKEN_VALIDITY: Duration::seconds(
                config.cookies.refresh_token_validity_seconds,
            ),
            JWT_TOKEN_VALIDITY: Duration::seconds(config.cookies.jwt_token_validity_seconds),
            API_TOKEN_VALIDITY: Duration::weeks(520), // 10 years
        })
    }

    fn jwt_public_key_(&self) -> Result<String> {
        // let key_pair = Ed25519KeyPair

        use ed25519_dalek::SigningKey;
        use ed25519_dalek::pkcs8::DecodePrivateKey;

        let signing_key = SigningKey::from_pkcs8_pem(&self.jwt_private_key)
            .map_err(|e| anyhow::anyhow!("Invalid PKCS8 PEM private key: {}", e))?;

        let verifying_key = signing_key
            .verifying_key()
            .to_public_key_pem(ed25519_dalek::pkcs8::spki::der::pem::LineEnding::LF)
            .context("encode public key to PEM")?;
        Ok(verifying_key)
    }

    pub fn jwt_public_key(&self) -> &'static str {
        static JWT_PUBLIC_KEY: Lazy<String> = Lazy::new(|| {
            let cfg = AppConfiguration::INSTANCE();
            cfg.jwt_public_key_().expect("Failed to get JWT public key")
        });
        &JWT_PUBLIC_KEY
    }

    pub async fn db(&self) -> &DatabaseManager {
        if !std::path::Path::new(&self.db_path).exists() {
            std::fs::File::create(&self.db_path).expect("Failed to create database file");
        }

        self.db_manager
            .get_or_init(|| async {
                let pool = sqlx::SqlitePool::connect(&self.db_path)
                    .await
                    .expect("Failed to connect to SQLite database");

                let schema = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/schema.sql"));

                sqlx::query(AssertSqlSafe(schema))
                    .execute(&pool)
                    .await
                    .context("creating database schema")
                    .expect("Failed to create database schema");

                let manager = DatabaseManager::new(pool);

                manager.start_one_time_code_cleanup_timer();

                manager
            })
            .await
    }

    #[allow(non_snake_case)]
    pub fn INSTANCE() -> &'static Self {
        static INSTANCE: Lazy<AppConfiguration> = Lazy::new(|| {
            let e = AppConfiguration::from_env()
                .expect("Failed to load configuration from environment");
            if let Environnement::Development = e.environment {
                println!(
                    "=========================================\n WARNING: Running in development mode. This is not secure and should not be used in production.\n========================================="
                );
            }
            e
        });
        &INSTANCE
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environnement::Development)
    }

    pub fn new_jwt_verifier(&self) -> Result<JWTVerifier> {
        use jsonwebtoken::{Algorithm, DecodingKey, Validation};
        let key = self.jwt_public_key();

        let public_key = DecodingKey::from_ed_pem(key.as_bytes()).context("decoding public key")?;

        let mut v = Validation::new(Algorithm::EdDSA);
        v.set_audience(&[format!("https://{}", self.tld)]);
        v.leeway = 1;

        Ok(JWTVerifier {
            public_key,
            validation_settings: v,
        })
    }
}

mod config_parsing {
    use serde::Deserializer;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::fs;
    use tracing::error;

    use anyhow::Result;

    use std::str::FromStr;

    /// A configuration input that can be supplied either as a direct numeric value (`8080`)
    /// or as a string reference (`"env:PORT"` / `"file:/etc/secrets/port"`).
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum StringOrNumber<T> {
        Number(T),
        String(String),
    }

    impl<T> StringOrNumber<T>
    where
        T: FromStr + Copy,
        T::Err: std::fmt::Display,
    {
        /// Resolves the underlying value to `T`.
        ///
        /// - If direct number: returns value.
        /// - If string starting with `env:`: fetches environment variable and parses as `T`.
        /// - If string starting with `file:`: reads file content and parses as `T`.
        /// - Otherwise: attempts to parse raw string into `T`.
        pub fn resolve(&self) -> Result<T, String> {
            match self {
                Self::Number(val) => Ok(*val),
                Self::String(raw) => {
                    let resolved_str = resolve_value(raw)?;
                    resolved_str.trim().parse::<T>().map_err(|err| {
                        format!(
                            "Failed to parse '{}' as number: {}",
                            resolved_str.trim(),
                            err
                        )
                    })
                }
            }
        }
    }

    #[cfg(feature = "schema-gen")]
    impl<T: schemars::JsonSchema> schemars::JsonSchema for StringOrNumber<T> {
        fn schema_name() -> String {
            format!("StringOrNumber_{}", T::schema_name())
        }

        fn json_schema(
            generator: &mut schemars::r#gen::SchemaGenerator,
        ) -> schemars::schema::Schema {
            let number_schema = generator.subschema_for::<T>();
            let string_schema = generator.subschema_for::<String>();

            let mut schema_obj = schemars::schema::SchemaObject::default();
            schema_obj.metadata().description = Some(
                "Can be specified either directly as a number (e.g. 8080) or as a string reference ('env:PORT:<optional default_value>', 'file:/path:<optional default_value>')".to_string()
            );
            schema_obj.subschemas().any_of = Some(vec![number_schema, string_schema]);
            schemars::schema::Schema::Object(schema_obj)
        }
    }

    /// The environment the application is running in
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    #[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
    pub enum Environnement {
        #[default]
        Production = 1,
        Development = 2,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
    #[serde(deny_unknown_fields)]
    /// Represents the configuration for the authentication service.
    /// You can use `env:VAR_NAME:<optional default_value>` to load a value from an environment variable, or `file:PATH:<optional default_value>` to load a value from a file.
    pub struct AuthConfig {
        /// Path or URL to the JSON Schema for validation and auto-completion
        #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
        pub schema: Option<String>,

        /// Domain name and protocol of the **internet facing** auth domain (used for redirecting in case the user isn't logged in, to log in)
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub auth_url: String,

        /// Endpoint to call internally to renew tokens, or control other authentication stuff; it can be a local url
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub auth_endpoint: String,

        /// Domain of global cookies, most importantly token/refreshtoken/smalldata
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub cookie_domain: String,

        /// Domain name
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub tld: String,

        /// Default redirect url after login/logout
        #[serde(
            default = "default_redirect_url",
            deserialize_with = "deserialize_optional_resolved_string"
        )]
        pub default_redirect_url: String,

        /// Port to run the auth service on
        #[serde(
            default = "default_running_port",
            deserialize_with = "deserialize_positive_port"
        )]
        #[cfg_attr(feature = "schema-gen", schemars(with = "StringOrNumber<i32>"))]
        pub running_port: i32,

        /// Path to the database file
        #[serde(
            default = "default_db_path",
            deserialize_with = "deserialize_optional_resolved_string"
        )]
        pub db_path: String,

        /// Your private key to sign JWTs with. It should be a PEM encoded Ed25519 private key.
        /// You can generate one with: openssl genpkey -algorithm Ed25519 -out jwt_private_key.pem
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub jwt_private_key: String,

        /// Your private key to sign CSRF tokens with.
        /// You can generate one with: openssl rand -base64 32
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub csrf_private_key: String,

        /// Discord application id
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub discord_application_id: String,

        /// Discord Client ID for OAuth2
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub discord_client_id: String,

        /// Discord Client Secret for OAuth2
        #[serde(deserialize_with = "deserialize_required_resolved_string")]
        pub discord_client_secret: String,

        /// Running environment
        #[serde(default)]
        pub environment: Environnement,

        /// Optional nested cookie configuration
        #[serde(default)]
        pub cookies: CookiesConfig,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
    pub struct CookiesConfig {
        /// The name of the cookie that will hold the JWT token
        #[serde(
            default = "default_jwt_cookie_name",
            deserialize_with = "deserialize_optional_resolved_string"
        )]
        pub jwt_token_cookie_name: String,

        /// The name of the cookie that will hold the refresh token
        #[serde(
            default = "default_refresh_cookie_name",
            deserialize_with = "deserialize_optional_resolved_string"
        )]
        pub refresh_token_cookie_name: String,

        /// The name of the cookie that will hold the small data
        #[serde(
            default = "default_small_data_cookie_name",
            deserialize_with = "deserialize_optional_resolved_string"
        )]
        pub small_data_cookie_name: String,

        /// The validity of the refresh token, in seconds
        #[serde(
            default = "default_refresh_validity",
            deserialize_with = "deserialize_positive_i64"
        )]
        #[cfg_attr(feature = "schema-gen", schemars(with = "StringOrNumber<i32>"))]
        pub refresh_token_validity_seconds: i64,

        /// The validity of the JWT token, in seconds
        #[serde(
            default = "default_jwt_validity",
            deserialize_with = "deserialize_positive_i64"
        )]
        #[cfg_attr(feature = "schema-gen", schemars(with = "StringOrNumber<i32>"))]
        pub jwt_token_validity_seconds: i64,
    }

    pub fn default_redirect_url() -> String {
        "/".to_string()
    }

    pub fn default_running_port() -> i32 {
        8080
    }

    pub fn default_db_path() -> String {
        "./auth.db".to_string()
    }

    pub fn default_jwt_cookie_name() -> String {
        "HXI2_TOKEN".to_string()
    }

    pub fn default_refresh_cookie_name() -> String {
        "HXI2_REFRESH_TOKEN".to_string()
    }

    pub fn default_small_data_cookie_name() -> String {
        "HXI2_SMALL_DATA".to_string()
    }

    pub fn default_refresh_validity() -> i64 {
        2_592_000
    }

    // 30 days
    pub fn default_jwt_validity() -> i64 {
        900
    }

    impl Default for CookiesConfig {
        fn default() -> Self {
            Self {
                jwt_token_cookie_name: default_jwt_cookie_name(),
                refresh_token_cookie_name: default_refresh_cookie_name(),
                small_data_cookie_name: default_small_data_cookie_name(),
                refresh_token_validity_seconds: default_refresh_validity(),
                jwt_token_validity_seconds: default_jwt_validity(),
            }
        }
    }

    /// Resolves `env:VAR_NAME:?<default_value>` or `file:PATH:?<default_value>` prefixes.
    /// Returns raw string if no prefix matches.
    pub fn resolve_value(raw: &str) -> Result<String, String> {
        let trimmed = raw.trim();
        if let Some(var_name) = trimmed.strip_prefix("env:") {
            let parts: Vec<&str> = var_name.split(':').collect();
            if parts.len() < 2 {
                return env::var(parts[0])
                    .map_err(|_| format!("Environment variable '{}' is not set", var_name));
            }
            let var_name = parts[0];
            let default_value = parts[1];
            env::var(var_name)
                .map_err(|_| format!("Environment variable '{}' is not set", var_name))
                .or_else(|_| Ok(default_value.into()))
        } else if let Some(file_path) = trimmed.strip_prefix("file:") {
            let parts: Vec<&str> = file_path.split(':').collect();
            let fcontent = fs::read_to_string(parts[0]).map(|content| content.trim().to_string());
            if parts.len() < 2 {
                fcontent.map_err(|_| format!("File '{}' is not readable", file_path))
            } else {
                let default_value = parts[1];
                fcontent
                    .map_err(|e| {
                        if let Ok(true) = fs::exists(parts[0]) {
                            error!(
                                "Reading configuration: File '{}' exits but is not readable: {}",
                                parts[0], e
                            )
                        };
                        e
                    })
                    .or_else(|_| Ok(default_value.into()))
            }
        } else {
            Ok(trimmed.to_string())
        }
    }

    /// Resolves string and ensures non-empty output for required fields
    pub fn deserialize_required_resolved_string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let resolved = resolve_value(&raw).map_err(serde::de::Error::custom)?;

        if resolved.is_empty() {
            return Err(serde::de::Error::custom(
                "field is required and resolved value cannot be empty",
            ));
        }

        Ok(resolved)
    }

    /// Resolves optional string fields (if provided)
    pub fn deserialize_optional_resolved_string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        resolve_value(&raw).map_err(serde::de::Error::custom)
    }

    /// Deserializes a `StringOrNumber<i32>`, resolves env/file, and validates 1 <= port <= 65535
    pub fn deserialize_positive_port<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = StringOrNumber::<i32>::deserialize(deserializer)?;
        let resolved_port = input.resolve().map_err(serde::de::Error::custom)?;

        if resolved_port <= 0 || resolved_port > 65_535 {
            return Err(serde::de::Error::custom(format!(
                "invalid port number '{}': must be between 1 and 65535",
                resolved_port
            )));
        }

        Ok(resolved_port)
    }

    /// Deserializes a `StringOrNumber<i64>`, resolves env/file, and validates > 0
    pub fn deserialize_positive_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = StringOrNumber::<i64>::deserialize(deserializer)?;
        let resolved = input.resolve().map_err(serde::de::Error::custom)?;

        if resolved <= 0 {
            return Err(serde::de::Error::custom(
                "duration seconds must be greater than 0",
            ));
        }

        Ok(resolved)
    }

    #[cfg(all(test, feature = "schema-gen"))]
    mod schema_gen_test {
        use super::*;
        use schemars::schema_for;
        use serde_json::Serializer;
        use serde_json::ser::PrettyFormatter;
        use std::fs;

        #[test]
        fn generate_json_schema() -> anyhow::Result<()> {
            let schema = schema_for!(AuthConfig);

            // Serialize with 4 spaces
            let mut buf = Vec::new();
            let formatter = PrettyFormatter::with_indent(b"    ");
            let mut serializer = Serializer::with_formatter(&mut buf, formatter);
            schema.serialize(&mut serializer)?;

            let mut schema_json = String::from_utf8(buf)?;
            schema_json.push('\n'); // Add a newline at the end for better formatting

            // Write to file
            fs::write("auth_config.schema.json", schema_json)?;
            println!(
                "\n✅ Successfully generated auth_config.schema.json with 4-space indentation!"
            );

            Ok(())
        }
    }
}
