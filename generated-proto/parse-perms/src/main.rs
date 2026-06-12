use std::collections::BTreeMap;

use anyhow::{Context as _, Result};

use buffa::{Enumeration, ExtensionSet};
use buffa_descriptor::DescriptorPool;
use hxi2_proto::proto::auth::v2::{
    PERMISSION_LEVEL, PERMISSION_LEVEL_SERVICE, Permission, Permissions,
};
use serde::Serializer;

const BIN_FILE_PATH: &str = "../hxi2.binpb";
const PROTO_FILES_PATH: &str = "../../protos";

#[derive(Clone, Debug, serde::Serialize)]
pub struct MethodPermissions {
    #[serde(serialize_with = "serialize_roles_as_ints")]
    pub allow_roles: Vec<Permission>,
    pub is_public: bool,
    pub public_url: Option<String>,
    pub compiled_permissions_bitfield: Option<i64>,
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
    if let Some(url) = perms_msg.public_url {
        base.public_url = Some(url);
    }
}

fn get_proto_folder_hash() -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    let mut entries: Vec<_> = walkdir::WalkDir::new(PROTO_FILES_PATH)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    entries.sort_by_key(|e| e.path().to_path_buf());
    for entry in entries {
        let path = entry.path();
        let content =
            std::fs::read(path).context(format!("Failed to read proto file: {:?}", path))?;
        hasher.update(&content);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn get_permissions(descriptor_bytes: &[u8]) -> Result<BTreeMap<String, MethodPermissions>> {
    let mut cache: BTreeMap<String, MethodPermissions> = BTreeMap::new();

    let pool = DescriptorPool::decode(descriptor_bytes).context("parse FileDescriptorSet")?;

    for service in pool.services() {
        let mut default_perms = MethodPermissions {
            allow_roles: vec![],
            is_public: false,
            public_url: None,
            compiled_permissions_bitfield: None,
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

#[derive(serde::Serialize)]
pub struct PermissionsOutput {
    pub permissions: BTreeMap<String, MethodPermissions>,
}

fn main() -> Result<()> {
    let descriptor_bytes =
        std::fs::read(BIN_FILE_PATH).context("Failed to read descriptor set binary")?;

    let perms: BTreeMap<String, MethodPermissions> = get_permissions(&descriptor_bytes)
        .context("get_permissions")?
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                MethodPermissions {
                    compiled_permissions_bitfield: Some(
                        v.allow_roles
                            .iter()
                            .fold(0, |acc, r| acc | i64::from(r.to_i32())),
                    ),
                    ..v.clone()
                },
            )
        })
        .collect();
    println!("Permissions: {:#?}", perms);

    let output = PermissionsOutput { permissions: perms };
    let json =
        serde_json::to_string_pretty(&output).context("Failed to serialize permissions to JSON")?;

    let hash = get_proto_folder_hash().unwrap_or_else(|_| "unknown".into());

    // doing this so that the hash is always first in the output, for determinism
    let json = json.replacen("{", &format!("{{\n\t\"hash\": \"{}\",", hash), 1);

    std::fs::write("../permissions.json", json).context("Failed to write permissions to file")?;
    println!("Permissions written to ../permissions.json");

    Ok(())
}
