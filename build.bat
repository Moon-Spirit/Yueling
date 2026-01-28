@echo off
setlocal
chcp 65001 > nul

echo 开始构建流程...

REM 构建后端服务器
echo 1. 构建后端服务器...
cd /d "%~dp0server"
call cargo build || (echo 后端构建失败 & exit /b 1)

REM 补全前端环境
echo.
echo 2. 补全前端环境...
cd /d "%~dp0Yueling"
call npm install || (echo 前端环境补全失败 & exit /b 1)

REM 构建前端应用
echo.
echo 3. 构建前端应用...
call npm run build || (echo 前端构建失败 & exit /b 1)

REM 构建 Tauri 应用
echo.
echo 4. 构建 Tauri 应用...
call cargo tauri build || (echo Tauri 构建失败 & exit /b 1)

echo.
echo 全部构建完成!
