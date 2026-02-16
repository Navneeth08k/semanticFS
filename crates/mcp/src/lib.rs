use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use fuse_bridge::FuseBridge;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
pub struct McpServer {
    bridge: Arc<FuseBridge>,
}

impl McpServer {
    pub fn new(bridge: FuseBridge) -> Self {
        Self {
            bridge: Arc::new(bridge),
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
            .with_state(self.bridge);

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
}

async fn search_codebase(
    State(bridge): State<Arc<FuseBridge>>,
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let path = format!("/search/{}.md", params.query.replace(' ', "_"));
    let snapshot = params.snapshot_version.unwrap_or(0);
    let active = params.active_version.unwrap_or(snapshot);

    match bridge.read_virtual(&path, snapshot, active) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

#[derive(Debug, Deserialize)]
struct MapParams {
    path: String,
    snapshot_version: Option<u64>,
}

async fn get_directory_map(
    State(bridge): State<Arc<FuseBridge>>,
    Query(params): Query<MapParams>,
) -> Json<Value> {
    let path = format!("/map/{}/directory_overview.md", params.path);
    let snapshot = params.snapshot_version.unwrap_or(0);

    match bridge.read_virtual(&path, snapshot, snapshot) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn resource_health(State(bridge): State<Arc<FuseBridge>>) -> Json<Value> {
    let (inode, content) = bridge.cache_stats();
    Json(json!({
        "live": true,
        "ready": true,
        "cache": {
            "inode_entries": inode,
            "content_entries": content
        }
    }))
}

async fn resource_search(
    State(bridge): State<Arc<FuseBridge>>,
    Path(query): Path<String>,
) -> Json<Value> {
    let path = format!("/search/{}.md", query);
    match bridge.read_virtual(&path, 0, 0) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn resource_map(
    State(bridge): State<Arc<FuseBridge>>,
    Path(path): Path<String>,
) -> Json<Value> {
    let virtual_path = format!("/map/{}/directory_overview.md", path);
    match bridge.read_virtual(&virtual_path, 0, 0) {
        Ok(bytes) => Json(json!({ "ok": true, "content": String::from_utf8_lossy(&bytes) })),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn prompt_template() -> Json<Value> {
    Json(json!({
        "name": "semanticfs_search_then_raw_verify",
        "instructions": "First read /search/<query>.md to locate grounded ranges, then verify exact bytes through /raw/<path> before making edits."
    }))
}
