# Quick Fix Benchmark Compilation Errors
# Automatically fixes the 3 most common benchmark compilation issues

param(
    [switch]$DryRun,
    [switch]$Verbose
)

$ErrorActionPreference = "Continue"

function Write-Fix {
    param([string]$Message, [string]$Type = "INFO")
    $colors = @{"INFO"="Cyan"; "SUCCESS"="Green"; "ERROR"="Red"; "WARN"="Yellow"}
    $symbols = @{"INFO"="ℹ️"; "SUCCESS"="✅"; "ERROR"="❌"; "WARN"="⚠️"}
    Write-Host "$($symbols[$Type]) $Message" -ForegroundColor $colors[$Type]
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "   Benchmark Compilation Error Auto-Fix" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host ""

$fixCount = 0

# ============================================================================
# FIX 1: astraweave-ai GOAP imports (use astraweave_behavior, not astraweave_ai)
# ============================================================================
Write-Fix "Fixing astraweave-ai GOAP imports..." "INFO"

$aiFiles = @(
    "astraweave-ai\benches\goap_bench.rs",
    "astraweave-ai\benches\goap_performance_bench.rs",
    "astraweave-ai\benches\goap_vs_rule_bench.rs"
)

foreach ($file in $aiFiles) {
    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        
        if ($content -match 'use astraweave_ai::goap') {
            Write-Fix "  • $file needs fixing" "WARN"
            
            if (-not $DryRun) {
                $newContent = $content -replace 'use astraweave_ai::goap', 'use astraweave_behavior::goap'
                Set-Content -Path $file -Value $newContent -NoNewline
                Write-Fix "    Fixed GOAP imports" "SUCCESS"
                $fixCount++
            }
            else {
                Write-Fix "    Would fix GOAP imports (dry run)" "INFO"
            }
        }
        else {
            Write-Fix "  • $file already correct" "SUCCESS"
        }
    }
}

# ============================================================================
# FIX 2: astraweave-asset missing imports (tracing, VecDeque)
# ============================================================================
Write-Fix "Fixing astraweave-asset missing imports..." "INFO"

$assetLib = "astraweave-asset\src\lib.rs"
if (Test-Path $assetLib) {
    $content = Get-Content $assetLib -Raw
    
    $needsVecDeque = $content -match 'VecDeque' -and $content -notmatch 'use std::collections::VecDeque'
    $needsTracing = $content -match 'tracing::' -and $content -notmatch 'use tracing;'
    
    if ($needsVecDeque -or $needsTracing) {
        Write-Fix "  • $assetLib needs imports" "WARN"
        
        if (-not $DryRun) {
            # Read file as array of lines to insert at top
            $lines = Get-Content $assetLib
            $insertions = @()
            
            if ($needsVecDeque) {
                $insertions += "use std::collections::VecDeque;"
                Write-Fix "    Adding VecDeque import" "INFO"
            }
            
            if ($needsTracing) {
                # Check if tracing is in Cargo.toml dependencies
                $cargoToml = Get-Content "astraweave-asset\Cargo.toml" -Raw
                if ($cargoToml -match 'tracing\s*=') {
                    $insertions += "use tracing;"
                    Write-Fix "    Adding tracing import" "INFO"
                }
                else {
                    $insertions += "#[cfg(feature = `"tracing`")]"
                    $insertions += "use tracing;"
                    Write-Fix "    Adding conditional tracing import" "INFO"
                }
            }
            
            # Insert at line 1 (after any #! attributes)
            $insertLine = 0
            for ($i = 0; $i -lt $lines.Count; $i++) {
                if ($lines[$i] -notmatch '^#!\[') {
                    $insertLine = $i
                    break
                }
            }
            
            $newLines = $lines[0..($insertLine-1)] + $insertions + $lines[$insertLine..($lines.Count-1)]
            Set-Content -Path $assetLib -Value $newLines
            Write-Fix "    Fixed missing imports" "SUCCESS"
            $fixCount++
        }
        else {
            Write-Fix "    Would add missing imports (dry run)" "INFO"
        }
    }
    else {
        Write-Fix "  • $assetLib already has necessary imports" "SUCCESS"
    }
}

# ============================================================================
# FIX 3: astraweave-core duplicate criterion_group (more complex, just warn)
# ============================================================================
Write-Fix "Checking astraweave-core for duplicate criterion_group..." "INFO"

$coreFile = "astraweave-core\benches\full_game_loop.rs"
if (Test-Path $coreFile) {
    $content = Get-Content $coreFile -Raw
    $groupCount = ([regex]::Matches($content, 'criterion_group!')).Count
    $mainCount = ([regex]::Matches($content, 'criterion_main!')).Count
    
    if ($groupCount -gt 1 -or $mainCount -gt 1) {
        Write-Fix "  • $coreFile has $groupCount criterion_group! and $mainCount criterion_main! calls" "WARN"
        Write-Fix "    MANUAL FIX REQUIRED: Merge into one group or split into separate files" "ERROR"
        Write-Fix "    See BENCHMARK_DISCOVERY_REPORT.md for details" "INFO"
    }
    else {
        Write-Fix "  • $coreFile looks correct" "SUCCESS"
    }
}

# ============================================================================
# FIX 4: multi_agent_pipeline.rs IVec2::new issues
# ============================================================================
Write-Fix "Fixing astraweave-ai multi_agent_pipeline IVec2 issues..." "INFO"

$pipelineFile = "astraweave-ai\benches\multi_agent_pipeline.rs"
if (Test-Path $pipelineFile) {
    $content = Get-Content $pipelineFile -Raw
    
    if ($content -match 'IVec2::new') {
        Write-Fix "  • $pipelineFile uses IVec2::new (should use glam::ivec2 or IVec2 { x, y })" "WARN"
        
        if (-not $DryRun) {
            # Replace IVec2::new(x, y) with glam::ivec2(x, y)
            $newContent = $content -replace 'IVec2::new\(', 'glam::ivec2('
            
            # Also ensure glam is imported
            if ($newContent -notmatch 'use glam::') {
                $lines = $newContent -split "`n"
                # Find first non-comment, non-use line
                for ($i = 0; $i -lt $lines.Count; $i++) {
                    if ($lines[$i] -match '^use ' -and $lines[$i+1] -notmatch '^use ') {
                        $lines = $lines[0..$i] + "use glam::ivec2;" + $lines[($i+1)..($lines.Count-1)]
                        break
                    }
                }
                $newContent = $lines -join "`n"
            }
            
            Set-Content -Path $pipelineFile -Value $newContent -NoNewline
            Write-Fix "    Fixed IVec2::new → glam::ivec2" "SUCCESS"
            $fixCount++
        }
        else {
            Write-Fix "    Would fix IVec2::new calls (dry run)" "INFO"
        }
    }
    else {
        Write-Fix "  • $pipelineFile IVec2 usage looks correct" "SUCCESS"
    }
}

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "   Fix Summary" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host ""

if ($DryRun) {
    Write-Fix "Dry run complete. Would have applied $fixCount fixes." "INFO"
    Write-Host ""
    Write-Host "Run without -DryRun to apply fixes:" -ForegroundColor Yellow
    Write-Host "  .\scripts\fix_benchmark_errors.ps1" -ForegroundColor Cyan
}
else {
    Write-Fix "Applied $fixCount automatic fixes" "SUCCESS"
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Manual fix for astraweave-core duplicate criterion_group (if needed)" -ForegroundColor Cyan
    Write-Host "  2. Re-run benchmarks: .\scripts\run_all_working_benchmarks.ps1" -ForegroundColor Cyan
    Write-Host "  3. Validate: .\scripts\validate_dashboard.ps1" -ForegroundColor Cyan
}

Write-Host ""
