#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Audits .unwrap() usage across the AstraWeave codebase.

.DESCRIPTION
    Scans all Rust files for .unwrap() calls, categorizes them by risk level,
    and generates a detailed CSV report for review and remediation.

.PARAMETER OutputPath
    Path to output CSV file. Default: unwrap_audit_report.csv

.PARAMETER ExcludeDirs
    Directories to exclude from scan. Default: target, .git

.EXAMPLE
    .\scripts\audit_unwrap.ps1
    .\scripts\audit_unwrap.ps1 -OutputPath custom_report.csv
#>

param(
    [string]$OutputPath = "unwrap_audit_report.csv",
    [string[]]$ExcludeDirs = @("target", ".git", "node_modules")
)

# Color output helpers
function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

# Risk categorization based on context
function Get-RiskLevel {
    param([string]$Line, [string]$FilePath)
    
    # 🔴 P0 - Critical: Production code unwraps without context
    if ($Line -match '\.unwrap\(\)\s*;?\s*$' -and $FilePath -notmatch 'test|example|demo') {
        return "P0-Critical"
    }
    
    # 🟠 P1 - High: Unwraps in core engine systems
    if ($FilePath -match 'astraweave-(ecs|physics|render|ai|audio|nav)/src/' -and $FilePath -notmatch 'test') {
        return "P1-High"
    }
    
    # 🟡 P2 - Medium: Unwraps with error messages or in gameplay
    if ($Line -match '\.expect\(' -or $FilePath -match 'astraweave-gameplay') {
        return "P2-Medium"
    }
    
    # 🟢 P3 - Low: Test code or examples
    if ($FilePath -match 'test|example|demo|bench') {
        return "P3-Low"
    }
    
    # 🟠 P1 - Default to high for safety
    return "P1-High"
}

# Extract context around unwrap
function Get-UnwrapContext {
    param([string]$Line)
    
    # Try to extract the expression being unwrapped
    if ($Line -match '([a-zA-Z_][a-zA-Z0-9_:]*(?:\([^)]*\))?(?:\.[a-zA-Z_][a-zA-Z0-9_]*(?:\([^)]*\))?)*)\.unwrap\(\)') {
        return $Matches[1]
    }
    
    return $Line.Trim()
}

# Main audit logic
function Start-UnwrapAudit {
    Write-ColorOutput "`n🔍 AstraWeave .unwrap() Audit - Starting...`n" "Cyan"
    
    # Find all Rust files
    $rustFiles = Get-ChildItem -Path . -Filter "*.rs" -Recurse | Where-Object {
        $fullPath = $_.FullName
        -not ($ExcludeDirs | Where-Object { $fullPath -match [regex]::Escape($_) })
    }
    
    Write-ColorOutput "📁 Found $($rustFiles.Count) Rust files to scan`n" "Yellow"
    
    $results = @()
    $totalUnwraps = 0
    $fileCount = 0
    
    foreach ($file in $rustFiles) {
        $fileCount++
        $relativePath = $file.FullName.Replace((Get-Location).Path, "").TrimStart('\', '/')
        
        # Progress indicator
        if ($fileCount % 50 -eq 0) {
            Write-Host "." -NoNewline
        }
        
        $lineNumber = 0
        Get-Content $file.FullName | ForEach-Object {
            $lineNumber++
            $line = $_
            
            # Match .unwrap() calls (excluding commented lines)
            if ($line -match '\.unwrap\(\)' -and $line -notmatch '^\s*//') {
                $totalUnwraps++
                
                # Extract crate name from path
                $crate = "unknown"
                if ($relativePath -match '^(astraweave-[^/\\]+|examples/[^/\\]+|tools/[^/\\]+)') {
                    $crate = $Matches[1]
                }
                
                $riskLevel = Get-RiskLevel -Line $line -FilePath $relativePath
                $context = Get-UnwrapContext -Line $line
                
                $results += [PSCustomObject]@{
                    File = $relativePath
                    Line = $lineNumber
                    Crate = $crate
                    RiskLevel = $riskLevel
                    Context = $context
                    Code = $line.Trim()
                }
            }
        }
    }
    
    Write-Host "`n"
    
    # Sort by risk level then by file
    $results = $results | Sort-Object @{Expression={
        switch ($_.RiskLevel) {
            "P0-Critical" { 0 }
            "P1-High" { 1 }
            "P2-Medium" { 2 }
            "P3-Low" { 3 }
            default { 4 }
        }
    }}, File
    
    # Export to CSV
    $results | Export-Csv -Path $OutputPath -NoTypeInformation -Encoding UTF8
    
    # Generate summary statistics
    $byRisk = $results | Group-Object RiskLevel
    $byCrate = $results | Group-Object Crate | Sort-Object Count -Descending
    
    Write-Host ("=" * 70) -ForegroundColor Cyan
    Write-Host "  AUDIT SUMMARY" -ForegroundColor Cyan
    Write-Host ("=" * 70) -ForegroundColor Cyan
    
    Write-ColorOutput "`n📊 Total .unwrap() calls found: $totalUnwraps`n" "White"
    
    Write-ColorOutput "🎯 By Risk Level:" "Yellow"
    foreach ($group in $byRisk) {
        $color = switch ($group.Name) {
            "P0-Critical" { "Red" }
            "P1-High" { "DarkYellow" }
            "P2-Medium" { "Yellow" }
            "P3-Low" { "Green" }
            default { "Gray" }
        }
        Write-ColorOutput "   $($group.Name.PadRight(15)) : $($group.Count)" $color
    }
    
    Write-ColorOutput "`n📦 Top 10 Crates:" "Yellow"
    $byCrate | Select-Object -First 10 | ForEach-Object {
        Write-ColorOutput "   $($_.Name.PadRight(35)) : $($_.Count)" "White"
    }
    
    Write-ColorOutput "`n📄 Report saved to: $OutputPath" "Green"
    
    # Show critical cases
    $critical = $results | Where-Object { $_.RiskLevel -eq "P0-Critical" }
    if ($critical.Count -gt 0) {
        Write-ColorOutput "`n🔴 CRITICAL UNWRAPS (P0) - Immediate attention required:`n" "Red"
        $critical | Select-Object -First 10 | ForEach-Object {
            Write-ColorOutput "   $($_.File):$($_.Line)" "Red"
            Write-ColorOutput "      $($_.Context)" "Gray"
        }
        
        if ($critical.Count -gt 10) {
            Write-ColorOutput "`n   ... and $($critical.Count - 10) more (see CSV report)" "DarkGray"
        }
    }
    
    Write-ColorOutput "`n" "White"
    Write-Host ("=" * 70) -ForegroundColor Cyan
    Write-ColorOutput "✅ Audit complete!" "Green"
    Write-Host ("=" * 70) -ForegroundColor Cyan
    Write-ColorOutput "`n💡 Next steps:" "Yellow"
    Write-ColorOutput "   1. Review CSV report: $OutputPath" "White"
    Write-ColorOutput "   2. Create GitHub issues for P0/P1 cases" "White"
    Write-ColorOutput "   3. Prioritize fixes based on risk and usage frequency" "White"
    Write-ColorOutput "`n"
}

# Run the audit
try {
    Start-UnwrapAudit
    exit 0
} catch {
    Write-ColorOutput "`n❌ Error during audit: $_" "Red"
    exit 1
}
