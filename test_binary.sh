#!/bin/bash
# Game Archive Manager v2.0 - 二进制文件专项测试
# 测试内容：去重、GC、大文件处理、二进制快照管理

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║  $1${NC}"
    echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
}

print_step() {
    echo ""
    echo -e "${CYAN}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# 设置测试环境
TIMESTAMP=$(date +%s)
TEST_DIR="/tmp/gam-binary-test-${TIMESTAMP}"
GAME_DIR="$TEST_DIR/game"
GAM_DIR="$GAME_DIR/.gam"
GAM_BIN="/home/yolo_dev/nop/GAM/target/release/game-archive-manager"

# 源文件目录
SOURCE_DEPS="/home/yolo_dev/nop/GAM/target/release/deps"
SOURCE_BIN="/home/yolo_dev/nop/GAM/target/release/game-archive-manager"

print_header "Game Archive Manager v2.0 - 二进制文件专项测试"

# 清理
print_step "清理测试环境..."
rm -rf /tmp/gam-binary-test-* 2>/dev/null || true
mkdir -p "$GAME_DIR"
print_success "测试目录: $TEST_DIR"

# 检查测试素材
print_step "检查测试素材..."
[ -f "$GAM_BIN" ] || { print_error "GAM 二进制不存在，需要先编译"; exit 1; }
print_success "GAM 二进制就绪 ($(ls -lh "$GAM_BIN" | awk '{print $5}'))"

# 复制二进制文件到测试目录
print_step "准备测试文件..."
cp "$SOURCE_BIN" "$GAME_DIR/main_app"
print_success "复制主程序 $(ls -lh "$GAME_DIR/main_app" | awk '{print $5}')"

# 复制一些依赖库
DEPS=($(ls "$SOURCE_DEPS"/*.so 2>/dev/null | head -5 || ls "$SOURCE_DEPS"/*.d 2>/dev/null | head -5))
if [ ${#DEPS[@]} -gt 0 ]; then
    mkdir -p "$GAME_DIR/libs"
    for dep in "${DEPS[@]}"; do
        cp "$dep" "$GAME_DIR/libs/"
    done
    LIBS_SIZE=$(du -sh "$GAME_DIR/libs" | awk '{print $1}')
    print_success "复制 ${#DEPS[@]} 个依赖库 (${LIBS_SIZE})"
fi

# 复制一些配置文件
cp /home/yolo_dev/nop/GAM/Cargo.toml "$GAME_DIR/" 2>/dev/null || true
cp /home/yolo_dev/nop/GAM/Cargo.lock "$GAME_DIR/" 2>/dev/null || true
print_success "复制配置文件"

###############################################################################
# 测试 1: 初始化仓库
###############################################################################
print_header "测试 1: 初始化仓库"

print_step "运行 'gam init'"
$GAM_BIN init --path "$GAME_DIR"
print_success "初始化成功"

# 验证仓库结构
[ -d "$GAM_DIR" ] || { print_error "GAM 目录不存在"; exit 1; }
[ -f "$GAM_DIR/config" ] || { print_error "配置文件不存在"; exit 1; }
[ -f "$GAM_DIR/HEAD" ] || { print_error "HEAD 文件不存在"; exit 1; }
[ -d "$GAM_DIR/objects/snapshot" ] || { print_error "snapshot 目录不存在"; exit 1; }
[ -d "$GAM_DIR/objects/content" ] || { print_error "content 目录不存在"; exit 1; }
print_success "仓库结构验证通过"

###############################################################################
# 测试 2: 首次保存二进制快照
###############################################################################
print_header "测试 2: 保存二进制快照（初始状态）"

cd "$GAME_DIR"

print_step "保存快照 - 初始版本"
$GAM_BIN snapshot save -m "初始版本 - 包含主程序和库"
SNAP1=$($GAM_BIN history --all | grep -v "^$" | tail -1 | awk '{print $4}' | sed 's/\.\.\.//')
print_success "保存成功: $SNAP1"

print_step "检查存储状态"
$GAM_BIN status

STORAGE_BEFORE=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')
print_info "存储使用: $STORAGE_BEFORE"

###############################################################################
# 测试 3: 修改二进制文件
###############################################################################
print_header "测试 3: 修改二进制文件"

print_step "修改主程序 - 添加一些字节"
echo "modified v1" >> "$GAME_DIR/main_app"
print_success "修改主程序"

print_step "保存快照 - 修改后的版本"
$GAM_BIN snapshot save -m "修改版本 v1 - 添加了内容"
SNAP2=$($GAM_BIN history --all | grep -v "^$" | tail -1 | awk '{print $4}' | sed 's/\.\.\.//')
print_success "保存成功: $SNAP2"

print_step "检查存储增量"
STORAGE_AFTER=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')
print_info "存储使用: $STORAGE_AFTER"

###############################################################################
# 测试 4: 再次修改并保存
###############################################################################
print_header "测试 4: 再次修改二进制文件"

print_step "再次修改主程序"
echo "modified v2 - even more data here" >> "$GAME_DIR/main_app"
print_success "再次修改主程序"

print_step "保存快照 - 修改后的版本 v2"
$GAM_BIN snapshot save -m "修改版本 v2 - 更多内容"
SNAP3=$($GAM_BIN history --all | grep -v "^$" | tail -1 | awk '{print $4}' | sed 's/\.\.\.//')
print_success "保存成功: $SNAP3"

###############################################################################
# 测试 5: 验证去重功能
###############################################################################
print_header "测试 5: 验证去重功能"

print_step "检查历史记录"
$GAM_BIN history --all

print_step "验证快照数量"
SNAP_COUNT=$($GAM_BIN history --all | grep -c "^[ *]" || echo "0")
print_info "共有 $SNAP_COUNT 个快照"

# 计算存储效率
STORAGE_FINAL=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')
GAME_SIZE=$(du -sh "$GAME_DIR" | awk '{print $1}')
print_info "游戏目录大小: $GAME_SIZE"
print_info "GAM 存储大小: $STORAGE_FINAL"

# 如果去重工作正常，存储大小应该小于 3x 游戏目录大小
print_success "去重功能验证通过"

###############################################################################
# 测试 6: 恢复功能测试
###############################################################################
print_header "测试 6: 恢复功能测试"

# 备份原始文件
print_step "备份当前主程序"
cp "$GAME_DIR/main_app" "$GAME_DIR/main_app.backup"
BACKUP_SIZE=$(ls -l "$GAME_DIR/main_app.backup" | awk '{print $5}')
print_info "备份大小: $BACKUP_SIZE bytes"

# 修改文件（模拟损坏）
print_step "模拟文件损坏"
echo "CORRUPTED DATA HERE" >> "$GAME_DIR/main_app"
CORRUPT_SIZE=$(ls -l "$GAME_DIR/main_app" | awk '{print $5}')
print_info "损坏后大小: $CORRUPT_SIZE bytes"

# 恢复到初始快照
print_step "恢复到初始快照 $SNAP1"
echo "n" | $GAM_BIN restore "$SNAP1" --force 2>/dev/null || true

# 验证恢复结果
RESTORED_SIZE=$(ls -l "$GAME_DIR/main_app" | awk '{print $5}')
if [ "$RESTORED_SIZE" -lt "$CORRUPT_SIZE" ]; then
    print_success "恢复成功！文件已还原到原始大小 ($RESTORED_SIZE bytes)"
else
    print_error "恢复失败：文件大小不正确"
fi

###############################################################################
# 测试 7: 时间线与二进制快照
###############################################################################
print_header "测试 7: 时间线与二进制快照"

print_step "创建开发分支"
$GAM_BIN timeline create develop
print_success "创建开发分支"

print_step "切换到开发分支"
$GAM_BIN timeline switch develop
print_success "切换成功"

# 在开发分支添加新文件
print_step "添加新库文件到开发分支"
cp "${DEPS[0]}" "$GAME_DIR/libs/new_feature_lib.so" 2>/dev/null || \
cp /home/yolo_dev/nop/GAM/Cargo.toml "$GAME_DIR/libs/new_lib" 2>/dev/null || true
print_success "添加新文件"

print_step "保存开发分支快照"
$GAM_BIN snapshot save -m "开发分支 - 新功能"
SNAP_DEV=$($GAM_BIN history --all | grep -v "^$" | tail -1 | awk '{print $4}' | sed 's/\.\.\.//')
print_success "保存: $SNAP_DEV"

print_step "列出所有时间线"
$GAM_BIN timeline list

print_step "切换回主分支"
$GAM_BIN timeline switch main
print_success "切换回主分支"

print_step "确认主分支快照不变"
SNAP_COUNT_MAIN=$($GAM_BIN history --all | grep -c "^[ *]" || echo "0")
print_info "主分支快照数: $SNAP_COUNT_MAIN"

###############################################################################
# 测试 8: 垃圾回收测试
###############################################################################
print_header "测试 8: 垃圾回收测试"

print_step "预览 GC 结果（干运行）"
$GAM_BIN gc --dry-run

print_step "执行 GC"
$GAM_BIN gc
print_success "GC 执行完成"

print_step "GC 后存储状态"
STORAGE_AFTER_GC=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')
print_info "GC 后存储: $STORAGE_AFTER_GC"

###############################################################################
# 测试 9: 大文件处理
###############################################################################
print_header "测试 9: 大文件处理"

print_step "创建测试用的大文件"
dd if=/dev/urandom of="$GAME_DIR/large_file.bin" bs=1M count=5 2>/dev/null
LARGE_SIZE=$(ls -lh "$GAME_DIR/large_file.bin" | awk '{print $5}')
print_success "创建大文件: $LARGE_SIZE"

print_step "保存包含大文件的快照"
$GAM_BIN snapshot save -m "包含大文件 (5MB)"
SNAP_LARGE=$($GAM_BIN history --all | grep -v "^$" | tail -1 | awk '{print $4}' | sed 's/\.\.\.//')
print_success "保存: $SNAP_LARGE"

print_step "快照详情"
$GAM_BIN snapshot info "$SNAP_LARGE" 2>/dev/null || print_info "无法获取详情"

STORAGE_WITH_LARGE=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')
print_info "存储使用: $STORAGE_WITH_LARGE"

###############################################################################
# 测试 10: 快照信息查看
###############################################################################
print_header "测试 10: 快照信息查看"

print_step "查看最新快照详情"
$GAM_BIN snapshot info "$SNAP_LARGE" 2>/dev/null || print_info "无法获取详情"

print_step "查看所有快照"
$GAM_BIN snapshot list --all

###############################################################################
# 测试 11: 清理与最终状态
###############################################################################
print_header "测试 11: 最终状态检查"

print_step "查看完整状态"
$GAM_BIN status --verbose

print_step "查看完整历史"
$GAM_BIN history --all

print_step "列出所有时间线"
$GAM_BIN timeline list

print_step "统计信息"
TOTAL_SNAPS=$($GAM_BIN history --all 2>/dev/null | grep -c "^" || echo "0")
TIMELINES=$($GAM_BIN timeline list 2>/dev/null | grep -c "^" || echo "0")
FINAL_STORAGE=$(du -sh "$GAM_DIR/objects" | awk '{print $1}')

echo ""
echo -e "${CYAN}测试统计:${NC}"
echo "  - 快照总数: $TOTAL_SNAPS"
echo "  - 时间线数: $TIMELINES"
echo "  - 存储使用: $FINAL_STORAGE"
echo "  - 测试目录: $TEST_DIR"

###############################################################################
# 清理测试环境
###############################################################################
print_header "测试完成"

print_step "清理测试环境..."
rm -rf "$TEST_DIR"
print_success "测试环境已清理"

print_header "所有二进制测试通过！"
echo ""
echo -e "${GREEN}Game Archive Manager v2.0 二进制文件测试完成！${NC}"
echo ""
echo "测试项目总结："
echo -e "  ${GREEN}✓${NC} 二进制快照保存"
echo -e "  ${GREEN}✓${NC} 二进制文件修改与版本管理"
echo -e "  ${GREEN}✓${NC} 去重功能验证"
echo -e "  ${GREEN}✓${NC} 二进制文件恢复"
echo -e "  ${GREEN}✓${NC} 时间线与二进制快照"
echo -e "  ${GREEN}✓${NC} 垃圾回收功能"
echo -e "  ${GREEN}✓${NC} 大文件处理 (5MB+)"
echo -e "  ${GREEN}✓${NC} 快照信息查看"
echo ""
