use std::sync::Arc;

use anyhow::Result;
use axum_extra::extract::cookie::Cookie;
use connectrpc::ConnectError;
use http::header::ACCEPT;
use hxi2_proto::proto::auth::v2::JwtClaims;

use axum::extract::{FromRef, FromRequestParts};
use axum::response::{IntoResponse, Response};

use crate::api_login;
use crate::login_manager::LoginManager;
use crate::permissions_checking::{self, MethodPermissionsOptionExt};

#[derive(Debug, Clone)]
pub struct ReqAuthState {
    pub claims: Option<JwtClaims>,
}

pub fn to_http_error(
    e: ConnectError,
    headers: &http::HeaderMap,
    is_frontend: Option<bool>,
) -> Response {
    let is_front = if let Some(is_frontend) = is_frontend {
        is_frontend
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
    (Arc<api_login::APILoginManager>, Arc<LoginManager>): FromRef<S>,
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

        // let token = parts
        //     .headers
        //     .get(http::header::AUTHORIZATION)
        //     .and_then(|v| v.to_str().ok())
        //     .and_then(|s| s.strip_prefix("Bearer "))
        //     .map(String::from)
        //     .or_else(|| extract_auth_cookies(&parts.headers).0);

        // let claims = if let Some(t) = token {
        //     match crate::jwt_verifier::GLOBAL_JWT_VERIFIER.verify_token(&t) {
        //         Ok(c) => Some(c),
        //         Err(e) => {
        //             let decoded_claims =
        //                 crate::jwt_verifier::GLOBAL_JWT_VERIFIER.verify_token_ignore_expiry(&t);
        //             if !decoded_claims.is_ok() && !is_public {
        //                 return Err(to_http_error(
        //                     ConnectError::unauthenticated(format!("completely invalid token: {e}")),
        //                     &parts.headers,
        //                     Some(perms),
        //                 ));
        //             }
        //             if decoded_claims.is_ok() {
        //                 let decoded_claims = decoded_claims.unwrap();
        //                 if decoded_claims.exp < chrono::Utc::now().timestamp() {
        //                     return Err(to_http_error(
        //                         ConnectError::unauthenticated(format!("expired token: {e}")),
        //                         &parts.headers,
        //                         Some(perms),
        //                     ));
        //                 }
        //             }

        //             if !is_public {
        //                 return Err(to_http_error(
        //                     ConnectError::unauthenticated(format!("invalid token: {e}")),
        //                     &parts.headers,
        //                     Some(perms),
        //                 ));
        //             }
        //             None
        //         }
        //     }
        // } else if !is_public {
        //     return Err(to_http_error(
        //         ConnectError::unauthenticated("missing Bearer token"),
        //         &parts.headers,
        //         Some(perms),
        //     ));
        // } else {
        //     None
        // };
        let (api_login_manager, login_manager) =
            <(Arc<api_login::APILoginManager>, Arc<LoginManager>)>::from_ref(state);
        let (claims, _) = login_manager
            .extract_claims_from_request(&parts.headers, Some(perms.is_frontend))
            .await
            .map_err(|e| e.into_response())?;

        if !is_public && let Some(ref c) = claims {
            if !perms.check_permissions(c.data.permissions) {
                return Err(to_http_error(
                    ConnectError::permission_denied("insufficient permissions"),
                    &parts.headers,
                    Some(perms.is_frontend),
                ));
            } else if c.data.api_token_data.is_set() {
                //
                if !api_login_manager
                    .check_valid_api_token_id(c.data.api_token_data.token_id)
                    .await
                    .unwrap_or(false)
                {
                    return Err(to_http_error(
                        ConnectError::permission_denied("invalid API token"),
                        &parts.headers,
                        Some(perms.is_frontend),
                    ));
                }
            }
        }

        parts.extensions.insert(ReqAuthState { claims });

        Ok(Self)
    }
}
