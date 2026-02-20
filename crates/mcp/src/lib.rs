use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use fuse_bridge::FuseBridge;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct McpServer {
    state: Arc<McpState>,
}

#[derive(Clone)]
struct McpState {
    bridge: Arc<FuseBridge>,
    sessions: Arc<RwLock<HashMap<String, u64>>>,
}

impl McpServer {
    pub fn new(bridge: FuseBridge) -> Self {
        Self {
            state: Arc::new(McpState {
                bridge: Arc::new(bridge),
                sessions: Arc::new(RwLock::new(HashMap::new())),
            }),
        }
    }

    pub async fn serve(self, bind: &str) -> Result<()> {
        let app = Router::new()
            .route("/tools/search_codebase", get(search_codebase))
            .route("/tools/get_directory_map", get(get_directory_map))
            .route("/resources/health", get(resource_health))
            .route("/resources/search/:query", get(resource_search))
            .route("/resources/map/:path", get(resource_map))
            .route(
                "/prompts/semanticfs_search_then_raw_verify",
                get(prompt_template),
            )
            .with_state(self.state);

        let addr: SocketAddr = bind.parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    query: String,
    snapshot_version: Option<u64>,
    active_version: Option<u64>,
    session_id: Option<String>,
    refresh_session: Option<bool>,
}

async fn search_codebase(
    State(state): State<Arc<McpState>>,
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let path = format!("/search/{}.md", params.query.replace(' ', "_"));
    let refresh = params.refresh_session.unwrap_or(false);
    let active = params
        .active_version
        .filter(|v| *v > 0)
        .unwrap_or_else(|| state.bridge.active_version().unwrap_or(0));
    let snapshot = resolve_snapshot(
        &state,
        params.snapshot_version,
        params.session_id.as_deref(),
        refresh,
    )
    .await;

    match state.bridge.read_virtual(&path, snapshot, active) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

#[derive(Debug, Deserialize)]
struct MapParams {
    path: String,
    snapshot_version: Option<u64>,
    session_id: Option<String>,
    refresh_session: Option<bool>,
}

async fn get_directory_map(
    State(state): State<Arc<McpState>>,
    Query(params): Query<MapParams>,
) -> Json<Value> {
    let path = format!("/map/{}/directory_overview.md", params.path);
    let snapshot = resolve_snapshot(
        &state,
        params.snapshot_version,
        params.session_id.as_deref(),
        params.refresh_session.unwrap_or(false),
    )
    .await;

    match state.bridge.read_virtual(&path, snapshot, snapshot) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn resource_health(State(state): State<Arc<McpState>>) -> Json<Value> {
    let (inode, content) = state.bridge.cache_stats();
    Json(json!({
        "live": true,
        "ready": true,
        "cache": {
            "inode_entries": inode,
            "content_entries": content
        }
    }))
}

#[derive(Debug, Deserialize)]
struct SessionQuery {
    snapshot_version: Option<u64>,
    session_id: Option<String>,
    refresh_session: Option<bool>,
}

async fn resource_search(
    State(state): State<Arc<McpState>>,
    Path(query): Path<String>,
    Query(params): Query<SessionQuery>,
) -> Json<Value> {
    let path = format!("/search/{}.md", query);
    let snapshot = resolve_snapshot(
        &state,
        params.snapshot_version,
        params.session_id.as_deref(),
        params.refresh_session.unwrap_or(false),
    )
    .await;
    let active = state.bridge.active_version().unwrap_or(snapshot);
    match state.bridge.read_virtual(&path, snapshot, active) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn resource_map(
    State(state): State<Arc<McpState>>,
    Path(path): Path<String>,
    Query(params): Query<SessionQuery>,
) -> Json<Value> {
    let virtual_path = format!("/map/{}/directory_overview.md", path);
    let snapshot = resolve_snapshot(
        &state,
        params.snapshot_version,
        params.session_id.as_deref(),
        params.refresh_session.unwrap_or(false),
    )
    .await;
    match state.bridge.read_virtual(&virtual_path, snapshot, snapshot) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn prompt_template() -> Json<Value> {
    Json(json!({
        "name": "semanticfs_search_then_raw_verify",
        "instructions": "First read /search/<query>.md to locate grounded ranges, then verify exact bytes through /raw/<path> before making edits. Use session_id to keep a stable snapshot; set refresh_session=true when you want latest index state."
    }))
}

async fn resolve_snapshot(
    state: &McpState,
    requested_snapshot: Option<u64>,
    session_id: Option<&str>,
    refresh_session: bool,
) -> u64 {
    if let Some(version) = requested_snapshot.filter(|v| *v > 0) {
        if let Some(id) = session_id {
            state.sessions.write().await.insert(id.to_string(), version);
        }
        return version;
    }

    let active = state.bridge.active_version().unwrap_or(0);
    if let Some(id) = session_id {
        let mut sessions = state.sessions.write().await;
        if refresh_session || !sessions.contains_key(id) {
            sessions.insert(id.to_string(), active);
        }
        return sessions.get(id).copied().unwrap_or(active);
    }

    active
}
