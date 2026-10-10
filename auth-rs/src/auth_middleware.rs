use std::sync::Arc;

use anyhow::Result;
use axum_extra::extract::cookie::Cookie;
use connectrpc::ConnectError;
use http::header::ACCEPT;
use hxi2_proto::proto::auth::v2::JwtClaims;

use axum::extract::{FromRef, FromRequestParts};
use axum::response::{IntoResponse, Response};

use crate::api_login;
use crate::permissions_checking::{self, MethodPermissionsOptionExt};

#[derive(Debug, Clone)]
pub struct ReqAuthState {
    pub claims: Option<JwtClaims>,
}

fn extract_auth_cookies(headers: &http::HeaderMap) -> (Option<String>, Option<String>) {
    let cookies = headers
        .get_all(http::header::COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());
    let mut jwt_token = None;
    let mut refresh_token = None;

    for cookie in cookies {
        match cookie.name() {
            name if name
                == crate::app_config::AppConfiguration::INSTANCE().COOKIE_JWT_TOKEN_NAME =>
            {
                jwt_token = Some(cookie.value().to_string());
            }
            name if name
                == crate::app_config::AppConfiguration::INSTANCE().COOKIE_REFRESH_TOKEN_NAME =>
            {
                refresh_token = Some(cookie.value().to_string());
            }
            _ => {}
        }
    }

    (jwt_token, refresh_token)
}

fn to_http_error(
    e: ConnectError,
    headers: &http::HeaderMap,
    perms: Option<&permissions_checking::MethodPermissions>,
) -> Response {
    let is_front = if let Some(perms) = perms {
        perms.is_frontend
    } else {
        headers
            .get(ACCEPT)
            .is_some_and(|v| v.to_str().map(|s| s.contains("text/html")).unwrap_or(false))
    };
    if is_front {
        let status_code = e.http_status();
        let body = format!(
            "<html><body><h1>{}</h1><p>{}</p></body></html>",
            status_code,
            e.message.unwrap_or("An error occurred".to_owned())
        );
        let mut response = Response::builder()
            .status(status_code)
            .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(body.into())
            .unwrap();
        response.headers_mut().extend(headers.clone());
        response
    } else {
        e.into_http_response(headers).into_response()
    }
}

pub struct RequireAuthMiddleware;

impl<S> FromRequestParts<S> for RequireAuthMiddleware
where
    S: Send + Sync,
    Arc<api_login::APILoginManager>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let route =
            permissions_checking::get_route_from_public_url(parts.uri.path()).ok_or_else(|| {
                to_http_error(
                    ConnectError::not_found("route not found"),
                    &parts.headers,
                    None,
                )
            })?;
        let perms = permissions_checking::get_by_route(route)
            .expect("impossible: route can't be non empty, and not be in the permissions list");
        let is_public = perms.is_public;

        let token = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(String::from)
            .or_else(|| extract_auth_cookies(&parts.headers).0);

        let claims = if let Some(t) = token {
            match crate::jwt_verifier::GLOBAL_JWT_VERIFIER.verify_token(&t) {
                Ok(c) => Some(c),
                Err(e) => {
                    if !is_public {
                        return Err(to_http_error(
                            ConnectError::unauthenticated(format!("invalid token: {e}")),
                            &parts.headers,
                            Some(perms),
                        ));
                    }
                    None
                }
            }
        } else if !is_public {
            return Err(to_http_error(
                ConnectError::unauthenticated("missing Bearer token"),
                &parts.headers,
                Some(perms),
            ));
        } else {
            None
        };

        if !is_public && let Some(ref c) = claims {
            if !perms.check_permissions(c.data.permissions) {
                return Err(to_http_error(
                    ConnectError::permission_denied("insufficient permissions"),
                    &parts.headers,
                    Some(perms),
                ));
            } else if c.data.api_token_data.is_set() {
                let api_login_manager = Arc::<api_login::APILoginManager>::from_ref(state);
                if !api_login_manager
                    .check_valid_api_token_id(c.data.api_token_data.token_id)
                    .await
                    .unwrap_or(false)
                {
                    return Err(to_http_error(
                        ConnectError::permission_denied("invalid API token"),
                        &parts.headers,
                        Some(perms),
                    ));
                }
            }
        }

        parts.extensions.insert(ReqAuthState { claims });

        Ok(Self)
    }
}
