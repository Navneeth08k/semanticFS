# Contributing to SemanticFS

## Development setup

### Prerequisites

- Rust stable (`rustup update stable`)
- On Linux: `libfuse3-dev` and `protobuf-compiler` for the FUSE bridge and LanceDB
- Python 3.10+ (only needed for ONNX sidecar and benchmark harness scripts)

### Build

```bash
cargo build -p semanticfs-cli
cargo build --release -p semanticfs-cli   # for benchmarks
```

### Run tests

```bash
# All workspace unit tests
cargo test --workspace

# A specific crate
cargo test -p indexer
```

### Check formatting

```bash
cargo fmt --check
cargo clippy --workspace
```

---

## Running the golden retrieval suite

The frozen golden suites are the primary regression gate. Any change to retrieval
or indexing logic **must** pass v14 at recall=1.000, MRR=1.000 before merging.

```bash
# Requires a fixture repo — use the project itself or tests/fixtures/benchmark_repo
cargo run --release -p semanticfs-cli -- \
  --config config/semanticfs.sample.toml \
  benchmark relevance \
  --fixture-repo /absolute/path/to/repo \
  --golden tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json
```

Expected output: `recall=1.000  mrr=1.000  symbol_hit=1.000`

---

## Adding a new golden query

1. Identify a query that should be retrieval-correct for the fixture corpus.
2. Run the benchmark to find the expected result file path and line range.
3. Edit `tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json` and add your entry (or create a new `v15.json` for a new frozen suite).
4. Re-run the benchmark and paste the new metrics.

---

## Crate overview

| Crate | Purpose |
|---|---|
| `semanticfs-common` | Shared config types (`SemanticFsConfig`), health structs |
| `policy-guard` | Trust boundaries, file filtering, multi-root ownership |
| `indexer` | File watching, chunking, symbol extraction, embedding |
| `retrieval-core` | Hybrid retrieval planner, RRF fusion, ranking priors |
| `map-engine` | Directory summary generation and LLM enrichment |
| `fuse-bridge` | Virtual filesystem: inode/content LRU cache, snapshot versioning |
| `mcp` | MCP HTTP server and native stdio MCP transport |
| `semanticfs-cli` | CLI entry point: `index`, `serve`, `health`, `benchmark`, `model` |

---

## PR checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace` passes (no new warnings)
- [ ] `cargo test --workspace` passes
- [ ] Golden suite v14 still at recall=1.000, MRR=1.000 (if retrieval/indexing changed)
- [ ] New public types/functions have doc comments
- [ ] Config changes are reflected in `config/semanticfs.sample.toml` with inline comments

---

## Commit style

Use conventional-ish commit messages:

```
feat: add native stdio MCP transport (serve mcp-stdio)
fix: default port in semanticfs_mcp_stdio.py (9466 → 9464)
docs: add inline TOML comments to sample configs
chore: add GitHub Actions CI matrix for Linux/macOS/Windows
```

---

## Questions / issues

Open an issue on GitHub. For security issues, see [SECURITY.md](SECURITY.md).
