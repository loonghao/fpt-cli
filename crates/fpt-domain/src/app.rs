use std::env;

use fpt_core::{AppError, Result};
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};

use crate::capability::{command_specs, find_command_spec};

use crate::config::{api_version_or_default, ConnectionOverrides, ConnectionSettings};
use crate::filter_dsl::parse_filter_dsl;
use crate::transport::{
    FindParams, RequestPlan, RestTransport, ShotgridTransport, plan_entity_create,
    plan_entity_delete, plan_entity_update,
};



pub struct App<T = RestTransport> {
    transport: T,
}

impl Default for App<RestTransport> {
    fn default() -> Self {
        Self {
            transport: RestTransport::default(),
        }
    }
}

impl<T> App<T>
where
    T: ShotgridTransport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn capabilities(&self) -> Value {

        json!({
            "name": "fpt",
            "version": env!("CARGO_PKG_VERSION"),
            "transports": [
                { "name": "rest", "status": "available" },
                { "name": "rpc", "status": "planned" }
            ],
            "required_environment": [
                "FPT_SITE"
            ],
            "optional_environment": [
                "FPT_AUTH_MODE",
                "FPT_SCRIPT_NAME",
                "FPT_SCRIPT_KEY",
                "FPT_USERNAME",
                "FPT_PASSWORD",
                "FPT_AUTH_TOKEN",
                "FPT_SESSION_TOKEN",
                "FPT_API_VERSION"
            ],
            "auth_modes": [
                {
                    "name": "script",
                    "grant_type": "client_credentials",
                    "required": ["FPT_SCRIPT_NAME", "FPT_SCRIPT_KEY"]
                },
                {
                    "name": "user_password",
                    "grant_type": "password",
                    "required": ["FPT_USERNAME", "FPT_PASSWORD"],
                    "optional": ["FPT_AUTH_TOKEN"]
                },
                {
                    "name": "session_token",
                    "grant_type": "session_token",
                    "required": ["FPT_SESSION_TOKEN"]
                }
            ],
            "commands": command_specs(),
        })
    }

    pub fn inspect_command(&self, name: &str) -> Result<Value> {
        let spec = find_command_spec(name)
            .ok_or_else(|| AppError::unsupported(format!("未知命令 `{name}`")))?;
        Ok(json!(spec))
    }

    pub async fn auth_test(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.auth_test(&config).await
    }

    pub async fn schema_entities(&self, overrides: ConnectionOverrides) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_entities(&config).await
    }

    pub async fn schema_fields(
        &self,
        overrides: ConnectionOverrides,
        entity: &str,
    ) -> Result<Value> {
        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.schema_fields(&config, entity).await
    }

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
        self.transport.entity_update(&config, entity, id, &body).await
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
                "删除操作需要显式传入 `--yes`，或先使用 `--dry-run` 查看计划",
            ));
        }

        let config = ConnectionSettings::resolve(overrides)?;
        self.transport.entity_delete(&config, entity, id).await
    }

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
                "批量删除需要显式传入 `--yes`，或先使用 `--dry-run` 查看计划",
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

const DEFAULT_BATCH_CONCURRENCY: usize = 8;
const MAX_BATCH_CONCURRENCY: usize = 32;

fn batch_concurrency_limit() -> usize {
    env::var("FPT_BATCH_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .map(|value| value.min(MAX_BATCH_CONCURRENCY))
        .unwrap_or(DEFAULT_BATCH_CONCURRENCY)
}

fn sort_batch_results(results: &mut [Value]) {
    results.sort_by_key(batch_result_index);
}

fn batch_result_index(value: &Value) -> usize {
    value
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX)
}

fn build_find_params(input: Option<Value>, filter_dsl: Option<String>) -> Result<FindParams> {

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
        .ok_or_else(|| AppError::invalid_input("entity find 的输入必须是 JSON object"))?;

    let inline_filter_dsl = object
        .get("filter_dsl")
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| AppError::invalid_input("`filter_dsl` 必须是字符串"))
        })
        .transpose()?;

    if filter_dsl.is_some() && inline_filter_dsl.is_some() {
        return Err(AppError::invalid_input(
            "`--filter-dsl` 与输入 JSON 中的 `filter_dsl` 不能同时传入",
        ));
    }
    let effective_filter_dsl = filter_dsl.or(inline_filter_dsl);

    if let Some(fields) = object.get("fields") {
        let fields = string_list(fields, "fields")?;
        if !fields.is_empty() {
            params.query.push(("fields".to_string(), fields.join(",")));
        }
    }

    if let Some(include) = object.get("include") {
        let include = string_list(include, "include")?;
        if !include.is_empty() {
            params.query.push(("include".to_string(), include.join(",")));
        }
    }

    if let Some(sort) = object.get("sort") {
        let sort = sort
            .as_str()
            .ok_or_else(|| AppError::invalid_input("`sort` 必须是字符串"))?;
        params.query.push(("sort".to_string(), sort.to_string()));
    }

    if let Some(page) = object.get("page") {
        let page = page
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`page` 必须是 object"))?;
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
        if effective_filter_dsl.is_some() {
            return Err(AppError::invalid_input(
                "`filters` 与 `filter_dsl` 不能同时使用",
            ));
        }

        let filters = filters
            .as_object()
            .ok_or_else(|| AppError::invalid_input("`filters` 必须是 object"))?;
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
            .ok_or_else(|| AppError::invalid_input("`options` 必须是 object"))?;
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
            .ok_or_else(|| AppError::invalid_input("`query` 必须是 object"))?;
        for (key, value) in query {
            params.query.push((
                key.clone(),
                value_to_query(value, &format!("query.{key}"))?,
            ));
        }
    }

    if let Some(filter_dsl) = effective_filter_dsl {
        params.search = Some(json!({
            "filters": parse_filter_dsl(&filter_dsl)?,
        }));
    }

    Ok(params)
}


#[derive(Debug, Clone)]
struct BatchUpdateItem {
    id: u64,
    body: Value,
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
                .ok_or_else(|| AppError::invalid_input("entity batch get 需要 `ids` 字段"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`ids` 必须是数组"))?;
            let ids = u64_list(ids, "ids")?;
            let fields = object
                .get("fields")
                .map(|value| string_list(value, "fields"))
                .transpose()?;
            Ok((ids, fields.filter(|items| !items.is_empty())))
        }
        _ => Err(AppError::invalid_input(
            "entity batch get 的输入必须是 JSON array，或包含 `ids` 的 object",
        )),
    }
}

fn parse_batch_find_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => object_items(values, "entity batch find") ,
        Value::Object(object) => {
            let requests = object
                .get("requests")
                .ok_or_else(|| AppError::invalid_input("entity batch find 需要 `requests` 字段"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`requests` 必须是数组"))?
                .clone();
            object_items(requests, "requests")
        }
        _ => Err(AppError::invalid_input(
            "entity batch find 的输入必须是 JSON array，或包含 `requests` 的 object",
        )),
    }
}

fn parse_batch_create_input(input: Value) -> Result<Vec<Value>> {
    match input {
        Value::Array(values) => non_empty_items(values, "entity batch create") ,
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| AppError::invalid_input("entity batch create 需要 `items` 字段"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` 必须是数组"))?
                .clone();
            non_empty_items(items, "items")
        }
        _ => Err(AppError::invalid_input(
            "entity batch create 的输入必须是 JSON array，或包含 `items` 的 object",
        )),
    }
}

fn parse_batch_update_input(input: Value) -> Result<Vec<BatchUpdateItem>> {
    let items = match input {
        Value::Array(values) => non_empty_items(values, "entity batch update")?,
        Value::Object(object) => {
            let items = object
                .get("items")
                .ok_or_else(|| AppError::invalid_input("entity batch update 需要 `items` 字段"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`items` 必须是数组"))?
                .clone();
            non_empty_items(items, "items")?
        }
        _ => {
            return Err(AppError::invalid_input(
                "entity batch update 的输入必须是 JSON array，或包含 `items` 的 object",
            ));
        }
    };

    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let object = item.as_object().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "entity batch update 的第 {} 项必须是 object",
                    index + 1
                ))
            })?;
            let id = object
                .get("id")
                .ok_or_else(|| AppError::invalid_input(format!("entity batch update 的第 {} 项缺少 `id`", index + 1)))
                .and_then(|value| value.as_u64().ok_or_else(|| {
                    AppError::invalid_input(format!("entity batch update 的第 {} 项 `id` 必须是正整数", index + 1))
                }))?;
            let body = object
                .get("body")
                .cloned()
                .ok_or_else(|| AppError::invalid_input(format!("entity batch update 的第 {} 项缺少 `body`", index + 1)))?;
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
                .ok_or_else(|| AppError::invalid_input("entity batch delete 需要 `ids` 字段"))?
                .as_array()
                .ok_or_else(|| AppError::invalid_input("`ids` 必须是数组"))?;
            u64_list(ids, "ids")
        }
        _ => Err(AppError::invalid_input(
            "entity batch delete 的输入必须是 JSON array，或包含 `ids` 的 object",
        )),
    }
}

fn non_empty_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    if values.is_empty() {
        return Err(AppError::invalid_input(format!("`{field_name}` 不能为空数组")));
    }
    Ok(values)
}

fn object_items(values: Vec<Value>, field_name: &str) -> Result<Vec<Value>> {
    let values = non_empty_items(values, field_name)?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_object() {
            return Err(AppError::invalid_input(format!(
                "`{field_name}` 的第 {} 项必须是 object",
                index + 1
            )));
        }
    }
    Ok(values)
}

fn u64_list(values: &[Value], field_name: &str) -> Result<Vec<u64>> {
    if values.is_empty() {
        return Err(AppError::invalid_input(format!("`{field_name}` 不能为空数组")));
    }

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.as_u64().ok_or_else(|| {
                AppError::invalid_input(format!(
                    "`{field_name}` 的第 {} 项必须是正整数",
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
        AppError::invalid_input(format!("`{field_name}` 必须是字符串或字符串数组"))
    })?;

    array
        .iter()
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                AppError::invalid_input(format!("`{field_name}` 只能包含字符串"))
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
            "`{field_name}` 必须是标量值"
        ))),
    }
}

fn value_to_query(value: &Value, field_name: &str) -> Result<String> {
    match value {
        Value::Null => Err(AppError::invalid_input(format!(
            "`{field_name}` 不能为 null"
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
