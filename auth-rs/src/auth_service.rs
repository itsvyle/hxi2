use anyhow::{Context as _, Result};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use hxi2_proto::connect::auth::v2::{AuthService, AuthServiceExt};
use hxi2_proto::proto::auth::v2::{
    GetJWTPublicKeyRequest, GetJWTPublicKeyResponse, ListUsersRequest, ListUsersResponse,
    RenewJWTRequest, RenewJWTResponse,
};

struct AuthServiceImpl;

#[allow(refining_impl_trait)]
impl AuthService for AuthServiceImpl {
    async fn get_jwt_public_key(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetJWTPublicKeyRequest>,
    ) -> ServiceResult<GetJWTPublicKeyResponse> {
        Response::ok(GetJWTPublicKeyResponse {
            public_key: "fake-public-key".to_string(),
            ..Default::default()
        })
    }

    async fn renew_jwt(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, RenewJWTRequest>,
    ) -> ServiceResult<RenewJWTResponse> {
        Err(ConnectError::unimplemented(
            "renew_jwt is not implemented yet".to_string(),
        ))
    }

    async fn list_users(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListUsersRequest>,
    ) -> ServiceResult<ListUsersResponse> {
        Err(ConnectError::unimplemented(
            "list_users is not implemented yet".to_string(),
        ))
    }
}
