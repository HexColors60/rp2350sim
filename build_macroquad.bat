@echo off
REM Build RP2350 Simulator with Macroquad GUI (default)
REM This is the original macroquad + egui backend

echo Building RP2350 Simulator with Macroquad GUI...
echo.

cd /d "%~dp0"

cargo build --release -p rp2350sim-app --features gui-macroquad

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo Binary: target\release\rp2350sim.exe
) else (
    echo.
    echo Build failed!
    exit /b 1
)