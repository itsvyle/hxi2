use anyhow::{Context as _, Result};
use buffa::Enumeration;
use std::collections::BTreeMap;

use hxi2_proto::proto::auth::v2::Permission;

pub fn find_permissions_path() -> Result<String> {
    // walk up the directory tree until root, where we'll go into generated-proto/permissions.json
    let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;
    loop {
        let potential_path = current_dir.join("generated-proto/permissions.json");
        if potential_path.exists() {
            return Ok(potential_path
                .to_str()
                .context("Failed to convert path to string")?
                .to_string());
        }
        current_dir = current_dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?
            .to_path_buf();
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MethodPermissions {
    allow_roles: Vec<i32>,
    is_public: bool,
    public_url: Option<String>,
    compiled_permissions_bitfield: i64,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct PermissionsOutput {
    permissions: BTreeMap<String, MethodPermissions>,
    hash: String,
}

// Two functions to output to stdout basically:
// 1. Take in a public url, outputs the route as an option
// 2. Take in a route, and the permissions of a user, and returns if the user is allowed on that route

fn write_public_to_route(perms: &PermissionsOutput) -> Option<String> {
    let mut s = String::new();
    s.push_str("pub fn get_route_from_public_url(url: &str) -> Option<String> {\n");

    for (route, perms) in &perms.permissions {
        if let Some(public_url) = &perms.public_url {
            s.push_str(&format!(
                "\tif url == \"{}\" {{ return Some(\"{}\".to_string()) }}\n",
                public_url, route
            ));
        } else {
            s.push_str(&format!(
                "\tif url == \"{}\" {{ return Some(\"{}\".to_string()) }}\n",
                route, route
            ));
        }
    }

    s.push_str("\tNone\n}");
    Some(s)
}

fn write_route_to_permissions_check(perms: &PermissionsOutput) -> Option<String> {
    let mut s = String::new();
    s.push_str(
        "pub fn check_permissions_for_route(route: &str, user_permissions: i64) -> bool {\n",
    );

    s.push_str("\tmatch route {\n");

    for (route, perms) in &perms.permissions {
        if perms.is_public {
            s.push_str(&format!("\t\t\"{}\" => return true,\n", route));
            continue;
        }
        // just check if the user_permissions fits the compiled_permissions_bitfield, since the bitfield is just an OR of all the permissions
        s.push_str(&format!(
            "\t\t\"{}\" => return user_permissions & {} == {},\n",
            route, perms.compiled_permissions_bitfield, perms.compiled_permissions_bitfield
        ));
    }

    s.push_str("\t\t_ => println!(\"Warning: route not found: {}\", route),\n");
    s.push_str("\t}\n\tfalse\n}");
    Some(s)
}

fn write_embedded_structs(perms: &PermissionsOutput) -> Option<String> {
    let mut s = String::new();

    s.push_str("use hxi2_proto::proto::auth::v2::Permission;\n\n");

    s.push_str("#[derive(Debug, Clone)]\n");
    s.push_str("pub struct MethodPermissions {\n");
    s.push_str("\tpub allow_roles: &'static [Permission],\n");
    s.push_str("\tpub is_public: bool,\n");
    s.push_str("\tpub public_url: Option<&'static str>,\n");
    s.push_str("\tpub compiled_permissions_bitfield: i64,\n");
    s.push_str("}\n\n");

    s.push_str("#[derive(Debug, Clone)]\n");
    s.push_str("pub struct CompiledPermissions {\n");
    s.push_str("\tpub permissions: &'static [(&'static str, MethodPermissions)],\n");
    s.push_str("\tpub hash: &'static str,\n");
    s.push_str("}\n\n");

    s.push_str("impl CompiledPermissions {\n");
    s.push_str("\tpub fn get_by_route(&self, route: &str) -> Option<&MethodPermissions> {\n");
    s.push_str("\t\tself.permissions\n");
    s.push_str("\t\t\t.binary_search_by_key(&route, |&(k, _)| k)\n");
    s.push_str("\t\t\t.ok()\n");
    s.push_str("\t\t\t.map(|idx| &self.permissions[idx].1)\n");
    s.push_str("\t}\n");
    s.push_str("}\n\n");

    let mut sorted_perms: Vec<(&String, &MethodPermissions)> = perms.permissions.iter().collect();
    sorted_perms.sort_by_key(|&(route, _)| route);

    s.push_str("pub fn get_compiled_permissions() -> &'static CompiledPermissions {\n");
    s.push_str("\tstatic INSTANCE: CompiledPermissions = CompiledPermissions {\n");
    s.push_str("\t\tpermissions: &[\n");

    for (route, perms) in sorted_perms {
        s.push_str(&format!("\t\t\t(\"{}\", MethodPermissions {{\n", route));
        s.push_str("\t\t\t\tallow_roles: &[\n");
        for role in &perms.allow_roles {
            s.push_str(&format!(
                "\t\t\t\t\tPermission::{},\n",
                Permission::from_i32(*role)
                    .unwrap_or(Permission::PermissionUnspecified)
                    .proto_name()
            ));
        }
        s.push_str("\t\t\t\t],\n");
        s.push_str(&format!("\t\t\t\tis_public: {},\n", perms.is_public));

        if let Some(public_url) = &perms.public_url {
            s.push_str(&format!("\t\t\t\tpublic_url: Some(\"{}\"),\n", public_url));
        } else {
            s.push_str("\t\t\t\tpublic_url: None,\n");
        }

        s.push_str(&format!(
            "\t\t\t\tcompiled_permissions_bitfield: {},\n",
            perms.compiled_permissions_bitfield
        ));
        s.push_str("\t\t\t}),\n");
    }

    s.push_str("\t\t],\n");
    s.push_str(&format!("\t\thash: \"{}\",\n", perms.hash));
    s.push_str("\t};\n\n");
    s.push_str("\t&INSTANCE\n");
    s.push('}');

    Some(s)
}

pub fn main(path: &str) -> Result<String> {
    let contents = std::fs::read_to_string(path).context("Failed to read permissions file")?;
    let perms: PermissionsOutput =
        serde_json::from_str(&contents).context("Failed to parse permissions JSON")?;

    let mut s = String::new();

    if let Some(route) = write_public_to_route(&perms) {
        s.push_str(&route);
    }

    if let Some(check) = write_route_to_permissions_check(&perms) {
        s.push_str("\n\n");
        s.push_str(&check);
    }

    if let Some(structs) = write_embedded_structs(&perms) {
        s.push_str("\n\n");
        s.push_str(&structs);
    }

    if s.is_empty() {
        return Err(anyhow::anyhow!("No permissions found to generate code for"));
    } else {
        s.insert_str(0, "#![allow(clippy::all, warnings)]\n// This file is generated by global-rs/generate-permissions-checking. Do not edit manually.\n\n");
    }
    Ok(s)
}
