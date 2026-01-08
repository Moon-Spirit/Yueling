@echo off
REM 构建后端服务器
cd /d "%~dp0server" && cargo build
if %errorlevel% neq 0 (
    echo 后端构建失败
    exit /b %errorlevel%
)

REM 构建前端
cd /d "%~dp0Yueling" && npm run build
if %errorlevel% neq 0 (
    echo 前端构建失败
    exit /b %errorlevel%
)

REM 构建 Tauri 应用
cd /d "%~dp0Yueling" && cargo tauri build
if %errorlevel% neq 0 (
    echo Tauri 构建失败
    exit /b %errorlevel%
)

echo 全部构建完成
