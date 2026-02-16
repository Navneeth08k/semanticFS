# SemanticFS v1.1 Scaffold

This repository contains a production-oriented scaffold for SemanticFS v1.1.

## Workspace crates
- `semanticfs-common`: shared config/types/audit/health structures.
- `policy-guard`: allow/deny filtering, secret detection, and redaction.
- `indexer`: indexing pipeline, schema management, symbol extraction, map precompute, two-phase publish.
- `retrieval-core`: symbol-first planner + BM25 + plain RRF fusion.
- `map-engine`: deterministic map summary retrieval + optional enrichment merge.
- `fuse-bridge`: virtual path router with inode/content LRU caches and snapshot-scoped reads.
- `mcp`: minimal MCP-compatible HTTP surface.
- `semanticfs-cli`: user-facing commands.

## Current implementation status
Implemented now:
- Full config model.
- SQLite schema and index versioning.
- Symbol table extraction and lookup.
- Policy guard + audit event model.
- Search and map virtual rendering paths.
- Vector persistence + retrieval path with RRF integration.
- Optional LanceDB vector backend (index sync + nearest-neighbor retrieval).
- Optional ONNX embedding backend with hash fallback.
- MCP minimal tools/resources/prompt endpoints.
- systemd service templates.

Stubbed next increments:
- Async LLM enrichment worker.
- Full observability server with metrics endpoints.

## Quickstart (after Rust toolchain install)
1. `cargo build`
2. `cp config/semanticfs.sample.toml local.toml` and edit `repo_root`/`mount_point`
3. `cargo run -p semanticfs-cli -- --config local.toml index build`
4. `cargo run -p semanticfs-cli -- --config local.toml serve mcp`
5. `cargo run -p semanticfs-cli -- --config local.toml index watch` for continuous reindexing

Vector backend selection:
1. `SEMANTICFS_VECTOR_BACKEND=sqlite` (default behavior if unset)
2. `SEMANTICFS_VECTOR_BACKEND=lancedb` to sync/query vectors via LanceDB
3. Optional LanceDB location override: `SEMANTICFS_LANCEDB_URI=./.semanticfs/lancedb`

Embedding backend selection:
1. `embedding.runtime = "hash"` for deterministic local hash embeddings
2. `embedding.runtime = "onnx"` plus `SEMANTICFS_ONNX_MODEL=/abs/path/model.onnx`
3. Build ONNX-capable binary with: `cargo build -p semanticfs-cli --features onnx`

Windows note:
1. LanceDB build requires `protoc` on PATH.
2. ONNX feature currently may require a newer MSVC toolchain than this machine has (linker unresolved `__std_*` symbols).

Linux-only mount:
1. Ensure FUSE userspace tooling is installed and mountpoint exists.
2. Run `cargo run -p semanticfs-cli -- --config local.toml serve fuse`.
3. On non-Linux targets, `serve fuse` exits with a clear unsupported-platform error.

## Security model
Policy guard is enforced in indexing and retrieval/render pathways and supports:
- deny/allow glob filters
- secret heuristics (regex + entropy)
- retrieval redaction

## Recovery
Use:
- `semanticfs recover mount --force-unmount`
- then remount/restart service using systemd
