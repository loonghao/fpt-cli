use fpt_core::{AppError, Result};
use serde::Serialize;
use std::{env, str::FromStr};

const DEFAULT_API_VERSION: &str = "v1.1";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Script,
    UserPassword,
    SessionToken,
}

impl AuthMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::UserPassword => "user_password",
            Self::SessionToken => "session_token",
        }
    }

    pub const fn grant_type(self) -> &'static str {
        match self {
            Self::Script => "client_credentials",
            Self::UserPassword => "password",
            Self::SessionToken => "session_token",
        }
    }
}

impl FromStr for AuthMode {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "script" | "client_credentials" => Ok(Self::Script),
            "user-password" | "user_password" | "password" | "user" => {
                Ok(Self::UserPassword)
            }
            "session-token" | "session_token" | "session" => Ok(Self::SessionToken),
            other => Err(AppError::invalid_input(format!(
                "unsupported auth mode `{other}`; expected one of: script / user-password / session-token"
            ))),

        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionOverrides {
    pub site: Option<String>,
    pub auth_mode: Option<AuthMode>,
    pub script_name: Option<String>,
    pub script_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_token: Option<String>,
    pub session_token: Option<String>,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Credentials {
    Script {
        script_name: String,
        script_key: String,
    },
    UserPassword {
        username: String,
        password: String,
        auth_token: Option<String>,
    },
    SessionToken {
        session_token: String,
    },
}

impl Credentials {
    pub const fn auth_mode(&self) -> AuthMode {
        match self {
            Self::Script { .. } => AuthMode::Script,
            Self::UserPassword { .. } => AuthMode::UserPassword,
            Self::SessionToken { .. } => AuthMode::SessionToken,
        }
    }

    pub fn principal(&self) -> Option<String> {
        match self {
            Self::Script { script_name, .. } => Some(script_name.clone()),
            Self::UserPassword { username, .. } => Some(username.clone()),
            Self::SessionToken { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionSettings {
    pub site: String,
    pub credentials: Credentials,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionSummary {
    pub site: String,
    pub auth_mode: AuthMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    pub api_version: String,
}

impl ConnectionSettings {
    pub fn resolve(overrides: ConnectionOverrides) -> Result<Self> {
        let env_site = env_var_compat("FPT_SITE", "SG_SITE");
        let env_auth_mode = env_var_compat("FPT_AUTH_MODE", "SG_AUTH_MODE");
        let env_script_name = env_var_compat("FPT_SCRIPT_NAME", "SG_SCRIPT_NAME");
        let env_script_key = env_var_compat("FPT_SCRIPT_KEY", "SG_SCRIPT_KEY");
        let env_username = env_var_compat("FPT_USERNAME", "SG_USERNAME");
        let env_password = env_var_compat("FPT_PASSWORD", "SG_PASSWORD");
        let env_auth_token = env_var_compat("FPT_AUTH_TOKEN", "SG_AUTH_TOKEN");
        let env_session_token = env_var_compat("FPT_SESSION_TOKEN", "SG_SESSION_TOKEN");
        let env_api_version = env_var_compat("FPT_API_VERSION", "SG_API_VERSION");


        let site = overrides.site.or(env_site).unwrap_or_default();
        let script_name = overrides.script_name.or(env_script_name).unwrap_or_default();
        let script_key = overrides.script_key.or(env_script_key).unwrap_or_default();
        let username = overrides.username.or(env_username).unwrap_or_default();
        let password = overrides.password.or(env_password).unwrap_or_default();
        let auth_token = overrides.auth_token.or(env_auth_token).unwrap_or_default();
        let session_token = overrides.session_token.or(env_session_token).unwrap_or_default();
        let api_version = api_version_or_default(
            overrides.api_version.as_deref().or(env_api_version.as_deref()),
        );
        let auth_mode = overrides
            .auth_mode
            .or(parse_optional_auth_mode(env_auth_mode.as_deref())?)
            .unwrap_or_else(|| infer_auth_mode(&username, &password, &auth_token, &session_token));

        let mut missing = Vec::new();
        if site.trim().is_empty() {
            missing.push("FPT_SITE / SG_SITE / --site");
        }


        let credentials = match auth_mode {
            AuthMode::Script => {
                if script_name.trim().is_empty() {
                    missing.push("FPT_SCRIPT_NAME / SG_SCRIPT_NAME / --script-name");
                }
                if script_key.trim().is_empty() {
                    missing.push("FPT_SCRIPT_KEY / SG_SCRIPT_KEY / --script-key");
                }

                Credentials::Script {
                    script_name,
                    script_key,
                }
            }
            AuthMode::UserPassword => {
                if username.trim().is_empty() {
                    missing.push("FPT_USERNAME / SG_USERNAME / --username");
                }
                if password.trim().is_empty() {
                    missing.push("FPT_PASSWORD / SG_PASSWORD / --password");
                }

                Credentials::UserPassword {
                    username,
                    password,
                    auth_token: (!auth_token.trim().is_empty()).then_some(auth_token),
                }
            }
            AuthMode::SessionToken => {
                if session_token.trim().is_empty() {
                    missing.push("FPT_SESSION_TOKEN / SG_SESSION_TOKEN / --session-token");
                }

                Credentials::SessionToken { session_token }
            }
        };

        if !missing.is_empty() {
            return Err(AppError::invalid_input("缺少 ShotGrid 连接配置").with_details(
                serde_json::json!({
                    "auth_mode": auth_mode,
                    "missing": missing,
                    "hint": "可通过命令行参数或环境变量提供认证信息"
                }),
            ));
        }

        Ok(Self {
            site: site.trim_end_matches('/').to_string(),
            credentials,
            api_version,
        })
    }

    pub fn auth_mode(&self) -> AuthMode {
        self.credentials.auth_mode()
    }

    pub fn summary(&self) -> ConnectionSummary {
        ConnectionSummary {
            site: self.site.clone(),
            auth_mode: self.auth_mode(),
            principal: self.credentials.principal(),
            api_version: self.api_version.clone(),
        }
    }
}

pub fn api_version_or_default(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_API_VERSION)
        .to_string()
}

pub fn resolve_site(overrides: ConnectionOverrides) -> Result<String> {
    let site = overrides
        .site
        .or(env_var_compat("FPT_SITE", "SG_SITE"))
        .unwrap_or_default();

    if site.trim().is_empty() {
        return Err(AppError::invalid_input("缺少 ShotGrid 连接配置").with_details(
            serde_json::json!({
                "missing": ["FPT_SITE / SG_SITE / --site"],
                "hint": "可通过命令行参数或环境变量提供站点信息"
            }),
        ));
    }

    Ok(site.trim_end_matches('/').to_string())
}

fn env_var_compat(primary: &str, fallback: &str) -> Option<String> {

    env::var(primary)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var(fallback)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn parse_optional_auth_mode(value: Option<&str>) -> Result<Option<AuthMode>> {

    value.map(AuthMode::from_str).transpose()
}

fn infer_auth_mode(
    username: &str,
    password: &str,
    auth_token: &str,
    session_token: &str,
) -> AuthMode {
    if !username.trim().is_empty() || !password.trim().is_empty() || !auth_token.trim().is_empty() {
        return AuthMode::UserPassword;
    }

    if !session_token.trim().is_empty() {
        return AuthMode::SessionToken;
    }

    AuthMode::Script
}
