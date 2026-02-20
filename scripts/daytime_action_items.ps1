param(
  [string]$SemanticFsRepo = (Get-Location).Path,
  [string]$AiTestgenRepo = (Join-Path (Resolve-Path "..").Path "ai-testgen"),
  [string]$BuckitRepo = "C:\Users\navneeth\Desktop\NavneethThings\Projects\buckit",
  [string]$TensorflowModelsRepo = "C:\Users\navneeth\Desktop\NavneethThings\Projects\Robot\TFODCourse\Tensorflow\models",
  [int]$SoakSeconds = 2,
  [switch]$IncludeReleaseGate
)

$ErrorActionPreference = "Stop"

function Run-Step([string]$Label, [string]$Cmd) {
  Write-Host ""
  Write-Host "== $Label =="
  Write-Host $Cmd
  Invoke-Expression $Cmd
  if ($LASTEXITCODE -ne 0) {
    throw "step failed ($Label) with exit code $LASTEXITCODE"
  }
}

function Assert-Path([string]$PathValue, [string]$Kind) {
  if (-not (Test-Path $PathValue)) {
    throw "$Kind not found: $PathValue"
  }
}

Write-Host "== SemanticFS Daytime Action Items =="
Write-Host "semanticfs repo:       $SemanticFsRepo"
Write-Host "ai-testgen repo:       $AiTestgenRepo"
Write-Host "buckit repo:           $BuckitRepo"
Write-Host "tensorflow/models:     $TensorflowModelsRepo"
Write-Host "soak seconds:          $SoakSeconds"
Write-Host "include release gate:  $($IncludeReleaseGate.IsPresent)"

Assert-Path $SemanticFsRepo "semanticfs repo"
Assert-Path $AiTestgenRepo "ai-testgen repo"
Assert-Path $BuckitRepo "buckit repo"
Assert-Path $TensorflowModelsRepo "tensorflow/models repo"

Run-Step "Split buckit bootstrap into tune/holdout" "python scripts/split_golden_suite.py --input tests/retrieval_golden/buckit_bootstrap.json --tune-output tests/retrieval_golden/buckit_tune.json --holdout-output tests/retrieval_golden/buckit_holdout.json --tune-count 10"
Run-Step "Split tensorflow bootstrap into tune/holdout" "python scripts/split_golden_suite.py --input tests/retrieval_golden/tensorflow_models_bootstrap.json --tune-output tests/retrieval_golden/tensorflow_models_tune.json --holdout-output tests/retrieval_golden/tensorflow_models_holdout.json --tune-count 10"

$smokeCmd = "powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SemanticFsRepo `"$SemanticFsRepo`" -AiTestgenRepo `"$AiTestgenRepo`" -SoakSeconds $SoakSeconds"
if ($IncludeReleaseGate.IsPresent) {
  $smokeCmd += " -IncludeReleaseGate"
}
Run-Step "Representative daytime smoke" $smokeCmd

Run-Step "Tune/Holdout sweep - buckit" "powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label buckit -RepoRoot `"$BuckitRepo`" -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/buckit_tune.json -HoldoutGolden tests/retrieval_golden/buckit_holdout.json -History"
Run-Step "Tune/Holdout sweep - tensorflow_models" "powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label tensorflow_models -RepoRoot `"$TensorflowModelsRepo`" -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/tensorflow_models_tune.json -HoldoutGolden tests/retrieval_golden/tensorflow_models_holdout.json -History"

Run-Step "Drift summary refresh" "powershell -ExecutionPolicy Bypass -File scripts/drift_summary.ps1"

Write-Host ""
Write-Host "Daytime action items complete."
Write-Host "Key artifacts:"
Write-Host "  .semanticfs/bench/relevance_latest.json"
Write-Host "  .semanticfs/bench/head_to_head_latest.json"
Write-Host "  .semanticfs/bench/tune_holdout_buckit_latest.json"
Write-Host "  .semanticfs/bench/tune_holdout_tensorflow_models_latest.json"
Write-Host "  .semanticfs/bench/drift_summary_latest.json"
