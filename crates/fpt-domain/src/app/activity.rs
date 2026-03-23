use fpt_core::{AppError, Result};
use serde_json::Value;

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;
use super::query_helpers::{scalar_to_string, string_list_to_csv};

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

/// Public wrapper for `build_query_params` — used by entity_relationships
/// and user_following to convert optional JSON input into query pairs.
pub fn build_query_params_public(input: Option<Value>) -> Result<Vec<(String, String)>> {
    build_query_params(input)
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
