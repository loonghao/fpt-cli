use fpt_core::{AppError, Result};
use serde_json::Value;

/// Convert a JSON scalar (string, number, or bool) to its string representation.
///
/// Used for query-string parameter encoding where all values must be strings.
pub fn scalar_to_string(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err(AppError::invalid_input(format!(
            "`{field_name}` must be a scalar query value of type string, number, or boolean"
        ))
        .with_invalid_field(field_name)
        .with_expected_shape("a scalar value of type string, number, or boolean")),
    }
}

/// Convert a JSON value that is either a comma-separated string or an array of
/// strings into a single comma-joined string.
///
/// This is the variant used by activity and note query parameter builders.
pub fn string_list_to_csv(value: &Value, field_name: &str) -> Result<String> {
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    let array = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be either a comma-separated string or an array of strings"
        ))
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;
    let items: Result<Vec<String>> = array
        .iter()
        .map(|v| {
            v.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect();
    Ok(items?.join(","))
}

/// Parse a JSON value into a `Vec<String>`.
///
/// Accepts either a comma-separated string or a JSON array of strings.
/// Empty entries are filtered out.
pub fn string_list(value: &Value, field_name: &str) -> Result<Vec<String>> {
    if let Some(value) = value.as_str() {
        let items = value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect();
        return Ok(items);
    }

    let array = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be either a comma-separated string or an array of strings"
        ))
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect()
}
