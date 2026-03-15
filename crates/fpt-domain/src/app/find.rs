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

    let object = input
        .as_object()
        .ok_or_else(|| AppError::invalid_input("entity find input must be a JSON object"))?;

    let inline_filter_dsl = object
        .get("filter_dsl")
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| AppError::invalid_input("`filter_dsl` must be a string"))
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
            "`--filter-dsl` and `filter_dsl` in the input JSON cannot be provided at the same time",
        ));
    }
    let effective_filter_dsl = filter_dsl.or(inline_filter_dsl);

    if effective_filter_dsl.is_some() && inline_search.is_some() {
        return Err(AppError::invalid_input(
            "`search` and `filter_dsl` cannot be used together",
        ));
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
        let sort = sort
            .as_str()
            .ok_or_else(|| AppError::invalid_input("`sort` must be a string"))?;
        params.query.push(("sort".to_string(), sort.to_string()));
    }

    if let Some(page) = object.get("page") {
        let page = page
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`page` must be an object"))?;
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
                "`filters` cannot be used together with `filter_dsl`, `search`, or `additional_filter_presets`",
            ));
        }

        let filters = filters
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`filters` must be an object"))?;
        for (key, value) in filters {
            params.query.push((
                format!("filter[{key}]"),
                value_to_query(value, &format!("filters.{key}"))?,
            ));
        }
    }

    if let Some(options) = object.get("options") {
        let options = options
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`options` must be an object"))?;
        for (key, value) in options {
            params.query.push((
                format!("options[{key}]"),
                value_to_query(value, &format!("options.{key}"))?,
            ));
        }
    }

    if let Some(query) = object.get("query") {
        let query = query
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`query` must be an object"))?;
        for (key, value) in query {
            params
                .query
                .push((key.clone(), value_to_query(value, &format!("query.{key}"))?));
        }
    }

    let mut search = inline_search;

    if let Some(presets) = top_level_presets {
        let search_object = search.get_or_insert_with(|| json!({}));
        let search_map = search_object
            .as_object_mut()
            .ok_or_else(|| AppError::internal("search payload must be an object"))?;
        if search_map.contains_key("additional_filter_presets") {
            return Err(AppError::invalid_input(
                "`additional_filter_presets` cannot be provided both at the top level and inside `search`",
            ));
        }
        search_map.insert(
            "additional_filter_presets".to_string(),
            Value::Array(presets),
        );
    }

    if let Some(filter_dsl) = effective_filter_dsl {
        let search_object = search.get_or_insert_with(|| json!({}));
        let search_map = search_object
            .as_object_mut()
            .ok_or_else(|| AppError::internal("search payload must be an object"))?;
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
            AppError::api("ShotGrid find response is missing `data`").with_details(response)
        );
    };

    match data {
        Value::Array(items) => Ok(items.first().cloned().unwrap_or(Value::Null)),
        Value::Null => Ok(Value::Null),
        Value::Object(_) => Ok(data.clone()),
        _ => Err(
            AppError::api("ShotGrid find response contains unsupported `data` shape")
                .with_details(response),
        ),
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
            "`{field_name}` must be a string or an array of strings"
        ))
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` may contain only strings"))
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
            "`{field_name}` must be a scalar value"
        ))),
    }
}

fn value_to_query(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::Null => Err(AppError::invalid_input(format!(
            "`{field_name}` cannot be null"
        ))),
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
    let mut search = value
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::invalid_input("`search` must be an object"))?;

    if let Some(filters) = search.get("filters") {
        validate_search_filters(filters, "`search.filters`")?;
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

fn validate_search_filters(value: &Value, field_name: &str) -> Result<()> {
    match value {
        Value::Object(_) | Value::Array(_) => Ok(()),
        _ => Err(AppError::invalid_input(format!(
            "{field_name} must be an object or an array"
        ))),
    }
}

fn normalize_filter_presets(value: &Value, field_name: &str) -> Result<Vec<Value>> {
    let presets = value
        .as_array()
        .ok_or_else(|| AppError::invalid_input(format!("{field_name} must be an array")))?;

    let mut normalized = Vec::with_capacity(presets.len());
    for (index, preset) in presets.iter().enumerate() {
        let object = preset.as_object().ok_or_else(|| {
            AppError::invalid_input(format!(
                "item {} in {field_name} must be an object",
                index + 1
            ))
        })?;
        let preset_name = object
            .get("preset_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in {field_name} must contain a string `preset_name`",
                    index + 1
                ))
            })?;
        if preset_name.trim().is_empty() {
            return Err(AppError::invalid_input(format!(
                "item {} in {field_name} must contain a non-empty `preset_name`",
                index + 1
            )));
        }
        normalized.push(preset.clone());
    }

    Ok(normalized)
}
