use fpt_core::{AppError, Result};
use serde_json::{Map, Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings, api_version_or_default};
use crate::filter_dsl::parse_filter_dsl;
use crate::transport::{
    ShotgridTransport, plan_entity_create, plan_entity_delete, plan_entity_revive,
    plan_entity_update,
};

use super::App;
use super::find::{build_find_params, extract_find_one_response, upsert_query_param};

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_get(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_get(&config, entity, id, fields).await
    }

    pub async fn entity_find(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = build_find_params(input, filter_dsl)?;
        self.transport.entity_find(&config, entity, params).await
    }

    pub async fn entity_find_one(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let mut params = build_find_params(input, filter_dsl)?;
        upsert_query_param(&mut params.query, "page[size]", "1");
        let response = self.transport.entity_find(&config, entity, params).await?;
        extract_find_one_response(response)
    }

    pub async fn entity_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        body: Value,
        dry_run: bool,
    ) -> Result<Value> {
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(json!({
                "dry_run": true,
                "plan": plan_entity_create(&api_version, entity, body),
            }));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_create(&config, entity, &body).await
    }

    pub async fn entity_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        body: Value,
        dry_run: bool,
    ) -> Result<Value> {
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(json!({
                "dry_run": true,
                "plan": plan_entity_update(&api_version, entity, id, body),
            }));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .entity_update(&config, entity, id, &body)
            .await
    }

    pub async fn entity_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        dry_run: bool,
        yes: bool,
    ) -> Result<Value> {
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            return Ok(json!({
                "dry_run": true,
                "plan": plan_entity_delete(&api_version, entity, id),
            }));
        }

        if !yes {
            return Err(AppError::policy_blocked(
                "entity delete is a destructive operation; pass `--yes` to execute it, or use `--dry-run` to inspect the request plan first",
            )
            .with_operation("entity_delete")
            .with_hint("Add `--yes` to confirm the deletion, or `--dry-run` to preview the request without executing it."));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_delete(&config, entity, id).await
    }

    pub async fn entity_revive(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        dry_run: bool,
    ) -> Result<Value> {
        if dry_run {
            return Ok(json!({
                "dry_run": true,
                "plan": plan_entity_revive(entity, id),
            }));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_revive(&config, entity, id).await
    }

    pub async fn entity_relationships(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        id: u64,
        related_field: &str,
        input: Option<Value>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let params = super::activity::build_common_query_params(input)?;
        self.transport
            .entity_relationships(&config, entity, id, related_field, &params)
            .await
    }

    pub async fn project_update_last_accessed(
        &self,
        overrides: ConnectionOverrides,
        project_id: u64,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport
            .project_update_last_accessed(&config, project_id)
            .await
    }

    pub async fn text_search(&self, overrides: ConnectionOverrides, input: Value) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = normalize_text_search_input(input)?;
        self.transport.text_search(&config, &payload).await
    }

    pub async fn entity_count(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Option<Value>,
        filter_dsl: Option<String>,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = build_count_payload(input, filter_dsl)?;
        self.transport
            .entity_summarize(&config, entity, &payload)
            .await
    }
}

fn normalize_text_search_input(input: Value) -> Result<Value> {
    let object = input.as_object().ok_or_else(|| {
        AppError::invalid_input("entity text-search input must be a JSON object")
            .with_operation("normalize_text_search_input")
            .with_expected_shape(
                "a JSON object containing `text` (search query string) and optional `entity_types`",
            )
    })?;

    if !object.contains_key("text") {
        return Err(AppError::invalid_input(
            "text-search requires a `text` field with the search query string",
        )
        .with_operation("normalize_text_search_input")
        .with_missing_fields(["text"])
        .with_expected_shape("a JSON object containing at least a `text` string field"));
    }

    let text = object.get("text").and_then(Value::as_str).ok_or_else(|| {
        AppError::invalid_input("`text` must be a non-empty string")
            .with_operation("normalize_text_search_input")
            .with_invalid_field("text")
            .with_expected_shape("a non-empty string")
    })?;

    if text.trim().is_empty() {
        return Err(AppError::invalid_input("`text` cannot be empty")
            .with_operation("normalize_text_search_input")
            .with_invalid_field("text")
            .with_hint("Provide a non-empty search query string."));
    }

    Ok(input)
}

/// Build an RPC `summarize` payload that counts records matching an
/// optional set of filters.
///
/// The returned JSON is suitable for passing directly to `entity_summarize`
/// and always requests `record_count` on the `id` field.  Filters can be
/// supplied via:
///
/// * `input` — a JSON object with an optional `filters` field (array or
///   object form) and an optional `filter_operator` field.
/// * `filter_dsl` — a DSL expression that is parsed into RPC-compatible
///   conditions.
///
/// When neither is provided the resulting payload uses an empty filter set
/// (count all records of the entity type).
fn build_count_payload(input: Option<Value>, filter_dsl: Option<String>) -> Result<Value> {
    let mut payload = Map::new();

    // Always request record_count on "id".
    payload.insert(
        "summaries".to_string(),
        json!([{"field": "id", "type": "record_count"}]),
    );

    // Determine filter source.
    let filters = match (input, filter_dsl) {
        (Some(_input), Some(_)) => {
            return Err(AppError::invalid_input(
                "`--input` and `--filter-dsl` are mutually exclusive for entity count",
            )
            .with_operation("build_count_payload")
            .with_conflicting_fields(["--input", "--filter-dsl"])
            .with_hint("Provide either `--input` with a JSON filter payload, or `--filter-dsl` with a DSL expression, but not both."));
        }
        (Some(input), None) => {
            let object = input.as_object().ok_or_else(|| {
                AppError::invalid_input("entity count input must be a JSON object")
                    .with_operation("build_count_payload")
                    .with_expected_shape("a JSON object containing optional `filters` and `filter_operator`")
            })?;
            let filters = object.get("filters").cloned().unwrap_or(json!([]));
            let filter_operator = object.get("filter_operator").cloned();
            normalize_count_filters(filters, filter_operator)?
        }
        (None, Some(dsl)) => {
            let parsed = parse_filter_dsl(&dsl)?;
            // The DSL parser returns a logical-group object with
            // `logical_operator` + `conditions` — translate to the RPC
            // `filter_operator` + `filters` shape.
            let operator = parsed
                .get("logical_operator")
                .and_then(Value::as_str)
                .map(|op| match op {
                    "and" => "all",
                    "or" => "any",
                    other => other,
                })
                .unwrap_or("all");
            let conditions = parsed
                .get("conditions")
                .cloned()
                .unwrap_or(json!([]));
            json!({
                "filter_operator": operator,
                "filters": conditions,
            })
        }
        (None, None) => {
            json!({
                "filter_operator": "all",
                "filters": [],
            })
        }
    };

    payload.insert("filters".to_string(), filters);
    Ok(Value::Object(payload))
}

/// Normalize the `filters` value for a count payload.
///
/// Accepts either:
/// * An array of filter conditions (wrapped with the default operator `all`)
/// * An object with `filter_operator` and `filters` keys
fn normalize_count_filters(filters: Value, filter_operator: Option<Value>) -> Result<Value> {
    match filters {
        Value::Array(items) => {
            let operator = match filter_operator {
                Some(op) => {
                    let op_str = op.as_str().ok_or_else(|| {
                        AppError::invalid_input("`filter_operator` must be `all` or `any`")
                            .with_operation("normalize_count_filters")
                            .with_invalid_field("filter_operator")
                            .with_allowed_values(["all", "any"])
                    })?;
                    match op_str {
                        "all" | "any" => op_str.to_string(),
                        _ => return Err(AppError::invalid_input("`filter_operator` must be `all` or `any`")
                            .with_operation("normalize_count_filters")
                            .with_invalid_field("filter_operator")
                            .with_received_value(op_str)
                            .with_allowed_values(["all", "any"])),
                    }
                }
                None => "all".to_string(),
            };
            Ok(json!({
                "filter_operator": operator,
                "filters": items,
            }))
        }
        Value::Object(ref map) => {
            if filter_operator.is_some() && map.contains_key("filter_operator") {
                return Err(AppError::invalid_input(
                    "`filter_operator` cannot be set both at the top level and inside `filters`",
                )
                .with_operation("normalize_count_filters")
                .with_conflicting_fields(["filter_operator", "filters.filter_operator"]));
            }
            if let Some(op) = filter_operator {
                let mut m = map.clone();
                m.insert("filter_operator".to_string(), op);
                Ok(Value::Object(m))
            } else {
                Ok(filters)
            }
        }
        _ => Err(AppError::invalid_input("`filters` must be a JSON array or object")
            .with_operation("normalize_count_filters")
            .with_invalid_field("filters")
            .with_expected_shape("a JSON array of filter conditions, or a JSON object with `filter_operator` and `filters`")),
    }
}
