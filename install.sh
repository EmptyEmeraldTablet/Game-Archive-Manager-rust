#!/bin/bash
#
# Game Archive Manager 安装脚本
# 自动将 GAM 添加到系统 PATH
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BINARY_NAME="game-archive-manager"
INSTALL_DIR=""
NEED_SUDO=false

detect_os() {
    case "$(uname -s)" in
        Linux*)
            OS="linux"
            INSTALL_DIR="/usr/local/bin"
            ;;
        Darwin*)
            OS="macos"
            INSTALL_DIR="/usr/local/bin"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            OS="windows"
            INSTALL_DIR="/usr/local/bin"
            ;;
        *)
            echo -e "${RED}不支持的操作系统${NC}"
            exit 1
            ;;
    esac
}

check_permissions() {
    if [ ! -w "$INSTALL_DIR" ]; then
        echo -e "${YELLOW}提示: $INSTALL_DIR 需要 root 权限${NC}"
        NEED_SUDO=true
    else
        NEED_SUDO=false
    fi
}

find_binary() {
    local binary_path=""

    if [ -f "target/release/$BINARY_NAME" ]; then
        binary_path="target/release/$BINARY_NAME"
    elif [ -f "target/release/${BINARY_NAME}.exe" ]; then
        binary_path="target/release/${BINARY_NAME}.exe"
    fi

    if [ -z "$binary_path" ] && [ -f "./$BINARY_NAME" ]; then
        binary_path="./$BINARY_NAME"
    fi

    if [ -z "$binary_path" ]; then
        for path in "/usr/local/bin/$BINARY_NAME" "/usr/bin/$BINARY_NAME" "$HOME/.local/bin/$BINARY_NAME"; do
            if [ -f "$path" ]; then
                binary_path="$path"
                break
            fi
        done
    fi

    if [ -z "$binary_path" ]; then
        echo -e "${RED}错误: 找不到 $BINARY_NAME${NC}"
        echo "请先编译: cargo build --release"
        exit 1
    fi

    echo -e "${GREEN}找到: $binary_path${NC}"
    BINARY_PATH="$binary_path"
}

create_symlink() {
    local install_target="$INSTALL_DIR/$BINARY_NAME"

    if [ -L "$install_target" ]; then
        echo -e "${YELLOW}更新现有链接...${NC}"
        rm "$install_target"
    elif [ -f "$install_target" ]; then
        echo -e "${YELLOW}备份现有文件...${NC}"
        mv "$install_target" "${install_target}.backup.$(date +%s)"
    fi

    local cmd
    if [ "$NEED_SUDO" = true ]; then
        cmd="sudo ln -sf"
    else
        cmd="ln -sf"
    fi

    $cmd "$BINARY_PATH" "$install_target"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ 已安装: $install_target${NC}"
    else
        echo -e "${RED}✗ 安装失败${NC}"
        exit 1
    fi
}

verify_install() {
    echo ""
    echo "验证..."
    local gam_path
    gam_path=$(command -v "$BINARY_NAME" 2>/dev/null || echo "")

    if [ -n "$gam_path" ]; then
        echo -e "${GREEN}✓ 命令可用: $gam_path${NC}"
        echo ""
        "$gam_path" --version 2>/dev/null || "$gam_path" --help | head -2
    else
        echo -e "${YELLOW}⚠ 请重新打开终端或运行: export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    fi
}

print_usage() {
    echo ""
    echo "========================================"
    echo "      Game Archive Manager 安装完成"
    echo "========================================"
    echo ""
    echo "使用示例:"
    echo "  gam init --path /path/to/game/saves"
    echo "  gam snapshot save -m '游戏进度'"
    echo "  gam snapshot list"
    echo "  gam restore <snapshot-id>"
    echo ""
    echo "详细文档: README.md"
    echo ""
}

main() {
    echo "========================================"
    echo "   Game Archive Manager 安装脚本"
    echo "========================================"
    echo ""

    detect_os
    echo "系统: $OS"
    echo "安装目录: $INSTALL_DIR"
    echo ""

    check_permissions
    find_binary
    echo ""
    echo "正在安装..."

    create_symlink
    verify_install
    print_usage
}

if [ ! -f "target/release/$BINARY_NAME" ] && [ ! -f "target/release/${BINARY_NAME}.exe" ]; then
    if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
        echo "Usage: $0 [--binary PATH]"
        echo "  --binary PATH  指定可执行文件路径"
        exit 0
    fi
    if [ -n "$1" ] && [ -f "$1" ]; then
        BINARY_PATH="$1"
        detect_os
    else
        echo -e "${RED}错误: 找不到可执行文件${NC}"
        echo "请先编译: cargo build --release"
        exit 1
    fi
fi

main
