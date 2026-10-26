@echo off
setlocal
cd /d "%~dp0"

echo Building the Windows installer (.exe)...
where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Rust/Cargo not found. See README.md.
  pause
  exit /b 1
)
if not exist node_modules\ (
  call npm install
  if errorlevel 1 (
    echo [ERROR] npm dependency installation failed.
    pause
    exit /b 1
  )
)
call npm run tauri:build
if errorlevel 1 (
  echo.
  echo [ERROR] Build failed. Review the messages above.
  pause
  exit /b 1
)
echo.
echo Done. The installer is usually in:
echo src-tauri\target\release\bundle\nsis\
pause
