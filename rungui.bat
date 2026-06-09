@echo off
REM RP2350 Simulator - GUI Launcher
REM This script runs the simulator with GUI enabled

cd /d "%~dp0"

echo Building RP2350 Simulator...
cargo build --release -q -p rp2350sim-app

if %ERRORLEVEL% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Starting RP2350 Simulator GUI...
echo.

REM Run the simulator with GUI (default mode, no --headless flag)
cargo run --release -q -p rp2350sim-app --bin rp2350sim -- %*