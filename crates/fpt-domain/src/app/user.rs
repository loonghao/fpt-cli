use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::build_query_params;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn current_user(
        &self,
        overrides: ConnectionOverrides,
        user_type: &str,
        input: Option<Value>,
    ) -> Result<Value> {
        validate_user_type(user_type)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .current_user(&config, user_type, &params)
            .await
    }
}

fn validate_user_type(user_type: &str) -> Result<()> {
    match user_type {
        "HumanUser" | "human_user" | "human" | "ApiUser" | "api_user" | "api" => Ok(()),
        _ => Err(AppError::invalid_input(format!(
            "unsupported user type `{user_type}`; expected one of: HumanUser, ApiUser"
        ))
        .with_operation("current_user")
        .with_invalid_field("user_type")
        .with_expected_shape("one of: HumanUser, human_user, human, ApiUser, api_user, api")
        .with_received_value(user_type)),
    }
}
