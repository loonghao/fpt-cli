use fpt_core::{AppError, Result};
use serde_json::{Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings, api_version_or_default};
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

    pub async fn text_search(&self, overrides: ConnectionOverrides, input: Value) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        let payload = normalize_text_search_input(input)?;
        self.transport.text_search(&config, &payload).await
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
