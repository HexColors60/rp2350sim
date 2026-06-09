@echo off
REM Build RP2350 Simulator with WinAPI GUI
REM Uses native Windows API with wgpu rendering

echo Building RP2350 Simulator with WinAPI GUI...
echo.

cd /d "%~dp0"

cargo build --release -p rp2350sim-app --no-default-features --features gui-winapi

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo Binary: target\release\rp2350sim.exe
) else (
    echo.
    echo Build failed!
    exit /b 1
)