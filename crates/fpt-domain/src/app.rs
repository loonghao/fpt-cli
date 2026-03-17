mod activity;
mod auth;
pub mod batch;
mod entity;
mod find;
mod follow;
mod hierarchy;
mod note;
mod schema;
mod server;
mod summarize;
mod upload;
mod work_schedule;

use std::env;

use fpt_core::{AppError, Result};
use serde_json::{Value, json};

use crate::capability::{command_specs, find_command_spec};
use crate::transport::{RestTransport, ShotgridTransport};

pub struct App<T = RestTransport> {
    transport: T,
}

impl Default for App<RestTransport> {
    fn default() -> Self {
        Self {
            transport: RestTransport::default(),
        }
    }
}

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn capabilities(&self) -> Value {
        json!({
            "name": "fpt",
            "version": env!("CARGO_PKG_VERSION"),
            "transports": [
                { "name": "rest", "status": "available" },
                { "name": "rpc", "status": "planned" }
            ],
            "required_environment": [
                "FPT_SITE"
            ],
            "optional_environment": [
                "FPT_AUTH_MODE",
                "FPT_SCRIPT_NAME",
                "FPT_SCRIPT_KEY",
                "FPT_USERNAME",
                "FPT_PASSWORD",
                "FPT_AUTH_TOKEN",
                "FPT_SESSION_TOKEN",
                "FPT_API_VERSION"
            ],
            "auth_modes": [
                {
                    "name": "script",
                    "grant_type": "client_credentials",
                    "required": ["FPT_SCRIPT_NAME", "FPT_SCRIPT_KEY"]
                },
                {
                    "name": "user_password",
                    "grant_type": "password",
                    "required": ["FPT_USERNAME", "FPT_PASSWORD"],
                    "optional": ["FPT_AUTH_TOKEN"]
                },
                {
                    "name": "session_token",
                    "grant_type": "session_token",
                    "required": ["FPT_SESSION_TOKEN"]
                }
            ],
            "commands": command_specs(),
        })
    }

    pub fn inspect_command(&self, name: &str) -> Result<Value> {
        let spec = find_command_spec(name)
            .ok_or_else(|| AppError::unsupported(format!("未知命令 `{name}`")))?;
        Ok(json!(spec))
    }
}

const DEFAULT_BATCH_CONCURRENCY: usize = 8;
const MAX_BATCH_CONCURRENCY: usize = 32;

fn batch_concurrency_limit() -> usize {
    env::var("FPT_BATCH_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .map(|value| value.min(MAX_BATCH_CONCURRENCY))
        .unwrap_or(DEFAULT_BATCH_CONCURRENCY)
}

fn sort_batch_results(results: &mut [Value]) {
    results.sort_by_key(batch_result_index);
}

fn batch_result_index(value: &Value) -> usize {
    value
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX)
}

#[derive(Debug, Clone)]
struct BatchUpdateItem {
    id: u64,
    body: Value,
}
