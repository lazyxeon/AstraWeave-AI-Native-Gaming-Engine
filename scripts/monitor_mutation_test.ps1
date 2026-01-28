# Mutation Test Progress Monitor
# Usage: .\scripts\monitor_mutation_test.ps1

param(
    [string]$Package = "",
    [int]$RefreshSeconds = 30
)

function Get-MutationProgress {
    if (Test-Path "mutants.out/outcomes.json") {
        $json = Get-Content "mutants.out/outcomes.json" -Raw | ConvertFrom-Json
        $caught = ($json | Where-Object {$_.scenario.outcome -eq "Caught"}).Count
        $missed = ($json | Where-Object {$_.scenario.outcome -eq "Missed"}).Count
        $timeout = ($json | Where-Object {$_.scenario.outcome -eq "Timeout"}).Count
        $unviable = ($json | Where-Object {$_.scenario.outcome -eq "Unviable"}).Count
        $total = $caught + $missed + $timeout + $unviable
        $viable = $caught + $missed
        
        $score = if ($viable -gt 0) { [math]::Round(100.0 * $caught / $viable, 2) } else { 0 }
        
        return @{
            Total = $total
            Caught = $caught
            Missed = $missed
            Timeout = $timeout
            Unviable = $unviable
            Viable = $viable
            Score = $score
        }
    }
    return $null
}

Write-Host "=== MUTATION TEST MONITOR ===" -ForegroundColor Cyan
Write-Host "Package: $Package"
Write-Host "Refresh: Every $RefreshSeconds seconds"
Write-Host "Press Ctrl+C to stop monitoring"
Write-Host ""

$lastTotal = 0
while ($true) {
    $progress = Get-MutationProgress
    
    if ($progress) {
        $timestamp = Get-Date -Format "HH:mm:ss"
        
        if ($progress.Total -ne $lastTotal) {
            Write-Host "[$timestamp] Progress: $($progress.Total) mutants tested" -ForegroundColor Green
            Write-Host "  Caught: $($progress.Caught) | Missed: $($progress.Missed) | Timeout: $($progress.Timeout) | Unviable: $($progress.Unviable)"
            Write-Host "  Mutation Score: $($progress.Score)% (from $($progress.Viable) viable mutants)"
            Write-Host ""
            $lastTotal = $progress.Total
        }
    } else {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Waiting for test to start..." -ForegroundColor Yellow
    }
    
    Start-Sleep -Seconds $RefreshSeconds
}
