@echo off
setlocal
cd /d "%~dp0"

echo ============================================================
echo Media Library MVP - dependency setup
echo ============================================================
echo.

where node >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Node.js was not found.
  echo Install the current LTS version from https://nodejs.org/
  echo Then run install.cmd again.
  pause
  exit /b 1
)

where npm >nul 2>nul
if errorlevel 1 (
  echo [ERROR] npm was not found. Reinstall Node.js with npm.
  pause
  exit /b 1
)

echo [1/2] Installing npm packages...
call npm install
if errorlevel 1 (
  echo.
  echo [ERROR] npm install failed.
  echo Check your internet connection and Node.js version.
  pause
  exit /b 1
)

echo.
echo [2/2] Checking Rust for the desktop build...
where cargo >nul 2>nul
if errorlevel 1 (
  echo [WARNING] Cargo/Rust was not found. Browser preview can still run.
  echo Tauri requires Rust MSVC, Visual Studio Build Tools with C++ workload, and WebView2.
  echo Instructions: https://v2.tauri.app/start/prerequisites/
) else (
  echo Rust found. Fetching Rust dependencies...
  cargo fetch --manifest-path src-tauri\Cargo.toml
  if errorlevel 1 echo [WARNING] cargo fetch failed; run-desktop.cmd may retry later.
)

echo.
echo Done. Run run-web.cmd for browser preview.
echo Run run-desktop.cmd for the Tauri desktop window after installing system tools.
echo.
pause
exit /b 0
