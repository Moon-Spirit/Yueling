@echo off
setlocal
chcp 65001 > nul

echo 开始构建流程...

REM 构建后端服务器
echo 1. 构建后端服务器...
cd /d "%~dp0server"
call cargo build || (echo 后端构建失败 & exit /b 1)

REM 构建前端
echo.
echo 2. 构建前端...
cd /d "%~dp0Yueling"
call npm run build || (echo 前端构建失败 & exit /b 1)

REM 构建 Tauri 应用
echo.
echo 3. 构建 Tauri 应用...
call cargo tauri build || (echo Tauri 构建失败 & exit /b 1)

echo.
echo 全部构建完成!
