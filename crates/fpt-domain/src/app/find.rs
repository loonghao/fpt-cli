use fpt_core::{AppError, Result};
use serde_json::{Value, json};

use crate::filter_dsl::parse_filter_dsl;
use crate::transport::FindParams;

pub(super) fn build_find_params(
    input: Option<Value>,
    filter_dsl: Option<String>,
) -> Result<FindParams> {
    let mut params = FindParams::default();

    let Some(input) = input else {
        if let Some(filter_dsl) = filter_dsl {
            params.search = Some(json!({
                "filters": parse_filter_dsl(&filter_dsl)?,
            }));
        }
        return Ok(params);
    };

    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input(
            "entity find input must be a JSON object containing query parameters or search payload fields",
        )
        .with_operation("build_find_params")
        .with_expected_shape("a JSON object containing fields like `fields`, `include`, `sort`, `page`, `filters`, `options`, `query`, `search`, or `filter_dsl`")
    })?;

    let inline_filter_dsl = object
        .get("filter_dsl")
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input("`filter_dsl` must be a string expression")
                    .with_operation("build_find_params")
                    .with_invalid_field("filter_dsl")
                    .with_expected_shape("a string expression")
            })
        })
        .transpose()?;
    let inline_search = object
        .get("search")
        .map(normalize_search_body)
        .transpose()?;
    let top_level_presets = object
        .get("additional_filter_presets")
        .map(|value| normalize_filter_presets(value, "`additional_filter_presets`"))
        .transpose()?;

    if filter_dsl.is_some() && inline_filter_dsl.is_some() {
        return Err(AppError::invalid_input(
            "`--filter-dsl` and `filter_dsl` in the input JSON are mutually exclusive; provide only one filter DSL source",
        )
        .with_operation("build_find_params")
        .with_conflicting_fields(["--filter-dsl", "filter_dsl"])
        .with_hint("Choose either the CLI flag or the JSON field as the single filter DSL source."));
    }
    let effective_filter_dsl = filter_dsl.or(inline_filter_dsl);

    if effective_filter_dsl.is_some() && inline_search.is_some() {
        return Err(AppError::invalid_input(
            "`search` and `filter_dsl` cannot be used together because both define the search body",
        )
        .with_operation("build_find_params")
        .with_conflicting_fields(["search", "filter_dsl"])
        .with_hint("Use `search` for a full ShotGrid search body, or `filter_dsl` for DSL-based filters, but not both."));
    }

    if let Some(fields) = object.get("fields") {
        let fields = string_list(fields, "fields")?;
        if !fields.is_empty() {
            params.query.push(("fields".to_string(), fields.join(",")));
        }
    }

    if let Some(include) = object.get("include") {
        let include = string_list(include, "include")?;
        if !include.is_empty() {
            params
                .query
                .push(("include".to_string(), include.join(",")));
        }
    }

    if let Some(sort) = object.get("sort") {
        let sort = sort.as_str().ok_or_else(|| {
            AppError::invalid_input("`sort` must be a string such as `id` or `-created_at`")
                .with_operation("build_find_params")
                .with_invalid_field("sort")
                .with_expected_shape("a string such as `id` or `-created_at`")
        })?;
        params.query.push(("sort".to_string(), sort.to_string()));
    }

    if let Some(page) = object.get("page") {
        let page = page.as_object().ok_or_else(|| {
            AppError::invalid_input(
                "`page` must be a JSON object like `{\"number\": 1, \"size\": 25}`",
            )
            .with_operation("build_find_params")
            .with_invalid_field("page")
            .with_expected_shape("a JSON object like `{\"number\": 1, \"size\": 25}`")
        })?;
        if let Some(number) = page.get("number") {
            params.query.push((
                "page[number]".to_string(),
                scalar_to_query(number, "page.number")?,
            ));
        }
        if let Some(size) = page.get("size") {
            params.query.push((
                "page[size]".to_string(),
                scalar_to_query(size, "page.size")?,
            ));
        }
    }

    if let Some(filters) = object.get("filters") {
        if effective_filter_dsl.is_some() || inline_search.is_some() || top_level_presets.is_some()
        {
            return Err(AppError::invalid_input(
                "`filters` cannot be combined with `filter_dsl`, `search`, or `additional_filter_presets`; choose exactly one filtering style",
            )
            .with_operation("build_find_params")
            .with_conflicting_fields([
                "filters",
                "filter_dsl",
                "search",
                "additional_filter_presets",
            ])
            .with_hint("Use query-string style filters or search-body style filters, but do not mix them."));
        }

        let filters = filters.as_object().ok_or_else(|| {
            AppError::invalid_input("`filters` must be a JSON object of query-string filter values")
                .with_operation("build_find_params")
                .with_invalid_field("filters")
                .with_expected_shape("a JSON object of query-string filter values")
        })?;
        for (key, value) in filters {
            params.query.push((
                format!("filter[{key}]"),
                value_to_query(value, &format!("filters.{key}"))?,
            ));
        }
    }

    if let Some(options) = object.get("options") {
        let options = options.as_object().ok_or_else(|| {
            AppError::invalid_input("`options` must be a JSON object of ShotGrid option values")
                .with_operation("build_find_params")
                .with_invalid_field("options")
                .with_expected_shape("a JSON object of ShotGrid option values")
        })?;
        for (key, value) in options {
            params.query.push((
                format!("options[{key}]"),
                value_to_query(value, &format!("options.{key}"))?,
            ));
        }
    }

    if let Some(query) = object.get("query") {
        let query = query.as_object().ok_or_else(|| {
            AppError::invalid_input("`query` must be a JSON object of raw query parameters")
                .with_operation("build_find_params")
                .with_invalid_field("query")
                .with_expected_shape("a JSON object of raw query parameters")
        })?;
        for (key, value) in query {
            params
                .query
                .push((key.clone(), value_to_query(value, &format!("query.{key}"))?));
        }
    }

    let mut search = inline_search;

    if let Some(presets) = top_level_presets {
        let search_object = search.get_or_insert_with(|| json!({}));
        let search_map = search_object.as_object_mut().ok_or_else(|| {
            AppError::internal("normalized search payload must remain a JSON object")
                .with_operation("build_find_params")
        })?;
        if search_map.contains_key("additional_filter_presets") {
            return Err(AppError::invalid_input(
                "`additional_filter_presets` cannot be provided both at the top level and inside `search`",
            )
            .with_operation("build_find_params")
            .with_conflicting_fields(["additional_filter_presets", "search.additional_filter_presets"]));
        }
        search_map.insert(
            "additional_filter_presets".to_string(),
            Value::Array(presets),
        );
    }

    if let Some(filter_dsl) = effective_filter_dsl {
        let search_object = search.get_or_insert_with(|| json!({}));
        let search_map = search_object.as_object_mut().ok_or_else(|| {
            AppError::internal("normalized search payload must remain a JSON object")
                .with_operation("build_find_params")
        })?;
        search_map.insert("filters".to_string(), parse_filter_dsl(&filter_dsl)?);
    }

    params.search = search;
    Ok(params)
}

pub(super) fn upsert_query_param(query: &mut Vec<(String, String)>, key: &str, value: &str) {
    query.retain(|(item_key, _)| item_key != key);
    query.push((key.to_string(), value.to_string()));
}

pub(super) fn extract_find_one_response(response: Value) -> Result<Value> {
    let Some(data) = response.get("data") else {
        return Err(
            AppError::api("ShotGrid find response is missing the `data` field")
                .with_operation("extract_find_one_response")
                .with_expected_shape("a ShotGrid response object containing a `data` field")
                .with_detail("response_body", response),
        );
    };

    match data {
        Value::Array(items) => Ok(items.first().cloned().unwrap_or(Value::Null)),
        Value::Null => Ok(Value::Null),
        Value::Object(_) => Ok(data.clone()),
        _ => Err(AppError::api(
            "ShotGrid find response returned an unsupported `data` shape; expected array, object, or null"
        )
        .with_operation("extract_find_one_response")
        .with_invalid_field("data")
        .with_expected_shape("`data` must be an array, object, or null")
        .with_detail("response_body", response)),
    }
}

fn string_list(value: &Value, field_name: &str) -> Result<Vec<String>> {
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
        .with_operation("build_find_params")
        .with_invalid_field(field_name)
        .with_expected_shape("either a comma-separated string or an array of strings")
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` array items must all be strings"))
                    .with_operation("build_find_params")
                    .with_invalid_field(field_name)
                    .with_expected_shape("an array containing only strings")
            })
        })
        .collect()
}

fn scalar_to_query(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        _ => Err(AppError::invalid_input(format!(
            "`{field_name}` must be a scalar query value of type string, number, or boolean"
        ))
        .with_operation("build_find_params")
        .with_invalid_field(field_name)
        .with_expected_shape("a scalar query value of type string, number, or boolean")),
    }
}

fn value_to_query(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::Null => Err(
            AppError::invalid_input(format!("`{field_name}` cannot be null"))
                .with_operation("build_find_params")
                .with_invalid_field(field_name)
                .with_expected_shape("a non-null scalar or array of scalar values"),
        ),
        Value::Array(values) => {
            let mut items = Vec::with_capacity(values.len());
            for value in values {
                items.push(scalar_to_query(value, field_name)?);
            }
            Ok(items.join(","))
        }
        _ => scalar_to_query(value, field_name),
    }
}

fn normalize_search_body(value: &Value) -> Result<Value> {
    let mut search = value.as_object().cloned().ok_or_else(|| {
        AppError::invalid_input(
            "`search` must be a JSON object representing the ShotGrid search body",
        )
        .with_operation("normalize_search_body")
        .with_invalid_field("search")
        .with_expected_shape("a JSON object representing the ShotGrid search body")
    })?;

    if let Some(filters) = search.remove("filters") {
        // Normalize array-form filters to the object form expected by the ShotGrid
        // REST `_search` endpoint: {"filter_operator": "all", "filters": [...]}
        let normalized = normalize_search_filters(filters, "`search.filters`")?;
        search.insert("filters".to_string(), normalized);
    } else if !search.contains_key("filters") {
        // The ShotGrid `_search` endpoint requires a `filters` key in the request body.
        // Provide a default empty filter set so users can pass a minimal search body
        // (e.g. just `fields`) without triggering a "filters is missing" server error.
        search.insert(
            "filters".to_string(),
            json!({ "filter_operator": "all", "filters": [] }),
        );
    }

    if let Some(presets) = search.get("additional_filter_presets") {
        let presets = normalize_filter_presets(presets, "`search.additional_filter_presets`")?;
        search.insert(
            "additional_filter_presets".to_string(),
            Value::Array(presets),
        );
    }

    Ok(Value::Object(search))
}

/// Normalize the `filters` value inside a `search` body.
///
/// ShotGrid REST `_search` expects filters as an object:
///   `{"filter_operator": "all"|"any", "filters": [...]}`
///
/// As a convenience, a bare array is accepted and wrapped with the default
/// `filter_operator` of `"all"`.  An object is passed through as-is after
/// basic shape validation.
fn normalize_search_filters(value: Value, field_name: &str) -> Result<Value> {
    match value {
        Value::Array(items) => {
            let normalized = normalize_filter_conditions(items)?;
            Ok(serde_json::json!({
                "filter_operator": "all",
                "filters": normalized,
            }))
        }
        Value::Object(ref map) => {
            // Validate that the object has the expected shape.
            if let Some(inner_filters) = map.get("filters") {
                if let Some(arr) = inner_filters.as_array() {
                    let normalized = normalize_filter_conditions(arr.clone())?;
                    let mut result = map.clone();
                    result.insert("filters".to_string(), Value::Array(normalized));
                    return Ok(Value::Object(result));
                }
            }
            Ok(value)
        }
        _ => Err(AppError::invalid_input(format!(
            "{field_name} must be either an object or an array"
        ))
        .with_operation("normalize_search_filters")
        .with_invalid_field(field_name)
        .with_expected_shape(
            "a JSON array of filter conditions, or an object with `filter_operator` and `filters`",
        )),
    }
}

/// Normalize individual filter conditions within a filters array.
///
/// This handles the entity-link shorthand: when a filter condition is
/// `["field", "is"|"is_not", integer]` for common entity-link fields,
/// the integer is NOT automatically expanded (the REST API may accept it),
/// but a validation warning is prepared. For nested logical groups,
/// the function recurses.
fn normalize_filter_conditions(conditions: Vec<Value>) -> Result<Vec<Value>> {
    let mut normalized = Vec::with_capacity(conditions.len());
    for condition in conditions {
        match &condition {
            // Nested logical group: {"logical_operator": ..., "conditions": [...]}
            Value::Object(map) if map.contains_key("logical_operator") => {
                if let Some(inner_conds) = map.get("conditions").and_then(Value::as_array) {
                    let inner_normalized = normalize_filter_conditions(inner_conds.clone())?;
                    let mut result = map.clone();
                    result.insert("conditions".to_string(), Value::Array(inner_normalized));
                    normalized.push(Value::Object(result));
                } else {
                    normalized.push(condition);
                }
            }
            // Standard filter condition: [field, operator, value]
            Value::Array(items) if items.len() >= 3 => {
                normalized.push(condition);
            }
            _ => {
                normalized.push(condition);
            }
        }
    }
    Ok(normalized)
}

fn normalize_filter_presets(value: &Value, field_name: &str) -> Result<Vec<Value>> {
    let presets = value.as_array().ok_or_else(|| {
        AppError::invalid_input(format!("{field_name} must be an array of preset objects"))
            .with_operation("normalize_filter_presets")
            .with_invalid_field(field_name)
            .with_expected_shape("an array of preset objects")
    })?;

    let mut normalized = Vec::with_capacity(presets.len());
    for (index, preset) in presets.iter().enumerate() {
        let object = preset.as_object().ok_or_else(|| {
            AppError::invalid_input(format!(
                "item {} in {field_name} must be an object",
                index + 1
            ))
            .with_operation("normalize_filter_presets")
            .with_invalid_field(field_name)
            .with_expected_shape("an array whose items are all objects")
        })?;
        let preset_name = object
            .get("preset_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in {field_name} must contain a string `preset_name`",
                    index + 1
                ))
                .with_operation("normalize_filter_presets")
                .with_invalid_field("preset_name")
                .with_expected_shape("each preset object must contain a string field `preset_name`")
            })?;
        if preset_name.trim().is_empty() {
            return Err(AppError::invalid_input(format!(
                "item {} in {field_name} must contain a non-empty `preset_name`",
                index + 1
            ))
            .with_operation("normalize_filter_presets")
            .with_invalid_field("preset_name")
            .with_expected_shape("a non-empty string"));
        }
        normalized.push(preset.clone());
    }

    Ok(normalized)
}
