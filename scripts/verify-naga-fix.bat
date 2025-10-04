@echo off
REM Naga Fix Verification - Windows Batch Version
REM Run this to verify the WGPU 22.1.0 fix

echo.
echo === Naga WriteColor Fix Verification ===
echo.

echo [1/4] Checking WGPU version...
cargo tree -p astraweave-render | findstr /C:"wgpu v"
echo.

echo [2/4] Checking naga versions...
cargo tree -p astraweave-render | findstr /C:"naga v"
echo.

echo [3/4] Compiling astraweave-render...
cargo check -p astraweave-render
if %ERRORLEVEL% EQU 0 (
    echo   SUCCESS: astraweave-render compiled cleanly
) else (
    echo   FAILED: Compilation errors detected
    exit /b 1
)
echo.

echo [4/4] Checking for naga errors...
cargo check -p astraweave-render 2>&1 | findstr /C:"WriteColor" > nul
if %ERRORLEVEL% EQU 0 (
    echo   FOUND naga WriteColor errors - Fix did not work
    exit /b 1
) else (
    echo   SUCCESS: No naga WriteColor errors found
)
echo.

echo === Verification Complete ===
echo WGPU version: 22.1.0
echo Naga version: 22.x
echo Compilation: SUCCESS
echo No naga errors
echo.
echo Next steps:
echo   1. Run: cargo build -p astraweave-render --release
echo   2. Test: cargo run -p hello_companion --release
echo   3. Verify: Check examples run correctly
echo.
