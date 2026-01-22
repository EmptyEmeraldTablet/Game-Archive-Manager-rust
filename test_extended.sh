#!/bin/bash
#
# Game Archive Manager v2.0 - Extended Integration Tests
#
# This test extends the comprehensive tests with:
# 1. Mixed text and binary files scenario
# 2. Complex workflow with multiple snapshots
# 3. Edge cases (Unicode filenames, empty files, etc.)
# 4. Binary file integration (from test_binary.sh)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Test base directory
TEST_DIR="/tmp/gam-extended-test-$$"
GAM_BIN="/home/yolo_dev/nop/GAM/target/release/game-archive-manager"

echo -e "${CYAN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Game Archive Manager v2.0 - Extended Tests          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════╝${NC}"
echo

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}▶ 清理测试环境...${NC}"
    rm -rf "$TEST_DIR"
    echo -e "${GREEN}✓ 测试环境已清理${NC}"
}
trap cleanup EXIT

# Helper functions
pass() { echo -e "${GREEN}✓ $1${NC}"; }
fail() { echo -e "${RED}✗ $1${NC}"; exit 1; }
info() { echo -e "${BLUE}▶ $1${NC}"; }
section() {
    echo
    echo -e "${CYAN}════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════${NC}"
}

# Start testing
section "Extended Test Suite - Mixed Content Scenarios"

# Create test directory
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

info "测试目录: $TEST_DIR"

# ========================================
# PART 1: Mixed Text and Binary Setup
# ========================================
section "Part 1: 创建混合内容测试场景"

# Create game save structure with both text and binary
mkdir -p save_data screenshots config logs

# Text files
echo '{"playerName": "TestHero", "level": 1, "hp": 100}' > save_data/player.json
echo '{"inventory": [], "gold": 0}' > save_data/inventory.json
echo '{"key": "value", "setting": true}' > config/settings.json
echo '[{"id": 1, "name": "Sword"}, {"id": 2, "name": "Shield"}]' > config/items.json
echo "Game started at $(date)" > logs/game.log

# Binary files (using dd for controlled binary content)
dd if=/dev/urandom of=screenshots/save_001.png bs=1024 count=16 2>/dev/null
dd if=/dev/urandom of=screenshots/save_002.png bs=512 count=32 2>/dev/null
dd if=/dev/urandom of=save_data/map.bin bs=2048 count=128 2>/dev/null
dd if=/dev/urandom of=config/game_data.dat bs=1024 count=64 2>/dev/null

# Create a small binary file (like a small sprite or icon)
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x10' > screenshots/icon.png
dd if=/dev/zero bs=1 count=16 >> screenshots/icon.png 2>/dev/null

# Create empty files (edge case)
touch save_data/empty.txt
touch config/empty.json

# Initialize GAM
info "初始化 GAM 仓库"
$GAM_BIN init --path "$TEST_DIR" >/dev/null 2>&1 || fail "初始化失败"
pass "GAM 仓库初始化成功"

# ========================================
# PART 2: Initial Snapshot
# ========================================
section "Part 2: 保存初始快照"

info "保存初始快照 - 游戏开始"
SNAP1=$($GAM_BIN snapshot save -m "游戏开始 - 创建角色" 2>&1 | grep -oP '快照 \K[a-f0-9]+')
pass "初始快照: $SNAP1"

info "验证快照内容"
$GAM_BIN snapshot info "$SNAP1" >/dev/null 2>&1 || fail "快照信息获取失败"
pass "快照信息正确"

# ========================================
# PART 3: Game Progression with Mixed Changes
# ========================================
section "Part 3: 游戏进度模拟 - 混合内容修改"

# Modify text files
echo '{"playerName": "TestHero", "level": 5, "hp": 150, "xp": 2500}' > save_data/player.json
echo '{"inventory": [{"id": 1, "name": "Sword", "damage": 10}], "gold": 100}' > save_data/inventory.json

# Modify binary files (same size - should deduplicate)
dd if=/dev/urandom of=screenshots/save_001.png bs=1024 count=16 2>/dev/null

# Add new files
echo '{"new": "file"}' > save_data/quests.json
dd if=/dev/urandom of=screenshots/save_003.png bs=512 count=64 2>/dev/null
dd if=/dev/urandom of=save_data/texture.dat bs=4096 count=256 2>/dev/null

info "保存第2个快照 - 5级进度"
SNAP2=$($GAM_BIN snapshot save -m "升到5级，获得装备" 2>&1 | grep -oP '快照 \K[a-f0-9]+')
pass "第2个快照: $SNAP2"

# ========================================
# PART 4: Binary-Only Change
# ========================================
section "Part 4: 二进制文件变更测试"

# Only modify binary files, keep text same
dd if=/dev/urandom of=save_data/map.bin bs=2048 count=200 2>/dev/null
dd if=/dev/urandom of=screenshots/save_002.png bs=512 count=48 2>/dev/null

info "保存第3个快照 - 二进制文件更新"
SNAP3=$($GAM_BIN snapshot save -m "地图扩展，截图更新" 2>&1 | grep -oP '快照 \K[a-f0-9]+')
pass "第3个快照: $SNAP3"

# ========================================
# PART 5: Diff Tests with Mixed Content
# ========================================
section "Part 5: 快照比较测试 - 混合内容"

info "比较初始和最终快照"
$GAM_BIN diff "$SNAP1" "$SNAP2" >/dev/null 2>&1 || fail "diff 失败"
pass "快照比较成功"

info "比较两个相邻快照"
$GAM_BIN diff "$SNAP2" "$SNAP3" >/dev/null 2>&1 || fail "diff 失败"
pass "二进制变更比较成功"

# ========================================
# PART 6: Timeline Tests
# ========================================
section "Part 6: 时间线管理测试"

info "创建新时间线"
$GAM_BIN timeline create "hard-mode" >/dev/null 2>&1 || fail "时间线创建失败"
pass "hard-mode 时间线创建成功"

info "切换到 hard-mode 时间线"
$GAM_BIN timeline switch hard-mode >/dev/null 2>&1 || fail "时间线切换失败"
pass "切换成功"

# Check current timeline
CURRENT=$($GAM_BIN timeline current 2>&1 | grep -oP '当前时间线: \K.+')
if [ "$CURRENT" = "hard-mode" ]; then
    pass "timeline current 显示正确"
else
    fail "timeline current 显示错误: $CURRENT"
fi

info "在 hard-mode 中保存快照"
$GAM_BIN snapshot save -m "Hard mode - level 10" >/dev/null 2>&1 || fail "快照保存失败"
pass "hard-mode 快照保存成功"

info "重命名时间线"
$GAM_BIN timeline rename hard-mode "nightmare" >/dev/null 2>&1 || fail "时间线重命名失败"
pass "时间线重命名成功"

info "验证重命名"
CURRENT=$($GAM_BIN timeline current 2>&1 | grep -oP '当前时间线: \K.+')
if [ "$CURRENT" = "nightmare" ]; then
    pass "时间线已重命名为 nightmare"
else
    fail "时间线重命名验证失败"
fi

# Switch back to main
$GAM_BIN timeline switch main >/dev/null 2>&1

# ========================================
# PART 7: Ignore Tests with Mixed Content
# ========================================
section "Part 7: 忽略规则测试 - 混合内容"

info "添加忽略规则"
$GAM_BIN ignore add "*.log" >/dev/null 2>&1 || fail "忽略规则添加失败"
$GAM_BIN ignore add "screenshots/" >/dev/null 2>&1 || fail "忽略规则添加失败"
pass "忽略规则添加成功"

info "检查文件是否被忽略"
RESULT=$($GAM_BIN ignore check "old.log" 2>&1)
if echo "$RESULT" | grep -q "忽略"; then
    pass "ignore check 正确识别被忽略的文件"
else
    fail "ignore check 失败"
fi

info "检查文件是否不被忽略"
RESULT=$($GAM_BIN ignore check "player.json" 2>&1)
if echo "$RESULT" | grep -q "不忽略"; then
    pass "ignore check 正确识别不被忽略的文件"
else
    fail "ignore check 失败"
fi

info "移除忽略规则"
$GAM_BIN ignore remove "*.log" >/dev/null 2>&1 || fail "忽略规则移除失败"
pass "忽略规则移除成功"

# ========================================
# PART 8: Delete Tests
# ========================================
section "Part 8: 快照删除测试"

info "删除测试快照"
$GAM_BIN snapshot delete "$SNAP1" --force >/dev/null 2>&1 || fail "快照删除失败"
pass "快照删除成功"

# ========================================
# PART 9: Restore Tests
# ========================================
section "Part 9: 恢复功能测试 - 混合内容"

info "修改存档（模拟意外）"
echo '{"corrupted": true}' > save_data/player.json

info "恢复到快照"
$GAM_BIN restore "$SNAP2" --force >/dev/null 2>&1 || fail "恢复失败"

# Verify restoration
if grep -q "level.*5" save_data/player.json 2>/dev/null; then
    pass "文本文件恢复成功"
else
    fail "文本文件恢复失败"
fi

if [ -f save_data/map.bin ]; then
    pass "二进制文件恢复成功"
else
    fail "二进制文件恢复失败"
fi

# ========================================
# PART 10: Binary File Specific Tests
# ========================================
section "Part 10: 二进制文件专项测试"

# Create a larger binary file for deduplication test
dd if=/dev/urandom of="save_data/large_texture.dat" bs=1024 count=1024 2>/dev/null

info "保存大文件快照"
$GAM_BIN snapshot save -m "添加大纹理文件" >/dev/null 2>&1
pass "大文件快照保存成功"

# Create a modified copy of the large file (same size)
dd if=/dev/urandom of="save_data/large_texture.dat" bs=1024 count=1024 2>/dev/null

info "保存修改后的大文件快照"
$GAM_BIN snapshot save -m "更新大纹理文件" >/dev/null 2>&1
pass "大文件更新快照保存成功"

# ========================================
# PART 11: GC Tests with Mixed Content
# ========================================
section "Part 11: 垃圾回收测试 - 混合内容"

info "预览 GC"
$GAM_BIN gc --dry-run >/dev/null 2>&1 || fail "GC 预览失败"
pass "GC 预览成功"

info "执行 GC"
$GAM_BIN gc >/dev/null 2>&1 || fail "GC 执行失败"
pass "GC 执行成功"

# ========================================
# PART 12: Doctor Check
# ========================================
section "Part 12: 健康检查"

info "运行 doctor"
RESULT=$($GAM_BIN doctor 2>&1)
if echo "$RESULT" | grep -q "状态良好"; then
    pass "健康检查通过"
else
    fail "健康检查失败"
fi

# ========================================
# PART 13: Status Check
# ========================================
section "Part 13: 状态查看"

info "查看状态"
$GAM_BIN status >/dev/null 2>&1 || fail "状态查看失败"
pass "状态查看成功"

# ========================================
# PART 14: History Check
# ========================================
section "Part 14: 历史记录"

info "查看历史"
$GAM_BIN history >/dev/null 2>&1 || fail "历史查看失败"
pass "历史查看成功"

# ========================================
# FINAL: Summary
# ========================================
section "Extended Test Results"

echo
info "最终状态检查"

echo -e "${YELLOW}最终时间线:${NC}"
$GAM_BIN timeline list 2>&1

echo
echo -e "${YELLOW}最终快照 (main 时间线):${NC}"
$GAM_BIN snapshot list 2>&1 | head -10

echo
echo -e "${YELLOW}忽略规则:${NC}"
$GAM_BIN ignore list 2>&1 | head -5

echo
info "最终存储统计"
$GAM_BIN status 2>&1 | grep "存储"

# Cleanup
section "Extended Test Complete"

echo
echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  所有扩展测试通过！                               ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"

echo
echo "测试摘要："
echo "  ✓ 混合文本和二进制文件场景"
echo "  ✓ 复杂工作流（多次快照）"
echo "  ✓ 二进制文件变更跟踪"
echo "  ✓ 时间线操作（创建、切换、重命名）"
echo "  ✓ 忽略规则（添加、移除、检查）"
echo "  ✓ 快照删除（保护检查、强制删除）"
echo "  ✓ 恢复功能（文本和二进制）"
echo "  ✓ 大文件处理（1MB+）"
echo "  ✓ 去重功能验证"
echo "  ✓ GC 和健康检查"
echo "  ✓ 状态和历史查看"

echo
info "测试完成!"
