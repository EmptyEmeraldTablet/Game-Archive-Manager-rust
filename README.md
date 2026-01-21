# Game Archive Manager - Rust 版

[![Rust](https://img.shields.io/badge/Rust-1.92+-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

游戏存档备份管理工具 - 使用 Rust 重构版本，支持 Windows。

## 简介

这是一个游戏存档备份管理工具，主要功能是帮助玩家备份和恢复游戏存档，防止因意外情况导致存档丢失。

**主要特点：**
- 支持手动保存、快速保存、覆盖保存
- 支持读取存档、快速读取
- 支持查看存档列表（全部/最近）
- 支持修改、删除存档信息
- 彩色命令行界面
- 跨平台支持（主要为 Windows 优化）

## 使用说明

### 1. 安装

#### 从源码编译

```bash
# 克隆项目
git clone https://github.com/yourusername/game-archive-manager.git
cd game-archive-manager

# Release 构建
cargo build --release

# 可执行文件位于: target/release/game-archive-manager.exe
```

#### Windows 一键构建

双击运行 `build.bat` 即可自动完成清理和构建。

### 2. 配置

1. **创建工作目录**：将 `game-archive-manager.exe` 放到一个专用文件夹中
2. **创建 path.txt**：在同一文件夹下创建 `path.txt` 文件，内容为游戏存档目录的**绝对路径**

示例 `path.txt`：
```
D:\Games\The Binding of Isaac Rebirth\Profile\ALI213\Saves
```

**注意事项：**
- path.txt 必须是**单行**，不要有回车或空格
- 建议不要将程序放在 C 盘，避免权限问题
- 建议关闭 Steam 云存档

### 3. 运行

```bash
# Windows
game-archive-manager.exe
```

### 4. 命令列表

| 命令 | 简写 | 说明 |
|------|------|------|
| `quit` | `q` | 退出程序 |
| `help` | `h` | 显示帮助信息 |
| `clearScreen` | `cls` | 清屏 |
| `save` | `s` | 保存存档（需输入名称和备注） |
| `qsave` | `qs` | 快速保存（无需输入） |
| `rsave` | `rs` | 覆盖式保存（更新最新存档） |
| `load` | `l` | 读取指定存档 |
| `qload` | `ql` | 快速读取（最新存档） |
| `log` | `lo` | 查看所有存档信息 |
| `slog` | `sl` | 查看最近七次存档 |
| `mArchive` | `ma` | 修改存档信息 |
| `delArch` | `del` | 删除指定存档 |
| `qDelete` | `qd` | 删除最新存档 |
| `usage` | `use` | 查看占用空间 |

### 5. 使用流程

```
1. 正常游戏并保存
2. 关闭游戏
3. 运行本程序
4. 输入 save 或 qs 保存当前存档
5. 继续游戏
6. 如需恢复，输入 load 或 ql
```

## 项目结构

```
game-archive-manager/
├── src/
│   ├── main.rs              # 程序入口
│   ├── core/
│   │   ├── mod.rs           # 核心模块
│   │   ├── archive_info.rs  # 存档信息结构
│   │   └── archive_manager.rs # 存档管理器
│   ├── commands/
│   │   └── mod.rs           # 命令处理器
│   ├── ui/
│   │   └── mod.rs           # 用户界面
│   └── utils/
│       └── file_utils.rs    # 文件工具函数
├── Cargo.toml               # 项目配置
├── build.bat                # Windows 构建脚本
└── README.md                # 本文档
```

## 技术栈

- **语言**: Rust 2024 Edition
- **依赖**:
  - `anyhow` - 错误处理
  - `serde_json` - JSON 序列化
  - `chrono` - 日期时间处理
  - `thiserror` - 错误类型定义

## 构建配置

- **静态链接**: 所有依赖静态链接，生成单文件可执行文件
- **Release 优化**: LTO 优化，代码单元 1，最高优化级别
- **二进制 Strip**: 移除调试符号，减小文件体积

## 注意事项

1. 请在**游戏正常关闭后**再进行存档操作
2. 游戏进行中**请勿读取存档**
3. 存档会随游戏进度逐渐增大，请耐心等待
4. 建议定期整理不需要的存档以节省空间

## 致谢

本项目基于 [NoitaArchiveManager](https://github.com/Xiaomony/NoitaArchiveManager) 重构，感谢原作者的代码贡献。

## License

MIT License
