use fpt_domain::transport::{FindParams, RestTransport, ShotgridTransport};
use fpt_domain::{ConnectionSettings, Credentials};
use httpmock::Mock;
use httpmock::prelude::*;

use serde_json::json;

fn script_config(server: &MockServer) -> ConnectionSettings {
    ConnectionSettings {
        site: server.base_url(),
        credentials: Credentials::Script {
            script_name: "openclaw".to_string(),
            script_key: "secret-key".to_string(),
        },
        api_version: "v1.1".to_string(),
    }
}

fn mock_auth(server: &MockServer) -> Mock<'_> {
    server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/auth/access_token")
            .header("accept", "application/json")
            .body_contains("grant_type=client_credentials")
            .body_contains("client_id=openclaw")
            .body_contains("client_secret=secret-key");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "access_token": "token-123",
                "token_type": "Bearer",
                "expires_in": 3600
            }));
    })
}

#[tokio::test]
async fn auth_test_uses_script_oauth_form_payload() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let transport = RestTransport::default();

    let response = transport
        .auth_test(&script_config(&server))
        .await
        .expect("auth test succeeds");

    assert_eq!(response["ok"], true);
    assert_eq!(response["grant_type"], "client_credentials");
    assert_eq!(response["token_received"], true);
    assert_eq!(auth.hits(), 1);
}

#[tokio::test]
async fn schema_commands_reuse_cached_token_and_hit_expected_paths() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let schema_entities = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": ["Asset", "Shot"]}));
    });
    let schema_fields = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/schema/Shot/fields")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"code": {"name": "Code"}}}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .schema_entities(&config)
        .await
        .expect("schema entities succeeds");
    transport
        .schema_fields(&config, "Shot")
        .await
        .expect("schema fields succeeds");

    assert_eq!(auth.hits(), 1, "access token should be reused in-process");
    assert_eq!(schema_entities.hits(), 1);
    assert_eq!(schema_fields.hits(), 1);
}

#[tokio::test]
async fn entity_get_and_find_use_expected_read_endpoints() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_get = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/42")
            .query_param("fields", "code,sg_status_list")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 42}}));
    });
    let entity_find = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/assets")
            .query_param("fields", "code")
            .query_param("page[size]", "25")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 7}]}));
    });
    let entity_search = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/assets/_search")
            .query_param("fields", "code")
            .header("authorization", "Bearer token-123")
            .header("content-type", "application/vnd+shotgun.api3_hash+json")
            .json_body(json!({
                "filters": {
                    "logical_operator": "and",
                    "conditions": [
                        {"path": "code", "relation": "contains", "values": ["bunny"]}
                    ]
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": [{"id": 8}]}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .entity_get(
            &config,
            "Shot",
            42,
            Some(vec!["code".to_string(), "sg_status_list".to_string()]),
        )
        .await
        .expect("entity get succeeds");
    transport
        .entity_find(
            &config,
            "Asset",
            FindParams {
                query: vec![
                    ("fields".to_string(), "code".to_string()),
                    ("page[size]".to_string(), "25".to_string()),
                ],
                search: None,
            },
        )
        .await
        .expect("entity find succeeds");
    transport
        .entity_find(
            &config,
            "Asset",
            FindParams {
                query: vec![("fields".to_string(), "code".to_string())],
                search: Some(json!({
                    "filters": {
                        "logical_operator": "and",
                        "conditions": [
                            {"path": "code", "relation": "contains", "values": ["bunny"]}
                        ]
                    }
                })),
            },
        )
        .await
        .expect("entity search succeeds");

    assert_eq!(auth.hits(), 1);
    assert_eq!(entity_get.hits(), 1);
    assert_eq!(entity_find.hits(), 1);
    assert_eq!(entity_search.hits(), 1);
}

#[tokio::test]
async fn note_threads_use_documented_note_thread_endpoint() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let note_threads = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/notes/100/thread_contents")
            .query_param("fields", "content,user")
            .header("authorization", "Bearer token-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": []}));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    transport
        .note_threads(
            &config,
            100,
            &[("fields".to_string(), "content,user".to_string())],
        )
        .await
        .expect("note threads succeeds");

    assert_eq!(auth.hits(), 1);
    assert_eq!(note_threads.hits(), 1);
}

#[tokio::test]
async fn rpc_methods_use_expected_paths_and_payloads() {
    let server = MockServer::start();
    let revive = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "revive",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "type": "Shot",
                    "id": 860
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"results": true}));
    });
    let work_schedule = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "work_schedule_read",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "start_date": "2026-03-16",
                    "end_date": "2026-03-20"
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": {
                    "2026-03-16": {
                        "working": true,
                        "reason": "STUDIO_WORK_WEEK"
                    }
                }
            }));
    });
    let summarize = server.mock(|when, then| {
        when.method(POST).path("/api3/json").json_body(json!({
            "method_name": "summarize",
            "params": [
                {
                    "script_name": "openclaw",
                    "script_key": "secret-key"
                },
                {
                    "type": "Version",
                    "filters": {
                        "filter_operator": "all",
                        "filters": [["sg_status_list", "is", "ip"]]
                    },
                    "summaries": [
                        {
                            "field": "id",
                            "type": "record_count"
                        }
                    ]
                }
            ]
        }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "results": {
                    "summaries": {
                        "id": {
                            "record_count": 3
                        }
                    },
                    "groups": []
                }
            }));
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let revive_response = transport
        .entity_revive(&config, "Shot", 860)
        .await
        .expect("entity revive succeeds");
    let work_schedule_response = transport
        .work_schedule_read(
            &config,
            &json!({
                "start_date": "2026-03-16",
                "end_date": "2026-03-20"
            }),
        )
        .await
        .expect("work schedule succeeds");
    let summarize_response = transport
        .entity_summarize(
            &config,
            "Version",
            &json!({
                "filters": {
                    "filter_operator": "all",
                    "filters": [["sg_status_list", "is", "ip"]]
                },
                "summaries": [
                    {
                        "field": "id",
                        "type": "record_count"
                    }
                ]
            }),
        )
        .await
        .expect("entity summarize succeeds");

    assert_eq!(revive.hits(), 1);
    assert_eq!(work_schedule.hits(), 1);
    assert_eq!(summarize.hits(), 1);
    assert_eq!(revive_response, json!(true));
    assert_eq!(work_schedule_response["2026-03-16"]["working"], true);
    assert_eq!(summarize_response["summaries"]["id"]["record_count"], 3);
}

#[tokio::test]
async fn entity_write_commands_use_expected_methods_and_parse_empty_delete() {
    let server = MockServer::start();
    let auth = mock_auth(&server);
    let entity_create = server.mock(|when, then| {
        when.method(POST)
            .path("/api/v1.1/entity/versions")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data": {"type": "Version"}}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 10}}));
    });
    let entity_update = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/v1.1/entity/tasks/42")
            .header("authorization", "Bearer token-123")
            .json_body(json!({"data": {"id": 42}}));
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({"data": {"id": 42, "updated": true}}));
    });
    let entity_delete = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/v1.1/entity/playlists/99")
            .header("authorization", "Bearer token-123");
        then.status(204);
    });
    let transport = RestTransport::default();
    let config = script_config(&server);

    let create_response = transport
        .entity_create(&config, "Version", &json!({"data": {"type": "Version"}}))
        .await
        .expect("entity create succeeds");
    let update_response = transport
        .entity_update(&config, "Task", 42, &json!({"data": {"id": 42}}))
        .await
        .expect("entity update succeeds");
    let delete_response = transport
        .entity_delete(&config, "Playlist", 99)
        .await
        .expect("entity delete succeeds");

    assert_eq!(auth.hits(), 1);
    assert_eq!(entity_create.hits(), 1);
    assert_eq!(entity_update.hits(), 1);
    assert_eq!(entity_delete.hits(), 1);
    assert_eq!(create_response["data"]["id"], 10);
    assert_eq!(update_response["data"]["updated"], true);
    assert_eq!(delete_response["ok"], true);
    assert_eq!(delete_response["status"], 204);
}

#[tokio::test]
async fn rest_errors_map_auth_and_api_failures() {
    let auth_server = MockServer::start();
    let auth_failure = auth_server.mock(|when, then| {
        when.method(POST).path("/api/v1.1/auth/access_token");
        then.status(400)
            .header("content-type", "application/json")
            .json_body(json!({"errors": ["Can't authenticate script 'openclaw'"]}));
    });
    let auth_transport = RestTransport::default();
    let auth_error = auth_transport
        .auth_test(&script_config(&auth_server))
        .await
        .expect_err("auth failure should be surfaced");

    let api_server = MockServer::start();
    let api_auth = mock_auth(&api_server);
    let api_failure = api_server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1.1/entity/shots/404")
            .header("authorization", "Bearer token-123");
        then.status(404)
            .header("content-type", "application/json")
            .json_body(json!({"errors": ["record not found"]}));
    });
    let api_transport = RestTransport::default();
    let api_error = api_transport
        .entity_get(&script_config(&api_server), "Shot", 404, None)
        .await
        .expect_err("api failure should be surfaced");

    assert_eq!(auth_failure.hits(), 1);
    assert_eq!(auth_error.envelope().code, "AUTH_FAILED");
    assert_eq!(auth_error.envelope().transport.as_deref(), Some("rest"));
    assert_eq!(api_auth.hits(), 1);
    assert_eq!(api_failure.hits(), 1);
    assert_eq!(api_error.envelope().code, "API_ERROR");
    assert_eq!(api_error.envelope().transport.as_deref(), Some("rest"));
}
