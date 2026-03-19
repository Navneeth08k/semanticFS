#!/usr/bin/env python3
"""
SemanticFS stdio MCP wrapper.

Translates Claude Code's MCP JSON-RPC stdio protocol into SemanticFS HTTP API
calls, so Claude Code can use SemanticFS search/map as native tools.

Usage (via Claude Code --mcp-config):
  The wrapper is launched as a subprocess by Claude Code and communicates via
  stdin/stdout using the MCP Content-Length framed JSON-RPC protocol.

Direct test:
  python semanticfs_mcp_stdio.py --url http://127.0.0.1:9464
"""

import sys
import json
import argparse
import urllib.request
import urllib.parse
import urllib.error

# ── Tool definitions exposed to Claude ────────────────────────────────────────

TOOLS = [
    {
        "name": "search_codebase",
        "description": (
            "Semantically search the indexed codebase. Returns the most relevant "
            "files with precise file paths and line ranges. Use this FIRST to locate "
            "code before reading it. Much faster than grep or find for concept queries."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": (
                        "Natural language or keyword query. "
                        "Examples: 'Python signature extraction', 'Gemini API call', "
                        "'CLI argument parsing', 'error handling save file'"
                    )
                }
            },
            "required": ["query"]
        }
    },
    {
        "name": "get_directory_map",
        "description": (
            "Get a structural overview of a directory in the codebase. "
            "Useful for orientation before diving into specific files."
        ),
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
]

# ── MCP stdio transport ────────────────────────────────────────────────────────

def read_message(stream=None):
    """Read one Content-Length framed MCP message from stdin."""
    if stream is None:
        stream = sys.stdin

    headers = {}
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.rstrip("\r\n")
        if line == "":
            break
        if ":" in line:
            k, _, v = line.partition(":")
            headers[k.strip().lower()] = v.strip()

    content_length = int(headers.get("content-length", 0))
    if content_length == 0:
        return None

    body = stream.read(content_length)
    return json.loads(body)


def write_message(msg, stream=None):
    """Write one Content-Length framed MCP message to stdout."""
    if stream is None:
        stream = sys.stdout

    body = json.dumps(msg, ensure_ascii=False)
    framed = f"Content-Length: {len(body.encode('utf-8'))}\r\n\r\n{body}"
    stream.write(framed)
    stream.flush()


# ── SemanticFS HTTP calls ──────────────────────────────────────────────────────

def semanticfs_search(base_url: str, query: str) -> str:
    params = urllib.parse.urlencode({"query": query})
    url = f"{base_url}/tools/search_codebase?{params}"
    try:
        with urllib.request.urlopen(url, timeout=15) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            if data.get("ok"):
                return data["content"]
            return f"[SemanticFS] Search error: {data.get('error', 'unknown')}"
    except urllib.error.URLError as e:
        return f"[SemanticFS] HTTP error (is the server running?): {e}"
    except Exception as e:
        return f"[SemanticFS] Unexpected error: {e}"


def semanticfs_map(base_url: str, path: str) -> str:
    params = urllib.parse.urlencode({"path": path})
    url = f"{base_url}/tools/get_directory_map?{params}"
    try:
        with urllib.request.urlopen(url, timeout=15) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            if data.get("ok"):
                return data["content"]
            return f"[SemanticFS] Map error: {data.get('error', 'unknown')}"
    except urllib.error.URLError as e:
        return f"[SemanticFS] HTTP error (is the server running?): {e}"
    except Exception as e:
        return f"[SemanticFS] Unexpected error: {e}"


# ── MCP server loop ───────────────────────────────────────────────────────────

def run(base_url: str):
    while True:
        msg = read_message()
        if msg is None:
            break

        method = msg.get("method", "")
        msg_id = msg.get("id")

        # ── Initialization ──
        if method == "initialize":
            write_message({
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "semanticfs", "version": "1.0.0"}
                }
            })

        # ── Notifications (no response) ──
        elif method in ("notifications/initialized", "initialized"):
            pass

        # ── List tools ──
        elif method == "tools/list":
            write_message({
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {"tools": TOOLS}
            })

        # ── Call tool ──
        elif method == "tools/call":
            params = msg.get("params", {})
            tool_name = params.get("name", "")
            args = params.get("arguments", {})

            if tool_name == "search_codebase":
                query = args.get("query", "")
                result_text = semanticfs_search(base_url, query)
            elif tool_name == "get_directory_map":
                path = args.get("path", ".")
                result_text = semanticfs_map(base_url, path)
            else:
                result_text = f"Unknown tool: {tool_name}"

            write_message({
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "content": [{"type": "text", "text": result_text}]
                }
            })

        # ── Unknown method ──
        else:
            if msg_id is not None:
                write_message({
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "error": {
                        "code": -32601,
                        "message": f"Method not found: {method}"
                    }
                })


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="SemanticFS MCP stdio wrapper")
    parser.add_argument(
        "--url",
        default="http://127.0.0.1:9464",
        help="SemanticFS MCP HTTP base URL (default: http://127.0.0.1:9464)"
    )
    args = parser.parse_args()
    run(args.url)
