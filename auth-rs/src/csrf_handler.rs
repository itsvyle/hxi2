use axum::{
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use connectrpc::ConnectError;
use hmac::{Hmac, KeyInit, Mac};
use rand::{RngExt, distr::Alphanumeric};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::permissions_checking;

type HmacSha256 = Hmac<Sha256>;

pub static GLOBAL_CSRF_PROTECTION: once_cell::sync::Lazy<CsrfProtection> =
    once_cell::sync::Lazy::new(CsrfProtection::from_env);

#[derive(Clone)]
pub struct CsrfProtection {
    signing_key: Vec<u8>,
}

#[allow(dead_code)]
impl CsrfProtection {
    pub fn from_env() -> Self {
        let config_key = std::env::var("CONFIG_CSRF_SIGNING_KEY")
            .expect("CONFIG_CSRF_SIGNING_KEY environment variable not set");
        Self::new(&config_key)
    }

    pub fn new(config_key: &str) -> Self {
        let decoded_key = base64::engine::general_purpose::STANDARD
            .decode(config_key)
            .expect("Failed to decode CONFIG_CSRF_SIGNING_KEY from base64");

        Self {
            signing_key: decoded_key,
        }
    }

    fn sign_message(&self, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.signing_key).unwrap();
        mac.update(message.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }

    pub fn generate_token(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut rng = rand::rng();

        let random_str: String = (0..16).map(|_| rng.sample(Alphanumeric) as char).collect();

        let payload = format!("{}.{}", now, random_str);

        let signature = self.sign_message(&payload);

        format!("{}.{}", payload, signature)
    }

    pub async fn middleware(req: Request<axum::body::Body>, next: Next) -> Response {
        let csrf = &GLOBAL_CSRF_PROTECTION;

        let _err_not_found = || {
            ConnectError::not_found("route not found")
                .into_http_response(req.headers())
                .into_response()
        };

        let err_forbidden = |s: &str| {
            ConnectError::permission_denied(s)
                .into_http_response(req.headers())
                .into_response()
        };

        let route = match permissions_checking::get_route_from_public_url(req.uri().path()) {
            Some(r) => r,
            None => return next.run(req).await, // return _err_not_found(),
        };

        let perms = permissions_checking::get_by_route(route)
            .expect("impossible: route can't be non empty, and not be in the permissions list");

        if !perms.enforce_csrf {
            return next.run(req).await;
        }

        // 2. Extract Header Value
        let header_token = match req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
        {
            Some(t) => t,
            None => return err_forbidden("X-CSRF-Token header missing"),
        };

        // 3. Extract Cookie Value manually
        let cookie_token: String = match req
            .headers()
            .get("Cookie")
            .and_then(|c| c.to_str().ok())
            .and_then(|c_str| {
                c_str
                    .split(';')
                    .map(|s| s.trim())
                    .find(|s| s.starts_with("csrf_token="))
                    .map(|s| s["csrf_token=".len()..].to_string())
            }) {
            Some(t) => t,
            None => return err_forbidden("CSRF token cookie not found"),
        };

        // 4. Basic string match check
        if header_token != cookie_token {
            return err_forbidden("CSRF header and cookie tokens do not match");
        }

        // 5. Cryptographic signature check
        let parts: Vec<&str> = cookie_token.split('.').collect();
        if parts.len() != 3 {
            return err_forbidden("Invalid CSRF token format");
        }

        let payload = format!("{}.{}", parts[0], parts[1]);
        let signature = parts[2];

        if csrf.sign_message(&payload) != signature {
            return err_forbidden("Invalid CSRF token signature");
        }

        // 6. Expiration check
        let timestamp: u64 = match parts[0].parse() {
            Ok(t) => t,
            Err(_) => return err_forbidden("Invalid CSRF token timestamp"),
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - timestamp > 86400 {
            return err_forbidden("CSRF token expired");
        }

        next.run(req).await
    }
}
