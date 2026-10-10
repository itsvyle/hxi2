use std::sync::Arc;

use anyhow::Context as _;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use buffa::MessageField;
use buffa_types::{Duration, Empty};
use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
use http::header::SET_COOKIE;
pub use hxi2_proto::connect::auth::v2::AuthServiceExt;
use hxi2_proto::proto::auth::v2::{
    APILoginRequest, APILoginResponse, AddAPIUserRequest, AddAPIUserResponse, AddPasswordRequest,
    CreateUserRequest, CreateUserResponse, DBUser, GetCSRFTokenRequest, GetCSRFTokenResponse,
    GetJWTPublicKeyRequest, GetJWTPublicKeyResponse, ListAPIUsersResponse, ListUsersRequest,
    ListUsersResponse, PasswordLoginRequest, PasswordLoginResponse, Permission,
    RemoveAPIUserRequest, RemovePasswordRequest, RenewAPIUserTokenRequest,
    RenewAPIUserTokenResponse, RenewJWTRequest, RenewJWTResponse, SmallData,
};
use hxi2_proto::utils::compile_bitfield;
use hxi2_proto::{
    connect::auth::v2::AuthService,
    proto::auth::v2::{GetDevTokenRequest, GetDevTokenResponse},
};
use tracing::{error, trace, warn};

use crate::api_login::APILoginManager;
use crate::app_config::AppConfiguration;
use crate::auth_middleware::ReqAuthState;
use crate::connect_result::ToConnectError;
use crate::database::DbUserIdentifier;
use crate::login_manager::LoginManager;
use crate::password_login::PasswordLoginManager;
use crate::permissions_checking;

pub struct AuthServiceImpl {
    pub subdomain: String,
    pub signer: &'static crate::jwt_signer::JWTSigner,
    #[allow(unused)]
    pub verifier: &'static crate::jwt_verifier::JWTVerifier,
    pub login_manager: Arc<LoginManager>,
    pub password_login_manager: Arc<PasswordLoginManager>,
    pub api_login_manager: Arc<APILoginManager>,
}

#[allow(unused)]
macro_rules! impl_unimplemented_rpc {
    ($name:ident, $req_type:ty, $res_type:ty) => {
        async fn $name(
            &self,
            _ctx: RequestContext,
            _request: ServiceRequest<'_, $req_type>,
        ) -> ServiceResult<$res_type> {
            Err(ConnectError::unimplemented(
                concat!(stringify!($name), " is not implemented yet").to_string(),
            ))
        }
    };
}

macro_rules! impl_otherplace_rpc {
    ($name:ident, $req_type:ty, $res_type:ty) => {
        async fn $name(
            &self,
            _ctx: RequestContext,
            _request: ServiceRequest<'_, $req_type>,
        ) -> ServiceResult<$res_type> {
            Err(ConnectError::unimplemented(
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
        if !AppConfiguration::INSTANCE().is_development() {
            return Err(ConnectError::permission_denied(
                "get_dev_token is only available in development mode",
            ));
        }

        // compile to a bitfield
        let final_permissions = compile_bitfield(req.roles.iter().filter_map(|r| r.as_known()));
        let final_attributes = compile_bitfield(req.attributes.iter().filter_map(|a| a.as_known()));

        let data = SmallData {
            user_id: 42,
            username: "TESTING".to_owned(),
            first_name: "Test".to_owned(),
            last_name: Some("T".to_owned()),
            permissions: final_permissions,
            promotion: 2024,
            expiration: MessageField::none(),
            roles: req.roles.clone().to_vec(),
            attributes: final_attributes,
            attribute_list: req.attributes.clone().to_vec(),
            api_token_data: MessageField::none(),
            __buffa_unknown_fields: Default::default(),
        };

        let token = self
            .signer
            .new_token(
                &format!("{}", data.user_id),
                &data,
                &crate::jwt_signer::JWTSignerOptions::default().with_validity(
                    chrono::Duration::seconds(req.validity_seconds.unwrap_or(86400)),
                ),
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
        ctx: RequestContext,
        _request: ServiceRequest<'_, GetCSRFTokenRequest>,
    ) -> ServiceResult<GetCSRFTokenResponse> {
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
            error!(
                "CSRF token cookie or header not configured for route {}",
                ctx.path().unwrap_or("<unknown>")
            );
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
        if !AppConfiguration::INSTANCE().is_development() {
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

    // ignore the _ctx.headers
    // #[instrument(skip(self), err)]
    async fn renew_jwt(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, RenewJWTRequest>,
    ) -> ServiceResult<RenewJWTResponse> {
        let r = self
            .login_manager
            .renew_authentication(request.jwt, request.refresh_token)
            .await
            .obfuscate()
            .to_connect_permission_denied()?;

        Response::ok(RenewJWTResponse {
            jwt: r.token,
            refresh_token: r.refresh_token,
            refresh_token_max_age: r.refresh_token_max_age,
            jwt_max_age: r.token_max_age,
            small_data: r.small_data.into(),
            __buffa_unknown_fields: Default::default(),
        })
    }

    async fn list_users(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListUsersRequest>,
    ) -> ServiceResult<ListUsersResponse> {
        let db = AppConfiguration::INSTANCE().db().await;
        let users = db
            .list_users()
            .await
            .context("listing users")
            .obfuscate()
            .to_connect_internal()?;
        Response::ok(ListUsersResponse {
            users: users
                .into_iter()
                .filter(|user| !user.is_api)
                .map(hxi2_proto::proto::auth::v2::DBUser::from)
                .collect(),
            ..Default::default()
        })
    }

    async fn create_user(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, CreateUserRequest>,
    ) -> ServiceResult<CreateUserResponse> {
        let req = request.to_owned_message();

        let mut new_user = crate::database::DbUser::from(
            req.user
                .ok_or_else(|| ConnectError::invalid_argument("User data is missing"))?,
        );

        new_user
            .check_schema()
            .map_err(|e| ConnectError::invalid_argument(format!("Invalid user data: {}", e)))?;

        let cfg = AppConfiguration::INSTANCE();
        cfg.db()
            .await
            .add_new_db_user(&mut new_user)
            .await
            .context("creating user in database")
            .obfuscate()
            .to_connect_internal()?;
        let return_user = hxi2_proto::proto::auth::v2::DBUser::from(new_user);

        Response::ok(CreateUserResponse {
            user: return_user.into(),
            ..Default::default()
        })
    }

    async fn update_user(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, DBUser>,
    ) -> ServiceResult<DBUser> {
        let req = request.to_owned_message();

        let mut updated_user = crate::database::DbUser::from(req);

        updated_user
            .check_schema()
            .map_err(|e| ConnectError::invalid_argument(format!("Invalid user data: {}", e)))?;

        let cfg = AppConfiguration::INSTANCE();
        cfg.db()
            .await
            .update_user(&mut updated_user)
            .await
            .context("updating user in database")
            .obfuscate()
            .to_connect_internal()?;
        let return_user = hxi2_proto::proto::auth::v2::DBUser::from(updated_user);

        Response::ok(return_user)
    }
    async fn add_password(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, AddPasswordRequest>,
    ) -> ServiceResult<Empty> {
        let auth_state = ctx.extensions().get::<ReqAuthState>().ok_or_else(|| {
            ConnectError::permission_denied("ReqAuthState not found in request extensions")
        })?;

        let current_user = &auth_state
            .claims
            .as_ref()
            .ok_or_else(|| ConnectError::permission_denied("Claims not found in ReqAuthState"))?
            .data;

        let user_identifier =
            if (current_user.permissions & (1 << Permission::PERMISSION_ADMIN as i64) != 0)
                && let Some(target_user_identifier) = req.user_identifier.as_ref()
            {
                use hxi2_proto::proto::auth::v2::add_password_request::UserIdentifierView;
                match target_user_identifier {
                    UserIdentifierView::UserId(user_id) => DbUserIdentifier::Id(*user_id),
                    UserIdentifierView::Username(username) => {
                        DbUserIdentifier::Username(username.to_string())
                    }
                }
            } else if req.user_identifier.is_some() {
                return Err(ConnectError::permission_denied(
                    "You do not have permission to add a password for another user",
                ));
            } else {
                DbUserIdentifier::Id(current_user.user_id)
            };

        let req = req.to_owned_message();

        if req.password.trim().is_empty() {
            return Err(ConnectError::invalid_argument(
                "Password cannot be empty or whitespace",
            ));
        }

        self.password_login_manager
            .add_password(&user_identifier, &req.password)
            .await
            .obfuscate()
            .to_connect_internal()?;

        Response::ok(Empty {
            ..Default::default()
        })
    }

    async fn remove_password(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, RemovePasswordRequest>,
    ) -> ServiceResult<Empty> {
        let auth_state = ctx.extensions().get::<ReqAuthState>().ok_or_else(|| {
            ConnectError::permission_denied("ReqAuthState not found in request extensions")
        })?;

        let current_user = &auth_state
            .claims
            .as_ref()
            .ok_or_else(|| ConnectError::permission_denied("Claims not found in ReqAuthState"))?
            .data;

        let user_identifier =
            if (current_user.permissions & (1 << Permission::PERMISSION_ADMIN as i64) != 0)
                && let Some(target_user_identifier) = req.user_identifier.as_ref()
            {
                use hxi2_proto::proto::auth::v2::remove_password_request::UserIdentifierView;
                match target_user_identifier {
                    UserIdentifierView::UserId(user_id) => DbUserIdentifier::Id(*user_id),
                    UserIdentifierView::Username(username) => {
                        DbUserIdentifier::Username(username.to_string())
                    }
                }
            } else if req.user_identifier.is_some() {
                return Err(ConnectError::permission_denied(
                    "You do not have permission to remove a password for another user",
                ));
            } else {
                DbUserIdentifier::Id(current_user.user_id)
            };

        self.password_login_manager
            .remove_password(&user_identifier)
            .await
            .obfuscate()
            .to_connect_internal()?;
        Response::ok(Empty {
            ..Default::default()
        })
    }

    // #[cfg_attr(debug_assertions, instrument(skip(self), level = "trace", ret))]
    async fn password_login(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, PasswordLoginRequest>,
    ) -> ServiceResult<PasswordLoginResponse> {
        let jar = CookieJar::from_headers(ctx.headers());

        let user_id = self
            .password_login_manager
            .try_login(req.username, req.password)
            .await
            .map_err(|_| ConnectError::permission_denied("Invalid username or password"))?;

        let login_response = self
            .login_manager
            .login_as(&DbUserIdentifier::Id(user_id), None)
            .await
            .map_err(|err| {
                if matches!(
                    err.downcast_ref::<crate::database::DbError>(),
                    Some(crate::database::DbError::NotFound)
                ) {
                    warn!(
                        "User {} (id={}) not found in database, yet tried to login with UserID.",
                        req.username, user_id
                    );
                    return ConnectError::permission_denied("Invalid username or password");
                }
                error!(error = %err, "Failed to login user by UserID.");
                ConnectError::new(ErrorCode::Internal, "Failed to login user by UserID.")
            })?;

        let mut jar = login_response.make_cookies(jar).map_err(|err| {
            error!(
                err_code = err.0.as_u16(),
                err_message = err.1,
                "Failed to make cookies for login response"
            );
            ConnectError::new(
                ErrorCode::Internal,
                "Failed to make cookies for login response",
            )
        })?;

        let redirect_to = {
            if let Some(r) = req.redirect_to {
                Some(r.to_owned())
            } else if let Some(cookie) = jar.get("redirect_to") {
                let redirect_to = cookie.value().to_string();
                let mut c = Cookie::from("redirect_to");
                LoginManager::apply_secure_cookie_options(&mut c);
                jar = jar.remove(c);
                Some(redirect_to)
            } else {
                None
            }
        }
        .unwrap_or_else(|| "/".to_string());

        trace!(
            "User {} (id={}) logged in via password, redirecting to {}",
            req.username, user_id, redirect_to
        );

        let mut res = Response::from(PasswordLoginResponse {
            redirect_to,
            ..Default::default()
        });

        // this is so incredibly stupid... i hate it... I hate types... it sucks
        // this only works well here because we know that manu of the cookies will change
        // basically, the axum CookieJar doesn't expose the "delta" like the basic one does
        // but I'm having problems with adapting make_cookies to use either the axum one or the basic one
        // so im doing this as a workaround for now...
        let jar = jar.into_response();
        for (header_name, header_value) in
            jar.headers().iter().filter(|(name, _)| name == &SET_COOKIE)
        {
            // trace!(
            //     "Adding Set-Cookie header to response: {} = {}",
            //     header_name,
            //     header_value.to_str().unwrap_or("<invalid utf8>")
            // );
            res = res.try_with_header(header_name, header_value.as_bytes())?;
        }

        Ok(res)
    }

    async fn add_api_user(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, AddAPIUserRequest>,
    ) -> ServiceResult<AddAPIUserResponse> {
        let req = _request.to_owned_message();

        let (user_id, token) = self
            .api_login_manager
            .new_api_user(
                &req.username,
                &req.roles
                    .iter()
                    .filter_map(|r| r.as_known())
                    .collect::<Vec<_>>(),
                &req.attributes
                    .iter()
                    .filter_map(|a| a.as_known())
                    .collect::<Vec<_>>(),
            )
            .await
            .obfuscate()
            .to_connect_internal()?;
        Response::ok(AddAPIUserResponse {
            user_id,
            api_token: token,
            ..Default::default()
        })
    }

    async fn list_api_users(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, Empty>,
    ) -> ServiceResult<ListAPIUsersResponse> {
        let db = AppConfiguration::INSTANCE().db().await;
        let users = db
            .list_api_users()
            .await
            .context("listing users")
            .obfuscate()
            .to_connect_internal()?;

        Response::ok(ListAPIUsersResponse {
            users,
            ..Default::default()
        })
    }
    async fn renew_api_user_token(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, RenewAPIUserTokenRequest>,
    ) -> ServiceResult<RenewAPIUserTokenResponse> {
        let identifier = {
            use hxi2_proto::proto::auth::v2::renew_api_user_token_request::UserIdentifierView;
            match _request.user_identifier {
                Some(UserIdentifierView::UserId(user_id)) => DbUserIdentifier::Id(user_id),
                Some(UserIdentifierView::Username(username)) => {
                    DbUserIdentifier::Username(username.to_string())
                }
                None => {
                    return Err(ConnectError::invalid_argument(
                        "user_identifier is required",
                    ));
                }
            }
        };

        let token = self
            .api_login_manager
            .renew_api_token(&identifier)
            .await
            .obfuscate()
            .to_connect_internal()?;

        Response::ok(RenewAPIUserTokenResponse {
            api_token: token,
            __buffa_unknown_fields: Default::default(),
        })
    }

    async fn api_login(
        &self,
        _ctx: RequestContext,
        req: ServiceRequest<'_, APILoginRequest>,
    ) -> ServiceResult<APILoginResponse> {
        let identifier = {
            use hxi2_proto::proto::auth::v2::api_login_request::UserIdentifierView;
            match req.user_identifier {
                Some(UserIdentifierView::UserId(user_id)) => DbUserIdentifier::Id(user_id),
                Some(UserIdentifierView::Username(username)) => {
                    DbUserIdentifier::Username(username.to_string())
                }
                None => {
                    return Err(ConnectError::invalid_argument(
                        "user_identifier is required",
                    ));
                }
            }
        };

        let res = self
            .api_login_manager
            .login_with_api_token(&identifier, req.api_token)
            .await
            .obfuscate()
            .to_connect_permission_denied()?;

        Response::ok(APILoginResponse {
            jwt: res.token,
            jwt_max_age: Duration::from_secs(res.token_max_age).into(),
            ..Default::default()
        })
    }

    impl_unimplemented_rpc!(remove_api_user, RemoveAPIUserRequest, Empty);

    impl_otherplace_rpc!(logout, Empty, Empty);
    impl_otherplace_rpc!(frontend_index, Empty, Empty);
    impl_otherplace_rpc!(discord_login, Empty, Empty);
}
