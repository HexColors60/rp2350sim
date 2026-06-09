@echo off
REM RP2350 Simulator - GUI Launcher with System Fonts
REM This script runs the simulator with GUI and system font loading

cd /d "%~dp0"

echo ============================================
echo RP2350 Simulator - GUI with System Fonts
echo ============================================
echo.

echo [1/2] Building RP2350 Simulator (release)...
cargo build --release -q -p rp2350sim-app

if %ERRORLEVEL% neq 0 (
    echo.
    echo Build failed!
    pause
    exit /b 1
)

echo [2/2] Starting GUI...
echo.
echo Font loading: System fonts from C:\Windows\Fonts
echo   - Monospace: Consolas, Courier New, Lucida Console
echo   - UI: Segoe UI, Arial, Tahoma, Calibri
echo   - Fallback: CJK, Emoji, Symbols
echo.
echo Press Ctrl+C to exit
echo ============================================
echo.

REM Run the simulator with GUI
cargo run --release -q -p rp2350sim-app --bin rp2350sim -- %*