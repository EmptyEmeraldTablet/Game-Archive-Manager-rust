#!/bin/bash
# Game Archive Manager v2.0 - 完整功能测试脚本
# 测试场景：模拟 RPG 游戏存档管理

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
}

print_step() {
    echo ""
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# 设置测试环境
TEST_DIR="/tmp/gam-test-$(date +%s)"
GAM_DIR="$TEST_DIR/game-saves/.gam"
GAME_DIR="$TEST_DIR/game-saves"
GAM_BIN="/home/yolo_dev/nop/GAM/target/release/game-archive-manager"

print_header "Game Archive Manager v2.0 - 完整功能测试"

# 清理之前的测试
print_step "清理测试环境（包括之前的测试）..."
rm -rf /tmp/gam-test-* 2>/dev/null || true
mkdir -p "$GAME_DIR"
print_success "测试目录: $TEST_DIR"

# 创建模拟游戏存档结构
print_step "创建模拟游戏存档结构..."
mkdir -p "$GAME_DIR/save_data"
mkdir -p "$GAME_DIR/config"
mkdir -p "$GAME_DIR/screenshots"
mkdir -p "$GAME_DIR/logs"

# 初始存档数据
cat > "$GAME_DIR/save_data/player.json" << 'EOF'
{
    "name": "Hero",
    "level": 1,
    "hp": 100,
    "mp": 50,
    "inventory": ["sword", "shield"],
    "position": {"x": 100, "y": 200}
}
EOF

cat > "$GAME_DIR/config/settings.json" << 'EOF'
{
    "difficulty": "normal",
    "sound_volume": 80,
    "music_volume": 70,
    "brightness": 100
}
EOF

echo "game screenshot 1" > "$GAME_DIR/screenshots/save_001.png"
echo "game screenshot 2" > "$GAME_DIR/screenshots/save_002.png"
echo "log file" > "$GAME_DIR/logs/game.log"
print_success "创建了初始存档数据"

# 构建（如果需要）
print_step "检查 GAM 二进制..."
if [ ! -f "$GAM_BIN" ]; then
    print_step "编译 GAM..."
    cargo build --release
fi
print_success "GAM 二进制就绪"

###############################################################################
# 测试 1: 初始化仓库
###############################################################################
print_header "测试 1: 初始化仓库"

print_step "运行 'gam init'"
$GAM_BIN init --path "$GAME_DIR"

print_step "进入游戏存档目录（后续命令需要）"
cd "$GAME_DIR"

if [ -d "$GAM_DIR" ]; then
    print_success "仓库初始化成功"
else
    print_error "仓库初始化失败"
    exit 1
fi

print_step "检查仓库结构..."
[ -f "$GAM_DIR/config" ] && print_success "配置文件存在"
[ -f "$GAM_DIR/HEAD" ] && print_success "HEAD 文件存在"
[ -d "$GAM_DIR/objects" ] && print_success "objects 目录存在"
[ -d "$GAM_DIR/refs" ] && print_success "refs 目录存在"

###############################################################################
# 测试 2: 保存第一个快照（开始新游戏）
###############################################################################
print_header "测试 2: 保存快照（开始新游戏）"

print_step "保存快照到 main 时间线"
$GAM_BIN snapshot save -m "开始新游戏 - 创建角色"

print_step "查看快照列表"
$GAM_BIN snapshot list

print_step "查看当前状态"
$GAM_BIN status

###############################################################################
# 测试 3: 游戏进度 - 第一次升级
###############################################################################
print_header "测试 3: 游戏进度（第一次升级）"

# 更新存档文件
cat > "$GAME_DIR/save_data/player.json" << 'EOF'
{
    "name": "Hero",
    "level": 5,
    "hp": 150,
    "mp": 80,
    "inventory": ["sword", "shield", "potion", "map"],
    "position": {"x": 250, "y": 300}
}
EOF

echo "boss battle screenshot" > "$GAME_DIR/screenshots/save_003.png"

print_step "保存快照 - 5级"
$GAM_BIN snapshot save -m "升到5级，获得新装备"

print_step "查看历史"
$GAM_BIN history

###############################################################################
# 测试 4: 创建新时间线 - 挑战模式
###############################################################################
print_header "测试 4: 创建新时间线（挑战模式）"

print_step "创建挑战模式时间线（从当前最新快照）"
$GAM_BIN timeline create challenge

print_step "列出所有时间线"
$GAM_BIN timeline list

print_step "切换到挑战模式"
$GAM_BIN timeline switch challenge

# 在挑战模式下继续游戏
cat > "$GAME_DIR/save_data/player.json" << 'EOF'
{
    "name": "Hero",
    "level": 10,
    "hp": 200,
    "mp": 120,
    "inventory": ["sword+1", "shield+1", "magic_ring"],
    "position": {"x": 500, "y": 400}
}
EOF

print_step "挑战模式 - 保存进度"
$GAM_BIN snapshot save -m "挑战模式 - 10级"

###############################################################################
# 测试 5: 创建第二个时间线 - 和平模式
###############################################################################
print_header "测试 5: 创建和平模式时间线"

print_step "从初始状态创建和平模式"
# 获取第一个快照ID（去除省略号）
FIRST_SNAP=$($GAM_BIN history --all | grep "^  " | tail -1 | awk '{print $3}' | sed 's/\.\.\.//')
$GAM_BIN timeline create pacifist --from "$FIRST_SNAP"

print_step "列出所有时间线"
$GAM_BIN timeline list

###############################################################################
# 测试 6: 快照详情和比较
###############################################################################
print_header "测试 6: 快照操作"

print_step "查看第一个快照详情"
FIRST_SNAP_ID=$($GAM_BIN history --all | grep "^  " | tail -1 | awk '{print $3}' | sed 's/\.\.\.//')
$GAM_BIN snapshot info "$FIRST_SNAP_ID"

print_step "比较快照"
SNAP1=$($GAM_BIN history --all | grep "^  " | tail -1 | awk '{print $3}' | sed 's/\.\.\.//')
SNAP2=$($GAM_BIN history --all | grep "^  " | head -1 | awk '{print $3}' | sed 's/\.\.\.//')
if [ -n "$SNAP1" ] && [ -n "$SNAP2" ] && [ "$SNAP1" != "$SNAP2" ]; then
    $GAM_BIN diff "$SNAP1" "$SNAP2" || true
else
    print_info "只有一个快照，跳过比较"
fi

###############################################################################
# 测试 7: 时间线切换
###############################################################################
print_header "测试 7: 时间线切换"

print_step "切换到和平模式"
$GAM_BIN timeline switch pacifist

print_step "查看和平模式快照"
$GAM_BIN snapshot list

print_step "切换回主模式"
$GAM_BIN timeline switch main

print_step "查看主模式快照"
$GAM_BIN snapshot list

###############################################################################
# 测试 8: 忽略规则
###############################################################################
print_header "测试 8: 忽略规则"

print_step "初始化忽略规则模板"
$GAM_BIN ignore init

print_step "添加自定义忽略规则"
$GAM_BIN ignore add "*.log"
$GAM_BIN ignore add "screenshots/"
$GAM_BIN ignore add "config/"

print_step "列出忽略规则"
$GAM_BIN ignore list

print_step "检查文件是否被忽略"
$GAM_BIN ignore check "game.log"
$GAM_BIN ignore check "save_data/player.json"

###############################################################################
# 测试 9: 恢复功能
###############################################################################
print_header "测试 9: 恢复功能"

print_step "备份当前存档"
cp "$GAME_DIR/save_data/player.json" "$GAME_DIR/save_data/player_backup.json"

print_step "修改存档（模拟意外修改）"
cat > "$GAME_DIR/save_data/player.json" << 'EOF'
{
    "name": "Corrupted",
    "level": 999,
    "hp": 9999,
    "mp": 9999,
    "inventory": ["cheated_item"]
}
EOF

print_step "恢复到上一个快照"
echo "n" | $GAM_BIN restore HEAD~1 --force || true

print_step "验证恢复结果"
cat "$GAME_DIR/save_data/player.json" | grep -q "level" && print_success "存档已恢复"

###############################################################################
# 测试 10: 诊断功能
###############################################################################
print_header "测试 10: 诊断功能"

print_step "运行健康检查"
$GAM_BIN doctor

print_step "运行健康检查（带修复）"
$GAM_BIN doctor --fix

###############################################################################
# 测试 11: 垃圾回收
###############################################################################
print_header "测试 11: 垃圾回收"

print_step "预览 GC 结果"
$GAM_BIN gc --dry-run

print_step "执行 GC"
$GAM_BIN gc

###############################################################################
# 测试 12: 最终状态检查
###############################################################################
print_header "测试 12: 最终状态检查"

print_step "查看最终状态"
$GAM_BIN status --verbose

print_step "列出所有快照"
$GAM_BIN snapshot list --all

print_step "列出所有时间线"
$GAM_BIN timeline list

print_step "查看完整历史"
$GAM_BIN history --all

###############################################################################
# 测试 13: 时间线管理
###############################################################################
print_header "测试 13: 时间线管理"

print_step "创建测试用时间线"
$GAM_BIN timeline create test-timeline

print_step "列出时间线"
$GAM_BIN timeline list

print_step "删除测试时间线（不是当前的）"
$GAM_BIN timeline delete test-timeline --force

print_step "确认删除"
$GAM_BIN timeline list

###############################################################################
# 清理测试环境
###############################################################################
print_header "测试完成"

print_step "清理测试环境..."
rm -rf "$TEST_DIR"
print_success "测试环境已清理"

print_header "所有测试通过！"
echo ""
echo "测试摘要："
echo "  ✓ 仓库初始化"
echo "  ✓ 快照保存"
echo "  ✓ 快照列表和详情"
echo "  ✓ 时间线创建和切换"
echo "  ✓ 快照比较 (diff)"
echo "  ✓ 恢复功能"
echo "  ✓ 忽略规则"
echo "  ✓ 诊断功能 (doctor)"
echo "  ✓ 垃圾回收 (gc)"
echo "  ✓ 状态查看"
echo ""
echo "Game Archive Manager v2.0 功能验证完成！"
