use anyhow::{Context as _, Result};
use std::collections::BTreeMap;
const PERMISSIONS_PATH: &str = "../../generated-proto/permissions.json";

use hxi2_proto::proto::auth::v2::{Permission, Permissions};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct MethodPermissions {
    pub allow_roles: Vec<i32>,
    pub is_public: bool,
    pub public_url: Option<String>,
    pub compiled_permissions_bitfield: Option<i32>,
}

#[derive(serde::Deserialize)]
pub struct PermissionsOutput {
    pub permissions: BTreeMap<String, MethodPermissions>,
    pub hash: String,
}

pub fn main() -> Result<()> {
    let contents =
        std::fs::read_to_string(PERMISSIONS_PATH).context("Failed to read permissions file")?;
    let perms: BTreeMap<String, MethodPermissions> =
        serde_json::from_str(&contents).context("Failed to parse permissions JSON")?;
    println!("Permissions: {:#?}", perms);
    Ok(())
}
