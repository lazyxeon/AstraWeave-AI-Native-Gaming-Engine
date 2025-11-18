# Root Directory Organization Script
# Moves documents from root to organized docs/ subdirectories

$rootPath = "c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine"
Set-Location $rootPath

# Create directory structure
$directories = @(
    "docs\audits",
    "docs\reports",
    "docs\guides",
    "docs\archive\logs",
    "docs\archive\test-outputs",
    "docs\archive\scripts"
)

foreach ($dir in $directories) {
    $fullPath = Join-Path $rootPath $dir
    if (-not (Test-Path $fullPath)) {
        New-Item -Path $fullPath -ItemType Directory -Force | Out-Null
        Write-Host "Created: $dir"
    }
}

# Audit reports to docs/audits/
$auditFiles = @(
    "COMPREHENSIVE_AUDIT_REPORT.md",
    "DOCUMENTATION_AUDIT_REPORT.md",
    "DOCUMENTATION_AUDIT_SUMMARY.md",
    "COMPETITIVE_ANALYSIS_SUMMARY.md",
    "COMPETITIVE_MATRIX.md",
    "EXTERNAL_RESEARCH_COMPETITIVE_ANALYSIS.md",
    "EXTERNAL_RESEARCH_INDEX.md",
    "GAP_ANALYSIS_ACTION_PLAN.md",
    "SECURITY_REMEDIATION_REPORT.md"
)

Write-Host "`nMoving audit reports to docs/audits/..."
foreach ($file in $auditFiles) {
    if (Test-Path $file) {
        git mv $file "docs\audits\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

# Status reports to docs/reports/
$reportFiles = @(
    "BENCHMARK_DASHBOARD_FIX.md",
    "BENCHMARK_DISCOVERY_REPORT.md",
    "BENCHMARK_FIX_COMPLETION_REPORT.md",
    "BENCHMARK_RECONCILIATION_REPORT.md",
    "PROBLEM_RESOLUTION_PROGRESS.md",
    "PROBLEM_RESOLUTION_REPORT.md",
    "REMEDIATION_ROADMAP.md",
    "REMEDIATION_STATUS.md",
    "RENDERING_ENHANCEMENTS_REPORT.md",
    "RENDERING_MASTER_DOCS_UPDATE_SUMMARY.md",
    "MASTER_DOCS_UPDATE_COMPLETE.md",
    "MASTER_DOCS_UPDATE_STATUS.md",
    "PHASE1_COMPLETE.md",
    "PHASE2_COMPLETE.md",
    "SESSION_3_SUMMARY.md"
)

Write-Host "`nMoving status reports to docs/reports/..."
foreach ($file in $reportFiles) {
    if (Test-Path $file) {
        git mv $file "docs\reports\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

# Guides to docs/guides/
$guideFiles = @(
    "HMAC_QUICK_REFERENCE.md",
    "HMAC_SHA256_IMPLEMENTATION.md",
    "QUICK_START_GLB_ASSETS.md"
)

Write-Host "`nMoving guides to docs/guides/..."
foreach ($file in $guideFiles) {
    if (Test-Path $file) {
        git mv $file "docs\guides\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

# Archive logs
$logFiles = @("benchmark_run.log", "server_test.log")

Write-Host "`nMoving logs to docs/archive/logs/..."
foreach ($file in $logFiles) {
    if (Test-Path $file) {
        git mv $file "docs\archive\logs\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

# Archive test outputs
$testOutputs = @(
    "aw_editor_check.txt", "bench_errors.txt", "check_output_session4.txt",
    "clippy_errors.txt", "clippy_output.txt", "coverage_output.txt",
    "examples_check.txt", "final_check.txt", "lib_check.txt", "output.txt",
    "shader_errors.txt", "shader_test_full.txt", "shader_test_output.txt",
    "temp_check_output.txt", "test_output.txt", "wgsl_list.txt"
)

Write-Host "`nMoving test outputs to docs/archive/test-outputs/..."
foreach ($file in $testOutputs) {
    if (Test-Path $file) {
        git mv $file "docs\archive\test-outputs\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

# Archive scripts
$scriptFiles = @("fix_packet_tests.ps1", "fix_tests.ps1")

Write-Host "`nMoving old scripts to docs/archive/scripts/..."
foreach ($file in $scriptFiles) {
    if (Test-Path $file) {
        git mv $file "docs\archive\scripts\" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $file"
        } else {
            Write-Host "  ✗ $file (skipped)"
        }
    }
}

Write-Host ""
Write-Host "Root directory cleanup complete!"
Write-Host ""
Write-Host "Run git status to review changes."
