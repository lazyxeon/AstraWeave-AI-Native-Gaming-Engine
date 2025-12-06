off
cargo run -p unified_showcase > test_output.txt 2>&1  
timeout /t 10 /nobreak >nul  
taskkill /F /IM unified_showcase.exe >nul 2>&1  
type test_output.txt 
