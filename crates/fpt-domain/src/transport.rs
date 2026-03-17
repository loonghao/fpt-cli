use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use fpt_core::{AppError, Result, RiskLevel};
use reqwest::{Client, Method, Response, StatusCode};
use serde::Serialize;
use serde_json::{Value, json};
use url::Url;

use crate::config::{ConnectionSettings, Credentials};

/// Maximum number of retry attempts for rate-limited or transient failures.
const MAX_RETRY_ATTEMPTS: u32 = 5;
/// Base delay for exponential backoff (milliseconds).
const RETRY_BASE_DELAY_MS: u64 = 500;
/// Maximum backoff delay cap (milliseconds).
const RETRY_MAX_DELAY_MS: u64 = 30_000;

#[derive(Debug, Clone, Default)]
pub struct FindParams {
    pub query: Vec<(String, String)>,
    pub search: Option<Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct UploadUrlRequest<'a> {
    pub entity: &'a str,
    pub id: u64,
    pub field_name: &'a str,
    pub file_name: &'a str,
    pub content_type: Option<&'a str>,
    pub multipart_upload: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestPlan {
    pub transport: &'static str,
    pub method: &'static str,
    pub path: String,
    pub risk: RiskLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct AccessTokenPayload {
    access_token: String,
    token_type: Option<String>,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
struct CachedAccessToken {
    cache_key: String,
    payload: AccessTokenPayload,
    expires_at: Option<Instant>,
}

#[async_trait]
pub trait ShotgridTransport {
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn server_info(&self, site: &str) -> Result<Value>;
    async fn schema_entities(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn schema_fields(&self, config: &ConnectionSettings, entity: &str) -> Result<Value>;
    async fn entity_get(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value>;
    async fn entity_find(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        params: FindParams,
    ) -> Result<Value>;
    async fn entity_summarize(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value>;
    async fn entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn work_schedule_read(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
    async fn upload_url(
        &self,
        config: &ConnectionSettings,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value>;
    async fn download_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value>;
    async fn thumbnail_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn activity_stream(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn event_log_entries(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn preferences_get(&self, config: &ConnectionSettings) -> Result<Value>;
    async fn entity_followers(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value>;
    async fn entity_follow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value>;
    async fn entity_unfollow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value>;
    async fn note_threads(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        params: &[(String, String)],
    ) -> Result<Value>;
    async fn schema_field_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_field_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
        body: &Value,
    ) -> Result<Value>;
    async fn schema_field_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value>;
    async fn hierarchy(&self, config: &ConnectionSettings, body: &Value) -> Result<Value>;
}

#[derive(Debug, Clone)]
pub struct RestTransport {
    client: Client,
    token_cache: Arc<Mutex<Option<CachedAccessToken>>>,
}

impl Default for RestTransport {
    fn default() -> Self {
        Self {
            client: Client::new(),
            token_cache: Arc::new(Mutex::new(None)),
        }
    }
}

impl RestTransport {
    fn build_url(
        &self,
        config: &ConnectionSettings,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Url> {
        let mut url = Url::parse(&format!("{}/api/{}/", config.site, config.api_version))
            .map_err(|error| {
                AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                    .with_operation("build_rest_url")
                    .with_invalid_field("site")
                    .with_received_value(&config.site)
                    .with_hint("Provide a valid ShotGrid base URL, for example `https://example.shotgrid.autodesk.com`.")
            })?;
        url = url.join(path.trim_start_matches('/')).map_err(|error| {
            AppError::internal(format!("could not build REST URL `{path}`: {error}"))
                .with_operation("build_rest_url")
                .with_resource(path)
        })?;

        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }

        Ok(url)
    }

    fn build_rpc_url(&self, site: &str) -> Result<Url> {
        let normalized_site = site.trim_end_matches('/');
        let mut url = Url::parse(&format!("{normalized_site}/")).map_err(|error| {
            AppError::invalid_input(format!("invalid ShotGrid site URL: {error}"))
                .with_operation("build_rpc_url")
                .with_invalid_field("site")
                .with_received_value(site)
                .with_hint("Provide a valid ShotGrid base URL, for example `https://example.shotgrid.autodesk.com`.")
        })?;
        url = url.join("api3/json").map_err(|error| {
            AppError::internal(format!("could not build RPC URL: {error}"))
                .with_operation("build_rpc_url")
                .with_resource("api3/json")
        })?;
        Ok(url)
    }

    fn rpc_auth_params(config: &ConnectionSettings) -> Value {
        match &config.credentials {
            Credentials::Script {
                script_name,
                script_key,
            } => json!({
                "script_name": script_name,
                "script_key": script_key,
            }),
            Credentials::UserPassword {
                username,
                password,
                auth_token,
            } => {
                let mut payload = json!({
                    "user_login": username,
                    "user_password": password,
                });
                if let Some(auth_token) = auth_token {
                    payload["auth_token"] = Value::String(auth_token.clone());
                }
                payload
            }
            Credentials::SessionToken { session_token } => json!({
                "session_token": session_token,
                "reject_if_expired": true,
            }),
        }
    }

    fn extract_rpc_results(response: Value) -> Value {
        response.get("results").cloned().unwrap_or(response)
    }

    fn token_cache_key(config: &ConnectionSettings) -> String {
        format!(
            "{}|{}|{}|{:?}",
            config.site,
            config.api_version,
            config.auth_mode().grant_type(),
            config.credentials.principal(),
        )
    }

    fn cached_access_token(
        &self,
        config: &ConnectionSettings,
    ) -> Result<Option<AccessTokenPayload>> {
        let cache = self.token_cache.lock().map_err(|_| {
            AppError::internal("token cache is poisoned").with_operation("read_token_cache")
        })?;
        let Some(cached) = cache.as_ref() else {
            return Ok(None);
        };

        if cached.cache_key != Self::token_cache_key(config) {
            return Ok(None);
        }

        if let Some(expires_at) = cached.expires_at
            && Instant::now() >= expires_at
        {
            return Ok(None);
        }

        Ok(Some(cached.payload.clone()))
    }

    fn store_access_token(
        &self,
        config: &ConnectionSettings,
        payload: &AccessTokenPayload,
    ) -> Result<()> {
        let expires_at = payload.expires_in.map(|seconds| {
            let effective_seconds = if seconds > 30 { seconds - 30 } else { seconds };
            Instant::now() + Duration::from_secs(effective_seconds)
        });

        let mut cache = self.token_cache.lock().map_err(|_| {
            AppError::internal("token cache is poisoned").with_operation("write_token_cache")
        })?;
        *cache = Some(CachedAccessToken {
            cache_key: Self::token_cache_key(config),
            payload: payload.clone(),
            expires_at,
        });
        Ok(())
    }

    async fn access_token_response(
        &self,
        config: &ConnectionSettings,
    ) -> Result<AccessTokenPayload> {
        if let Some(cached) = self.cached_access_token(config)? {
            if std::env::var("FPT_DEBUG").is_ok() {
                eprintln!("[debug] reuse cached access token for {}", config.site);
            }
            return Ok(cached);
        }

        let url = self.build_url(config, "auth/access_token", &[])?;

        let mut form: Vec<(&str, &str)> = Vec::new();
        match &config.credentials {
            Credentials::Script {
                script_name,
                script_key,
            } => {
                form.push(("grant_type", "client_credentials"));
                form.push(("client_id", script_name));
                form.push(("client_secret", script_key));
            }
            Credentials::UserPassword {
                username,
                password,
                auth_token,
            } => {
                form.push(("grant_type", "password"));
                form.push(("username", username));
                form.push(("password", password));
                if let Some(token) = auth_token {
                    form.push(("auth_token", token));
                }
            }
            Credentials::SessionToken { session_token } => {
                form.push(("grant_type", "session_token"));
                form.push(("session_token", session_token));
            }
        };

        if std::env::var("FPT_DEBUG").is_ok() {
            let masked_form: Vec<String> = form
                .iter()
                .map(|(k, v)| {
                    if *k == "client_secret" || *k == "password" || *k == "session_token" {
                        format!("{k}=***REDACTED(len={})***", v.len())
                    } else {
                        format!("{k}={v}")
                    }
                })
                .collect();
            eprintln!("[debug] POST {} form=[{}]", url, masked_form.join("&"));
        }

        let response = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not request a ShotGrid access token: {error}"
                ))
                .with_operation("request_access_token")
                .with_transport("rest")
                .with_resource("auth/access_token")
                .with_retryable_reason("transient network failure while requesting an access token")
                .retryable(true)
            })?;

        let body = Self::parse_response(response, "rest").await?;
        let access_token = body
            .get("access_token")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::auth("ShotGrid access token response is missing `access_token`")
                    .with_operation("request_access_token")
                    .with_transport("rest")
                    .with_resource("auth/access_token")
                    .with_expected_shape("a JSON object containing a string field `access_token`")
                    .with_hint("Verify the credentials and auth mode, then retry the authentication request.")
                    .with_detail("response_body", body.clone())
            })?;

        let payload = AccessTokenPayload {
            access_token: access_token.to_string(),
            token_type: body
                .get("token_type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            expires_in: body.get("expires_in").and_then(Value::as_u64),
            refresh_token: body
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        };
        self.store_access_token(config, &payload)?;
        Ok(payload)
    }

    async fn authorized_json_request(
        &self,
        config: &ConnectionSettings,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let max_attempts = Self::max_retry_attempts();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let token = self.access_token_response(config).await?;
            let url = self.build_url(config, path, query)?;
            let mut request = self
                .client
                .request(method.clone(), url)
                .header("accept", "application/json")
                .bearer_auth(token.access_token);

            if let Some(body) = body {
                request = request.json(body);
            }

            let response = request.send().await.map_err(|error| {
                AppError::network(format!("could not send the ShotGrid REST request: {error}"))
                    .with_operation("authorized_json_request")
                    .with_transport("rest")
                    .with_resource(path)
                    .with_retryable_reason("transient network failure while sending a REST request")
                    .retryable(true)
            })?;

            let status = response.status();
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < max_attempts {
                let delay = Self::backoff_delay(attempt);
                if std::env::var("FPT_DEBUG").is_ok() {
                    eprintln!(
                        "[debug] rate-limited (429) on attempt {attempt}/{max_attempts}, retrying after {}ms",
                        delay.as_millis()
                    );
                }
                tokio::time::sleep(delay).await;
                continue;
            }

            return Self::parse_response(response, "rest").await;
        }
    }

    async fn authorized_search_request(
        &self,
        config: &ConnectionSettings,
        path: &str,
        query: &[(String, String)],
        body: &Value,
    ) -> Result<Value> {
        let serialized_body = serde_json::to_vec(body).map_err(|error| {
            AppError::internal(format!(
                "could not serialize the `_search` request body as JSON: {error}"
            ))
            .with_operation("serialize_search_request")
            .with_transport("rest")
            .with_resource(path)
            .with_expected_shape(
                "a JSON object or array accepted by the ShotGrid `_search` endpoint",
            )
        })?;

        let max_attempts = Self::max_retry_attempts();
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let token = self.access_token_response(config).await?;
            let url = self.build_url(config, path, query)?;

            let response = self
                .client
                .request(Method::POST, url)
                .header("accept", "application/json")
                .header("content-type", "application/vnd+shotgun.api3_hash+json")
                .bearer_auth(token.access_token)
                .body(serialized_body.clone())
                .send()
                .await
                .map_err(|error| {
                    AppError::network(format!(
                        "could not send the ShotGrid REST `_search` request: {error}"
                    ))
                    .with_operation("authorized_search_request")
                    .with_transport("rest")
                    .with_resource(path)
                    .with_retryable_reason(
                        "transient network failure while sending a `_search` request",
                    )
                    .retryable(true)
                })?;

            let status = response.status();
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < max_attempts {
                let delay = Self::backoff_delay(attempt);
                if std::env::var("FPT_DEBUG").is_ok() {
                    eprintln!(
                        "[debug] rate-limited (429) on attempt {attempt}/{max_attempts}, retrying after {}ms",
                        delay.as_millis()
                    );
                }
                tokio::time::sleep(delay).await;
                continue;
            }

            return Self::parse_response(response, "rest").await;
        }
    }

    /// Returns the maximum number of retry attempts, configurable via `FPT_MAX_RETRIES`.
    fn max_retry_attempts() -> u32 {
        std::env::var("FPT_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(MAX_RETRY_ATTEMPTS)
    }

    /// Computes the exponential backoff delay for a given attempt number (1-based).
    ///
    /// Uses full jitter: `delay = random(0, min(cap, base * 2^attempt))`.
    /// Falls back to a deterministic formula when the random source is unavailable.
    fn backoff_delay(attempt: u32) -> Duration {
        let exp = RETRY_BASE_DELAY_MS.saturating_mul(1u64 << attempt.min(10));
        let capped = exp.min(RETRY_MAX_DELAY_MS);
        // Simple deterministic jitter: use attempt as a pseudo-random seed.
        let jitter = (attempt as u64 * 137 + 42) % (capped.max(1));
        Duration::from_millis(jitter)
    }

    async fn rpc_request(
        &self,
        site: &str,
        method_name: &str,
        params: Vec<Value>,
    ) -> Result<Value> {
        let url = self.build_rpc_url(site)?;
        let body = json!({
            "method_name": method_name,
            "params": params,
        });

        let response = self
            .client
            .request(Method::POST, url)
            .header("accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                AppError::network(format!(
                    "could not send ShotGrid RPC request `{method_name}`: {error}"
                ))
                .with_operation("rpc_request")
                .with_transport("rpc")
                .with_resource(method_name)
                .with_retryable_reason("transient network failure while sending an RPC request")
                .retryable(true)
            })?;

        let parsed = Self::parse_response(response, "rpc").await?;
        Ok(Self::extract_rpc_results(parsed))
    }

    async fn parse_response(response: Response, transport: &'static str) -> Result<Value> {
        let status = response.status();
        let text = response.text().await.map_err(|error| {
            AppError::network(format!(
                "could not read ShotGrid {transport} response body: {error}"
            ))
            .with_operation("parse_response")
            .with_transport(transport)
            .with_retryable_reason("transient network failure while reading the response body")
            .retryable(true)
        })?;

        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(json!({
                    "ok": true,
                    "status": status.as_u16(),
                }));
            }

            return serde_json::from_str(&text).map_err(|error| {
                AppError::api(format!(
                    "ShotGrid {transport} response was not valid JSON: {error}"
                ))
                .with_operation("parse_response")
                .with_transport(transport)
                .with_expected_shape("a valid JSON response body")
                .with_hint("Inspect the raw response body to confirm whether the remote service returned HTML, plain text, or malformed JSON.")
                .with_detail("raw_response", text.clone())
            });
        }

        let details = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
        let is_auth_error = matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
            || (status == StatusCode::BAD_REQUEST && text.contains("authenticate"));

        let error = if is_auth_error {
            AppError::auth(format!(
                "ShotGrid rejected the {transport} request with HTTP status {status}"
            ))
        } else {
            AppError::api(format!(
                "ShotGrid returned HTTP status {status} for the {transport} request"
            ))
        };

        Err(error
            .with_operation("parse_response")
            .with_transport(transport)
            .with_http_status(status.as_u16())
            .with_hint("Inspect the response details and verify the request path, permissions, and payload.")
            .with_detail("response_body", details))
    }
}

#[async_trait]
impl ShotgridTransport for RestTransport {
    async fn auth_test(&self, config: &ConnectionSettings) -> Result<Value> {
        let token = self.access_token_response(config).await?;
        Ok(json!({
            "ok": true,
            "transport": "rest",
            "profile": config.summary(),
            "grant_type": config.auth_mode().grant_type(),
            "token_received": !token.access_token.is_empty(),
            "token_type": token.token_type,
            "expires_in": token.expires_in,
            "refresh_token_received": token.refresh_token.is_some(),
        }))
    }

    async fn server_info(&self, site: &str) -> Result<Value> {
        self.rpc_request(site, "info", Vec::new()).await
    }

    async fn schema_entities(&self, config: &ConnectionSettings) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "schema", &[], None)
            .await
    }

    async fn schema_fields(&self, config: &ConnectionSettings, entity: &str) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::GET,
            &format!("schema/{entity}/fields"),
            &[],
            None,
        )
        .await
    }

    async fn entity_get(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        fields: Option<Vec<String>>,
    ) -> Result<Value> {
        let mut query = Vec::new();
        if let Some(fields) = fields.filter(|fields| !fields.is_empty()) {
            query.push(("fields".to_string(), fields.join(",")));
        }

        self.authorized_json_request(
            config,
            Method::GET,
            &format!("entity/{}/{}", entity_collection_path(entity), id),
            &query,
            None,
        )
        .await
    }

    async fn entity_find(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        params: FindParams,
    ) -> Result<Value> {
        let path = format!("entity/{}", entity_collection_path(entity));

        if let Some(search_body) = params.search {
            return self
                .authorized_search_request(
                    config,
                    &format!("{path}/_search"),
                    &params.query,
                    &search_body,
                )
                .await;
        }

        self.authorized_json_request(config, Method::GET, &path, &params.query, None)
            .await
    }

    async fn entity_summarize(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let mut payload = body.clone();
        payload["type"] = Value::String(entity.to_string());
        self.rpc_request(
            &config.site,
            "summarize",
            vec![Self::rpc_auth_params(config), payload],
        )
        .await
    }

    async fn entity_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::POST,
            &format!("entity/{}", entity_collection_path(entity)),
            &[],
            Some(body),
        )
        .await
    }

    async fn entity_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        body: &Value,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::PUT,
            &format!("entity/{}/{}", entity_collection_path(entity), id),
            &[],
            Some(body),
        )
        .await
    }

    async fn entity_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::DELETE,
            &format!("entity/{}/{}", entity_collection_path(entity), id),
            &[],
            None,
        )
        .await
    }

    async fn entity_revive(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        self.rpc_request(
            &config.site,
            "revive",
            vec![
                Self::rpc_auth_params(config),
                json!({
                    "type": entity,
                    "id": id,
                }),
            ],
        )
        .await
    }

    async fn work_schedule_read(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.rpc_request(
            &config.site,
            "work_schedule_read",
            vec![Self::rpc_auth_params(config), body.clone()],
        )
        .await
    }

    async fn upload_url(
        &self,
        config: &ConnectionSettings,
        request: UploadUrlRequest<'_>,
    ) -> Result<Value> {
        let path = format!(
            "entity/{}/{}/{}/_upload",
            entity_collection_path(request.entity),
            request.id,
            request.field_name
        );
        let mut query = vec![
            ("filename".to_string(), request.file_name.to_string()),
            (
                "multipart_upload".to_string(),
                request.multipart_upload.to_string(),
            ),
        ];
        if let Some(ct) = request.content_type {
            query.push(("content_type".to_string(), ct.to_string()));
        }
        self.authorized_json_request(config, Method::GET, &path, &query, None)
            .await
    }

    async fn download_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!(
            "entity/{}/{}/{}/_download",
            entity_collection_path(entity),
            id,
            field_name
        );
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn thumbnail_url(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = format!("entity/{}/{}/image", entity_collection_path(entity), id);
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn activity_stream(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!(
            "entity/{}/{}/activity_stream",
            entity_collection_path(entity),
            id
        );
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn event_log_entries(
        &self,
        config: &ConnectionSettings,
        params: &[(String, String)],
    ) -> Result<Value> {
        self.authorized_json_request(
            config,
            Method::GET,
            "entity/event_log_entries",
            params,
            None,
        )
        .await
    }

    async fn preferences_get(&self, config: &ConnectionSettings) -> Result<Value> {
        self.authorized_json_request(config, Method::GET, "preferences", &[], None)
            .await
    }

    async fn entity_followers(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
    ) -> Result<Value> {
        let path = format!("entity/{}/{}/followers", entity_collection_path(entity), id);
        self.authorized_json_request(config, Method::GET, &path, &[], None)
            .await
    }

    async fn entity_follow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        let path = format!("entity/{}/{}/followers", entity_collection_path(entity), id);
        self.authorized_json_request(config, Method::POST, &path, &[], Some(user))
            .await
    }

    async fn entity_unfollow(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        id: u64,
        user: &Value,
    ) -> Result<Value> {
        let user_id = user.get("id").and_then(Value::as_u64).ok_or_else(|| {
            AppError::invalid_input("user object must contain a numeric `id`")
                .with_operation("entity_unfollow")
                .with_invalid_field("id")
                .with_expected_shape("a user JSON object containing a numeric field `id`")
                .with_detail("received_user", user.clone())
        })?;
        let path = format!(
            "entity/{}/{}/followers/{}",
            entity_collection_path(entity),
            id,
            user_id
        );
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn note_threads(
        &self,
        config: &ConnectionSettings,
        note_id: u64,
        params: &[(String, String)],
    ) -> Result<Value> {
        let path = format!("entity/notes/{note_id}/thread_contents");
        self.authorized_json_request(config, Method::GET, &path, params, None)
            .await
    }

    async fn schema_field_create(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields");
        self.authorized_json_request(config, Method::POST, &path, &[], Some(body))
            .await
    }

    async fn schema_field_update(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::PUT, &path, &[], Some(body))
            .await
    }

    async fn schema_field_delete(
        &self,
        config: &ConnectionSettings,
        entity: &str,
        field_name: &str,
    ) -> Result<Value> {
        let path = format!("schema/{entity}/fields/{field_name}");
        self.authorized_json_request(config, Method::DELETE, &path, &[], None)
            .await
    }

    async fn hierarchy(&self, config: &ConnectionSettings, body: &Value) -> Result<Value> {
        self.authorized_json_request(config, Method::POST, "hierarchy/_search", &[], Some(body))
            .await
    }
}

pub fn entity_collection_path(entity: &str) -> String {
    let entity = entity.trim();
    let chars: Vec<char> = entity.chars().collect();
    let mut output = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|value| chars.get(value))
            .copied();
        let next = chars.get(index + 1).copied();

        if ch.is_ascii_uppercase() {
            let should_split = index > 0
                && (previous
                    .is_some_and(|value| value.is_ascii_lowercase() || value.is_ascii_digit())
                    || (previous.is_some_and(|value| value.is_ascii_uppercase())
                        && next.is_some_and(|value| value.is_ascii_lowercase())));

            if should_split {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            continue;
        }

        if ch.is_ascii_digit() {
            if previous.is_some_and(|value| value.is_ascii_alphabetic()) && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch);
            continue;
        }

        if matches!(ch, '-' | ' ') {
            if !output.ends_with('_') {
                output.push('_');
            }
            continue;
        }

        output.push(ch.to_ascii_lowercase());
    }

    if !output.ends_with('s') {
        output.push('s');
    }

    output
}

pub fn plan_entity_create(api_version: &str, entity: &str, body: Value) -> RequestPlan {
    RequestPlan {
        transport: "rest",
        method: "POST",
        path: format!(
            "/api/{api_version}/entity/{}",
            entity_collection_path(entity)
        ),
        risk: RiskLevel::Write,
        query: Vec::new(),
        body: Some(body),
        notes: vec!["dry-run: shows the planned request without making a network call".to_string()],
    }
}

pub fn plan_entity_update(api_version: &str, entity: &str, id: u64, body: Value) -> RequestPlan {
    RequestPlan {
        transport: "rest",
        method: "PUT",
        path: format!(
            "/api/{api_version}/entity/{}/{}",
            entity_collection_path(entity),
            id
        ),
        risk: RiskLevel::Write,
        query: Vec::new(),
        body: Some(body),
        notes: vec!["dry-run: shows the planned request without making a network call".to_string()],
    }
}

pub fn plan_entity_delete(api_version: &str, entity: &str, id: u64) -> RequestPlan {
    RequestPlan {
        transport: "rest",
        method: "DELETE",
        path: format!(
            "/api/{api_version}/entity/{}/{}",
            entity_collection_path(entity),
            id
        ),
        risk: RiskLevel::Destructive,
        query: Vec::new(),
        body: None,
        notes: vec![
            "dry-run: shows the planned request without making a network call".to_string(),
            "actual deletion requires explicit `--yes` flag".to_string(),
        ],
    }
}

pub fn plan_entity_revive(entity: &str, id: u64) -> RequestPlan {
    RequestPlan {
        transport: "rpc",
        method: "POST",
        path: "/api3/json".to_string(),
        risk: RiskLevel::Write,
        query: Vec::new(),
        body: Some(json!({
            "method_name": "revive",
            "params": [
                {
                    "type": entity,
                    "id": id,
                }
            ]
        })),
        notes: vec![
            "dry-run: shows the planned request without making a network call".to_string(),
            "RPC auth params are injected from the connection config at execution time".to_string(),
        ],
    }
}
