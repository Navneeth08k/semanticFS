param(
    [string]$ConfigPath = "config/semanticfs.sample.toml",
    [string]$FixtureRepo = "tests/fixtures/benchmark_repo",
    [int]$SoakSeconds = 30
)

$ErrorActionPreference = "Stop"

Write-Host "[semanticfs] rc preflight starting"
Write-Host "  config:  $ConfigPath"
Write-Host "  fixture: $FixtureRepo"
Write-Host "  soak:    ${SoakSeconds}s"

cargo fmt --check
cargo test --workspace

cargo run --release -p semanticfs-cli -- --config $ConfigPath benchmark release-gate `
  --refresh `
  --fixture-repo $FixtureRepo `
  --soak-seconds 5

cargo run --release -p semanticfs-cli -- --config $ConfigPath benchmark soak `
  --duration-seconds $SoakSeconds `
  --fixture-repo $FixtureRepo `
  --max-soak-p95-ms 250 `
  --max-errors 0 `
  --max-rss-mb 2048

Write-Host "[semanticfs] rc preflight passed"
Write-Host "reports:"
Write-Host "  .semanticfs/bench/latest.json"
Write-Host "  .semanticfs/bench/lancedb_tuning.json"
Write-Host "  .semanticfs/bench/release_gate.json"
Write-Host "  .semanticfs/bench/soak_latest.json"
