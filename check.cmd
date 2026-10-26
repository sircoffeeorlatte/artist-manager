@echo off
setlocal
cd /d "%~dp0"

if not exist node_modules\ (
  echo npm packages not found. Run install.cmd first.
  pause
  exit /b 1
)
echo Checking TypeScript...
call npm run typecheck
if errorlevel 1 goto failed
echo.
echo Building the frontend...
call npm run build
if errorlevel 1 goto failed
echo.
echo Checks completed successfully.
pause
exit /b 0

:failed
echo.
echo A check failed. Review the messages above.
pause
exit /b 1
