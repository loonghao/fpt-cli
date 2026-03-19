use crate::cli::{ConfigClearArgs, ConfigCommands, ConfigSetArgs};
use fpt_core::{AppError, Result};
use fpt_domain::{
    PersistedConnectionConfig, config_file_path, load_persisted_config, save_persisted_config,
};
use serde_json::{Value, json};

pub fn run(command: ConfigCommands) -> Result<Value> {
    match command {
        ConfigCommands::Get => get_config(),
        ConfigCommands::Path => {
            let path = config_file_path()?;
            Ok(json!({
                "command": "config.path",
                "path": path.display().to_string(),
            }))
        }
        ConfigCommands::Set(args) => set_config(args),
        ConfigCommands::Clear(args) => clear_config(args),
    }
}

fn get_config() -> Result<Value> {
    let path = config_file_path()?;
    let config = load_persisted_config()?;
    Ok(json!({
        "command": "config.get",
        "path": path.display().to_string(),
        "config": config,
    }))
}

fn set_config(args: ConfigSetArgs) -> Result<Value> {
    if !has_any_set_arg(&args) {
        return Err(AppError::invalid_input(
            "`config set` requires at least one field to persist; pass options such as `--site`, `--auth-mode`, or credentials flags",
        ));
    }

    let mut config = load_persisted_config()?;
    if let Some(site) = args.site {
        config.site = Some(site.trim().trim_end_matches('/').to_string());
    }
    if let Some(auth_mode) = args.auth_mode {
        config.auth_mode = Some(auth_mode.into());
    }
    if let Some(script_name) = args.script_name {
        config.script_name = Some(script_name.trim().to_string());
    }
    if let Some(script_key) = args.script_key {
        config.script_key = Some(script_key.trim().to_string());
    }
    if let Some(username) = args.username {
        config.username = Some(username.trim().to_string());
    }
    if let Some(password) = args.password {
        config.password = Some(password);
    }
    if let Some(auth_token) = args.auth_token {
        config.auth_token = Some(auth_token.trim().to_string());
    }
    if let Some(session_token) = args.session_token {
        config.session_token = Some(session_token.trim().to_string());
    }
    if let Some(api_version) = args.api_version {
        config.api_version = Some(api_version.trim().to_string());
    }

    let path = save_persisted_config(&config)?;
    Ok(json!({
        "command": "config.set",
        "path": path.display().to_string(),
        "config": config,
    }))
}

/// All field names accepted by `config clear --fields`.
const VALID_CLEAR_FIELDS: &[&str] = &[
    "site",
    "auth-mode",
    "script-name",
    "script-key",
    "username",
    "password",
    "auth-token",
    "session-token",
    "api-version",
];

fn clear_config(args: ConfigClearArgs) -> Result<Value> {
    if !args.all && args.fields.is_empty() {
        return Err(AppError::invalid_input(
            "`config clear` requires `--all` or `--fields <name,...>`; \
             valid field names: site, auth-mode, script-name, script-key, \
             username, password, auth-token, session-token, api-version",
        ));
    }

    // Validate field names before doing any work.
    for name in &args.fields {
        if !VALID_CLEAR_FIELDS.contains(&name.as_str()) {
            return Err(AppError::invalid_input(format!(
                "unknown field name `{name}`; valid names: {}",
                VALID_CLEAR_FIELDS.join(", ")
            )));
        }
    }

    let mut config = load_persisted_config()?;
    if args.all {
        config = PersistedConnectionConfig::default();
    } else {
        let f = &args.fields;
        let has = |name: &str| f.iter().any(|n| n == name);
        if has("site") {
            config.site = None;
        }
        if has("auth-mode") {
            config.auth_mode = None;
        }
        if has("script-name") {
            config.script_name = None;
        }
        if has("script-key") {
            config.script_key = None;
        }
        if has("username") {
            config.username = None;
        }
        if has("password") {
            config.password = None;
        }
        if has("auth-token") {
            config.auth_token = None;
        }
        if has("session-token") {
            config.session_token = None;
        }
        if has("api-version") {
            config.api_version = None;
        }
    }

    let path = save_persisted_config(&config)?;
    Ok(json!({
        "command": "config.clear",
        "path": path.display().to_string(),
        "config": config,
    }))
}

fn has_any_set_arg(args: &ConfigSetArgs) -> bool {
    args.site.is_some()
        || args.auth_mode.is_some()
        || args.script_name.is_some()
        || args.script_key.is_some()
        || args.username.is_some()
        || args.password.is_some()
        || args.auth_token.is_some()
        || args.session_token.is_some()
        || args.api_version.is_some()
}
