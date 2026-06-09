@echo off
REM Build RP2350 Simulator with Bevy GUI
REM Uses Bevy game engine with egui integration

echo Building RP2350 Simulator with Bevy GUI...
echo.

cd /d "%~dp0"

cargo build --release -p rp2350sim-app --no-default-features --features gui-bevy

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo Binary: target\release\rp2350sim.exe
) else (
    echo.
    echo Build failed!
    exit /b 1
)