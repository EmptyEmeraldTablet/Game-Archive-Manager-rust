# Game Archive Manager v2.0 - 游戏存档版本控制系统

[![Rust](https://img.shields.io/badge/Rust-1.92+-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> 像 Git 一样管理你的游戏存档

## 简介

Game Archive Manager v2.0 是一个用 Rust 重构的游戏存档备份管理工具，支持 Windows、macOS 和 Linux。主要功能包括快照管理、时间线分支、忽略规则和去重存储。

**核心特性：**
- 📸 **快照管理** - 保存、查看、比较和恢复游戏存档
- 🌿 **时间线分支** - 像 Git 一样管理多条游戏进度
- 🔍 **差异比较** - 查看快照之间的变化
- 🚫 **忽略规则** - 使用 `.gamignore` 排除不需要的文件
- 💾 **内容去重** - 智能去重，节省存储空间
- 📊 **操作日志** - 记录所有操作历史

## 安装

### 方法一：直接下载二进制文件（推荐）

从 GitHub Releases 下载预编译的可执行文件：

1. 访问 [Game Archive Manager Releases](https://github.com/EmptyEmeraldTablet/Game-Archive-Manager-rust/releases)

2. 下载对应系统的版本：
   - **Linux**: `game-archive-manager-v2.0.0-linux-x86_64.tar.gz`
   - **macOS**: `game-archive-manager-v2.0.0-macos-universal.tar.gz`
   - **Windows**: `game-archive-manager-v2.0.0-windows-x86_64.zip`

3. 解压并安装：

   **Linux/macOS:**
   ```bash
   # 下载并解压
   wget https://github.com/EmptyEmeraldTablet/Game-Archive-Manager-rust/releases/download/v2.0.0/game-archive-manager-v2.0.0-linux-x86_64.tar.gz
   tar -xzf game-archive-manager-v2.0.0-linux-x86_64.tar.gz

   # 安装到系统（需要 root 权限）
   sudo mv game-archive-manager /usr/local/bin/gam

   # 验证安装
   gam --version
   ```

   **macOS (Apple Silicon):**
   ```bash
   # 下载并解压
   wget https://github.com/EmptyEmeraldTablet/Game-Archive-Manager-rust/releases/download/v2.0.0/game-archive-manager-v2.0.0-macos-aarch64.tar.gz
   tar -xzf game-archive-manager-v2.0.0-macos-aarch64.tar.gz

   # 移动到 Applications 或添加 PATH
   mv game-archive-manager ~/Applications/
   export PATH="~/Applications:$PATH"
   ```

   **Windows:**
   ```powershell
   # 使用 PowerShell 下载
   Invoke-WebRequest -Uri "https://github.com/EmptyEmeraldTablet/Game-Archive-Manager-rust/releases/download/v2.0.0/game-archive-manager-v2.0.0-windows-x86_64.zip" -OutFile "gam.zip"

   # 解压
   Expand-Archive -Path "gam.zip" -DestinationPath "C:\Games\GAM"

   # 添加到系统 PATH:
   # 右键"此电脑" -> 属性 -> 高级系统设置 -> 环境变量
   # 在用户变量或系统变量的 PATH 中添加: C:\Games\GAM
   ```

4. **验证安装:**
   ```bash
   gam --version
   gam --help
   ```

> **提示:** 下载解压后，使用 `./install.sh --binary /path/to/game-archive-manager` 可自动安装到系统 PATH。

### 方法二：从源码编译

```bash
# 克隆项目
git clone https://github.com/EmptyEmeraldTablet/Game-Archive-Manager-rust.git
cd game-archive-manager

# Release 构建
cargo build --release

# 可执行文件位于: target/release/game-archive-manager
```

### 一键安装到系统（Linux/macOS）

编译完成后，运行安装脚本自动添加到系统 PATH：

```bash
# 进入项目目录
cd game-archive-manager

# 运行安装脚本
./install.sh

# 需要 root 权限时会提示输入密码
# 安装位置: /usr/local/bin/gam

# 安装完成后需要重新打开终端，或执行:
source ~/.bashrc  # 或 ~/.zshrc
```

### Windows 安装

```bash
# 交叉编译（从 Linux/macOS）
cargo build --release --target x86_64-pc-windows-gnu

# 可执行文件: target/x86_64-pc-windows-gnu/release/game-archive-manager.exe

# 将 .exe 文件所在目录添加到系统 PATH:
#   设置 -> 系统 -> 关于 -> 高级系统设置 -> 环境变量
#   在 PATH 中添加 .exe 所在目录
```

## 快速开始

### 1. 初始化仓库

```bash
# 进入游戏存档目录
cd /path/to/your/game/saves

# 初始化 GAM
gam init --path /path/to/your/game/saves

# 或者交互式初始化
gam init
```

### 2. 保存第一个快照

```bash
# 保存当前状态（强烈建议使用 -m 参数添加描述）
gam snapshot save -m "游戏开始 - 创建角色"

# 保存到指定时间线
gam snapshot save -m "完成新手关卡" --timeline main

# 如果不提供 -m 参数，系统会自动生成时间戳名称
gam snapshot save
# 生成: "Snapshot 2026-01-29 12:54"
```

### 3. 查看快照

```bash
# 列出当前时间线快照
gam snapshot list

# 列出所有时间线快照
gam snapshot list --all

# 查看快照详情（可获取完整 ID）
gam snapshot info <short-id>

# 查看快照历史（用于恢复）
gam history
gam history --all  # 所有时间线的历史

# 查看操作日志（用于审计）
gam activity
gam activity --limit 10  # 最近 10 条
```

#### 📋 关于快照 ID

GAM 使用 **短 ID**（8 位十六进制字符）来标识快照，支持前缀匹配：

```
# snapshot list 输出示例
┌───────┬──────────┬────────────────────┬──────────────────────┐
│ 序号  │   ID     │ 时间               │ 名称                 │
├───────┼──────────┼────────────────────┼──────────────────────┤
│  1   │ e0bb142e │ 01-29 12:54 │ Level 3 │
│  2   │ 51dc4414 │ 01-29 12:54 │ Level 2 │
│  3   │ d108f6a6 │ 01-29 12:54 │ Level 1 │
└───────┴──────────┴────────────────────┴──────────────────────┘

# history 输出示例
* 2026-01-29 12:54:30  e0bb142e  Level 3
  2026-01-29 12:54:26  51dc4414  Level 2
```

**ID 使用规则：**

| 用法 | 示例 | 说明 |
|------|------|------|
| 完整 8 位 | `gam restore e0bb142e` | 推荐，最清晰 |
| 前缀匹配 | `gam restore e0bb` | 只要前缀唯一即可 |
| 完整哈希 | `gam restore e0bb142e24d92d89...` | `snapshot info` 中显示的完整 ID |

**注意事项：**

- ✅ **可以直接复制使用** - 输出中的 ID 如 `e0bb142e` 可直接复制粘贴
- ❌ **序号 #N 仅供显示** - 表格中的 `#1` 是显示序号，不能用于命令输入
- 📖 **查看完整 ID** - 使用 `gam snapshot info e0bb142e` 可查看完整哈希值

### 4. 时间线管理

```bash
# 创建新时间线
gam timeline create hard-mode

# 切换时间线
gam timeline switch hard-mode

# 查看当前时间线
gam timeline current

# 重命名时间线
gam timeline rename hard-mode nightmare

# 列出所有时间线
gam timeline list
```

### 5. 恢复存档

```bash
# 恢复到指定快照（使用短 ID）
gam restore e0bb142e

# 强制恢复（跳过确认）
gam restore e0bb142e --force

# 恢复时需要先切换到正确的时间线
gam timeline switch main
gam restore <snapshot-id>
```

### 6. 比较差异

```bash
# 比较两个快照
gam diff e0bb142e d108f6a6
```

## 命令参考

### 全局命令

| 命令 | 说明 |
|------|------|
| `gam init [--path PATH] [--force]` | 初始化版本控制 |
| `gam status` | 查看当前状态 |
| `gam history [--all]` | 查看操作历史 |
| `gam activity [--limit N]` | 查看活动日志 |
| `gam diff <id1> <id2>` | 比较两个快照 |
| `gam gc [--aggressive] [--dry-run]` | 垃圾回收 |
| `gam doctor [--fix]` | 诊断并修复问题 |
| `gam config [--list] [key] [value]` | 查看/设置配置 |
| `gam help` | 显示帮助信息 |

### 快照管理命令

| 命令 | 说明 |
|------|------|
| `gam snapshot save -m "描述"` | 保存当前状态为快照（推荐始终使用 -m） |
| `gam snapshot save -m "描述" --timeline NAME` | 保存到指定时间线 |
| `gam snapshot list [--all]` | 列出快照（显示短 ID） |
| `gam snapshot info <short-id>` | 查看快照详情（含完整 ID） |
| `gam snapshot delete <id> [--force]` | 删除快照 |
| `gam snapshot tag <id> <name>` | 为快照添加标签 |

### 时间线管理命令

| 命令 | 说明 |
|------|------|
| `gam timeline create <name>` | 创建新时间线 |
| `gam timeline create <name> --from <snapshot-id>` | 从指定快照创建时间线 |
| `gam timeline list` | 列出所有时间线 |
| `gam timeline switch <target>` | 切换时间线 |
| `gam timeline rename <old> <new>` | 重命名时间线 |
| `gam timeline delete <name> [--force]` | 删除时间线 |
| `gam timeline current` | 显示当前时间线 |

### 忽略规则命令

| 命令 | 说明 |
|------|------|
| `gam ignore add <pattern>` | 添加忽略规则 |
| `gam ignore remove <pattern>` | 移除忽略规则 |
| `gam ignore list` | 列出忽略规则 |
| `gam ignore check <file>` | 检查文件是否被忽略 |
| `gam ignore init [--force]` | 初始化默认规则模板 |

## 忽略规则 (.gamignore)

支持类似 `.gitignore` 的语法：

```gitignore
# 注释
*.log              # 忽略所有 .log 文件
screenshots/       # 忽略整个目录
!important.log     # 例外：不禁用 important.log
config/secrets.*   # 忽略 config 下 secrets 开头文件
backup~            # 忽略以 ~ 结尾的文件
```

## 文件结构

```
.gam/
├── config                 # 全局配置
├── HEAD                   # 当前时间线引用
├── .gamignore             # 忽略规则（可选）
├── refs/
│   └── timelines/         # 所有时间线指针
│       ├── main
│       └── hard-mode
├── objects/
│   ├── snapshot/          # 快照元数据
│   └── content/           # 内容可寻址存储
└── activity.log           # 操作日志
```

## 配置文件

```toml
[core]
game_path = "/path/to/game/saves"
default_timeline = "main"
use_gamignore = true

[storage]
strategy = "deduplication"
```

## 配置命令

```bash
# 列出所有配置
gam config --list

# 查看配置值
gam config core.default_timeline

# 设置配置值
gam config core.default_timeline main
```

## 标签功能

为重要快照添加语义化标签（便于识别和快速定位）：

```bash
# 添加标签
gam snapshot tag e0bb142e "v1.0-通关"

# 使用标签（未来支持）
# gam restore "v1.0-通关"
```

**提示：** 当前版本添加标签后，可通过 `gam history` 查看标签前缀：
```
* 2026-01-29 12:54:30  e0bb142e  [v1.0-通关] Level 3
```

## history 与 activity 的区别

GAM 提供两个看似相似但用途不同的命令：

### `history` - 快照历史

查看**快照列表**，回答"我有哪些存档版本"。

```bash
# 查看当前时间线的快照
gam history

# 查看所有时间线的快照
gam history --all
```

输出示例：
```
历史记录 (共 3 个快照)

* 2026-01-29 12:54:30  e0bb142e  Level 3
  2026-01-29 12:54:26  51dc4414  Level 2
  2026-01-29 12:54:23  d108f6a6  Level 1
```

**用途：** 找到要恢复的快照 ID

### `activity` - 操作日志

查看**操作记录**，回答"我之前做了什么"。

```bash
# 查看最近 20 条
gam activity

# 查看最近 10 条
gam activity --limit 10
```

输出示例：
```
活动记录 (最近 5 条):

  2026-01-29 12:53  init
  2026-01-29 12:54  snapshot save
  2026-01-29 12:54  snapshot save
  2026-01-29 12:54  snapshot save
  2026-01-29 13:16  restore
```

**用途：** 审计操作历史，跟踪所有操作

### 对比总结

| 特性 | `history` | `activity` |
|------|-----------|------------|
| **内容** | 快照 ID + 名称 | 操作类型 |
| **粒度** | 快照级别 | 操作级别 |
| **恢复存档时使用** | ✅ 是 | ❌ 否 |
| **审计操作时使用** | ❌ 否 | ✅ 是 |

### 快速选择

- 想**恢复存档** → 用 `history` 找快照 ID
- 想**查看操作记录** → 用 `activity` 追踪历史

## 项目结构

```
game-archive-manager/
├── src/
│   ├── main.rs              # 程序入口
│   ├── cli/                 # CLI 解析
│   │   ├── mod.rs
│   │   └── args.rs          # 参数定义
│   ├── core/                # 核心逻辑
│   │   ├── mod.rs
│   │   ├── activity.rs      # 活动日志
│   │   ├── tag.rs           # 标签管理
│   │   ├── commands.rs      # 命令实现
│   │   ├── error.rs         # 错误类型
│   │   ├── ignore.rs        # 忽略规则引擎
│   │   ├── messages/        # 国际化消息
│   │   │   └── mod.rs       # 消息目录和本地化管理
│   │   ├── store/           # 存储引擎
│   │   │   ├── mod.rs
│   │   │   ├── content_store.rs
│   │   │   └── snapshot_store.rs
│   │   └── types/           # 数据类型
│   ├── ui/                  # 用户界面
│   │   ├── mod.rs
│   │   └── formatter.rs     # 格式化输出
│   └── utils/               # 工具函数
│       ├── file_utils.rs
│       └── hash.rs
├── Cargo.toml               # 项目配置
├── build.bat               # Windows 构建脚本
└── README.md               # 本文档
```

## 技术栈

- **语言**: Rust 2024 Edition
- **依赖**:
  - `clap` - 命令行参数解析
  - `anyhow` - 错误处理
  - `toml` - TOML 配置
  - `chrono` - 日期时间处理
  - `thiserror` - 错误类型定义
  - `serde` - 序列化

## 国际化 (i18n)

Game Archive Manager v2.0 支持多语言界面，默认包含英文 (en) 和简体中文 (zh-CN)。

### 支持的语言

| 语言 | 区域 | 消息前缀 |
|------|------|----------|
| English | en | [Success] [Error] [Warning] [Info] |
| 简体中文 | zh-CN | [成功] [错误] [警告] [信息] |

### 消息键体系

所有用户可见消息都通过消息键 (message key) 获取，支持变量插值：

```rust
// 获取带变量的消息
let msg = messages.t("snapshot.save.success", &[
    ("short_id", "abc123"),
    ("name", "游戏进度"),
    ("timeline", "main"),
    ("file_count", "15"),
    ("size", "2.5MB"),
]);

// 输出: "Snapshot saved abc123 (游戏进度)\n  Timeline: main\n  Files: 15\n  Size: 2.5MB"
```

### 消息目录结构

```
src/core/messages/mod.rs
├── MessageManager       # 消息管理器
├── MessageCatalog      # 消息目录
├── chinese_catalog()   # 中文消息定义
└── english_catalog()   # 英文消息定义
```

### 添加新语言

1. 在 `MessageManager::new()` 中加载新目录：
```rust
manager.load_catalog("ja", japanese_catalog());
```

2. 创建消息目录函数：
```rust
fn japanese_catalog() -> MessageCatalog {
    let mut cat = MessageCatalog::new();
    cat.add("ui.success", "成功");
    cat.add("ui.error", "エラー");
    // ... 更多消息
    cat
}
```

### 消息前缀本地化

消息前缀（成功/错误/警告/信息）也从消息目录动态获取，确保 UI 输出一致性。

```rust
// print_success() 会自动使用当前语言的前缀
print_success("操作完成");  // 输出: [成功] 操作完成  或  [Success] Operation completed
```

### 消息键命名规范

- `模块.子模块.操作.状态`
- 示例: `snapshot.save.success`, `timeline.switch.error.not_found`

### 错误消息本地化

错误消息同样通过消息键本地化：
```rust
// 使用 thiserror 的 #[error] 注解配合消息键
#[error("{}", messages().t("common.error.not_found", &[("path", &path)))]
```

## 构建配置

- **静态链接**: 所有依赖静态链接，生成单文件可执行文件
- **Release 优化**: LTO 优化，代码单元 1，最高优化级别
- **二进制 Strip**: 移除调试符号，减小文件体积

## 注意事项

1. 请在**游戏正常关闭后**再进行存档操作
2. 游戏进行中**请勿读取存档**
3. 存档会随游戏进度逐渐增大，请耐心等待
4. 建议定期整理不需要的快照以节省空间
5. 时间线切换会改变后续快照的保存位置
6. ⚠️ **请勿将 GAM 程序放在游戏存档目录内** - 否则 restore 操作可能会尝试覆盖程序本身导致错误

## 常见问题 (FAQ)

### Q: 快照 ID 应该怎么输入？

**A:** GAM 使用 8 位短 ID，支持前缀匹配：
```bash
# 以下方式都可以（只要前缀唯一）
gam restore e0bb142e    # 完整 8 位
gam restore e0bb        # 前缀匹配
```

**注意：** 表格中显示的 `#1` 序号仅供显示，不能用于命令输入。

### Q: `gam restore` 提示 "Snapshot not found"？

**A:** 常见原因：
1. 复制了带 `...` 后缀的 ID（如 `e0bb142e...`）- 请去掉 `...` 后再试
2. 拼写错误 - 使用 `gam snapshot list` 查看正确的 ID
3. 当前时间线没有该快照 - 使用 `gam history --all` 查看所有快照

### Q: 快照名称太相似，无法区分？

**A:** 每次保存时使用 `-m` 参数提供有意义的描述：
```bash
gam snapshot save -m "Boss战前 - 装备检查"
gam snapshot save -m "通关存档 - 100%完成度"
```

### Q: 如何恢复到某个快照的父版本？

**A:** 使用 `gam snapshot info <id>` 查看父快照 ID：
```bash
gam snapshot info e0bb142e
# 输出中会显示 "父快照: 51dc4414"
gam restore 51dc4414
```

### Q: 想查看某个文件的修改历史？

**A:** 配合 `diff` 命令使用：
```bash
# 查看某文件在不同快照间的变化
gam diff <snapshot-id-1> <snapshot-id-2>
```

### Q: 误删了快照能恢复吗？

**A:** 目前版本**不支持**恢复已删除的快照。建议：
- 重要快照使用 `gam snapshot tag` 添加标签
- 定期检查快照列表，确认无误后再删除
- 考虑在删除前使用 `gam gc --dry-run` 检查影响范围

## 从 v1.0 迁移

v2.0 使用不同的存储格式，不直接兼容 v1.0。如需迁移，请手动导出/导入。

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT License

## 致谢

本项目基于 [NoitaArchiveManager](https://github.com/Xiaomony/NoitaArchiveManager) 重构，感谢原作者的代码贡献。
