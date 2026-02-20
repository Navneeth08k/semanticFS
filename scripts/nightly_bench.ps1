param(
  [string]$ConfigPath = "config/semanticfs.sample.toml",
  [string]$FixtureRepo = "tests/fixtures/benchmark_repo",
  [string]$GoldenDir = "tests/retrieval_golden",
  [int]$SoakSeconds = 30,
  [switch]$EnforceRelevance
)

$ErrorActionPreference = "Stop"

Write-Host "== SemanticFS Nightly Bench =="
Write-Host "config: $ConfigPath"
Write-Host "fixture repo: $FixtureRepo"
Write-Host "golden dir: $GoldenDir"

function Run-Step([string]$Label, [string]$Cmd) {
  Write-Host ""
  Write-Host "== $Label =="
  Write-Host $Cmd
  Invoke-Expression $Cmd
}

Run-Step "Relevance (multi-suite)" "cargo run --release -p semanticfs-cli -- --config `"$ConfigPath`" benchmark relevance --fixture-repo `"$FixtureRepo`" --golden-dir `"$GoldenDir`" --history"
Run-Step "Benchmark Run" "cargo run --release -p semanticfs-cli -- --config `"$ConfigPath`" benchmark run --fixture-repo `"$FixtureRepo`" --soak-seconds $SoakSeconds --history"
Run-Step "LanceDB Tune" "cargo run --release -p semanticfs-cli -- --config `"$ConfigPath`" benchmark tune-lancedb --fixture-repo `"$FixtureRepo`" --soak-seconds $SoakSeconds --history"
Run-Step "ONNX Tune" "cargo run --release -p semanticfs-cli -- --config `"$ConfigPath`" benchmark tune-onnx --fixture-repo `"$FixtureRepo`" --samples 300 --rounds 2 --batch-sizes 16,32 --max-lengths 128,256 --history"

$releaseGate = "cargo run --release -p semanticfs-cli -- --config `"$ConfigPath`" benchmark release-gate --refresh --fixture-repo `"$FixtureRepo`" --soak-seconds $SoakSeconds"
if ($EnforceRelevance) {
  $releaseGate += " --enforce-relevance --min-relevance-queries 20 --min-recall-at-5 0.90 --min-symbol-hit-rate 0.99 --min-mrr 0.80"
}
Run-Step "Release Gate" $releaseGate

Write-Host ""
Write-Host "Nightly bench complete."
