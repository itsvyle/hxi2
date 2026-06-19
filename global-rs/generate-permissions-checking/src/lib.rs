use anyhow::{Context as _, Result};
use buffa::Enumeration;
use indoc::indoc;
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
    public_url: Option<Vec<String>>,
    compiled_permissions_bitfield: i64,
    csrf_token_header: Option<String>,
    csrf_token_cookie: Option<String>,
    response_cors_headers: Option<BTreeMap<String, String>>,
    enforce_csrf: bool,
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
    let mut if_statements = String::new();

    for (route, perms) in &perms.permissions {
        let mut match_url = perms.public_url.clone().unwrap_or_default();
        match_url.push(route.clone());

        if_statements.push_str(&format!(
            r#"{} => return Some("{}"),{}"#,
            match_url
                .iter()
                .map(|url| format!("\"{}\"", url))
                .collect::<Vec<_>>()
                .join(" | "),
            route,
            "\n"
        ));
    }

    if_statements.push_str("_ => None,");

    Some(format!(
        indoc! {r#"
            pub fn get_route_from_public_url(url: &str) -> Option<&'static str> {{
                match url {{
                    {}
                }}
            }}
        "#},
        if_statements
    ))
}

fn write_embedded_structs(perms: &PermissionsOutput) -> Option<String> {
    let mut sorted_perms: Vec<(&String, &MethodPermissions)> = perms.permissions.iter().collect();
    sorted_perms.sort_by_key(|&(route, _)| route);

    let mut array_entries = String::new();
    for (route, perm_data) in &sorted_perms {
        let mut roles_list = String::new();
        for role in &perm_data.allow_roles {
            let role_name = Permission::from_i32(*role)
                .unwrap_or(Permission::PermissionUnspecified)
                .proto_name();
            roles_list.push_str(&format!("\t\t\t\t\tPermission::{},\n", role_name));
        }

        let public_url_val = match &perm_data.public_url {
            Some(url) => format!(
                "Some(&[{}])",
                url.iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None => "None".to_string(),
        };

        array_entries.push_str(&format!(
            indoc! {r#"
                        ("{route}", MethodPermissions {{
                            allow_roles: &[
            {roles_list}                ],
                            is_public: {is_public},
                            public_url: {public_url},
                            compiled_permissions_bitfield: {bitfield},
                            csrf_token_header: {csrf_header},
                            csrf_token_cookie: {csrf_cookie},
                            response_cors_headers: {cors_headers},
                            enforce_csrf: {enforce_csrf},
                        }}),
            "#},
            route = route,
            roles_list = roles_list,
            is_public = perm_data.is_public,
            public_url = public_url_val,
            bitfield = perm_data.compiled_permissions_bitfield,
            csrf_header = match &perm_data.csrf_token_header {
                Some(header) => format!("Some(\"{}\")", header),
                None => "None".to_string(),
            },
            csrf_cookie = match &perm_data.csrf_token_cookie {
                Some(cookie) => format!("Some(\"{}\")", cookie),
                None => "None".to_string(),
            },
            cors_headers = match &perm_data.response_cors_headers {
                Some(headers) => {
                    let mut headers_str = String::from("Some(BTreeMap::from([");
                    for (key, value) in headers {
                        headers_str.push_str(&format!("(\"{}\", \"{}\"), ", key, value));
                    }
                    headers_str.push_str("]))");
                    headers_str
                }
                None => "None".to_string(),
            },
            enforce_csrf = perm_data.enforce_csrf
        ));
    }

    let mut match_arms = String::new();
    for (idx, (route, _)) in sorted_perms.iter().enumerate() {
        match_arms.push_str(&format!(
            "\t\t\t\t\"{}\" => return Some(&self.permissions[{}].1),\n",
            route, idx
        ));
    }

    Some(format!(
        indoc! {r#"
            use std::collections::BTreeMap;
            use hxi2_proto::proto::auth::v2::Permission;

            #[derive(Debug, Clone)]
            pub struct MethodPermissions {{
                pub allow_roles: &'static [Permission],
                pub is_public: bool,
                pub public_url: Option<&'static [&'static str]>,
                pub compiled_permissions_bitfield: i64,
                pub csrf_token_header: Option<&'static str>,
                pub csrf_token_cookie: Option<&'static str>,
                pub response_cors_headers: Option<BTreeMap<&'static str, &'static str>>,
                pub enforce_csrf: bool,
            }}

            pub trait MethodPermissionsOptionExt {{
                fn check_permissions(&self, user_permissions: i64) -> bool;
            }}

            impl MethodPermissionsOptionExt for MethodPermissions {{
                fn check_permissions(&self, user_permissions: i64) -> bool {{
                    if self.is_public {{
                        return true;
                    }}
                    (user_permissions & self.compiled_permissions_bitfield) > 0
                }}
            }}

            impl MethodPermissionsOptionExt for Option<&MethodPermissions> {{
                fn check_permissions(&self, user_permissions: i64) -> bool {{
                    match self {{
                        Some(perm) => perm.check_permissions(user_permissions),
                        None => false,
                    }}
                }}
            }}

            #[derive(Debug, Clone)]
            pub struct CompiledPermissions {{
                pub permissions: &'static [(&'static str, MethodPermissions)],
                pub hash: &'static str,
            }}

            impl CompiledPermissions {{
                pub fn get_by_route(&self, route: &str) -> Option<&MethodPermissions> {{
                    match route {{
            {}            _ => None,
                    }}
                }}
            }}

            pub fn get_compiled_permissions() -> &'static CompiledPermissions {{
                static INSTANCE: CompiledPermissions = CompiledPermissions {{
                    permissions: &[
            {}        ],
                    hash: "{}",
                }};

                &INSTANCE
            }}

            pub fn get_by_route(route: &str) -> Option<&'static MethodPermissions> {{
                get_compiled_permissions().get_by_route(route)
            }}
        "#},
        match_arms, array_entries, perms.hash
    ))
}

pub fn main(path: &str) -> Result<String> {
    let contents = std::fs::read_to_string(path).context("Failed to read permissions file")?;
    let perms: PermissionsOutput =
        serde_json::from_str(&contents).context("Failed to parse permissions JSON")?;

    let mut s = String::new();

    if let Some(route) = write_public_to_route(&perms) {
        s.push_str(&route);
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
