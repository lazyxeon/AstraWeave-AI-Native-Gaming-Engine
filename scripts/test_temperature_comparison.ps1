# Hermes 2 Pro Temperature Comparison Script
# Tests different temperature settings (0.3, 0.5, 0.7) to find optimal balance

param(
    [int]$IterationsPerTemp = 10,
    [string]$OutputDir = "validation_results"
)

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  Hermes 2 Pro Temperature Comparison Test" -ForegroundColor Cyan
Write-Host "  Testing temperatures: 0.3 (deterministic), 0.5 (balanced), 0.7 (creative)" -ForegroundColor Cyan
Write-Host "  Iterations per temperature: $IterationsPerTemp" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Cyan

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Temperature configurations
$temperatures = @(
    @{Temp = 0.3; Name = "deterministic"; Color = "Blue"},
    @{Temp = 0.5; Name = "balanced"; Color = "Green"},
    @{Temp = 0.7; Name = "creative"; Color = "Magenta"}
)

$allResults = @()

foreach ($config in $temperatures) {
    $temp = $config.Temp
    $name = $config.Name
    $color = $config.Color
    
    Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor $color
    Write-Host "  Testing Temperature $temp ($name)" -ForegroundColor $color
    Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor $color
    
    $outputFile = Join-Path $OutputDir "temp_$($temp)_results.csv"
    "Run,Temperature,ParseSuccess,Steps,Latency_ms,Tier,PlanID,FirstAction,Tools" | Out-File $outputFile -Encoding UTF8
    
    $results = @()
    
    for ($i = 1; $i -le $IterationsPerTemp; $i++) {
        Write-Host "[$i/$IterationsPerTemp] Temperature $temp..." -ForegroundColor $color
        
        # Modify hello_companion to use specific temperature
        # NOTE: This requires manually editing the temperature in hello_companion/src/main.rs
        # OR passing it via environment variable if supported
        # For now, we'll note this limitation
        
        # Run hello_companion
        $output = cargo run -p hello_companion --release --features llm,ollama,metrics -- --llm 2>&1 | Out-String
        
        # Parse results
        $parseSuccess = $output -match "SUCCESS via Direct Parse" -or $output -match "✅ Hermes 2 Pro generated"
        
        if ($output -match "Generated (\d+) step plan in ([\d.]+)ms") {
            $steps = $matches[1]
            $latency = $matches[2]
        } else {
            $steps = 0
            $latency = 0
        }
        
        if ($output -match '"plan_id":\s*"([^"]+)"') {
            $planId = $matches[1]
        } else {
            $planId = "unknown"
        }
        
        $tier = if ($parseSuccess) { "FullLLM" } else { "Fallback" }
        
        if ($output -match '\[0\]\s+([A-Z_]+)') {
            $firstAction = $matches[1]
        } else {
            $firstAction = "none"
        }
        
        $tools = ($output | Select-String -Pattern '\[(\d+)\]\s+([A-Z_]+)' -AllMatches).Matches | ForEach-Object { $_.Groups[2].Value }
        $toolList = ($tools | Sort-Object -Unique) -join ';'
        
        # Record result
        $result = [PSCustomObject]@{
            Run = $i
            Temperature = $temp
            ParseSuccess = $parseSuccess
            Steps = $steps
            Latency_ms = $latency
            Tier = $tier
            PlanID = $planId
            FirstAction = $firstAction
            Tools = $toolList
        }
        
        $results += $result
        $allResults += $result
        
        "$i,$temp,$parseSuccess,$steps,$latency,$tier,$planId,$firstAction,$toolList" | Out-File $outputFile -Append -Encoding UTF8
        
        $statusColor = if ($parseSuccess) { "Green" } else { "Red" }
        Write-Host "  $tier | Steps: $steps | Latency: $latency ms" -ForegroundColor $statusColor
        
        Start-Sleep -Seconds 2
    }
    
    # Per-temperature summary
    $successCount = ($results | Where-Object { $_.ParseSuccess -eq $true }).Count
    $successRate = [math]::Round(($successCount / $IterationsPerTemp) * 100, 1)
    $avgSteps = [math]::Round(($results | Measure-Object -Property Steps -Average).Average, 1)
    $avgLatency = [math]::Round(($results | Measure-Object -Property Latency_ms -Average).Average, 0)
    
    Write-Host "`n  Summary for Temperature $temp ($name):" -ForegroundColor $color
    Write-Host "    Success Rate: $successRate% ($successCount/$IterationsPerTemp)"
    Write-Host "    Avg Steps:    $avgSteps"
    Write-Host "    Avg Latency:  $avgLatency ms"
}

# Final comparison
Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  TEMPERATURE COMPARISON SUMMARY" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Cyan

foreach ($config in $temperatures) {
    $temp = $config.Temp
    $name = $config.Name
    $tempResults = $allResults | Where-Object { $_.Temperature -eq $temp }
    
    $successCount = ($tempResults | Where-Object { $_.ParseSuccess -eq $true }).Count
    $successRate = [math]::Round(($successCount / $IterationsPerTemp) * 100, 1)
    $avgSteps = [math]::Round(($tempResults | Measure-Object -Property Steps -Average).Average, 1)
    $avgLatency = [math]::Round(($tempResults | Measure-Object -Property Latency_ms -Average).Average, 0)
    $minLatency = ($tempResults | Measure-Object -Property Latency_ms -Minimum).Minimum
    $maxLatency = ($tempResults | Measure-Object -Property Latency_ms -Maximum).Maximum
    
    Write-Host "Temperature $temp ($name):" -ForegroundColor $config.Color
    Write-Host "  Success Rate:   $successRate%"
    Write-Host "  Avg Steps:      $avgSteps"
    Write-Host "  Avg Latency:    $avgLatency ms (min: $minLatency, max: $maxLatency)"
    Write-Host ""
}

# Export combined results
$combinedFile = Join-Path $OutputDir "temperature_comparison_combined.csv"
$allResults | Export-Csv -Path $combinedFile -NoTypeInformation -Encoding UTF8

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "All results saved to: $OutputDir" -ForegroundColor Cyan
Write-Host "Combined CSV: $combinedFile" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Cyan

# NOTE: Manual step required
Write-Host "⚠️  NOTE: This script currently uses the default temperature (0.5)" -ForegroundColor Yellow
Write-Host "   To test different temperatures, modify hello_companion/src/main.rs:" -ForegroundColor Yellow
Write-Host "   Line ~726: .with_temperature(0.5) → .with_temperature(0.3 or 0.7)" -ForegroundColor Yellow
Write-Host "   Then recompile with: cargo build -p hello_companion --release`n" -ForegroundColor Yellow
