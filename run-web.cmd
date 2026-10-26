@echo off
setlocal
cd /d "%~dp0"

echo Starting browser preview at http://127.0.0.1:1420
echo Press Ctrl+C in this window to stop the server.
echo.
if not exist node_modules\ (
  echo npm packages not found. Running npm install...
  call npm install
  if errorlevel 1 (
    echo [ERROR] Dependency installation failed.
    pause
    exit /b 1
  )
)
call npm run dev:open
pause
