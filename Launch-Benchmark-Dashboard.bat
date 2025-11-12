@echo off
REM AstraWeave Benchmark Dashboard - Quick Launch
REM Double-click this file to run benchmarks and open dashboard

echo.
echo ========================================
echo  AstraWeave Benchmark Dashboard
echo ========================================
echo.

cd /d "%~dp0.."

powershell.exe -ExecutionPolicy Bypass -File "scripts\run_benchmark_dashboard.ps1" -SkipBench

pause
