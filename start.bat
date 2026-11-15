@echo off
setlocal
cd /d "%~dp0"

rem Honour REPIKACHAT_PORT so the URL and the port check match the server.
set "PORT=%REPIKACHAT_PORT%"
if "%PORT%"=="" set "PORT=7719"
set "URL=http://127.0.0.1:%PORT%/"
set "EXE=target\release\RePikaChat.exe"

echo.
echo   RePikaChat
echo   ----------
echo   URL:   %URL%
echo   Save:  %%LOCALAPPDATA%%\RePikaChat\repikachat.json
echo.

rem Already running? Just open it instead of failing on a busy port.
netstat -ano | findstr /R /C:":%PORT% .*LISTENING" >nul
if not errorlevel 1 (
  echo   Already running - opening %URL%
  start "" "%URL%"
  echo   Close this window whenever you like; the server keeps running.
  echo.
  pause
  exit /b 0
)

if not exist "%EXE%" (
  echo   No release build yet - building one now, this can take a minute...
  echo.
  set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
  cargo build --release
  if errorlevel 1 goto buildfailed
  echo.
)

echo   Starting the server on port %PORT%. Leave this window open while you chat;
echo   press Ctrl+C to stop it.
echo.

"%EXE%"
set "CODE=%ERRORLEVEL%"

echo.
if not "%CODE%"=="0" (
  echo   The server exited with code %CODE%.
) else (
  echo   Server stopped.
)
echo.
pause
exit /b %CODE%

:buildfailed
echo.
echo   Build failed - see the messages above.
echo.
pause
exit /b 1
