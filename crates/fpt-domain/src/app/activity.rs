use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn activity_stream(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport
            .activity_stream(&config, entity, id, &params)
            .await
    }

    pub async fn event_log_entries(
        &self,
        overrides: ConnectionOverrides,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_query_params(input)?;
        self.transport.event_log_entries(&config, &params).await
    }

    pub async fn preferences_get(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.preferences_get(&config).await
    }
}

/// Convert an optional JSON object into a flat list of query string key-value pairs.
///
/// Supported shapes:
/// - `{"fields": "id,event_type"}` or `{"fields": ["id", "event_type"]}`
/// - `{"sort": "-created_at"}`
/// - `{"page": {"number": 1, "size": 50}}`
/// - `{"entity_fields": "code,sg_status_list"}`
/// - Any other top-level string/number/bool scalar key is forwarded as-is.
fn build_query_params(input: Option<Value>) -> Result<Vec<(String, String)>> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };

    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input(
            "activity and event-log input must be a JSON object containing query parameters",
        )
        .with_operation("build_query_params")
        .with_expected_shape(
            "a JSON object containing fields like `fields`, `entity_fields`, `sort`, or `page`",
        )
    })?;

    let mut params: Vec<(String, String)> = Vec::new();

    for (key, value) in object {
        match key.as_str() {
            "page" => {
                let page = value.as_object().ok_or_else(|| {
                    AppError::invalid_input(
                        "`page` must be a JSON object like `{\"number\": 1, \"size\": 50}`",
                    )
                    .with_operation("build_query_params")
                    .with_invalid_field("page")
                    .with_expected_shape("a JSON object like `{\"number\": 1, \"size\": 50}`")
                })?;
                if let Some(number) = page.get("number") {
                    params.push((
                        "page[number]".to_string(),
                        scalar_to_string(number, "page.number")?,
                    ));
                }
                if let Some(size) = page.get("size") {
                    params.push((
                        "page[size]".to_string(),
                        scalar_to_string(size, "page.size")?,
                    ));
                }
            }
            "fields" | "entity_fields" => {
                let joined = string_list_to_csv(value, key)?;
                if !joined.is_empty() {
                    params.push((key.clone(), joined));
                }
            }
            _ => {
                params.push((key.clone(), scalar_to_string(value, key)?));
            }
        }
    }

    Ok(params)
}

fn scalar_to_string(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err(AppError::invalid_input(format!(
            "`{field_name}` must be a scalar query value of type string, number, or boolean"
        ))
        .with_operation("build_query_params")
        .with_invalid_field(field_name)
        .with_expected_shape("a scalar value of type string, number, or boolean")),
    }
}

fn string_list_to_csv(value: &Value, field_name: &str) -> Result<String> {
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    let array = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be either a comma-separated string or an array of strings"
        ))
        .with_operation("build_query_params")
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;
    let items: Result<Vec<String>> = array
        .iter()
        .map(|v| {
            v.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_operation("build_query_params")
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect();
    items.map(|items| items.join(","))
}
