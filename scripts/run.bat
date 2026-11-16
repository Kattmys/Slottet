@echo off
set "DOMAIN=%~1"
if "%DOMAIN%"=="" set "DOMAIN=cicada.kattmys.se:78987"
set "IP="
for /f "tokens=2 delims=[]" %%i in ('ping -n 1 -4 %DOMAIN% ^| findstr /i "Pinging"') do set "IP=%%i"
if not defined IP (echo Could not resolve %DOMAIN% & exit /b 1)
start "" "%~dp0client.exe" %IP%

