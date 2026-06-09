@echo off
REM RP2350 Simulator - Headless Launcher
REM This script runs the simulator in headless mode (no GUI)

cd /d "%~dp0"

echo Building RP2350 Simulator (headless)...
cargo build --release -q -p rp2350sim-app

if %ERRORLEVEL% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Starting RP2350 Simulator in headless mode...
echo.

REM Run the simulator in headless mode
cargo run --release -q -p rp2350sim-app --bin rp2350sim -- --headless %*