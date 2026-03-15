use fpt_core::{AppError, Result};
use futures::stream::{self, StreamExt};
use serde_json::{Value, json};

use crate::config::{ConnectionOverrides, ConnectionSettings, api_version_or_default};
use crate::transport::{
    RequestPlan, ShotgridTransport, plan_entity_create, plan_entity_delete, plan_entity_update,
};

use super::find::build_find_params;
use super::{App, BatchUpdateItem, batch_concurrency_limit, sort_batch_results};

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub async fn entity_batch_get(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let (ids, fields) = parse_batch_get_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let mut results = stream::iter(ids.into_iter().enumerate())
            .map(|(index, id)| {
                let fields = fields.clone();
                async move {
                    match transport.entity_get(config, entity, id, fields).await {
                        Ok(response) => json!({
                            "index": index,
                            "id": id,
                            "ok": true,
                            "response": response,
                        }),
                        Err(error) => json!({
                            "index": index,
                            "id": id,
                            "ok": false,
                            "error": error.envelope(),
                        }),
                    }
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response("entity.batch.get", entity, results))
    }

    pub async fn entity_batch_find(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
    ) -> Result<Value> {
        let requests = parse_batch_find_input(input)?;
        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let mut results = stream::iter(requests.into_iter().enumerate())
            .map(|(index, request)| async move {
                match build_find_params(Some(request.clone()), None) {
                    Ok(params) => match transport.entity_find(config, entity, params).await {
                        Ok(response) => json!({
                            "index": index,
                            "ok": true,
                            "request": request,
                            "response": response,
                        }),
                        Err(error) => json!({
                            "index": index,
                            "ok": false,
                            "request": request,
                            "error": error.envelope(),
                        }),
                    },
                    Err(error) => json!({
                        "index": index,
                        "ok": false,
                        "request": request,
                        "error": error.envelope(),
                    }),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response("entity.batch.find", entity, results))
    }

    pub async fn entity_batch_create(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
    ) -> Result<Value> {
        let items = parse_batch_create_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = items
                .into_iter()
                .map(|body| plan_entity_create(&api_version, entity, body))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.create", entity, plans));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let mut results = stream::iter(items.into_iter().enumerate())
            .map(|(index, body)| async move {
                match transport.entity_create(config, entity, &body).await {
                    Ok(response) => json!({
                        "index": index,
                        "ok": true,
                        "request": body,
                        "response": response,
                    }),
                    Err(error) => json!({
                        "index": index,
                        "ok": false,
                        "request": body,
                        "error": error.envelope(),
                    }),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response("entity.batch.create", entity, results))
    }

    pub async fn entity_batch_update(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
    ) -> Result<Value> {
        let items = parse_batch_update_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = items
                .iter()
                .map(|item| plan_entity_update(&api_version, entity, item.id, item.body.clone()))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.update", entity, plans));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let mut results = stream::iter(items.into_iter().enumerate())
            .map(|(index, item)| async move {
                match transport
                    .entity_update(config, entity, item.id, &item.body)
                    .await
                {
                    Ok(response) => json!({
                        "index": index,
                        "id": item.id,
                        "ok": true,
                        "request": item.body,
                        "response": response,
                    }),
                    Err(error) => json!({
                        "index": index,
                        "id": item.id,
                        "ok": false,
                        "request": item.body,
                        "error": error.envelope(),
                    }),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response("entity.batch.update", entity, results))
    }

    pub async fn entity_batch_delete(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
        input: Value,
        dry_run: bool,
        yes: bool,
    ) -> Result<Value> {
        let ids = parse_batch_delete_input(input)?;
        if dry_run {
            let api_version = api_version_or_default(overrides.api_version.as_deref());
            let plans = ids
                .into_iter()
                .map(|id| plan_entity_delete(&api_version, entity, id))
                .collect::<Vec<_>>();
            return Ok(batch_dry_run_response("entity.batch.delete", entity, plans));
        }

        if !yes {
            return Err(AppError::policy_blocked(
                "batch delete operations require an explicit `--yes`, or use `--dry-run` to inspect the plan first",
            ));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        let transport = &self.transport;
        let config = &config;
        let mut results = stream::iter(ids.into_iter().enumerate())
            .map(|(index, id)| async move {
                match transport.entity_delete(config, entity, id).await {
                    Ok(response) => json!({
                        "index": index,
                        "id": id,
                        "ok": true,
                        "response": response,
                    }),
                    Err(error) => json!({
                        "index": index,
                        "id": id,
                        "ok": false,
                        "error": error.envelope(),
                    }),
                }
            })
            .buffer_unordered(batch_concurrency_limit())
            .collect::<Vec<_>>()
            .await;
        sort_batch_results(&mut results);

        Ok(batch_response("entity.batch.delete", entity, results))
    }
}

fn batch_response(operation: &str, entity: &str, results: Vec<Value>) -> Value {
    let failure_count = results
        .iter()
        .filter(|item| item.get("ok").and_then(Value::as_bool) != Some(true))
        .count();
    let success_count = results.len().saturating_sub(failure_count);

    json!({
        "ok": failure_count == 0,
        "operation": operation,
        "entity": entity,
        "total": results.len(),
        "success_count": success_count,
        "failure_count": failure_count,
        "results": results,
    })
}

fn batch_dry_run_response(operation: &str, entity: &str, plans: Vec<RequestPlan>) -> Value {
    json!({
        "dry_run": true,
        "operation": operation,
        "entity": entity,
        "count": plans.len(),
        "plans": plans,
    })
}

fn parse_batch_get_input(input: Value) -> Result<(Vec<u64>, Option<Vec<String>>)> {
    match input {
        Value::Array(values) => Ok((u64_list(&values, "ids")?, None)),
        Value::Object(object) => {
            let ids = object
                .get("ids")
                .ok_or_else(|| AppError::invalid_input("entity batch get requires an `ids` field"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`ids` must be an array"))?;
            let ids = u64_list(ids, "ids")?;
            let fields = object
                .get("fields")
                .map(|value| string_list(value, "fields"))
                .transpose()?;
            Ok((ids, fields.filter(|items| !items.is_empty())))
        }
        _ => Err(AppError::invalid_input(
            "entity batch get input must be a JSON array or an object containing `ids`",
        )),
    }
}

fn parse_batch_find_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => object_items(values, "entity batch find"),
        Value::Object(object) => {
            let requests = object
                .get("requests")
                .ok_or_else(|| {
                    AppError::invalid_input("entity batch find requires a `requests` field")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`requests` must be an array"))?
                .clone();
            object_items(requests, "requests")
        }
        _ => Err(AppError::invalid_input(
            "entity batch find input must be a JSON array or an object containing `requests`",
        )),
    }
}

fn parse_batch_create_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => non_empty_items(values, "entity batch create"),
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| {
                    AppError::invalid_input("entity batch create requires an `items` field")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` must be an array"))?
                .clone();
            non_empty_items(items, "items")
        }
        _ => Err(AppError::invalid_input(
            "entity batch create input must be a JSON array or an object containing `items`",
        )),
    }
}

fn parse_batch_update_input(input: Value) -> Result<Vec<BatchUpdateItem>> {
    let items = match input {
        Value::Array(values) => non_empty_items(values, "entity batch update")?,
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| {
                    AppError::invalid_input("entity batch update requires an `items` field")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` must be an array"))?
                .clone();
            non_empty_items(items, "items")?
        }
        _ => {
            return Err(AppError::invalid_input(
                "entity batch update input must be a JSON array or an object containing `items`",
            ));
        }
    };

    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let object = item.as_object().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in entity batch update must be an object",
                    index + 1
                ))
            })?;
            let id = object
                .get("id")
                .ok_or_else(|| {
                    AppError::invalid_input(format!(
                        "item {} in entity batch update is missing `id`",
                        index + 1
                    ))
                })
                .and_then(|value| {
                    value.as_u64().ok_or_else(|| {
                        AppError::invalid_input(format!(
                            "`id` in item {} of entity batch update must be a positive integer",
                            index + 1
                        ))
                    })
                })?;
            let body = object.get("body").cloned().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in entity batch update is missing `body`",
                    index + 1
                ))
            })?;
            Ok(BatchUpdateItem { id, body })
        })
        .collect()
}

fn parse_batch_delete_input(input: Value) -> Result<Vec<u64>> {
    match input {
        Value::Array(values) => u64_list(&values, "ids"),
        Value::Object(object) => {
            let ids = object
                .get("ids")
                .ok_or_else(|| {
                    AppError::invalid_input("entity batch delete requires an `ids` field")
                })?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`ids` must be an array"))?;
            u64_list(ids, "ids")
        }
        _ => Err(AppError::invalid_input(
            "entity batch delete input must be a JSON array or an object containing `ids`",
        )),
    }
}

fn non_empty_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    if values.is_empty() {
        return Err(AppError::invalid_input(format!(
            "`{field_name}` cannot be an empty array"
        )));
    }
    Ok(values)
}

fn object_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    let values = non_empty_items(values, field_name)?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_object() {
            return Err(AppError::invalid_input(format!(
                "item {} in `{field_name}` must be an object",
                index + 1
            )));
        }
    }
    Ok(values)
}

fn u64_list(values: &[Value], field_name: &str) -> Result<Vec<u64>> {
    if values.is_empty() {
        return Err(AppError::invalid_input(format!(
            "`{field_name}` cannot be an empty array"
        )));
    }

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.as_u64().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "item {} in `{field_name}` must be a positive integer",
                    index + 1
                ))
            })
        })
        .collect()
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
