use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;

use crate::{
    app_config::AppConfiguration,
    login_manager::{self, LoginManager},
};

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct DiscordLoginManager {
    oauth_client: BasicClient,
    login_manager: Arc<LoginManager>,
}

impl DiscordLoginManager {
    pub fn new(login_manager: Arc<LoginManager>) -> anyhow::Result<Self> {
        let cfg = AppConfiguration::INSTANCE();

        // Passed parameters directly into BasicClient::new
        let oauth_client = BasicClient::new(
            ClientId::new(cfg.discord_client_id.to_string()),
            Some(ClientSecret::new(cfg.discord_client_secret.to_string())),
            AuthUrl::new("https://discord.com/oauth2/authorize".to_string())?,
            Some(TokenUrl::new(
                "https://discord.com/api/oauth2/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(format!(
            "{}/api/discord_callback",
            cfg.auth_url
        ))?);

        Ok(Self {
            oauth_client,
            login_manager,
        })
    }

    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/api/login", get(Self::login_handler))
            .route("/api/discord_callback", get(Self::callback_handler))
            .with_state(state)
    }

    /// Handler for GET /api/login
    async fn login_handler(State(manager): State<Arc<Self>>, jar: CookieJar) -> impl IntoResponse {
        let (auth_url, csrf_token) = manager
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".to_string()))
            .url();

        let csrf_cookie = Cookie::build(("csrf_token", csrf_token.secret().to_string()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(time::Duration::minutes(10))
            .build();

        (jar.add(csrf_cookie), Redirect::to(auth_url.as_str()))
    }

    /// Handler for GET /api/discord_callback
    async fn callback_handler(
        State(manager): State<Arc<Self>>,
        jar: CookieJar,
        Query(query): Query<CallbackQuery>,
    ) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
        let stored_csrf = jar.get("csrf_token").map(|c| c.value().to_string());
        let jar = jar.remove(Cookie::from("csrf_token"));

        match stored_csrf {
            Some(token) if token == query.state => {}
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "CSRF token mismatch or state cookie expired.",
                ));
            }
        }

        let token_result = manager
            .oauth_client
            .exchange_code(AuthorizationCode::new(query.code))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to exchange code with Discord");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to exchange code with Discord.",
                )
            })?;

        let http_client = reqwest::Client::new();
        let discord_user: DiscordUser = http_client
            .get("https://discord.com/api/v10/users/@me")
            .bearer_auth(token_result.access_token().secret())
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed request to Discord API.");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed request to Discord API.",
                )
            })?
            .json()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to parse Discord response JSON.");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse Discord response JSON.",
                )
            })?;

        Ok((
            jar,
            format!(
                "Successfully logged in as: {} (ID: {})",
                discord_user.username, discord_user.id
            ),
        ))
    }
}
