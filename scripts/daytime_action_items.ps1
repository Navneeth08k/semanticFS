param(
  [string]$SemanticFsRepo = (Get-Location).Path,
  [string]$AiTestgenRepo = (Join-Path (Resolve-Path "..").Path "ai-testgen"),
  [string]$BuckitRepo = "C:\Users\navneeth\Desktop\NavneethThings\Projects\buckit",
  [string]$TensorflowModelsRepo = "C:\Users\navneeth\Desktop\NavneethThings\Projects\Robot\TFODCourse\Tensorflow\models",
  [string[]]$DiscoveryRoots = @(),
  [int]$DiscoveryMinTrackedFiles = 500,
  [int]$DiscoveryTopN = 30,
  [int]$SoakSeconds = 2,
  [switch]$IncludeReleaseGate,
  [switch]$SkipFilesystemBacklog,
  [switch]$SkipDomainPlan
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
Write-Host "discovery roots:       $($DiscoveryRoots -join ', ')"
Write-Host "soak seconds:          $SoakSeconds"
Write-Host "include release gate:  $($IncludeReleaseGate.IsPresent)"
Write-Host "skip fs backlog:       $($SkipFilesystemBacklog.IsPresent)"
Write-Host "skip domain plan:      $($SkipDomainPlan.IsPresent)"

Assert-Path $SemanticFsRepo "semanticfs repo"
Assert-Path $AiTestgenRepo "ai-testgen repo"
Assert-Path $BuckitRepo "buckit repo"
Assert-Path $TensorflowModelsRepo "tensorflow/models repo"

Run-Step "Generate buckit bootstrap v2 (expanded)" "python scripts/bootstrap_golden_from_repo.py --repo-root `"$BuckitRepo`" --output tests/retrieval_golden/buckit_bootstrap_v2_full.json --dataset-name buckit_bootstrap_v2_full --max-queries 120"
Run-Step "Generate tensorflow bootstrap v2 (expanded)" "python scripts/bootstrap_golden_from_repo.py --repo-root `"$TensorflowModelsRepo`" --output tests/retrieval_golden/tensorflow_models_bootstrap_v2_full.json --dataset-name tensorflow_models_bootstrap_v2_full --max-queries 120"
Run-Step "Curate buckit mixed tune/holdout suites (>=30 each split)" "python scripts/build_curated_mixed_suites.py --input tests/retrieval_golden/buckit_bootstrap_v2_full.json --tune-output tests/retrieval_golden/buckit_curated_tune.json --holdout-output tests/retrieval_golden/buckit_curated_holdout.json --split-size 40 --non-symbol-per-split 10 --dataset-prefix buckit"
Run-Step "Curate tensorflow mixed tune/holdout suites (>=30 each split)" "python scripts/build_curated_mixed_suites.py --input tests/retrieval_golden/tensorflow_models_bootstrap_v2_full.json --tune-output tests/retrieval_golden/tensorflow_models_curated_tune.json --holdout-output tests/retrieval_golden/tensorflow_models_curated_holdout.json --split-size 40 --non-symbol-per-split 10 --dataset-prefix tensorflow_models --override 'build_losses=official/core/base_task.py;official/core/train_lib_test.py;official/projects/yt8m/tasks/yt8m_task.py'"

$smokeCmd = "powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SemanticFsRepo `"$SemanticFsRepo`" -AiTestgenRepo `"$AiTestgenRepo`" -SoakSeconds $SoakSeconds"
if ($IncludeReleaseGate.IsPresent) {
  $smokeCmd += " -IncludeReleaseGate"
}
Run-Step "Representative daytime smoke" $smokeCmd

Run-Step "Tune/Holdout sweep - buckit curated suites" "powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label buckit_curated -RepoRoot `"$BuckitRepo`" -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/buckit_curated_tune.json -HoldoutGolden tests/retrieval_golden/buckit_curated_holdout.json -History"
Run-Step "Tune/Holdout sweep - tensorflow curated suites" "powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label tensorflow_models_curated -RepoRoot `"$TensorflowModelsRepo`" -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/tensorflow_models_curated_tune.json -HoldoutGolden tests/retrieval_golden/tensorflow_models_curated_holdout.json -History"

if ($DiscoveryRoots.Count -gt 0) {
  $quotedRoots = ($DiscoveryRoots | ForEach-Object { "`"$_`"" }) -join " "
  $discoveryCmd = "powershell -ExecutionPolicy Bypass -File scripts/discover_repo_candidates.ps1 -Roots $quotedRoots -MinTrackedFiles $DiscoveryMinTrackedFiles -TopN $DiscoveryTopN -OutputPath .semanticfs/bench/filesystem_repo_candidates_latest.json"
  Run-Step "Filesystem candidate discovery" $discoveryCmd
  if (-not $SkipFilesystemBacklog.IsPresent) {
    Run-Step "Filesystem scope backlog build" "powershell -ExecutionPolicy Bypass -File scripts/build_filesystem_scope_backlog.ps1 -CandidatesPath .semanticfs/bench/filesystem_repo_candidates_latest.json -OutputPath .semanticfs/bench/filesystem_scope_backlog_latest.json"
    if (-not $SkipDomainPlan.IsPresent) {
      Run-Step "Phase 3 domain plan build" "powershell -ExecutionPolicy Bypass -File scripts/build_phase3_domain_plan.ps1 -BacklogPath .semanticfs/bench/filesystem_scope_backlog_latest.json -OutputPath .semanticfs/bench/filesystem_domain_plan_latest.json"
    }
  }
}

Run-Step "Drift summary refresh" "powershell -ExecutionPolicy Bypass -File scripts/drift_summary.ps1"

Write-Host ""
Write-Host "Daytime action items complete."
Write-Host "Key artifacts:"
Write-Host "  .semanticfs/bench/relevance_latest.json"
Write-Host "  .semanticfs/bench/head_to_head_latest.json"
Write-Host "  .semanticfs/bench/tune_holdout_buckit_curated_latest.json"
Write-Host "  .semanticfs/bench/tune_holdout_tensorflow_models_curated_latest.json"
Write-Host "  .semanticfs/bench/filesystem_repo_candidates_latest.json (if discovery roots were provided)"
Write-Host "  .semanticfs/bench/filesystem_scope_backlog_latest.json (if discovery roots were provided and backlog not skipped)"
Write-Host "  .semanticfs/bench/filesystem_domain_plan_latest.json (if discovery roots were provided and domain plan not skipped)"
Write-Host "  .semanticfs/bench/drift_summary_latest.json"
