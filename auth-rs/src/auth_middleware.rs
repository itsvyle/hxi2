use anyhow::Result;
use connectrpc::ConnectError;
use hxi2_proto::proto::auth::v2::JwtClaims;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};

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
        let route = crate::permissions_checking::get_route_from_public_url(parts.uri.path())
            .unwrap_or_default();
        let perms = crate::permissions_checking::get_compiled_permissions().get_by_route(&route);
        if perms.is_none() {
            return Err(ConnectError::not_found(format!(
                "route {} is not registered in permissions",
                route
            ))
            .into_http_response(&parts.headers)
            .into_response());
        }

        let mut auth_state = ReqAuthState { claims: None };

        let Some(token) = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
        else {
            parts.extensions.insert(auth_state);
            if perms.unwrap().is_public {
                return Ok(Self);
            }
            return Err(ConnectError::permission_denied("missing Bearer token")
                .into_http_response(&parts.headers)
                .into_response());
        };

        parts.extensions.insert(auth_state);

        println!(
            "Received request for route: {}, with token: {}",
            route, token
        );

        Ok(Self)
    }
}
