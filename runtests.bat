@echo off
REM RP2350 Simulator - Test Runner
REM This script runs all tests

cd /d "%~dp0"

echo Running RP2350 Simulator tests...
cargo test --all -q

echo.
echo Tests complete.
pause