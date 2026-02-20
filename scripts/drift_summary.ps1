param(
    [string]$HistoryDir = ".semanticfs/bench/history",
    [int]$LastN = 7,
    [int]$NightTarget = 7,
    [string[]]$NightRequiredDatasets = @("semanticfs_repo_v1", "ai_testgen_repo_v1"),
    [string]$OutputPath = ".semanticfs/bench/drift_summary_latest.json"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function New-StatSummary {
    param(
        [double[]]$Values
    )
    if (-not $Values -or $Values.Count -eq 0) {
        return @{
            min = $null
            avg = $null
            max = $null
            count = 0
        }
    }
    return @{
        min = ($Values | Measure-Object -Minimum).Minimum
        avg = ($Values | Measure-Object -Average).Average
        max = ($Values | Measure-Object -Maximum).Maximum
        count = $Values.Count
    }
}

function Parse-HistoryTimestampUtc {
    param(
        [string]$FileName
    )
    if ($FileName -match "([0-9]{8}T[0-9]{6}Z)") {
        return [datetime]::ParseExact($Matches[1], "yyyyMMdd'T'HHmmss'Z'", [System.Globalization.CultureInfo]::InvariantCulture, [System.Globalization.DateTimeStyles]::AssumeUniversal -bor [System.Globalization.DateTimeStyles]::AdjustToUniversal)
    }
    return $null
}

$historyPath = Resolve-Path -LiteralPath $HistoryDir -ErrorAction Stop
$headFiles = Get-ChildItem -LiteralPath $historyPath -Filter "head_to_head_latest_*.json" | Sort-Object LastWriteTime
$relevanceFiles = Get-ChildItem -LiteralPath $historyPath -Filter "relevance_latest_*.json" | Sort-Object LastWriteTime

$headRowsByDataset = @{}
$relevanceRowsByDataset = @{}
$headCounts = @{}
$relevanceCounts = @{}
$dateToDatasets = @{}

foreach ($f in $headFiles) {
    $timestampUtc = Parse-HistoryTimestampUtc -FileName $f.Name
    if (-not $timestampUtc) {
        continue
    }
    $dateKey = $timestampUtc.ToString("yyyy-MM-dd")
    if (-not $dateToDatasets.ContainsKey($dateKey)) {
        $dateToDatasets[$dateKey] = New-Object System.Collections.Generic.HashSet[string]
    }

    $j = Get-Content -LiteralPath $f.FullName -Raw | ConvertFrom-Json
    $hasDatasets = $j.PSObject.Properties.Name -contains "datasets"
    if ((-not $hasDatasets) -or (-not $j.datasets)) {
        continue
    }

    foreach ($d in $j.datasets) {
        $dataset = [string]$d.dataset_name
        if ([string]::IsNullOrWhiteSpace($dataset)) {
            continue
        }

        $null = $dateToDatasets[$dateKey].Add($dataset)

        if (-not $headRowsByDataset.ContainsKey($dataset)) {
            $headRowsByDataset[$dataset] = New-Object System.Collections.ArrayList
            $headCounts[$dataset] = 0
        }
        $headCounts[$dataset] += 1

        $row = [pscustomobject][ordered]@{
            file = $f.Name
            timestamp_utc = $timestampUtc.ToString("o")
            delta_mrr = [double]$j.delta_semanticfs_minus_baseline.mrr
            delta_recall = [double]$j.delta_semanticfs_minus_baseline.recall_at_topn
            delta_symbol = [double]$j.delta_semanticfs_minus_baseline.symbol_hit_rate
            delta_p95_ms = [double]$j.delta_semanticfs_minus_baseline.p95_latency_ms
            semanticfs = [ordered]@{
                recall = [double]$j.engines.semanticfs.recall_at_topn
                mrr = [double]$j.engines.semanticfs.mrr
                symbol_hit_rate = [double]$j.engines.semanticfs.symbol_hit_rate
                p95_ms = [double]$j.engines.semanticfs.latency_ms.p95
            }
            baseline = [ordered]@{
                recall = [double]$j.engines.baseline_rg.recall_at_topn
                mrr = [double]$j.engines.baseline_rg.mrr
                symbol_hit_rate = [double]$j.engines.baseline_rg.symbol_hit_rate
                p95_ms = [double]$j.engines.baseline_rg.latency_ms.p95
            }
        }
        [void]$headRowsByDataset[$dataset].Add($row)
    }
}

foreach ($f in $relevanceFiles) {
    $timestampUtc = Parse-HistoryTimestampUtc -FileName $f.Name
    if (-not $timestampUtc) {
        continue
    }
    $j = Get-Content -LiteralPath $f.FullName -Raw | ConvertFrom-Json
    $hasDatasets = $j.PSObject.Properties.Name -contains "datasets"
    if ((-not $hasDatasets) -or (-not $j.datasets)) {
        continue
    }
    foreach ($d in $j.datasets) {
        $dataset = [string]$d.dataset_name
        if ([string]::IsNullOrWhiteSpace($dataset)) {
            continue
        }
        if (-not $relevanceRowsByDataset.ContainsKey($dataset)) {
            $relevanceRowsByDataset[$dataset] = New-Object System.Collections.ArrayList
            $relevanceCounts[$dataset] = 0
        }
        $relevanceCounts[$dataset] += 1
        $row = [pscustomobject][ordered]@{
            file = $f.Name
            timestamp_utc = $timestampUtc.ToString("o")
            query_count = [int]$d.query_count
            recall = [double]$d.metrics.recall_at_5
            mrr = [double]$d.metrics.mrr
            symbol_hit_rate = [double]$d.metrics.symbol_hit_rate
        }
        [void]$relevanceRowsByDataset[$dataset].Add($row)
    }
}

$nightsComplete = 0
$completeDates = New-Object System.Collections.ArrayList
$partialDates = New-Object System.Collections.ArrayList
foreach ($date in ($dateToDatasets.Keys | Sort-Object)) {
    $set = $dateToDatasets[$date]
    $datasetsForDate = @()
    if (($set -is [System.Collections.IEnumerable]) -and -not ($set -is [string])) {
        $datasetsForDate = @($set)
    } elseif ($null -ne $set) {
        $datasetsForDate = @([string]$set)
    }
    $datasetsForDate = @($datasetsForDate | Sort-Object -Unique)

    $isComplete = $true
    foreach ($required in $NightRequiredDatasets) {
        if (-not ($datasetsForDate -contains $required)) {
            $isComplete = $false
            break
        }
    }
    if ($isComplete) {
        $nightsComplete += 1
        [void]$completeDates.Add([ordered]@{ date = $date; datasets = $datasetsForDate })
    } else {
        [void]$partialDates.Add([ordered]@{ date = $date; datasets = $datasetsForDate })
    }
}
$nightsRemaining = [Math]::Max(0, $NightTarget - $nightsComplete)

$datasetSummaries = [ordered]@{}
$allDatasets = @($headRowsByDataset.Keys + $relevanceRowsByDataset.Keys | Sort-Object -Unique)
foreach ($dataset in $allDatasets) {
    $headRows = @()
    if ($headRowsByDataset.ContainsKey($dataset)) {
        $headRows = @($headRowsByDataset[$dataset] | Sort-Object timestamp_utc)
    }
    $relRows = @()
    if ($relevanceRowsByDataset.ContainsKey($dataset)) {
        $relRows = @($relevanceRowsByDataset[$dataset] | Sort-Object timestamp_utc)
    }

    $lastHead = if ($headRows.Count -gt 0) { $headRows[-1] } else { $null }
    $lastRel = if ($relRows.Count -gt 0) { $relRows[-1] } else { $null }
    $headLastN = @($headRows | Sort-Object timestamp_utc -Descending | Select-Object -First $LastN)
    $relLastN = @($relRows | Sort-Object timestamp_utc -Descending | Select-Object -First $LastN)

    $datasetSummaries[$dataset] = [ordered]@{
        counts = [ordered]@{
            head_to_head = if ($headCounts.ContainsKey($dataset)) { [int]$headCounts[$dataset] } else { 0 }
            relevance = if ($relevanceCounts.ContainsKey($dataset)) { [int]$relevanceCounts[$dataset] } else { 0 }
        }
        latest = [ordered]@{
            head_to_head = $lastHead
            relevance = $lastRel
        }
        last_n = [ordered]@{
            n = $LastN
            head_to_head_delta = [ordered]@{
                mrr = New-StatSummary -Values @($headLastN | ForEach-Object { [double]$_.delta_mrr })
                recall = New-StatSummary -Values @($headLastN | ForEach-Object { [double]$_.delta_recall })
                symbol_hit_rate = New-StatSummary -Values @($headLastN | ForEach-Object { [double]$_.delta_symbol })
                p95_latency_ms = New-StatSummary -Values @($headLastN | ForEach-Object { [double]$_.delta_p95_ms })
            }
            relevance = [ordered]@{
                recall = New-StatSummary -Values @($relLastN | ForEach-Object { [double]$_.recall })
                mrr = New-StatSummary -Values @($relLastN | ForEach-Object { [double]$_.mrr })
                symbol_hit_rate = New-StatSummary -Values @($relLastN | ForEach-Object { [double]$_.symbol_hit_rate })
            }
        }
    }
}

$summary = [ordered]@{
    scenario = "drift_summary"
    generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
    history_dir = $historyPath.Path
    history_files = [ordered]@{
        head_to_head = $headFiles.Count
        relevance = $relevanceFiles.Count
    }
    nights = [ordered]@{
        target = $NightTarget
        complete = $nightsComplete
        remaining = $nightsRemaining
        required_datasets = $NightRequiredDatasets
        complete_dates = @($completeDates)
        partial_dates = @($partialDates)
    }
    datasets = $datasetSummaries
}

$outFile = [System.IO.Path]::GetFullPath($OutputPath)
$outDir = Split-Path -Parent $outFile
if (-not [string]::IsNullOrWhiteSpace($outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}
$summary | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $outFile

Write-Host "== Drift Summary =="
Write-Host ("nights complete: {0}/{1} (remaining: {2})" -f $nightsComplete, $NightTarget, $nightsRemaining)
Write-Host ("history files: head_to_head={0}, relevance={1}" -f $headFiles.Count, $relevanceFiles.Count)
foreach ($dataset in $datasetSummaries.Keys) {
    $ds = $datasetSummaries[$dataset]
    $hCount = $ds.counts.head_to_head
    $rCount = $ds.counts.relevance
    Write-Host ("dataset={0} counts: head_to_head={1}, relevance={2}" -f $dataset, $hCount, $rCount)
    if ($ds.latest.head_to_head) {
        $l = $ds.latest.head_to_head
        Write-Host ("  latest h2h: sem recall={0:N4} mrr={1:N4} symbol={2:N4} p95={3:N3}ms | base recall={4:N4} mrr={5:N4} symbol={6:N4} p95={7:N3}ms" -f `
            [double]$l.semanticfs.recall, [double]$l.semanticfs.mrr, [double]$l.semanticfs.symbol_hit_rate, [double]$l.semanticfs.p95_ms, `
            [double]$l.baseline.recall, [double]$l.baseline.mrr, [double]$l.baseline.symbol_hit_rate, [double]$l.baseline.p95_ms)
    }
    $d = $ds.last_n.head_to_head_delta
    if ($d.mrr.count -gt 0) {
        Write-Host ("  last-{0} delta: mrr(min/avg)={1:N4}/{2:N4} recall(min/avg)={3:N4}/{4:N4} symbol(min/avg)={5:N4}/{6:N4} p95(min/avg)={7:N3}/{8:N3}ms" -f `
            $LastN, [double]$d.mrr.min, [double]$d.mrr.avg, [double]$d.recall.min, [double]$d.recall.avg, [double]$d.symbol_hit_rate.min, [double]$d.symbol_hit_rate.avg, [double]$d.p95_latency_ms.min, [double]$d.p95_latency_ms.avg)
    }
}
Write-Host ("saved drift summary: {0}" -f $outFile)
