use std::collections::HashMap;

use anyhow::{Context as _, Result};

use buffa::{Enumeration, ExtensionSet};
use buffa_descriptor::DescriptorPool;
use hxi2_proto::proto::auth::v2::{
    PERMISSION_LEVEL, PERMISSION_LEVEL_SERVICE, Permission, Permissions,
};
use serde::Serializer;

#[derive(Clone, Debug, serde::Serialize)]
// allow serializing to json with serde, and also print with Debug
pub struct MethodPermissions {
    #[serde(serialize_with = "serialize_roles_as_ints")]
    pub allow_roles: Vec<Permission>,
    pub is_public: bool,
}

fn serialize_roles_as_ints<S>(roles: &[Permission], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(roles.len()))?;
    for role in roles {
        let num: i32 = role.to_i32();
        seq.serialize_element(&num)?;
    }
    seq.end()
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

fn get_permissions(descriptor_bytes: &[u8]) -> Result<HashMap<String, MethodPermissions>> {
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

    Ok(cache)
}

fn main() -> Result<()> {
    let descriptor_bytes =
        std::fs::read("../hxi2.binpb").context("Failed to read descriptor set binary")?;

    let perms = get_permissions(&descriptor_bytes).context("get_permissions")?;
    println!("Permissions: {:#?}", perms);

    // serialize to json and print
    let json =
        serde_json::to_string_pretty(&perms).context("Failed to serialize permissions to JSON")?;

    // write it to ../permissions.json
    std::fs::write("../permissions.json", json).context("Failed to write permissions to file")?;
    println!("Permissions written to ../permissions.json");

    Ok(())
}
