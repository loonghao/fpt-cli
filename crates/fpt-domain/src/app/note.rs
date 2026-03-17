use fpt_core::{AppError, Result};
use serde_json::Value;

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
    }
}

/// Convert an optional JSON object into query string key-value pairs for note thread requests.
fn build_note_query_params(input: Option<Value>) -> Result<Vec<(String, String)>> {
    let Some(input) = input else {
        return Ok(Vec::new());
    };

    let object = input
        .as_object()
        .ok_or_else(|| AppError::invalid_input("note threads input must be a JSON object"))?;

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
        ))),
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
    })?;
    let items: Result<Vec<String>> = array
        .iter()
        .map(|v| {
            v.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| AppError::invalid_input(format!("`{field_name}` items must be strings")))
        })
        .collect();
    Ok(items?.join(","))
}
