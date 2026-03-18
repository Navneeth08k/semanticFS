use anyhow::Result;
use fuse_bridge::FuseBridge;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::Arc;

const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Native Rust stdio MCP server.
///
/// Reads Content-Length framed JSON-RPC 2.0 messages from stdin, dispatches
/// search and map queries to the FuseBridge, and writes framed responses to
/// stdout.  Claude Code (and any MCP-capable agent) can launch this directly
/// as a subprocess without the Python sidecar.
pub struct McpStdioServer {
    bridge: Arc<FuseBridge>,
}

impl McpStdioServer {
    pub fn new(bridge: FuseBridge) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Run the stdio server until EOF on stdin.
    pub fn run(self) -> Result<()> {
        let mut reader = BufReader::new(io::stdin());

        loop {
            // ── Read Content-Length framed headers ──────────────────────────
            let mut content_length: usize = 0;
            loop {
                let mut line = String::new();
                let n = reader.read_line(&mut line)?;
                if n == 0 {
                    return Ok(());
                }
                let trimmed = line.trim_end_matches(['\r', '\n']);
                if trimmed.is_empty() {
                    break;
                }
                if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
                    content_length = rest.trim().parse().unwrap_or(0);
                }
            }

            if content_length == 0 {
                continue;
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;

            let msg: Value = match serde_json::from_slice(&body) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("mcp-stdio: invalid JSON-RPC message: {}", e);
                    continue;
                }
            };

            let method = msg["method"].as_str().unwrap_or("");
            let id = msg.get("id").cloned();

            // Notifications carry no id and need no response
            if method == "notifications/initialized" || method == "initialized" {
                continue;
            }

            if let Some(response) = self.handle(&msg, method, id) {
                write_message(&response)?;
            }
        }
    }

    fn handle(&self, msg: &Value, method: &str, id: Option<Value>) -> Option<Value> {
        match method {
            "initialize" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "semanticfs", "version": SERVER_VERSION }
                }
            })),

            "tools/list" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools_list() }
            })),

            "tools/call" => {
                let params = &msg["params"];
                let tool_name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];

                let result_text = match tool_name {
                    "search_codebase" => {
                        let query = args["query"].as_str().unwrap_or("");
                        self.search(query)
                    }
                    "get_directory_map" => {
                        let path = args["path"].as_str().unwrap_or(".");
                        self.get_map(path)
                    }
                    other => format!("[SemanticFS] Unknown tool: {other}"),
                };

                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": result_text }]
                    }
                }))
            }

            _ => id.map(|id| {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {method}")
                    }
                })
            }),
        }
    }

    fn search(&self, query: &str) -> String {
        let path = format!("/search/{}.md", query.replace(' ', "_"));
        let active = self.bridge.active_version().unwrap_or(0);
        match self.bridge.read_virtual(&path, active, active) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(e) => format!("[SemanticFS] Search error: {e}"),
        }
    }

    fn get_map(&self, path: &str) -> String {
        let vpath = format!("/map/{path}/directory_overview.md");
        let active = self.bridge.active_version().unwrap_or(0);
        match self.bridge.read_virtual(&vpath, active, active) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(e) => format!("[SemanticFS] Map error: {e}"),
        }
    }
}

fn tools_list() -> Value {
    json!([
        {
            "name": "search_codebase",
            "description": "Semantically search the indexed codebase. Returns the most relevant files with precise file paths and line ranges. Use this FIRST to locate code before reading it. Much faster than grep or find for concept queries.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language or keyword query. Examples: 'Python signature extraction', 'Gemini API call', 'CLI argument parsing', 'error handling save file'"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "get_directory_map",
            "description": "Get a structural overview of a directory in the codebase. Useful for orientation before diving into specific files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path relative to the workspace root, e.g. '.' or 'src'"
                    }
                },
                "required": ["path"]
            }
        }
    ])
}

fn write_message(msg: &Value) -> Result<()> {
    let body = serde_json::to_string(msg)?;
    let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    io::stdout().write_all(framed.as_bytes())?;
    io::stdout().flush()?;
    Ok(())
}
