# Hermes 2 Pro Extended Validation Script
# Runs hello_companion multiple times to collect statistics

param(
    [int]$Iterations = 10,
    [string]$OutputFile = "hermes2pro_validation_results.csv"
)

Write-Host "═══════════════════════════════════════════════════════════════"
Write-Host "  Hermes 2 Pro Extended Validation Test"
Write-Host "  Iterations: $Iterations"
Write-Host "  Output: $OutputFile"
Write-Host "═══════════════════════════════════════════════════════════════`n"

# Initialize results array
$results = @()

# CSV header
"Run,ParseSuccess,Steps,Latency_ms,Tier,PlanID,FirstAction,Tools" | Out-File $OutputFile -Encoding UTF8

for ($i = 1; $i -le $Iterations; $i++) {
    Write-Host "[$i/$Iterations] Running test..." -ForegroundColor Cyan
    
    # Run hello_companion and capture output
    $output = cargo run -p hello_companion --release --features llm,ollama,metrics -- --llm 2>&1 | Out-String
    
    # Parse results from output
    $parseSuccess = $output -match "SUCCESS via Direct Parse" -or $output -match "✅ Hermes 2 Pro generated"
    $fallback = $output -match "Hermes 2 Pro returned fallback"
    
    # Extract metrics
    if ($output -match "Generated (\d+) step plan in ([\d.]+)ms") {
        $steps = $matches[1]
        $latency = $matches[2]
    } else {
        $steps = 0
        $latency = 0
    }
    
    # Extract plan ID
    if ($output -match '"plan_id":\s*"([^"]+)"') {
        $planId = $matches[1]
    } else {
        $planId = "unknown"
    }
    
    # Determine tier
    if ($parseSuccess) {
        $tier = "FullLLM"
    } elseif ($fallback) {
        $tier = "Fallback"
    } else {
        $tier = "Failed"
    }
    
    # Extract first action
    if ($output -match '\[0\]\s+([A-Z_]+)') {
        $firstAction = $matches[1]
    } else {
        $firstAction = "none"
    }
    
    # Extract tools used
    $tools = ($output | Select-String -Pattern '\[(\d+)\]\s+([A-Z_]+)' -AllMatches).Matches | ForEach-Object { $_.Groups[2].Value }
    $toolList = ($tools | Sort-Object -Unique) -join ';'
    
    # Create result object
    $result = [PSCustomObject]@{
        Run = $i
        ParseSuccess = $parseSuccess
        Steps = $steps
        Latency_ms = $latency
        Tier = $tier
        PlanID = $planId
        FirstAction = $firstAction
        Tools = $toolList
    }
    
    $results += $result
    
    # Append to CSV
    "$i,$parseSuccess,$steps,$latency,$tier,$planId,$firstAction,$toolList" | Out-File $OutputFile -Append -Encoding UTF8
    
    # Display progress
    $color = if ($parseSuccess) { "Green" } elseif ($fallback) { "Yellow" } else { "Red" }
    Write-Host "  Status: $tier | Steps: $steps | Latency: $latency ms" -ForegroundColor $color
    
    # Small delay between runs
    Start-Sleep -Seconds 2
}

# Calculate statistics
Write-Host "`n═══════════════════════════════════════════════════════════════"
Write-Host "  VALIDATION SUMMARY"
Write-Host "═══════════════════════════════════════════════════════════════`n"

$successCount = ($results | Where-Object { $_.ParseSuccess -eq $true }).Count
$fallbackCount = ($results | Where-Object { $_.Tier -eq "Fallback" }).Count
$failedCount = ($results | Where-Object { $_.Tier -eq "Failed" }).Count

$successRate = [math]::Round(($successCount / $Iterations) * 100, 1)
$avgSteps = [math]::Round(($results | Measure-Object -Property Steps -Average).Average, 1)
$avgLatency = [math]::Round(($results | Measure-Object -Property Latency_ms -Average).Average, 0)
$minLatency = ($results | Measure-Object -Property Latency_ms -Minimum).Minimum
$maxLatency = ($results | Measure-Object -Property Latency_ms -Maximum).Maximum

Write-Host "Parse Success Rate: $successRate% ($successCount/$Iterations)" -ForegroundColor Green
Write-Host "Fallback Rate:      $(($fallbackCount / $Iterations) * 100)% ($fallbackCount/$Iterations)" -ForegroundColor Yellow
Write-Host "Failed Rate:        $(($failedCount / $Iterations) * 100)% ($failedCount/$Iterations)" -ForegroundColor Red
Write-Host ""
Write-Host "Average Steps:      $avgSteps"
Write-Host "Average Latency:    $avgLatency ms"
Write-Host "Min Latency:        $minLatency ms"
Write-Host "Max Latency:        $maxLatency ms"
Write-Host ""

# Tool usage frequency
$allTools = $results | ForEach-Object { $_.Tools -split ';' } | Where-Object { $_ -ne '' }
$toolFrequency = $allTools | Group-Object | Sort-Object Count -Descending | Select-Object -First 10

Write-Host "Top 10 Tools Used:"
foreach ($tool in $toolFrequency) {
    $pct = [math]::Round(($tool.Count / $Iterations) * 100, 1)
    Write-Host "  $($tool.Name): $($tool.Count)× ($pct%)"
}

Write-Host "`n═══════════════════════════════════════════════════════════════"
Write-Host "Results saved to: $OutputFile" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════`n"
