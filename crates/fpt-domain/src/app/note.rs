use fpt_core::{AppError, Result};
use serde_json::{Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings};
use crate::transport::ShotgridTransport;

use super::App;

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn note_threads(
        &self,
        overrides: ConnectionOverrides,
        note_id: u64,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_note_query_params(input)?;
        self.transport
            .note_threads(&config, note_id, &params)
            .await
            .map_err(|error| translate_note_threads_error(error, note_id))
    }
}

fn translate_note_threads_error(error: AppError, note_id: u64) -> AppError {
    let envelope = error.envelope();
    let not_found = envelope.code == "API_ERROR"
        && envelope
            .details
            .as_ref()
            .and_then(|details| details.get("errors"))
            .and_then(Value::as_array)
            .is_some_and(|errors| {
                errors.iter().any(|entry| {
                    entry.get("status") == Some(&json!(404))
                        && entry
                            .get("detail")
                            .and_then(Value::as_str)
                            .is_some_and(|detail| {
                                detail.contains(&format!("Note: {note_id} not found"))
                            })
                })
            });

    if not_found {
        return AppError::api(format!(
            "Note thread lookup failed: `{note_id}` is not a top-level Note record id or the Note does not exist"
        ))
        .with_operation("note_threads")
        .with_transport(envelope.transport.unwrap_or_else(|| "rest".to_string()))
        .with_resource(format!("Note/{note_id}"))
        .with_detail("note_id", note_id)
        .with_hint("Verify that the id belongs to a top-level Note entity, not a reply or a different entity type.")
        .with_details(
            envelope
                .details
                .unwrap_or_else(|| json!({ "note_id": note_id })),
        );
    }

    error
}

/// Convert an optional JSON object into query string key-value pairs for note thread requests.
fn build_note_query_params(input: Option<Value>) -> Result<Vec<(String, String)>> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };

    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input("note threads input must be a JSON object")
            .with_operation("build_note_query_params")
            .with_expected_shape("a JSON object containing fields like `fields` or `entity_fields`")
    })?;

    let mut params: Vec<(String, String)> = Vec::new();

    for (key, value) in object {
        match key.as_str() {
            "fields" | "entity_fields" => {
                let joined = string_list_to_csv(value, key)?;
                if !joined.is_empty() {
                    params.push((key.clone(), joined));
                }
            }
            _ => {
                let s = scalar_to_string(value, key)?;
                params.push((key.clone(), s));
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
            "`{field_name}` must be a scalar value (string, number, or bool)"
        ))
        .with_operation("build_note_query_params")
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
            "`{field_name}` must be a string or an array of strings"
        ))
        .with_operation("build_note_query_params")
        .with_invalid_field(field_name)
        .with_expected_shape("either a string or an array of strings")
    })?;
    let items: Result<Vec<String>> = array
        .iter()
        .map(|v| {
            v.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` items must be strings"))
                    .with_operation("build_note_query_params")
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect();
    Ok(items?.join(","))
}
