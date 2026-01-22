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

### 从源码编译

```bash
# 克隆项目
git clone https://github.com/yourusername/game-archive-manager.git
cd game-archive-manager

# Release 构建
cargo build --release

# 可执行文件位于: target/release/game-archive-manager
```

### Windows 一键构建

双击运行 `build.bat` 即可自动完成清理和构建。

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
# 保存当前状态
gam snapshot save -m "游戏开始 - 创建角色"

# 保存到指定时间线
gam snapshot save -m "完成新手关卡" --timeline main
```

### 3. 查看快照

```bash
# 列出当前时间线快照
gam snapshot list

# 列出所有时间线快照
gam snapshot list --all

# 查看快照详情
gam snapshot info <snapshot-id>
```

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
# 恢复到指定快照
gam restore <snapshot-id>

# 强制恢复（跳过确认）
gam restore <snapshot-id> --force
```

### 6. 比较差异

```bash
# 比较两个快照
gam diff <snapshot-id-1> <snapshot-id-2>
```

## 命令参考

### 全局命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `gam init [--path PATH] [--force]` | - | 初始化版本控制 |
| `gam status` | - | 查看当前状态 |
| `gam history [--all]` | - | 查看操作历史 |
| `gam activity [--limit N]` | - | 查看活动日志 |
| `gam diff <id1> <id2>` | - | 比较两个快照 |
| `gam gc [--aggressive] [--dry-run]` | - | 垃圾回收 |
| `gam doctor [--fix]` | - | 诊断并修复问题 |
| `gam config [--list] [key] [value]` | - | 查看/设置配置 |
| `gam help` | - | 显示帮助信息 |

### 快照管理命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `gam snapshot save [-m MESSAGE] [--timeline NAME]` | - | 保存当前状态为快照 |
| `gam snapshot list [--all] [--timeline NAME]` | - | 列出快照 |
| `gam snapshot info <id>` | - | 查看快照详情 |
| `gam snapshot delete <id> [--force]` | - | 删除快照 |
| `gam snapshot tag <id> <name>` | - | 为快照添加标签 |

### 时间线管理命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `gam timeline create <name> [--from SNAPSHOT]` | - | 创建时间线 |
| `gam timeline list` | - | 列出时间线 |
| `gam timeline switch <target>` | - | 切换时间线 |
| `gam timeline rename <old> <new>` | - | 重命名时间线 |
| `gam timeline delete <name> [--force]` | - | 删除时间线 |
| `gam timeline current` | - | 显示当前时间线 |

### 忽略规则命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `gam ignore add <pattern>` | - | 添加忽略规则 |
| `gam ignore remove <pattern>` | - | 移除忽略规则 |
| `gam ignore list` | - | 列出忽略规则 |
| `gam ignore check <file>` | - | 检查文件是否被忽略 |
| `gam ignore init [--force]` | - | 初始化默认规则模板 |

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
.ggam/
├── config                 # 全局配置
├── HEAD                   # 当前时间线引用
├── .gamignore            # 忽略规则（可选）
├── refs/
│   └── timelines/        # 所有时间线指针
│       ├── main
│       └── hard-mode
├── objects/
│   ├── snapshot/         # 快照元数据
│   └── content/          # 内容可寻址存储
├── activity.log          # 操作日志
└── refs/
    └── tags.json         # 快照标签
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

为重要快照添加语义化标签：

```bash
# 添加标签
gam snapshot tag ab273213 "v1.0 通关"

# 列出所有标签
# (暂不支持)

# 使用标签
gam restore "v1.0"
```

## 活动日志

查看所有操作记录：

```bash
# 查看最近 20 条
gam activity

# 查看最近 10 条
gam activity --limit 10
```

示例输出：
```
2024-01-22 10:30  init
2024-01-22 10:35  snapshot save #1
2024-01-22 11:00  timeline switch main → hard-mode
2024-01-22 11:30  snapshot save #2
```

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

## 从 v1.0 迁移

v2.0 使用不同的存储格式，不直接兼容 v1.0。如需迁移，请手动导出/导入。

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT License

## 致谢

本项目基于 [NoitaArchiveManager](https://github.com/Xiaomony/NoitaArchiveManager) 重构，感谢原作者的代码贡献。
