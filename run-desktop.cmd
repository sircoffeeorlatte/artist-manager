@echo off
setlocal
cd /d "%~dp0"

echo Starting Tauri desktop app...
where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Rust/Cargo not found.
  echo Open README.md and install the Tauri Windows prerequisites.
  pause
  exit /b 1
)
if not exist node_modules\ (
  echo npm packages not found. Running npm install...
  call npm install
  if errorlevel 1 (
    echo [ERROR] npm dependency installation failed.
    pause
    exit /b 1
  )
)
call npm run tauri:dev
if errorlevel 1 (
  echo.
  echo [ERROR] Tauri did not start. Check Rust, MSVC Build Tools, and WebView2.
  pause
  exit /b 1
)
