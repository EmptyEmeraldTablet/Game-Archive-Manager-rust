@echo off
chcp 65001 >nul
echo ============================================
echo Game Archive Manager - 构建脚本
echo ============================================
echo.

REM 检查 Rust 是否已安装
rustc --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未检测到 Rust，请先安装 Rust:
    echo       https://rustup.rs/
    echo.
    pause
    exit /b 1
)

echo [信息] 清理旧的构建文件...
cargo clean

echo.
echo [信息] 开始 Release 构建...
echo        (这可能需要几分钟时间...)
echo.

cargo build --release

if errorlevel 1 (
    echo.
    echo [错误] 构建失败！
    pause
    exit /b 1
)

echo.
echo ============================================
echo 构建成功！
echo ============================================
echo.
echo 可执行文件位置:
echo   %~dp0target\release\game-archive-manager.exe
echo.
echo 使用方法:
echo  1. 将 game-archive-manager.exe 放到一个文件夹中
echo  2. 在同一文件夹创建 path.txt，写入游戏存档目录的绝对路径
echo  3. 运行程序，输入 help 查看帮助
echo.
pause
