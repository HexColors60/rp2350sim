@echo off
REM Build all RP2350 Simulator GUI backends

echo ============================================
echo Building all RP2350 Simulator GUI backends
echo ============================================
echo.

cd /d "%~dp0"

echo [1/3] Building Macroquad GUI (default)...
echo ----------------------------------------
cargo build --release -p rp2350sim-app --features gui-macroquad
if %ERRORLEVEL% NEQ 0 (
    echo Macroquad build failed!
    exit /b 1
)
echo.

echo [2/3] Building Bevy GUI...
echo ----------------------------------------
cargo build --release -p rp2350sim-app --no-default-features --features gui-bevy
if %ERRORLEVEL% NEQ 0 (
    echo Bevy build failed!
    exit /b 1
)
echo.

echo [3/3] Building WinAPI GUI...
echo ----------------------------------------
cargo build --release -p rp2350sim-app --no-default-features --features gui-winapi
if %ERRORLEVEL% NEQ 0 (
    echo WinAPI build failed!
    exit /b 1
)
echo.

echo ============================================
echo All builds successful!
echo ============================================
echo.
echo Binaries:
echo   - target\release\rp2350sim.exe (last built)
echo.
echo To run specific backends:
echo   - run_macroquad.bat  (Macroquad + egui)
echo   - run_bevy.bat       (Bevy + egui)
echo   - run_winapi.bat     (WinAPI + wgpu + egui)