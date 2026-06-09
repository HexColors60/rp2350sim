@echo off
REM Run RP2350 Simulator with Macroquad GUI (default)

echo Running RP2350 Simulator with Macroquad GUI...
echo.

cd /d "%~dp0"

cargo run --release -p rp2350sim-app --features gui-macroquad --bin rp2350sim