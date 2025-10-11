# Export Benchmark Results to JSONL History
# Extracts Criterion benchmark data and appends to history.jsonl for dashboard visualization
#
# Usage:
#   .\scripts\export_benchmark_jsonl.ps1
#   .\scripts\export_benchmark_jsonl.ps1 -BenchmarkDir "custom/path"
#   .\scripts\export_benchmark_jsonl.ps1 -MaxAgeDays 60

param(
    [string]$BenchmarkDir = "target/criterion",
    [string]$OutputFile = "target/benchmark-data/history.jsonl",
    [int]$MaxAgeDays = 30,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    $color = switch ($Level) {
        "ERROR" { "Red" }
        "WARN" { "Yellow" }
        "SUCCESS" { "Green" }
        default { "White" }
    }
    Write-Host "[$timestamp] $Message" -ForegroundColor $color
}

function Get-GitMetadata {
    try {
        $sha = git rev-parse HEAD 2>&1
        $branch = git rev-parse --abbrev-ref HEAD 2>&1
        $dirty = (git status --porcelain 2>&1).Count -gt 0
        
        return @{
            sha = if ($sha -match "^[0-9a-f]{40}$") { $sha.Substring(0, 8) } else { "unknown" }
            branch = if ($branch -and $branch -notmatch "fatal") { $branch } else { "unknown" }
            dirty = $dirty
        }
    }
    catch {
        return @{ sha = "unknown"; branch = "unknown"; dirty = $false }
    }
}

function Parse-CriterionEstimates {
    param([string]$EstimatesJsonPath)
    
    if (-not (Test-Path $EstimatesJsonPath)) {
        Write-Log "Estimates file not found: $EstimatesJsonPath" "WARN"
        return $null
    }
    
    try {
        $estimates = Get-Content $EstimatesJsonPath -Raw | ConvertFrom-Json
        
        # Criterion stores times in nanoseconds
        $meanNs = $estimates.mean.point_estimate
        $stddevNs = $estimates.std_dev.point_estimate
        
        return @{
            mean_ns = [math]::Round($meanNs, 2)
            stddev_ns = [math]::Round($stddevNs, 2)
            unit = "ns"
        }
    }
    catch {
        Write-Log "Failed to parse $EstimatesJsonPath : $_" "WARN"
        return $null
    }
}

function Export-BenchmarkResults {
    param([string]$BenchmarkRoot, [string]$OutputPath)
    
    Write-Log "Scanning Criterion benchmark results in $BenchmarkRoot..."
    
    if (-not (Test-Path $BenchmarkRoot)) {
        Write-Log "Benchmark directory not found: $BenchmarkRoot" "ERROR"
        Write-Log "Run 'cargo bench' first to generate benchmark data" "ERROR"
        return 0
    }
    
    # Ensure output directory exists
    $outputDir = Split-Path $OutputPath -Parent
    if (-not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
        Write-Log "Created output directory: $outputDir"
    }
    
    # Get git metadata
    $git = Get-GitMetadata
    $timestamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    
    # Find all benchmark results (Criterion stores in <benchmark>/base/estimates.json)
    $benchmarkDirs = Get-ChildItem -Path $BenchmarkRoot -Recurse -Filter "estimates.json" -File
    
    $exportCount = 0
    $entries = @()
    
    foreach ($estimatesFile in $benchmarkDirs) {
        # Parse path to extract benchmark name
        # Example: target/criterion/astraweave-core/ecs_benchmarks/world_creation/base/estimates.json
        $fullPath = $estimatesFile.FullName
        
        # Extract the benchmark path between criterion/ and /base/estimates.json
        if ($fullPath -match 'criterion[/\\](.+)[/\\]base[/\\]estimates\.json$') {
            $benchmarkPath = $matches[1]
            $pathParts = $benchmarkPath -split '[/\\]'
            
            if ($pathParts.Count -lt 2) {
                if ($Verbose) { Write-Log "Skipping invalid path: $benchmarkPath" "WARN" }
                continue
            }
            
            # Extract crate and benchmark name
            # For paths like: astraweave-core/ecs_benchmarks/world_creation
            $crate = $pathParts[0]
            $benchGroup = $pathParts[1]
            
            # Remaining parts form the benchmark name
            if ($pathParts.Count -gt 2) {
                $benchName = $pathParts[2..($pathParts.Count - 1)] -join '/'
            } else {
                $benchName = "default"
            }
            
            # Full benchmark identifier (matches threshold JSON format)
            $fullName = "${crate}::${benchGroup}/${benchName}"
        } else {
            if ($Verbose) { Write-Log "Skipping non-matching path: $fullPath" "WARN" }
            continue
        }
        
        # Parse estimates
        $stats = Parse-CriterionEstimates -EstimatesJsonPath $estimatesFile.FullName
        
        if ($null -eq $stats) {
            continue
        }
        
        # Create JSONL entry
        $entry = @{
            timestamp = $timestamp
            benchmark_name = $fullName
            value = $stats.mean_ns
            stddev = $stats.stddev_ns
            unit = $stats.unit
            git_sha = $git.sha
            git_branch = $git.branch
            git_dirty = $git.dirty
            crate = $crate
            group = $benchGroup
            name = $benchName
        } | ConvertTo-Json -Compress
        
        $entries += $entry
        $exportCount++
        
        if ($Verbose) {
            Write-Log "Exported: $fullName = $($stats.mean_ns) ns"
        }
    }
    
    if ($exportCount -eq 0) {
        Write-Log "No benchmark results found in $BenchmarkRoot" "WARN"
        Write-Log "Run 'cargo bench' to generate data" "WARN"
        return 0
    }
    
    # Append to history file (JSONL = one JSON object per line)
    $entries | Out-File -FilePath $OutputPath -Append -Encoding utf8
    
    Write-Log "Exported $exportCount benchmark results to $OutputPath" "SUCCESS"
    
    # Rotate old entries (keep last N days)
    Rotate-OldEntries -HistoryFile $OutputPath -MaxAgeDays $MaxAgeDays
    
    return $exportCount
}

function Rotate-OldEntries {
    param([string]$HistoryFile, [int]$MaxAgeDays)
    
    if (-not (Test-Path $HistoryFile)) {
        return
    }
    
    Write-Log "Rotating entries older than $MaxAgeDays days..."
    
    $cutoffDate = (Get-Date).AddDays(-$MaxAgeDays).ToUniversalTime()
    $lines = Get-Content $HistoryFile
    $kept = 0
    $removed = 0
    
    $tempFile = "$HistoryFile.tmp"
    
    foreach ($line in $lines) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        
        try {
            $entry = $line | ConvertFrom-Json
            $entryDate = [DateTime]::Parse($entry.timestamp)
            
            if ($entryDate -gt $cutoffDate) {
                $line | Out-File -FilePath $tempFile -Append -Encoding utf8
                $kept++
            }
            else {
                $removed++
            }
        }
        catch {
            # Keep malformed entries (don't lose data)
            $line | Out-File -FilePath $tempFile -Append -Encoding utf8
            $kept++
        }
    }
    
    if ($removed -gt 0) {
        Move-Item -Path $tempFile -Destination $HistoryFile -Force
        Write-Log "Removed $removed old entries, kept $kept" "SUCCESS"
    }
    else {
        if (Test-Path $tempFile) {
            Remove-Item $tempFile
        }
        Write-Log "No entries to rotate (all within $MaxAgeDays days)"
    }
}

# Main execution
Write-Log "=== AstraWeave Benchmark JSONL Exporter ==="
Write-Log "Benchmark Directory: $BenchmarkDir"
Write-Log "Output File: $OutputFile"
Write-Log "Max Age: $MaxAgeDays days"
Write-Log ""

$count = Export-BenchmarkResults -BenchmarkRoot $BenchmarkDir -OutputPath $OutputFile

if ($count -gt 0) {
    Write-Log ""
    Write-Log "=== Export Complete ===" "SUCCESS"
    Write-Log "Total benchmarks exported: $count" "SUCCESS"
    
    $fileSize = (Get-Item $OutputFile).Length
    Write-Log "History file size: $([math]::Round($fileSize / 1KB, 2)) KB" "SUCCESS"
    
    Write-Log ""
    Write-Log "Next steps:"
    Write-Log "  1. View JSONL: Get-Content $OutputFile | Select -Last 5 | ConvertFrom-Json | Format-Table"
    Write-Log "  2. Build dashboard: cd tools/benchmark-dashboard && python -m http.server 8000"
    Write-Log "  3. Open dashboard: http://localhost:8000"
}
else {
    Write-Log ""
    Write-Log "=== Export Failed ===" "ERROR"
    Write-Log "No benchmark data found. Run 'cargo bench' first." "ERROR"
    exit 1
}
