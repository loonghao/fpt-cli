//! Shared helpers for converting JSON values into query-string representations.
//!
//! Several application modules (`activity`, `note`, `find`, `batch`) need to
//! convert user-supplied JSON values into flat query-string pairs.  This module
//! centralises the duplicated `scalar_to_string`, `string_list_to_csv`, and
//! `string_list` utilities so there is a single authoritative implementation.

use fpt_core::{AppError, Result};
use serde_json::Value;

/// Convert a JSON scalar (string | number | bool) to its string representation.
///
/// Returns a helpful error if the value is not one of the supported scalar types.
/// `operation` and `field_name` are used to annotate the error message.
pub(crate) fn scalar_to_string(
    value: &Value,
    field_name: &str,
    operation: &str,
) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Err(AppError::invalid_input(format!(
            "`{field_name}` must be a scalar value (string, number, or bool)"
        ))
        .with_operation(operation)
        .with_invalid_field(field_name)
        .with_expected_shape("a scalar value of type string, number, or boolean")),
    }
}

/// Accept a value that is either a plain string or a JSON array of strings and
/// return the items joined into a single comma-separated string.
///
/// - A plain string is returned as-is.
/// - An array of strings is joined with `,`.
///
/// Returns an error if the value is neither a string nor an array, or if the
/// array contains non-string items.
pub(crate) fn string_list_to_csv(
    value: &Value,
    field_name: &str,
    operation: &str,
) -> Result<String> {
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    let array = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!(
            "`{field_name}` must be either a comma-separated string or an array of strings"
        ))
        .with_operation(operation)
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;
    let items: Result<Vec<String>> = array
        .iter()
        .map(|v| {
            v.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_operation(operation)
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect();
    Ok(items?.join(","))
}

/// Accept a value that is either a comma-separated string or a JSON array of
/// strings and return a `Vec<String>` of individual items.
///
/// - A comma-separated string is split on `,`, trimmed, and empty items removed.
/// - A JSON array of strings is validated and converted directly.
pub(crate) fn string_list(
    value: &Value,
    field_name: &str,
    operation: &str,
) -> Result<Vec<String>> {
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
        .with_operation(operation)
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_operation(operation)
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect()
}
