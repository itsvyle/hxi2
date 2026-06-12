use std::{collections::HashMap, default, sync::Arc};

use anyhow::{Context as _, Result};

use buffa::ExtensionSet;
use buffa_descriptor::{DescriptorPool, ReflectMessage};
use hxi2_proto::proto::auth::v2::{
    PERMISSION_LEVEL, PERMISSION_LEVEL_SERVICE, Permission, Permissions,
};

mod auth_service;

#[derive(Clone, Debug)]
pub struct MethodPermissions {
    pub allow_roles: Vec<Permission>, // Store enum values as integers or your generated Enum type
    pub is_public: bool,
}

fn method_permissions_from_permissions(perms_msg: Permissions, base: &mut MethodPermissions) {
    base.is_public = perms_msg.is_public.unwrap_or(false);
    base.allow_roles.extend(
        perms_msg
            .allow_role
            .iter()
            .map(|r| r.as_known().unwrap_or(Permission::PermissionUnspecified))
            .filter(|&r| r != Permission::PermissionUnspecified),
    );
}

fn get_permissions(descriptor_bytes: &[u8]) -> Result<Arc<HashMap<String, MethodPermissions>>> {
    let mut cache: HashMap<String, MethodPermissions> = HashMap::new();

    let pool = DescriptorPool::decode(descriptor_bytes).context("parse FileDescriptorSet")?;

    for service in pool.services() {
        let mut default_perms = MethodPermissions {
            allow_roles: vec![],
            is_public: false,
        };
        if let Some(options) = service.options()
            && let Some(perms_msg) = options.extension(&PERMISSION_LEVEL_SERVICE)
        {
            method_permissions_from_permissions(perms_msg, &mut default_perms);
        }

        for method in service.methods() {
            let path = format!("/{}/{}", service.full_name(), method.name());
            let mut method_perms = default_perms.clone();

            if let Some(options) = method.options()
                && let Some(perms_msg) = options.extension(&PERMISSION_LEVEL)
            {
                method_permissions_from_permissions(perms_msg, &mut method_perms);
            }

            cache.insert(path, method_perms);
        }
    }

    Ok(Arc::new(cache))
}

fn main() -> Result<()> {
    let descriptor_bytes = std::fs::read("/home/gm/repos/hxi2/generated-proto/hxi2.binpb")
        .context("Failed to read descriptor set binary")?;

    let perms = get_permissions(&descriptor_bytes);
    println!("Permissions: {:#?}", perms);

    Ok(())
}

/*
use anyhow::{Context as _, Result};
use axum::{routing::get, Router};
use connectrpc::Router as ConnectRouter;
use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use hxi2_proto::proto::auth::v2::DBUser;
use std::sync::Arc;
#[tokio::main]
async fn main() -> Result<()> {
    let service = Arc::new(MyGreetService);
    let connect = service.register(ConnectRouter::new());

    let user = DBUser {
        id: 85,
        ..Default::default()
    };
    let json = serde_json::to_string(&user).context("serializing DBUser")?;
    let json = json.replace("\"85\"", "\"a random string\"");
    println!("DBUser as JSON: {json}");

    let decoded: DBUser = serde_json::from_str(&json).context("deserializing DBUser")?;
    println!("Decoded DBUser: {decoded:#?}");

    // Plain HTTP liveness probe for `kubectl`'s httpGet style. For the
    // standard gRPC Health protocol (grpc_health_probe, kubelet `grpc:`
    // probes), mount `connectrpc_health::HealthService` on the Connect
    // router instead — see docs/guide.md#health-checking.
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(connect.into_axum_service());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
 */
