# Implemented v1.1 Scaffold

## Implemented
- Workspace and crate decomposition aligned with v1.1 spec.
- Config schema and sample config.
- SQLite schema with two-phase index version publish.
- File filtering and policy guard enforcement.
- File-type routing and chunking strategies.
- Global symbol extraction table.
- Symbol-first retrieval planner + BM25 + plain RRF.
- Vector retrieval pipeline with persisted embeddings and cosine similarity.
- Optional LanceDB vector backend sync + nearest-neighbor query path.
- Optional ONNX embedding backend wiring (with hash fallback if unavailable).
- Virtual path router for `/raw`, `/search`, `/map`, `/.well-known/health.json`.
- Inode/content LRU cache structure.
- MCP minimal surface:
  - tools: `search_codebase`, `get_directory_map`
  - resources: `health`, `search/<query>`, `map/<path>`
  - prompt: `semanticfs_search_then_raw_verify`
- systemd units and operational runbook.
- Continuous `index watch` daemon loop with notify events.
- Linux-target mount adapter entrypoint (with Linux-only guard on non-Linux hosts).

## Remaining for full production parity
- Production-grade LanceDB schema/index tuning and large-scale performance profiling.
- Full ONNX model tokenization/inference pipeline for BGE-style text encoders.
- Async LLM enrichment worker for `/map`.
- Full observability server with metrics endpoints.
- End-to-end and soak test automation.
