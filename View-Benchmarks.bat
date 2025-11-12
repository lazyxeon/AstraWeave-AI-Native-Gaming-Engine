@echo off
REM Quick Benchmark Dashboard Launcher
REM Double-click this file to view benchmarks!

echo.
echo ╔════════════════════════════════════════════════════╗
echo ║   AstraWeave Benchmark Dashboard                  ║
echo ╚════════════════════════════════════════════════════╝
echo.

pwsh -File scripts\run_benchmarks_and_dashboard.ps1 -SkipBench

pause
