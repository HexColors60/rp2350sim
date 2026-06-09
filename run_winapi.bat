@echo off
REM Run RP2350 Simulator with WinAPI GUI

echo Running RP2350 Simulator with WinAPI GUI...
echo.

cd /d "%~dp0"

cargo run --release -p rp2350sim-app --no-default-features --features gui-winapi --bin rp2350sim