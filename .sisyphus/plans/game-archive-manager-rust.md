# Game Archive Manager - Rust 重构计划

## 项目概述
使用 Rust 重构现有的 C++ 游戏存档备份管理工具，确保 Windows 兼容性，提供更好的错误处理和类型安全。

## 任务清单

### 阶段 1: 项目初始化
- [ ] 1.1 初始化 Rust 项目 (Cargo.toml, 依赖配置)
- [ ] 1.2 配置 Windows 目标平台和静态链接
- [ ] 1.3 创建项目目录结构

### 阶段 2: 核心模块实现
- [ ] 2.1 实现 `ArchiveManager` 核心结构体
- [ ] 2.2 实现文件复制逻辑 (递归目录复制)
- [ ] 2.3 实现存档信息数据结构 (ArchiveInfo)
- [ ] 2.4 实现日志存储 (JSON/二进制格式)

### 阶段 3: 命令系统
- [ ] 3.1 实现命令行交互界面
- [ ] 3.2 实现保存命令 (save, qsave, rsave)
- [ ] 3.3 实现加载命令 (load, qload)
- [ ] 3.4 实现查询命令 (log, slog, usage)
- [ ] 3.5 实现维护命令 (mArchive, delArch, qDelete)

### 阶段 4: Windows 兼容性
- [ ] 4.1 处理路径编码 (UTF-8 支持)
- [ ] 4.2 Windows 特殊文件夹获取
- [ ] 4.3 控制台颜色输出 (Windows ANSI 转义序列)

### 阶段 5: 构建与测试
- [ ] 5.1 配置 Cargo.toml (release 优化, 静态链接)
- [ ] 5.2 编写构建脚本 (build.bat)
- [ ] 5.3 编写 README 文档

---

## 技术决策

### 依赖选择
- **文件操作**: 标准库 `std::fs` (跨平台)
- **路径处理**: 标准库 `std::path::Path`
- **JSON 存储**: `serde_json` (日志文件)
- **命令行解析**: `clap` 或手写解析

### 数据结构

```rust
struct ArchiveInfo {
    timestamp: chrono::DateTime<Local>,
    name: String,
    comment: String,
}

struct ArchiveManager {
    source_path: PathBuf,
    archive_path: PathBuf,
    info_file: PathBuf,
    archives: Vec<ArchiveInfo>,
}
```

### 命令映射

| C++ 命令 | Rust 实现 | 说明 |
|:--------:|:---------:|:----:|
| save | `save()` | 带名称和备注 |
| qsave | `quick_save()` | 快速保存 |
| rsave | `replace_save()` | 覆盖最新 |
| load | `load(index)` | 加载指定 |
| qload | `quick_load()` | 加载最新 |
| log | `list_all()` | 全部列表 |
| slog | `list_recent(7)` | 最近7个 |
| mArchive | `modify(index)` | 修改信息 |
| delArch | `delete(index)` | 删除指定 |
| qDelete | `quick_delete()` | 删除最新 |
| usage | `usage()` | 占用空间 |

## 注意事项

1. **Windows 静态链接**: 确保生成单文件可执行程序
2. **UTF-8 编码**: Windows 控制台需要特殊处理
3. **错误处理**: 使用 `Result` 和 `anyhow` 进行错误传播
4. **向后兼容**: 保持相同的使用方式 (path.txt)
