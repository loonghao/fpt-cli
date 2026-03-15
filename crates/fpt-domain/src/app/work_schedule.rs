use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn work_schedule_read(
        &self,
        overrides: ConnectionOverrides,
        input: Value,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = normalize_work_schedule_input(input)?;
        self.transport.work_schedule_read(&config, &payload).await
    }
}

fn normalize_work_schedule_input(input: Value) -> Result<Value> {
    let object = input
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::invalid_input("work-schedule read input must be a JSON object"))?;

    require_non_empty_string(&object, "start_date")?;
    require_non_empty_string(&object, "end_date")?;

    if let Some(project) = object.get("project") {
        validate_entity_link(project, "project", "Project")?;
    }

    if let Some(user) = object.get("user") {
        validate_entity_link(user, "user", "HumanUser")?;
    }

    Ok(Value::Object(object))
}

fn require_non_empty_string(
    object: &serde_json::Map<String, Value>,
    field_name: &str,
) -> Result<()> {
    let value = object.get(field_name).ok_or_else(|| {
        AppError::invalid_input(format!(
            "work-schedule read requires a `{field_name}` field"
        ))
    })?;

    let value = value.as_str().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be a string in YYYY-MM-DD format"
        ))
    })?;

    if value.trim().is_empty() {
        return Err(AppError::invalid_input(format!(
            "`{field_name}` cannot be empty"
        )));
    }

    Ok(())
}

fn validate_entity_link(value: &Value, field_name: &str, expected_type: &str) -> Result<()> {
    let object = value.as_object().ok_or_else(|| {
        AppError::invalid_input(format!("`{field_name}` must be an entity link object"))
    })?;

    let link_type = object.get("type").and_then(Value::as_str).ok_or_else(|| {
        AppError::invalid_input(format!("`{field_name}` must contain a string `type` field"))
    })?;
    if link_type != expected_type {
        return Err(AppError::invalid_input(format!(
            "`{field_name}.type` must be `{expected_type}`"
        )));
    }

    object.get("id").and_then(Value::as_u64).ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must contain a positive integer `id` field"
        ))
    })?;

    Ok(())
}
