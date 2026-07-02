//! Read-only localhost API for external tools.

use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    body::Body,
    extract::{Query, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::JoinHandle;
use tokio::sync::oneshot;

use crate::database::Repository;
use crate::settings::{LocalApiSettings, Settings};

const SCHEMA_VERSION: u32 = 1;

pub struct LocalApiManager {
    repository: Arc<Repository>,
    version: String,
    inner: Mutex<Option<RunningServer>>,
    status: Arc<Mutex<RuntimeState>>,
}

struct RunningServer {
    cfg: LocalApiSettings,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

#[derive(Default)]
struct RuntimeState {
    running: bool,
    port: Option<u16>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalApiStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub error: Option<String>,
}

impl LocalApiManager {
    pub fn new(repository: Arc<Repository>, version: String) -> Self {
        Self {
            repository,
            version,
            inner: Mutex::new(None),
            status: Arc::new(Mutex::new(RuntimeState::default())),
        }
    }

    pub fn status(&self) -> LocalApiStatus {
        let st = self.status.lock().unwrap();
        LocalApiStatus {
            running: st.running,
            port: st.port,
            error: st.error.clone(),
        }
    }

    pub fn shutdown(&self) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(mut server) = guard.take() {
            if let Some(tx) = server.shutdown.take() {
                let _ = tx.send(());
            }
            server.handle.abort();
        }
        let mut st = self.status.lock().unwrap();
        st.running = false;
        st.port = None;
    }

    pub fn reconcile(&self, settings: Option<&Settings>) {
        let cfg = settings.map(|s| s.local_api.clone()).unwrap_or_default();
        let should_run = cfg.enabled && !cfg.token.is_empty() && cfg.port > 0;

        let mut guard = self.inner.lock().unwrap();

        let running = self.status.lock().unwrap().running;
        let unchanged = match guard.as_ref() {
            Some(s) => {
                should_run
                    && running
                    && s.cfg.port == cfg.port
                    && s.cfg.token == cfg.token
                    && s.cfg.allowed_origins == cfg.allowed_origins
            }
            None => false,
        };
        if unchanged {
            return;
        }

        if let Some(mut server) = guard.take() {
            if let Some(tx) = server.shutdown.take() {
                let _ = tx.send(());
            }
            server.handle.abort();
        }

        {
            let mut st = self.status.lock().unwrap();
            st.running = false;
            st.port = None;
            st.error = if cfg.enabled && cfg.token.is_empty() {
                Some("no token configured".to_string())
            } else {
                None
            };
        }

        if !should_run {
            return;
        }

        *guard = Some(self.spawn(&cfg));
    }

    fn spawn(&self, cfg: &LocalApiSettings) -> RunningServer {
        let state = ApiState {
            repository: self.repository.clone(),
            version: self.version.clone(),
            token: Arc::new(cfg.token.clone()),
            allowed_origins: Arc::new(cfg.allowed_origins.clone()),
        };
        let status = self.status.clone();
        let (tx, rx) = oneshot::channel::<()>();
        let port = cfg.port;

        let handle = tauri::async_runtime::spawn(async move {
            let listener = match bind_with_retry(port).await {
                Ok(l) => l,
                Err(e) => {
                    error!("local api failed to bind 127.0.0.1:{port}: {e}");
                    let mut st = status.lock().unwrap();
                    st.running = false;
                    st.port = None;
                    st.error = Some(format!("could not bind port {port} ({e})"));
                    return;
                }
            };

            {
                let mut st = status.lock().unwrap();
                st.running = true;
                st.port = Some(port);
                st.error = None;
            }
            info!("local api listening on 127.0.0.1:{port}");

            let app = build_router(state);
            let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                let _ = rx.await;
            });
            if let Err(e) = server.await {
                error!("local api server error: {e}");
            }

            info!("local api stopped");
            let mut st = status.lock().unwrap();
            st.running = false;
            st.port = None;
        });

        RunningServer {
            cfg: cfg.clone(),
            shutdown: Some(tx),
            handle,
        }
    }
}

#[derive(Clone)]
struct ApiState {
    repository: Arc<Repository>,
    version: String,
    token: Arc<String>,
    allowed_origins: Arc<Vec<String>>,
}

fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/v1/status", get(status_handler))
        .route("/v1/encounters", get(encounters_handler))
        .route("/v1/characters", get(characters_handler))
        .fallback(fallback_handler)
        .layer(middleware::from_fn_with_state(state.clone(), cors_middleware))
        .with_state(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    ok: bool,
    app: &'static str,
    version: String,
    schema_version: u32,
    server_time_ms: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncountersResponse {
    schema_version: u32,
    encounters: Vec<ApiEncounter>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiEncounter {
    source_id: String,
    boss: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<String>,
    fight_start_ms: i64,
    duration_ms: i64,
    cleared: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_player: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    upload_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CharactersResponse {
    schema_version: u32,
    characters: Vec<ApiCharacter>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiCharacter {
    name: String,
    class_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,
    gear_score: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncountersQuery {
    since_ms: Option<i64>,
    until_ms: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CharactersQuery {
    names: Option<String>,
}

async fn status_handler(State(state): State<ApiState>, headers: HeaderMap) -> Response {
    if let Err(e) = check_auth(&headers, &state.token) {
        return e;
    }
    Json(StatusResponse {
        ok: true,
        app: "loa-logs",
        version: state.version.clone(),
        schema_version: SCHEMA_VERSION,
        server_time_ms: now_ms(),
    })
    .into_response()
}

async fn encounters_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(q): Query<EncountersQuery>,
) -> Response {
    if let Err(e) = check_auth(&headers, &state.token) {
        return e;
    }

    let until = q.until_ms.unwrap_or_else(now_ms);
    let since = q.since_ms.unwrap_or(0);

    match state.repository.get_cleared_encounters_in_range(since, until) {
        Ok(clears) => {
            let encounters = clears
                .into_iter()
                .map(|c| ApiEncounter {
                    source_id: c.id.to_string(),
                    boss: c.boss,
                    difficulty: c.difficulty,
                    fight_start_ms: c.fight_start_ms,
                    duration_ms: c.duration_ms,
                    cleared: true,
                    local_player: c.local_player,
                    upload_id: c.upload_id,
                })
                .collect();
            Json(EncountersResponse {
                schema_version: SCHEMA_VERSION,
                encounters,
            })
            .into_response()
        }
        Err(e) => {
            error!("local api encounters query failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn characters_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(q): Query<CharactersQuery>,
) -> Response {
    if let Err(e) = check_auth(&headers, &state.token) {
        return e;
    }

    let names: Vec<String> = q
        .names
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match state.repository.get_meter_characters(&names) {
        Ok(rows) => {
            let characters = rows
                .into_iter()
                .map(|c| ApiCharacter {
                    name: c.name,
                    class_id: c.class_id,
                    class: c.class,
                    gear_score: c.gear_score,
                })
                .collect();
            Json(CharactersResponse {
                schema_version: SCHEMA_VERSION,
                characters,
            })
            .into_response()
        }
        Err(e) => {
            error!("local api characters query failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn fallback_handler() -> Response {
    StatusCode::NOT_FOUND.into_response()
}

async fn cors_middleware(State(state): State<ApiState>, req: Request, next: Next) -> Response {
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let allowed = origin
        .as_deref()
        .map(|o| is_origin_allowed(&state.allowed_origins, o))
        .unwrap_or(false);

    if req.method() == Method::OPTIONS {
        if !allowed {
            return StatusCode::FORBIDDEN.into_response();
        }
        let pna_requested = req
            .headers()
            .get("access-control-request-private-network")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let mut resp = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap();
        let h = resp.headers_mut();
        set_origin(h, origin.as_deref());
        insert_static(h, header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS");
        insert_static(
            h,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Authorization, Content-Type",
        );
        if pna_requested {
            insert_static(
                h,
                HeaderName::from_static("access-control-allow-private-network"),
                "true",
            );
        }
        insert_static(
            h,
            header::VARY,
            "Origin, Access-Control-Request-Private-Network",
        );
        return resp;
    }

    let mut resp = next.run(req).await;
    let h = resp.headers_mut();
    if allowed {
        set_origin(h, origin.as_deref());
        insert_static(h, header::VARY, "Origin");
    }
    insert_static(h, header::CACHE_CONTROL, "no-store");
    resp
}

fn is_origin_allowed(allowed: &[String], origin: &str) -> bool {
    allowed.iter().any(|o| o == origin)
}

fn set_origin(headers: &mut HeaderMap, origin: Option<&str>) {
    if let Some(o) = origin {
        if let Ok(value) = HeaderValue::from_str(o) {
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        }
    }
}

fn insert_static(headers: &mut HeaderMap, name: HeaderName, value: &'static str) {
    headers.insert(name, HeaderValue::from_static(value));
}

fn check_auth(headers: &HeaderMap, token: &str) -> Result<(), Response> {
    let provided = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    if !token.is_empty() && constant_time_eq(provided.as_bytes(), token.as_bytes()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED.into_response())
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

async fn bind_with_retry(port: u16) -> std::io::Result<tokio::net::TcpListener> {
    let mut last_err: Option<std::io::Error> = None;
    for attempt in 0..10 {
        match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) => {
                last_err = Some(e);
                if attempt < 9 {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }
        }
    }
    Err(last_err.expect("bind loop ran at least once"))
}
