#!/usr/bin/env pwsh
# AstraWeave Validation Runner
# Runs comprehensive validation locally for development and pre-commit checks
#
# Usage:
#   ./scripts/validate.ps1              # Run tier-1 (fast) checks
#   ./scripts/validate.ps1 -Tier 2      # Run tier-2 (nightly) checks
#   ./scripts/validate.ps1 -Tier 3      # Run tier-3 (weekly) checks
#   ./scripts/validate.ps1 -Crate ecs   # Target specific crate
#   ./scripts/validate.ps1 -Sanitizer asan  # Run specific sanitizer

param(
    [ValidateSet(1, 2, 3)]
    [int]$Tier = 1,
    
    [string]$Crate = "",
    
    [ValidateSet("", "asan", "lsan", "tsan", "miri")]
    [string]$Sanitizer = "",
    
    [switch]$Quick,
    [switch]$Verbose,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host "⚠️  $msg" -ForegroundColor Yellow }
function Write-Failure { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Cyan }
function Write-Header { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Magenta }

if ($Help) {
    Write-Host @"
AstraWeave Validation Runner

Usage:
    ./scripts/validate.ps1 [options]

Options:
    -Tier <1|2|3>       Validation tier (default: 1)
                        1 = Fast PR checks (~5 min)
                        2 = Nightly checks (~30 min)
                        3 = Weekly full suite (~2 hours)
    
    -Crate <name>       Target specific crate (e.g., ecs, physics)
    
    -Sanitizer <type>   Run specific sanitizer only
                        asan = AddressSanitizer
                        lsan = LeakSanitizer
                        tsan = ThreadSanitizer
                        miri = Miri undefined behavior check
    
    -Quick              Skip non-essential checks
    -Verbose            Show detailed output
    -Help               Show this help message

Examples:
    ./scripts/validate.ps1                      # Fast PR checks
    ./scripts/validate.ps1 -Tier 2              # Nightly checks
    ./scripts/validate.ps1 -Sanitizer asan      # ASan only
    ./scripts/validate.ps1 -Crate astraweave-ecs  # ECS crate only

Tier Breakdown:
    Tier 1 (Every PR):
        - cargo check
        - cargo clippy
        - cargo test (via nextest if available)
        - cargo audit
        - wgpu validation (automatic in debug)
    
    Tier 2 (Nightly):
        - All Tier 1 checks
        - AddressSanitizer on critical crates
        - LeakSanitizer
        - Miri on pure-Rust crates
    
    Tier 3 (Weekly):
        - All Tier 2 checks
        - ThreadSanitizer
        - cargo-mutants
        - Extended fuzz runs

"@
    exit 0
}

# Track results
$script:Results = @{
    Passed = @()
    Failed = @()
    Skipped = @()
}

function Run-Check {
    param(
        [string]$Name,
        [scriptblock]$Command,
        [switch]$Required,
        [switch]$ContinueOnError
    )
    
    Write-Host "`n--- $Name ---" -ForegroundColor White
    
    try {
        & $Command
        if ($LASTEXITCODE -eq 0 -or $null -eq $LASTEXITCODE) {
            Write-Success "$Name passed"
            $script:Results.Passed += $Name
            return $true
        } else {
            if ($Required -and -not $ContinueOnError) {
                Write-Failure "$Name failed (exit code: $LASTEXITCODE)"
                $script:Results.Failed += $Name
                return $false
            } else {
                Write-Warning "$Name had issues (exit code: $LASTEXITCODE)"
                $script:Results.Failed += $Name
                return $false
            }
        }
    } catch {
        Write-Failure "$Name threw exception: $_"
        $script:Results.Failed += $Name
        if ($Required -and -not $ContinueOnError) {
            throw
        }
        return $false
    }
}

# Check for required tools
Write-Header "Checking Prerequisites"

$tools = @{
    "cargo" = { cargo --version }
    "rustc" = { rustc --version }
}

$optionalTools = @{
    "cargo-nextest" = { cargo nextest --version }
    "cargo-audit" = { cargo audit --version }
    "miri" = { cargo +nightly miri --version }
}

foreach ($tool in $tools.Keys) {
    try {
        $null = & $tools[$tool] 2>&1
        Write-Success "$tool available"
    } catch {
        Write-Failure "$tool not found - this is required!"
        exit 1
    }
}

foreach ($tool in $optionalTools.Keys) {
    try {
        $null = & $optionalTools[$tool] 2>&1
        Write-Success "$tool available"
    } catch {
        Write-Warning "$tool not found - some checks will be skipped"
    }
}

# Tier 1: Fast PR Checks
Write-Header "Tier 1: Fast Checks"

$cratesToTest = if ($Crate) { "-p $Crate" } else { "--workspace" }

# Basic compilation check
Run-Check -Name "cargo check" -Required -Command {
    if ($Crate) {
        cargo check -p $Crate
    } else {
        cargo check --workspace --exclude astraweave-llm --exclude llm_toolcall
    }
}

# Clippy lints
if (-not $Quick) {
    Run-Check -Name "cargo clippy" -Command {
        if ($Crate) {
            cargo clippy -p $Crate -- -D warnings
        } else {
            cargo clippy --workspace --exclude astraweave-llm --exclude llm_toolcall -- -D warnings
        }
    } -ContinueOnError
}

# Run tests
Run-Check -Name "cargo test" -Required -Command {
    # Use nextest if available
    $hasNextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue
    if ($hasNextest) {
        if ($Crate) {
            cargo nextest run -p $Crate
        } else {
            cargo nextest run --workspace --exclude astraweave-llm --exclude llm_toolcall
        }
    } else {
        if ($Crate) {
            cargo test -p $Crate
        } else {
            cargo test --workspace --exclude astraweave-llm --exclude llm_toolcall
        }
    }
}

# Security audit
if (-not $Quick) {
    $hasAudit = Get-Command cargo-audit -ErrorAction SilentlyContinue
    if ($hasAudit) {
        Run-Check -Name "cargo audit" -Command {
            cargo audit
        } -ContinueOnError
    } else {
        Write-Warning "Skipping cargo-audit (not installed)"
        $script:Results.Skipped += "cargo audit"
    }
}

if ($Tier -lt 2 -and -not $Sanitizer) {
    Write-Header "Tier 1 Complete"
    goto Summary
}

# Tier 2: Nightly Checks
Write-Header "Tier 2: Extended Checks"

# Determine if we're on Linux (sanitizers only work on Linux/macOS)
$isLinux = $IsLinux -or ($env:OS -ne "Windows_NT" -and $IsMacOS -eq $false)
$isMacOS = $IsMacOS
$canRunSanitizers = $isLinux -or $isMacOS

if (-not $canRunSanitizers) {
    Write-Warning "Sanitizers only work on Linux/macOS - skipping on Windows"
    Write-Info "Consider using WSL or CI for sanitizer checks"
}

# AddressSanitizer
if (($canRunSanitizers -and ($Sanitizer -eq "" -or $Sanitizer -eq "asan")) -or ($Sanitizer -eq "asan")) {
    $asanCrates = if ($Crate) { @($Crate) } else { @("astraweave-ecs", "astraweave-physics", "astraweave-core") }
    $target = if ($isMacOS) { "x86_64-apple-darwin" } else { "x86_64-unknown-linux-gnu" }
    
    foreach ($crate in $asanCrates) {
        Run-Check -Name "ASan: $crate" -Command {
            $env:RUSTFLAGS = "-Zsanitizer=address"
            $env:ASAN_OPTIONS = "detect_leaks=1"
            cargo +nightly test -p $crate --target $target -- --test-threads=1
        } -ContinueOnError
    }
}

# LeakSanitizer
if (($canRunSanitizers -and ($Sanitizer -eq "" -or $Sanitizer -eq "lsan")) -or ($Sanitizer -eq "lsan")) {
    $lsanCrates = if ($Crate) { @($Crate) } else { @("aw-save", "astraweave-ecs") }
    $target = if ($isMacOS) { "x86_64-apple-darwin" } else { "x86_64-unknown-linux-gnu" }
    
    foreach ($crate in $lsanCrates) {
        Run-Check -Name "LSan: $crate" -Command {
            $env:RUSTFLAGS = "-Zsanitizer=leak"
            cargo +nightly test -p $crate --target $target -- --test-threads=1
        } -ContinueOnError
    }
}

# Miri
if ($Sanitizer -eq "" -or $Sanitizer -eq "miri") {
    $hasMiri = Get-Command "cargo +nightly miri" -ErrorAction SilentlyContinue
    
    if (-not $hasMiri) {
        # Try to check if miri component is installed
        try {
            $null = cargo +nightly miri --version 2>&1
            $hasMiri = $true
        } catch {
            $hasMiri = $false
        }
    }
    
    if ($hasMiri) {
        $miriCrates = if ($Crate) { @($Crate) } else { @("astraweave-ecs", "astraweave-math", "astraweave-core") }
        
        foreach ($crate in $miriCrates) {
            Run-Check -Name "Miri: $crate" -Command {
                $env:MIRIFLAGS = "-Zmiri-disable-isolation"
                cargo +nightly miri test -p $crate -- --skip ffi --skip integration
            } -ContinueOnError
        }
    } else {
        Write-Warning "Miri not installed - run: rustup +nightly component add miri"
        $script:Results.Skipped += "Miri"
    }
}

if ($Tier -lt 3 -and -not $Sanitizer) {
    Write-Header "Tier 2 Complete"
    goto Summary
}

# Tier 3: Weekly Checks
Write-Header "Tier 3: Full Suite"

# ThreadSanitizer
if (($canRunSanitizers -and ($Sanitizer -eq "" -or $Sanitizer -eq "tsan")) -or ($Sanitizer -eq "tsan")) {
    $tsanCrates = if ($Crate) { @($Crate) } else { @("astraweave-ai", "astraweave-coordination") }
    $target = if ($isMacOS) { "x86_64-apple-darwin" } else { "x86_64-unknown-linux-gnu" }
    
    foreach ($crate in $tsanCrates) {
        Run-Check -Name "TSan: $crate" -Command {
            $env:RUSTFLAGS = "-Zsanitizer=thread"
            cargo +nightly test -p $crate --target $target -- --test-threads=1
        } -ContinueOnError
    }
}

# Mutation testing (very slow)
$hasMutants = Get-Command cargo-mutants -ErrorAction SilentlyContinue
if ($hasMutants) {
    $mutantCrates = if ($Crate) { @($Crate) } else { @("astraweave-ecs") }
    
    foreach ($crate in $mutantCrates) {
        Run-Check -Name "Mutation: $crate" -Command {
            cargo mutants -p $crate --timeout 300
        } -ContinueOnError
    }
} else {
    Write-Warning "cargo-mutants not installed - run: cargo install cargo-mutants"
    $script:Results.Skipped += "Mutation testing"
}

:Summary
# Summary
Write-Header "Validation Summary"

Write-Host "`nPassed: $($script:Results.Passed.Count)" -ForegroundColor Green
foreach ($check in $script:Results.Passed) {
    Write-Host "  ✅ $check" -ForegroundColor Green
}

if ($script:Results.Failed.Count -gt 0) {
    Write-Host "`nFailed: $($script:Results.Failed.Count)" -ForegroundColor Red
    foreach ($check in $script:Results.Failed) {
        Write-Host "  ❌ $check" -ForegroundColor Red
    }
}

if ($script:Results.Skipped.Count -gt 0) {
    Write-Host "`nSkipped: $($script:Results.Skipped.Count)" -ForegroundColor Yellow
    foreach ($check in $script:Results.Skipped) {
        Write-Host "  ⚠️  $check" -ForegroundColor Yellow
    }
}

$totalChecks = $script:Results.Passed.Count + $script:Results.Failed.Count
$passRate = if ($totalChecks -gt 0) { [math]::Round(($script:Results.Passed.Count / $totalChecks) * 100, 1) } else { 0 }

Write-Host "`nOverall: $passRate% pass rate ($($script:Results.Passed.Count)/$totalChecks checks)" -ForegroundColor $(if ($passRate -ge 90) { "Green" } elseif ($passRate -ge 70) { "Yellow" } else { "Red" })

if ($script:Results.Failed.Count -gt 0) {
    exit 1
}
