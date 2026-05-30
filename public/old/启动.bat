@echo off
title DevSetup Launcher v3 - Full Concurrent

:: ============================================
:: DevSetup Auto Installer Launcher v3
:: Run AutoSetup_EN_v3_Concurrent.ps1 with Admin Rights
:: Full Concurrent Installation Pipeline
:: ============================================

echo.
echo ================================================
echo     Windows DevSetup Auto Installer
echo           v3 - Full Concurrent Mode
echo ================================================
echo.
echo Revolutionary Features:
echo   - Full Concurrent Installation Pipeline
echo   - Each Component: Download -^> Install -^> Configure (Parallel)
echo   - Real-time Progress Monitoring
echo   - Independent Component Installation
echo   - Version Selection: JDK (8/17), Node.js (14/16/18)
echo   - Environment Detection with Smart Comparison
echo   - Multiple Mirror Sources with Auto Failover
echo.
echo Performance:
echo   - 4 Components Install Time: 3-5 minutes (60%% faster)
echo   - Fully Utilizes Multi-core CPU and Network
echo.
echo Components Available:
echo   - JDK: 8u392 or 17.0.10 (LTS)
echo   - Node.js: 14.21.3, 16.20.2, or 18.19.0 (LTS)
echo   - MySQL: 8.0.24 (password: 123456)
echo   - Maven: 3.9.6 (Aliyun mirror)
echo.
echo Install Directory: D:\DevSetup
echo.
echo ------------------------------------------------
echo.

:: Check if AutoSetup_EN_v3_Concurrent.ps1 exists
if not exist "%~dp0AutoSetup_EN_v3_Concurrent.ps1" (
    echo [ERROR] AutoSetup_EN_v3_Concurrent.ps1 not found!
    echo Please ensure AutoSetup_EN_v3_Concurrent.ps1 is in the same directory.
    echo.
    pause
    exit /b 1
)

echo [INFO] v3 Full Concurrent Installation Mode detected!
echo [INFO] All components will install in parallel pipelines
echo [INFO] Requesting administrator privileges...
echo [INFO] Please click "Yes" in the UAC dialog
echo.

:: Get the full path to the script
set "SCRIPT_PATH=%~dp0AutoSetup_EN_v3_Concurrent.ps1"
echo [DEBUG] Script path: %SCRIPT_PATH%
echo.
pause

:: Launch PowerShell with admin rights
PowerShell -Command "& {Start-Process PowerShell -ArgumentList '-NoExit', '-ExecutionPolicy', 'Bypass', '-File', '%SCRIPT_PATH%' -Verb RunAs}"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] PowerShell failed to start. Error code: %ERRORLEVEL%
    echo Possible reasons:
    echo   1. User denied administrator permission
    echo   2. PowerShell not installed correctly
    echo.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo [SUCCESS] v3 Full Concurrent PowerShell script launched
echo All components installing in parallel pipelines
echo Features: Download -^> Install -^> Configure (All Concurrent)
echo Please check the new PowerShell window for real-time progress
echo.
timeout /t 3 /nobreak >nul
exit /b 0