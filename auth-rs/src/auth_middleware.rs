use anyhow::Result;
use connectrpc::ConnectError;
use hxi2_proto::proto::auth::v2::JwtClaims;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};

use crate::permissions_checking;

#[derive(Debug, Clone)]
pub struct ReqAuthState {
    pub claims: Option<JwtClaims>,
}

pub struct RequireAuthMiddleware;

impl<S> FromRequestParts<S> for RequireAuthMiddleware
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let route =
            permissions_checking::get_route_from_public_url(parts.uri.path()).ok_or_else(|| {
                ConnectError::not_found("route not found")
                    .into_http_response(&parts.headers)
                    .into_response()
            })?;

        let is_public = permissions_checking::check_permissions_for_route(&route, 0);

        let token = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        let claims = if let Some(t) = token {
            match crate::jwt_verifier::GLOBAL_JWT_VERIFIER.verify_token(t) {
                Ok(c) => Some(c),
                Err(e) => {
                    if !is_public {
                        return Err(ConnectError::unauthenticated(format!("invalid token: {e}"))
                            .into_http_response(&parts.headers)
                            .into_response());
                    }
                    None
                }
            }
        } else if !is_public {
            return Err(ConnectError::unauthenticated("missing Bearer token")
                .into_http_response(&parts.headers)
                .into_response());
        } else {
            None
        };

        if !is_public
            && let Some(ref c) = claims
            && !permissions_checking::check_permissions_for_route(&route, c.data.permissions)
        {
            return Err(ConnectError::permission_denied("insufficient permissions")
                .into_http_response(&parts.headers)
                .into_response());
        }

        parts.extensions.insert(ReqAuthState { claims });

        Ok(Self)
    }
}
