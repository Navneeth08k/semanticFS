# Operations Runbook

## Stale mount recovery
1. Check service status:
   - `systemctl status semanticfs-fuse`
2. Force-unmount stale mount:
   - `semanticfs recover mount --force-unmount`
   - Linux fallback: `fusermount -uz /mnt/ai`
3. Restart services:
   - `sudo systemctl restart semanticfs-indexer semanticfs-fuse semanticfs-mcp semanticfs-observability`

## Health checks
1. CLI:
   - `semanticfs health --config /etc/semanticfs/config.toml`
2. MCP health resource:
   - `/resources/health`
3. Observability:
   - `/health/live`
   - `/health/ready`
   - `/health/index`
   - `/metrics`

## Common issues
1. High memory:
   - reduce `fuse_cache.max_virtual_inodes`
   - reduce `fuse_cache.max_cached_mb`
2. Slow indexing:
   - narrow `filter.allow_roots`
   - increase deny patterns for generated artifacts
3. Missing search hits:
   - verify index version published
   - check policy deny/redaction events

## Benchmark baseline
1. Run:
   - `semanticfs --config /etc/semanticfs/config.toml benchmark run --soak-seconds 60`
2. Review:
   - `.semanticfs/bench/latest.json`

## RC stability soak
1. Run:
   - `semanticfs --config /etc/semanticfs/config.toml benchmark soak --duration-seconds 1800 --fixture-repo tests/fixtures/benchmark_repo`
2. Review:
   - `.semanticfs/bench/soak_latest.json`

## Audit event fields
- `ts`, `actor`, `op`, `target`, `snapshot_version`, `policy_decision`, `reason`, `latency_ms`, `result_count`
