# AstraWeave Benchmark Production Validation Script
# Comprehensive validation for production readiness
#
# Usage:
#   .\scripts\validate_benchmark_production.ps1
#   .\scripts\validate_benchmark_production.ps1 -Strict          # Fail on warnings
#   .\scripts\validate_benchmark_production.ps1 -GenerateReport  # Output JSON report

param(
    [switch]$Strict,
    [switch]$GenerateReport,
    [string]$OutputPath = "benchmark_validation_report.json"
)

$ErrorActionPreference = "Stop"

# ANSI color codes
$Reset = "`e[0m"
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Cyan = "`e[36m"
$Bold = "`e[1m"

function Write-Header { param([string]$Msg) Write-Host "`n${Cyan}=== $Msg ===${Reset}" }
function Write-Success { param([string]$Msg) Write-Host "${Green}✅ $Msg${Reset}" }
function Write-Warning { param([string]$Msg) Write-Host "${Yellow}⚠️  $Msg${Reset}" }
function Write-Error { param([string]$Msg) Write-Host "${Red}❌ $Msg${Reset}" }
function Write-Info { param([string]$Msg) Write-Host "  ${Msg}" }

Write-Host ""
Write-Host "${Bold}${Cyan}════════════════════════════════════════════════════════${Reset}"
Write-Host "${Bold}${Cyan}   AstraWeave Benchmark Production Validation${Reset}"
Write-Host "${Bold}${Cyan}════════════════════════════════════════════════════════${Reset}"
Write-Host ""

$ValidationResults = @{
    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    grade = "A+"
    passed = 0
    warnings = 0
    errors = 0
    checks = @()
}

# ============================================================================
# CHECK 1: Benchmark Files Inventory
# ============================================================================
Write-Header "Benchmark Files Inventory"

$benchFiles = Get-ChildItem -Path . -Recurse -Include "*.rs" | 
    Where-Object { $_.FullName -like "*\benches\*" }

$benchCount = $benchFiles.Count
Write-Info "Found $benchCount benchmark source files"

if ($benchCount -ge 90) {
    Write-Success "Benchmark inventory: $benchCount files (excellent coverage)"
    $ValidationResults.passed++
    $ValidationResults.checks += @{ name = "benchmark_files"; status = "pass"; count = $benchCount }
} elseif ($benchCount -ge 50) {
    Write-Warning "Benchmark inventory: $benchCount files (good, but could expand)"
    $ValidationResults.warnings++
    $ValidationResults.checks += @{ name = "benchmark_files"; status = "warn"; count = $benchCount }
} else {
    Write-Error "Benchmark inventory: $benchCount files (insufficient coverage)"
    $ValidationResults.errors++
    $ValidationResults.checks += @{ name = "benchmark_files"; status = "fail"; count = $benchCount }
}

# ============================================================================
# CHECK 2: Criterion Configuration
# ============================================================================
Write-Header "Criterion Configuration Check"

$cargoFiles = Get-ChildItem -Path . -Recurse -Include "Cargo.toml" | 
    Where-Object { $_.FullName -notlike "*\target\*" }

$criterionCrates = 0
foreach ($cargo in $cargoFiles) {
    $content = Get-Content $cargo.FullName -Raw -ErrorAction SilentlyContinue
    if ($content -match "criterion") {
        $criterionCrates++
    }
}

Write-Info "Found $criterionCrates crates with Criterion dependency"

if ($criterionCrates -ge 30) {
    Write-Success "Criterion coverage: $criterionCrates crates (production-ready)"
    $ValidationResults.passed++
    $ValidationResults.checks += @{ name = "criterion_crates"; status = "pass"; count = $criterionCrates }
} else {
    Write-Warning "Criterion coverage: $criterionCrates crates (consider expanding)"
    $ValidationResults.warnings++
    $ValidationResults.checks += @{ name = "criterion_crates"; status = "warn"; count = $criterionCrates }
}

# ============================================================================
# CHECK 3: Dashboard Infrastructure
# ============================================================================
Write-Header "Dashboard Infrastructure Check"

$dashboardFiles = @(
    "tools/benchmark-dashboard/index.html",
    "tools/benchmark-dashboard/dashboard.js",
    "scripts/run_benchmark_dashboard.ps1",
    "scripts/export_benchmark_jsonl.ps1",
    "scripts/check_benchmark_thresholds.ps1",
    "scripts/generate_benchmark_graphs.py"
)

$missingFiles = @()
foreach ($file in $dashboardFiles) {
    if (-not (Test-Path $file)) {
        $missingFiles += $file
    }
}

if ($missingFiles.Count -eq 0) {
    Write-Success "Dashboard infrastructure complete (all $($dashboardFiles.Count) files present)"
    $ValidationResults.passed++
    $ValidationResults.checks += @{ name = "dashboard_files"; status = "pass"; files = $dashboardFiles.Count }
} else {
    Write-Error "Missing dashboard files: $($missingFiles -join ', ')"
    $ValidationResults.errors++
    $ValidationResults.checks += @{ name = "dashboard_files"; status = "fail"; missing = $missingFiles }
}

# ============================================================================
# CHECK 4: CI/CD Integration
# ============================================================================
Write-Header "CI/CD Integration Check"

$ciWorkflow = ".github/workflows/benchmark-dashboard.yml"
if (Test-Path $ciWorkflow) {
    $ciContent = Get-Content $ciWorkflow -Raw
    $hasSchedule = $ciContent -match "schedule"
    $hasGhPages = $ciContent -match "gh-pages"
    
    if ($hasSchedule -and $hasGhPages) {
        Write-Success "CI workflow: Nightly benchmark + GH Pages deployment configured"
        $ValidationResults.passed++
        $ValidationResults.checks += @{ name = "ci_workflow"; status = "pass"; schedule = $hasSchedule; ghpages = $hasGhPages }
    } else {
        Write-Warning "CI workflow exists but may be incomplete"
        $ValidationResults.warnings++
        $ValidationResults.checks += @{ name = "ci_workflow"; status = "warn" }
    }
} else {
    Write-Error "CI workflow not found: $ciWorkflow"
    $ValidationResults.errors++
    $ValidationResults.checks += @{ name = "ci_workflow"; status = "fail" }
}

# ============================================================================
# CHECK 5: Benchmark Data Freshness
# ============================================================================
Write-Header "Benchmark Data Freshness Check"

$historyFile = "tools/benchmark-dashboard/benchmark-data/history.jsonl"
$metadataFile = "tools/benchmark-dashboard/benchmark-data/metadata.json"

if (Test-Path $historyFile) {
    $historyLines = (Get-Content $historyFile).Count
    $lastModified = (Get-Item $historyFile).LastWriteTime
    $ageDays = ((Get-Date) - $lastModified).Days
    
    Write-Info "History file: $historyLines entries, last modified $ageDays days ago"
    
    if ($ageDays -le 7) {
        Write-Success "Benchmark data is fresh (within 7 days)"
        $ValidationResults.passed++
        $ValidationResults.checks += @{ name = "data_freshness"; status = "pass"; age_days = $ageDays; entries = $historyLines }
    } elseif ($ageDays -le 30) {
        Write-Warning "Benchmark data is $ageDays days old (consider re-running benchmarks)"
        $ValidationResults.warnings++
        $ValidationResults.checks += @{ name = "data_freshness"; status = "warn"; age_days = $ageDays }
    } else {
        Write-Error "Benchmark data is stale ($ageDays days old)"
        $ValidationResults.errors++
        $ValidationResults.checks += @{ name = "data_freshness"; status = "fail"; age_days = $ageDays }
    }
} else {
    Write-Warning "No benchmark history file found (run benchmarks first)"
    $ValidationResults.warnings++
    $ValidationResults.checks += @{ name = "data_freshness"; status = "warn"; missing = $true }
}

# ============================================================================
# CHECK 6: Master Report Consistency
# ============================================================================
Write-Header "Master Benchmark Report Check"

$masterReport = "docs/masters/MASTER_BENCHMARK_REPORT.md"
if (Test-Path $masterReport) {
    $reportContent = Get-Content $masterReport -Raw
    
    # Check version
    if ($reportContent -match "Version.*?(\d+\.\d+)") {
        $version = $matches[1]
        Write-Info "Master report version: v$version"
    }
    
    # Check for key sections
    $sections = @("Executive Summary", "Performance Highlights", "Reality Check")
    $missingSections = @()
    foreach ($section in $sections) {
        if ($reportContent -notmatch $section) {
            $missingSections += $section
        }
    }
    
    if ($missingSections.Count -eq 0) {
        Write-Success "Master benchmark report is complete and well-structured"
        $ValidationResults.passed++
        $ValidationResults.checks += @{ name = "master_report"; status = "pass"; version = $version }
    } else {
        Write-Warning "Master report missing sections: $($missingSections -join ', ')"
        $ValidationResults.warnings++
        $ValidationResults.checks += @{ name = "master_report"; status = "warn"; missing = $missingSections }
    }
} else {
    Write-Error "Master benchmark report not found"
    $ValidationResults.errors++
    $ValidationResults.checks += @{ name = "master_report"; status = "fail" }
}

# ============================================================================
# CHECK 7: Adversarial Benchmarks Coverage
# ============================================================================
Write-Header "Adversarial Benchmarks Coverage"

$adversarialFiles = Get-ChildItem -Path . -Recurse -Include "*adversarial*.rs" |
    Where-Object { $_.FullName -like "*\benches\*" }

$adversarialCount = $adversarialFiles.Count
Write-Info "Found $adversarialCount adversarial benchmark files"

if ($adversarialCount -ge 20) {
    Write-Success "Adversarial coverage: $adversarialCount files (comprehensive edge-case testing)"
    $ValidationResults.passed++
    $ValidationResults.checks += @{ name = "adversarial_benchmarks"; status = "pass"; count = $adversarialCount }
} elseif ($adversarialCount -ge 10) {
    Write-Warning "Adversarial coverage: $adversarialCount files (good, but could expand)"
    $ValidationResults.warnings++
    $ValidationResults.checks += @{ name = "adversarial_benchmarks"; status = "warn"; count = $adversarialCount }
} else {
    Write-Warning "Adversarial coverage: $adversarialCount files (consider adding edge-case benchmarks)"
    $ValidationResults.warnings++
    $ValidationResults.checks += @{ name = "adversarial_benchmarks"; status = "warn"; count = $adversarialCount }
}

# ============================================================================
# FINAL GRADE CALCULATION
# ============================================================================
Write-Header "Validation Summary"

$totalChecks = $ValidationResults.passed + $ValidationResults.warnings + $ValidationResults.errors
$passRate = [math]::Round(($ValidationResults.passed / $totalChecks) * 100, 1)

# Calculate grade
if ($ValidationResults.errors -gt 0) {
    if ($ValidationResults.errors -ge 3) {
        $ValidationResults.grade = "C"
    } else {
        $ValidationResults.grade = "B"
    }
} elseif ($ValidationResults.warnings -gt 2) {
    $ValidationResults.grade = "A"
} else {
    $ValidationResults.grade = "A+"
}

Write-Host ""
Write-Host "${Bold}Results:${Reset}"
Write-Host "  ✅ Passed:   $($ValidationResults.passed)"
Write-Host "  ⚠️  Warnings: $($ValidationResults.warnings)"
Write-Host "  ❌ Errors:   $($ValidationResults.errors)"
Write-Host ""
Write-Host "  ${Bold}Pass Rate:${Reset} ${passRate}%"
Write-Host "  ${Bold}Grade:${Reset} $($ValidationResults.grade)"
Write-Host ""

# Generate report if requested
if ($GenerateReport) {
    $ValidationResults | ConvertTo-Json -Depth 10 | Out-File $OutputPath -Encoding UTF8
    Write-Success "Validation report saved to: $OutputPath"
}

# Exit with appropriate code
if ($Strict -and ($ValidationResults.warnings -gt 0 -or $ValidationResults.errors -gt 0)) {
    Write-Host "${Yellow}Strict mode: Failing due to warnings/errors${Reset}"
    exit 1
} elseif ($ValidationResults.errors -gt 0) {
    exit 1
} else {
    Write-Success "Benchmark infrastructure is production-ready!"
    exit 0
}
