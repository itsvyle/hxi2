#![allow(refining_impl_trait)]
use std::sync::Arc;

use anyhow::Context;
use buffa_types::Empty;
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use hxi2_proto::{
    connect::auth::v2::AuthUpdateUserService,
    proto::auth::v2::{
        AddPasswordRequest, RemovePasswordRequest, UpdateDiscordIDRequest, UpdateUsernameRequest,
    },
};

use crate::{
    auth_service::{resolve_current_user, resolve_overridable_user_identifier_from_message},
    connect_result::ToConnectError,
    database::DbError::{Duplicate, Validation},
    login_manager::LoginManager,
    password_login::PasswordLoginManager,
};

pub struct AuthUpdateUserServiceImpl {
    pub login_manager: Arc<LoginManager>,
    pub password_login_manager: Arc<PasswordLoginManager>,
}

impl AuthUpdateUserService for AuthUpdateUserServiceImpl {
    async fn add_password(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, AddPasswordRequest>,
    ) -> ServiceResult<Empty> {
        let current_user = resolve_current_user(&ctx)?;

        let user_identifier =
            resolve_overridable_user_identifier_from_message(&req.user_identifier, current_user)?;

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
        let current_user = resolve_current_user(&ctx)?;

        let user_identifier =
            resolve_overridable_user_identifier_from_message(&req.user_identifier, current_user)?;

        self.password_login_manager
            .remove_password(&user_identifier)
            .await
            .obfuscate()
            .to_connect_internal()?;
        Response::ok(Empty {
            ..Default::default()
        })
    }

    async fn update_username(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, UpdateUsernameRequest>,
    ) -> ServiceResult<Empty> {
        let current_user = resolve_current_user(&ctx)?;

        let user_identifier =
            resolve_overridable_user_identifier_from_message(&req.user_identifier, current_user)?;

        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        match db.update_username(&user_identifier, req.new_username).await {
            Ok(()) => Ok(()),
            Err(Validation(msg)) => Err(ConnectError::invalid_argument(format!(
                "Invalid username: {}",
                msg
            ))),
            Err(Duplicate(_)) => Err(ConnectError::already_exists("Username already exists")),
            Err(e) => Err(e)
                .context("Failed to update username")
                .to_connect_internal(),
        }?;

        Response::ok(Empty {
            ..Default::default()
        })
    }

    async fn update_discord_id(
        &self,
        ctx: RequestContext,
        req: ServiceRequest<'_, UpdateDiscordIDRequest>,
    ) -> ServiceResult<Empty> {
        let current_user = resolve_current_user(&ctx)?;

        let user_identifier =
            resolve_overridable_user_identifier_from_message(&req.user_identifier, current_user)?;

        let db = crate::app_config::AppConfiguration::INSTANCE().db().await;
        match db
            .update_discord_id(&user_identifier, req.new_discord_id)
            .await
        {
            Ok(()) => Ok(()),
            Err(Validation(msg)) => Err(ConnectError::invalid_argument(format!(
                "Invalid discord ID: {}",
                msg
            ))),
            Err(Duplicate(_)) => Err(ConnectError::already_exists("Discord ID already exists")),
            Err(e) => Err(e)
                .context("Failed to update discord ID")
                .to_connect_internal(),
        }?;

        Response::ok(Empty {
            ..Default::default()
        })
    }
}
