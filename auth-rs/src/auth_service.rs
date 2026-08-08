use core::time;

use anyhow::Context as _;
use buffa_types::Empty;
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
pub use hxi2_proto::connect::auth::v2::AuthServiceExt;
use hxi2_proto::proto::auth::v2::{
    GetCSRFTokenRequest, GetCSRFTokenResponse, GetJWTPublicKeyRequest, GetJWTPublicKeyResponse,
    ListUsersRequest, ListUsersResponse, LoginRequest, LoginResponse, RenewJWTRequest,
    RenewJWTResponse, SmallData,
};
use hxi2_proto::{
    connect::auth::v2::AuthService,
    proto::auth::v2::{GetDevTokenRequest, GetDevTokenResponse},
};

use crate::app_config::AppConfiguration;
use crate::connect_result::ToConnectError;
use crate::{app_config, permissions_checking};

pub struct AuthServiceImpl {
    pub subdomain: String,
    pub signer: &'static crate::jwt_signer::JWTSigner,
    #[allow(unused)]
    pub verifier: &'static crate::jwt_verifier::JWTVerifier,
}

macro_rules! impl_unimplemented_rpc {
    ($name:ident, $req_type:ty, $res_type:ty) => {
        async fn $name(
            &self,
            _ctx: connectrpc::RequestContext,
            _request: connectrpc::ServiceRequest<'_, $req_type>,
        ) -> connectrpc::ServiceResult<$res_type> {
            Err(connectrpc::ConnectError::unimplemented(
                concat!(stringify!($name), " is not implemented yet").to_string(),
            ))
        }
    };
}

macro_rules! impl_otherplace_rpc {
    ($name:ident, $req_type:ty, $res_type:ty) => {
        async fn $name(
            &self,
            _ctx: connectrpc::RequestContext,
            _request: connectrpc::ServiceRequest<'_, $req_type>,
        ) -> connectrpc::ServiceResult<$res_type> {
            Err(connectrpc::ConnectError::unimplemented(
                concat!(stringify!($name), " is implemented some place else").to_string(),
            ))
        }
    };
}

#[allow(refining_impl_trait)]
impl AuthService for AuthServiceImpl {
    async fn get_jwt_public_key(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetJWTPublicKeyRequest>,
    ) -> ServiceResult<GetJWTPublicKeyResponse> {
        Response::ok(GetJWTPublicKeyResponse {
            public_key: AppConfiguration::INSTANCE().jwt_public_key().to_string(),
            ..Default::default()
        })
    }

    async fn get_dev_token(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, GetDevTokenRequest>,
    ) -> ServiceResult<GetDevTokenResponse> {
        if !app_config::AppConfiguration::INSTANCE().is_development() {
            return Err(ConnectError::permission_denied(
                "get_dev_token is only available in development mode",
            ));
        }

        // compile to a bitfield
        let final_permissions = req
            .roles
            .iter()
            .fold(0_i64, |acc, role| acc | ((role.to_i32() as i64) << 1_i64));

        let data = SmallData {
            user_id: 42,
            username: "TESTING".to_owned(),
            first_name: "Test".to_owned(),
            last_name: "T".to_owned(),
            permissions: final_permissions,
            promotion: 2000,
            ..Default::default()
        };

        let token = self
            .signer
            .new_token(
                &format!("{}", data.user_id),
                data,
                &crate::jwt_signer::JWTSignerOptions::default()
                    .with_validity(time::Duration::from_hours(24)),
            )
            .obfuscate()
            .to_connect_internal()?;

        let _claims = self
            .verifier
            .verify_token(&token.0)
            .context("verifying token after signing")
            .obfuscate()
            .to_connect_permission_denied()?;

        Response::ok(GetDevTokenResponse {
            jwt: token.0,
            ..Default::default()
        })
    }

    async fn get_csrf_token(
        &self,
        ctx: connectrpc::RequestContext,
        _request: connectrpc::ServiceRequest<'_, GetCSRFTokenRequest>,
    ) -> connectrpc::ServiceResult<GetCSRFTokenResponse> {
        let perms = permissions_checking::get_by_route(ctx.path().ok_or_else(|| {
            ConnectError::new(ErrorCode::Internal, "couldn't get path for my request")
        })?)
        .ok_or_else(|| {
            ConnectError::new(
                ErrorCode::Internal,
                "couldn't get permissions for my request",
            )
        })?;
        if perms.csrf_token_cookie.is_none() || perms.csrf_token_header.is_none() {
            return Err(ConnectError::permission_denied(
                "You need to configure the cookie name and the header name for the CSRF token in the permissions list for this route",
            ));
        }

        let csrf_manager = &crate::csrf_handler::GLOBAL_CSRF_PROTECTION;
        let new_token = csrf_manager.generate_token();

        let mut set_cookie = format!(
            "{}={}; SameSite=Strict; Path=/; Max-Age=86400;",
            perms.csrf_token_cookie.unwrap(),
            new_token,
        );
        if !app_config::AppConfiguration::INSTANCE().is_development() {
            set_cookie.push_str(" Secure");
            set_cookie.push_str(" ;Domain=");
            set_cookie.push_str(&self.subdomain);
        }

        let mut res = Response::from(GetCSRFTokenResponse {
            ..Default::default()
        });
        res = res.try_with_header("Set-Cookie", set_cookie)?;
        res = res.try_with_header(perms.csrf_token_header.unwrap(), new_token.clone())?;

        Ok(res)
    }

    async fn list_users(
        &self,
        _ctx: connectrpc::RequestContext,
        _request: connectrpc::ServiceRequest<'_, ListUsersRequest>,
    ) -> connectrpc::ServiceResult<ListUsersResponse> {
        let db = app_config::AppConfiguration::INSTANCE().db().await;
        let users = db
            .list_users()
            .await
            .context("listing users")
            .obfuscate()
            .to_connect_internal()?;
        Response::ok(ListUsersResponse {
            users: users
                .into_iter()
                .map(|u| hxi2_proto::proto::auth::v2::DBUser {
                    id: u.id,
                    username: u.username,
                    first_name: u.first_name,
                    last_name: u.last_name,
                    permissions: u.permissions,
                    promotion: u.promotion,
                    discord_id: u.discord_id,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        })
    }

    impl_unimplemented_rpc!(renew_jwt, RenewJWTRequest, RenewJWTResponse);
    impl_unimplemented_rpc!(login, LoginRequest, LoginResponse);
    impl_otherplace_rpc!(frontend_index, Empty, Empty);
    impl_otherplace_rpc!(discord_callback, Empty, Empty);
}
