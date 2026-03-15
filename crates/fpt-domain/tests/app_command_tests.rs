use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use fpt_core::{AppError, Result};
use fpt_domain::transport::{FindParams, ShotgridTransport};
use fpt_domain::{App, AuthMode, ConnectionOverrides, ConnectionSettings};

use serde_json::{Value, json};
use tokio::time::sleep;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityGetCall {
    entity: String,
    id: u64,
    fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
struct EntityUpdateCall {
    entity: String,
    id: u64,
    body: Value,
}

#[derive(Debug, Clone, Default)]
struct RecordedState {
    auth_calls: usize,
    server_info_sites: Vec<String>,
    schema_entities_calls: usize,
    schema_fields_entities: Vec<String>,
    entity_get_calls: Vec<EntityGetCall>,
    entity_find_calls: Vec<FindParams>,
    entity_summarize_calls: Vec<(String, Value)>,
    entity_create_calls: Vec<(String, Value)>,

    entity_update_calls: Vec<EntityUpdateCall>,
    entity_delete_calls: Vec<(String, u64)>,
    entity_revive_calls: Vec<(String, u64)>,
    work_schedule_calls: Vec<Value>,
}

#[derive(Debug, Clone, Default)]
struct RecordingTransport {
    state: Arc<Mutex<RecordedState>>,
}

impl RecordingTransport {
    fn snapshot(&self) -> RecordedState {
        self.state.lock().expect("state lock").clone()
    }
}

#[async_trait]
impl ShotgridTransport for RecordingTransport {
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value> {
        self.state.lock().expect("state lock").auth_calls += 1;
        Ok(json!({
            "ok": true,
            "site": config.site,
            "grant_type": config.auth_mode().grant_type(),
        }))
    }

    async fn server_info(&self, site: &str) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .server_info_sites
            .push(site.to_string());
        Ok(json!({"site": site, "full_version": [8, 40, 0, 0]}))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        self.state.lock().expect("state lock").schema_entities_calls += 1;
        Ok(json!({"data": ["Asset", "Shot"]}))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, entity: &str) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .schema_fields_entities
            .push(entity.to_string());
        Ok(json!({"entity": entity, "fields": ["code", "sg_status_list"]}))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_get_calls
            .push(EntityGetCall {
                entity: entity.to_string(),
                id,
                fields: fields.clone(),
            });
        Ok(json!({"entity": entity, "id": id, "fields": fields}))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_find_calls
            .push(params.clone());
        Ok(json!({
            "query": params.query,
            "search": params.search,
        }))
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_summarize_calls
            .push((entity.to_string(), body.clone()));
        Ok(json!({
            "entity": entity,
            "summaries": body["summaries"].clone(),
            "groups": body.get("grouping").cloned().unwrap_or_else(|| json!([])),
        }))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_create_calls
            .push((entity.to_string(), body.clone()));
        if body.get("fail").and_then(Value::as_bool) == Some(true) {
            return Err(AppError::api("create failed"));
        }
        Ok(json!({"entity": entity, "body": body}))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_update_calls
            .push(EntityUpdateCall {
                entity: entity.to_string(),
                id,
                body: body.clone(),
            });
        if id == 99 {
            return Err(AppError::api("update failed"));
        }
        Ok(json!({"entity": entity, "id": id, "body": body}))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_delete_calls
            .push((entity.to_string(), id));
        if id == 13 {
            return Err(AppError::api("delete failed"));
        }
        Ok(json!({"entity": entity, "id": id, "deleted": true}))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .entity_revive_calls
            .push((entity.to_string(), id));
        Ok(json!(true))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        body: &Value,
    ) -> Result<Value> {
        self.state
            .lock()
            .expect("state lock")
            .work_schedule_calls
            .push(body.clone());
        Ok(json!({
            "2026-03-16": {
                "working": true,
                "reason": "STUDIO_WORK_WEEK"
            }
        }))
    }
}

#[derive(Debug, Clone)]
struct FindOneTransport {
    response: Value,
    calls: Arc<Mutex<Vec<FindParams>>>,
}

impl FindOneTransport {
    fn new(response: Value) -> Self {
        Self {
            response,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn calls(&self) -> Vec<FindParams> {
        self.calls.lock().expect("calls lock").clone()
    }
}

#[async_trait]
impl ShotgridTransport for FindOneTransport {
    async fn auth_test(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, _entity: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        self.calls.lock().expect("calls lock").push(params);
        Ok(self.response.clone())
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
}

#[derive(Debug, Clone, Default)]
struct SlowGetTransport {
    current: Arc<AtomicUsize>,
    max_seen: Arc<AtomicUsize>,
}

impl SlowGetTransport {
    fn max_in_flight(&self) -> usize {
        self.max_seen.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ShotgridTransport for SlowGetTransport {
    async fn auth_test(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn server_info(&self, _site: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_entities(&self, _config: &ConnectionSettings) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn schema_fields(&self, _config: &ConnectionSettings, _entity: &str) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_get(
        &self,
        _config: &ConnectionSettings,
        entity: &str,
        id: u64,
        _fields: Option<Vec<String>>,
    ) -> Result<Value> {
        let in_flight = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_seen.fetch_max(in_flight, Ordering::SeqCst);

        let delay_ms = match id {
            1 => 80,
            2 => 10,
            3 => 60,
            _ => 20,
        };
        sleep(Duration::from_millis(delay_ms)).await;

        self.current.fetch_sub(1, Ordering::SeqCst);
        Ok(json!({"entity": entity, "id": id}))
    }

    async fn entity_find(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _params: FindParams,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_summarize(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_create(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_update(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_delete(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn entity_revive(
        &self,
        _config: &ConnectionSettings,
        _entity: &str,
        _id: u64,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }

    async fn work_schedule_read(
        &self,
        _config: &ConnectionSettings,
        _body: &Value,
    ) -> Result<Value> {
        Err(AppError::not_implemented("unused"))
    }
}

fn overrides() -> ConnectionOverrides {
    ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("openclaw".to_string()),
        script_key: Some("secret-key".to_string()),
        api_version: Some("v1.1".to_string()),
        ..Default::default()
    }
}

fn query_value<'a>(params: &'a FindParams, key: &str) -> Option<&'a str> {
    params
        .query
        .iter()
        .find(|(item_key, _)| item_key == key)
        .map(|(_, value)| value.as_str())
}

#[tokio::test]
async fn auth_schema_and_entity_read_commands_delegate_to_transport() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    let auth = app
        .auth_test(overrides())
        .await
        .expect("auth test succeeds");
    let server_info = app
        .server_info(ConnectionOverrides {
            site: overrides().site,
            ..Default::default()
        })
        .await
        .expect("server info succeeds");
    let schema_entities = app
        .schema_entities(overrides())
        .await
        .expect("schema entities succeeds");
    let schema_fields = app
        .schema_fields(overrides(), "Shot")
        .await
        .expect("schema fields succeeds");
    let entity_get = app
        .entity_get(
            overrides(),
            "Shot",
            42,
            Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        )
        .await
        .expect("entity get succeeds");
    let summarize = app
        .entity_summarize(
            overrides(),
            "Version",
            json!({
                "filters": [["sg_status_list", "is", "ip"]],
                "filter_operator": "any",
                "summary_fields": [
                    { "field": "id", "type": "record_count" }
                ],
                "grouping": [
                    { "field": "project", "type": "exact", "direction": "asc" }
                ],
                "include_archived_projects": false
            }),
        )
        .await
        .expect("entity summarize succeeds");
    let work_schedule = app
        .work_schedule_read(
            overrides(),
            json!({
                "start_date": "2026-03-16",
                "end_date": "2026-03-20",
                "project": { "type": "Project", "id": 123 },
                "user": { "type": "HumanUser", "id": 456 }
            }),
        )
        .await
        .expect("work schedule read succeeds");

    assert_eq!(auth["grant_type"], "client_credentials");
    assert_eq!(server_info["site"], "https://example.shotgrid.autodesk.com");
    assert_eq!(schema_entities["data"][0], "Asset");
    assert_eq!(schema_fields["entity"], "Shot");
    assert_eq!(entity_get["id"], 42);
    assert_eq!(summarize["entity"], "Version");
    assert_eq!(summarize["summaries"][0]["type"], "record_count");
    assert_eq!(work_schedule["2026-03-16"]["working"], true);

    let snapshot = transport.snapshot();
    assert_eq!(snapshot.auth_calls, 1);
    assert_eq!(
        snapshot.server_info_sites,
        vec!["https://example.shotgrid.autodesk.com".to_string()]
    );
    assert_eq!(snapshot.schema_entities_calls, 1);
    assert_eq!(snapshot.schema_fields_entities, vec!["Shot".to_string()]);
    assert_eq!(snapshot.entity_get_calls.len(), 1);
    assert_eq!(snapshot.entity_summarize_calls.len(), 1);
    assert_eq!(snapshot.work_schedule_calls.len(), 1);
    assert_eq!(
        snapshot.entity_summarize_calls[0],
        (
            "Version".to_string(),
            json!({
                "filters": {
                    "filter_operator": "any",
                    "filters": [["sg_status_list", "is", "ip"]]
                },
                "summaries": [
                    { "field": "id", "type": "record_count" }
                ],
                "grouping": [
                    { "field": "project", "type": "exact", "direction": "asc" }
                ],
                "include_archived_projects": false
            })
        )
    );
    assert_eq!(
        snapshot.work_schedule_calls[0],
        json!({
            "start_date": "2026-03-16",
            "end_date": "2026-03-20",
            "project": { "type": "Project", "id": 123 },
            "user": { "type": "HumanUser", "id": 456 }
        })
    );

    assert_eq!(
        snapshot.entity_get_calls[0],
        EntityGetCall {
            entity: "Shot".to_string(),
            id: 42,
            fields: Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        }
    );
}

#[tokio::test]
async fn entity_find_builds_query_and_search_payload_from_input() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Asset",
        Some(json!({
            "fields": ["code", "sg_status_list"],
            "include": "project",
            "sort": "code",
            "page": { "number": 2, "size": 50 },
            "options": { "include_archived_projects": true },
            "query": { "extra": "keep" },
            "filter_dsl": "code ~ 'bunny'"
        })),
        None,
    )
    .await
    .expect("entity find succeeds");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(query_value(params, "fields"), Some("code,sg_status_list"));
    assert_eq!(query_value(params, "include"), Some("project"));
    assert_eq!(query_value(params, "sort"), Some("code"));
    assert_eq!(query_value(params, "page[number]"), Some("2"));
    assert_eq!(query_value(params, "page[size]"), Some("50"));
    assert_eq!(
        query_value(params, "options[include_archived_projects]"),
        Some("true")
    );
    assert_eq!(query_value(params, "extra"), Some("keep"));
    assert!(params.search.is_some());
}

#[tokio::test]
async fn entity_find_accepts_structured_search_input() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Version",
        Some(json!({
            "fields": ["code", "sg_status_list"],
            "search": {
                "filters": {
                    "filter_operator": "all",
                    "filters": [
                        ["project", "is", {"type": "Project", "id": 123}],
                        ["sg_status_list", "is", "ip"]
                    ]
                },
                "additional_filter_presets": [
                    {
                        "preset_name": "LATEST",
                        "latest_by": "ENTITIES_CREATED_AT"
                    }
                ]
            }
        })),
        None,
    )
    .await
    .expect("structured search should succeed");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(query_value(params, "fields"), Some("code,sg_status_list"));
    assert_eq!(
        params.search,
        Some(json!({
            "filters": {
                "filter_operator": "all",
                "filters": [
                    ["project", "is", {"type": "Project", "id": 123}],
                    ["sg_status_list", "is", "ip"]
                ]
            },
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ]
        }))
    );
}

#[tokio::test]
async fn entity_find_merges_top_level_presets_with_filter_dsl() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    app.entity_find(
        overrides(),
        "Version",
        Some(json!({
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ]
        })),
        Some("code ~ 'hero'".to_string()),
    )
    .await
    .expect("filter presets should merge with filter_dsl");

    let snapshot = transport.snapshot();
    let params = snapshot
        .entity_find_calls
        .first()
        .expect("find params recorded");
    assert_eq!(
        params.search,
        Some(json!({
            "additional_filter_presets": [
                {
                    "preset_name": "LATEST",
                    "latest_by": "ENTITIES_CREATED_AT"
                }
            ],
            "filters": {
                "logical_operator": "and",
                "conditions": [
                    ["code", "contains", "hero"]
                ]
            }
        }))
    );
}

#[tokio::test]
async fn entity_find_rejects_conflicting_filter_inputs() {
    let app = App::new(RecordingTransport::default());

    let conflicting_filters = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "filters": { "id": 42 },
                "filter_dsl": "id == 42"
            })),
            None,
        )
        .await
        .expect_err("conflicting filters should be rejected");
    let conflicting_search = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "search": { "filters": [] },
                "filter_dsl": "id == 42"
            })),
            None,
        )
        .await
        .expect_err("search and filter_dsl should conflict");
    let duplicate_presets = app
        .entity_find(
            overrides(),
            "Asset",
            Some(json!({
                "search": {
                    "additional_filter_presets": [{ "preset_name": "LATEST" }]
                },
                "additional_filter_presets": [{ "preset_name": "LATEST" }]
            })),
            None,
        )
        .await
        .expect_err("duplicate presets should be rejected");

    assert_eq!(conflicting_filters.envelope().code, "INVALID_INPUT");
    assert_eq!(conflicting_search.envelope().code, "INVALID_INPUT");
    assert_eq!(duplicate_presets.envelope().code, "INVALID_INPUT");
}

#[tokio::test]
async fn entity_find_one_returns_first_match_and_forces_single_page() {
    let transport = FindOneTransport::new(json!({
        "data": [
            {"id": 7, "type": "Shot", "code": "sh010"},
            {"id": 8, "type": "Shot", "code": "sh020"}
        ]
    }));
    let app = App::new(transport.clone());

    let response = app
        .entity_find_one(
            overrides(),
            "Shot",
            Some(json!({
                "fields": ["code"],
                "page": { "number": 3, "size": 50 },
                "filter_dsl": "code ~ 'sh'"
            })),
            None,
        )
        .await
        .expect("find-one should succeed");

    assert_eq!(response["id"], 7);
    assert_eq!(response["code"], "sh010");

    let calls = transport.calls();
    let params = calls.first().expect("find params recorded");
    assert_eq!(query_value(params, "page[number]"), Some("3"));
    assert_eq!(query_value(params, "page[size]"), Some("1"));
}

#[tokio::test]
async fn entity_find_one_returns_null_when_no_match_exists() {
    let transport = FindOneTransport::new(json!({ "data": [] }));
    let app = App::new(transport);

    let response = app
        .entity_find_one(overrides(), "Shot", Some(json!({"fields": ["code"]})), None)
        .await
        .expect("empty find-one should still succeed");

    assert_eq!(response, Value::Null);
}

#[tokio::test]
async fn entity_write_commands_apply_dry_run_and_policy_guards() {
    let transport = RecordingTransport::default();
    let app = App::new(transport.clone());

    let create_dry_run = app
        .entity_create(
            overrides(),
            "Version",
            json!({"data": {"type": "Version"}}),
            true,
        )
        .await
        .expect("create dry-run succeeds");
    let update_dry_run = app
        .entity_update(
            overrides(),
            "Task",
            42,
            json!({"data": {"type": "Task", "id": 42}}),
            true,
        )
        .await
        .expect("update dry-run succeeds");
    let delete_dry_run = app
        .entity_delete(overrides(), "Playlist", 99, true, false)
        .await
        .expect("delete dry-run succeeds");
    let delete_error = app
        .entity_delete(overrides(), "Playlist", 99, false, false)
        .await
        .expect_err("delete without yes should be blocked");
    let revive_dry_run = app
        .entity_revive(overrides(), "Shot", 860, true)
        .await
        .expect("revive dry-run succeeds");
    let revive_response = app
        .entity_revive(overrides(), "Shot", 860, false)
        .await
        .expect("revive succeeds");

    assert_eq!(create_dry_run["dry_run"], true);
    assert_eq!(create_dry_run["plan"]["path"], "/api/v1.1/entity/versions");
    assert_eq!(update_dry_run["plan"]["path"], "/api/v1.1/entity/tasks/42");
    assert_eq!(
        delete_dry_run["plan"]["path"],
        "/api/v1.1/entity/playlists/99"
    );
    assert_eq!(revive_dry_run["plan"]["path"], "/api3/json");
    assert_eq!(revive_dry_run["plan"]["body"]["method_name"], "revive");
    assert_eq!(revive_response, json!(true));
    assert_eq!(delete_error.envelope().code, "POLICY_BLOCKED");
    assert_eq!(
        transport.snapshot().entity_revive_calls,
        vec![("Shot".to_string(), 860)]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn entity_batch_get_runs_concurrently_and_preserves_order() {
    let transport = SlowGetTransport::default();
    let app = App::new(transport.clone());

    let response = app
        .entity_batch_get(overrides(), "Shot", json!([1, 2, 3, 4]))
        .await
        .expect("batch get succeeds");

    let ids = response["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|item| item["id"].as_u64().expect("id"))
        .collect::<Vec<_>>();

    assert_eq!(ids, vec![1, 2, 3, 4]);
    assert!(
        transport.max_in_flight() > 1,
        "batch get should execute concurrently"
    );
}

#[tokio::test]
async fn entity_batch_find_reports_item_level_errors() {
    let app = App::new(RecordingTransport::default());

    let response = app
        .entity_batch_find(
            overrides(),
            "Asset",
            json!([
                {"fields": ["code"], "query": {"page[size]": 10}},
                {"filter_dsl": 42}
            ]),
        )
        .await
        .expect("batch find returns aggregated result");

    assert_eq!(response["total"], 2);
    assert_eq!(response["success_count"], 1);
    assert_eq!(response["failure_count"], 1);
    assert_eq!(response["results"][0]["ok"], true);
    assert_eq!(response["results"][1]["ok"], false);
    assert_eq!(response["results"][1]["error"]["code"], "INVALID_INPUT");
}

#[tokio::test]
async fn entity_batch_write_commands_aggregate_partial_failures() {
    let app = App::new(RecordingTransport::default());

    let create_response = app
        .entity_batch_create(
            overrides(),
            "Version",
            json!([
                {"data": {"type": "Version"}},
                {"fail": true}
            ]),
            false,
        )
        .await
        .expect("batch create returns aggregated result");
    let update_response = app
        .entity_batch_update(
            overrides(),
            "Task",
            json!([
                {"id": 42, "body": {"data": {"id": 42}}},
                {"id": 99, "body": {"data": {"id": 99}}}
            ]),
            false,
        )
        .await
        .expect("batch update returns aggregated result");
    let delete_response = app
        .entity_batch_delete(overrides(), "Task", json!([12, 13]), false, true)
        .await
        .expect("batch delete returns aggregated result");

    assert_eq!(create_response["failure_count"], 1);
    assert_eq!(create_response["results"][1]["error"]["code"], "API_ERROR");
    assert_eq!(update_response["failure_count"], 1);
    assert_eq!(update_response["results"][1]["id"], 99);
    assert_eq!(delete_response["failure_count"], 1);
    assert_eq!(delete_response["results"][1]["id"], 13);
}
