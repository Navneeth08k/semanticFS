# Operations Runbook

## Stale mount recovery
1. Check service status:
   - `systemctl status semanticfs-fuse`
2. Force-unmount stale mount:
   - `semanticfs recover mount --force-unmount`
   - Linux fallback: `fusermount -uz /mnt/ai`
3. Restart services:
   - `sudo systemctl restart semanticfs-indexer semanticfs-fuse semanticfs-mcp`

## Health checks
1. CLI:
   - `semanticfs health --config /etc/semanticfs/config.toml`
2. MCP health resource:
   - `/resources/health`

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

## Audit event fields
- `ts`, `actor`, `op`, `target`, `snapshot_version`, `policy_decision`, `reason`, `latency_ms`, `result_count`
