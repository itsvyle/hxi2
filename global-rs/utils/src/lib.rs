use anyhow::Result;

// if the var is empty, return the default; if parsing fails, return an error; otherwise return the parsed value
pub fn cfg_from_env_or<T: std::str::FromStr + Default>(
    env_var: &str,
    default: Option<T>,
) -> Result<T> {
    match std::env::var(env_var) {
        Ok(val) if !val.is_empty() => val.parse::<T>().map_err(|_| {
            anyhow::anyhow!("Failed to parse env var {} with value `{}`", env_var, val)
        }),
        _ => match default {
            Some(default_val) => Ok(default_val),
            None => Err(anyhow::anyhow!(
                "Env var {} is not set and no default value is provided",
                env_var
            )),
        },
    }
}
