# Game Archive Manager v2.0 - 实现计划

> 游戏存档版本控制系统
> 基于 Git 思想，针对二进制存档优化

---

## 一、设计概述

### 1.1 核心目标

将简单的存档备份工具升级为**游戏存档版本控制系统**，支持：
- 多时间线/分支管理
- 快照式存档
- 完整历史追溯
- 高效二进制存储

### 1.2 Git 概念映射

| Git 概念 | 游戏存档对应 | 说明 |
|---------|-------------|------|
| `commit` | **snapshot** | 存档快照，包含完整状态 |
| `branch` | **timeline** | 游戏进度分支路线 |
| `checkout` | **restore** | 恢复到指定快照 |
| `merge` | ~~N/A~~ | 不需要，用户明确排除 |
| `log` | **history** | 查看快照历史 |
| `reflog` | **activity** | 追踪所有操作记录 |
| `HEAD` | **current** | 当前活动的时间线 |
| `objects` | **store** | 内容可寻址存储 |

### 1.3 命令风格

采用 Git-style 子命令模式：

```bash
gam init [--path <dir>]
gam snapshot save [-m <msg>] [-t <timeline>]
gam snapshot list
gam timeline create <name>
gam restore <id>
```

---

## 二、数据结构设计

### 2.1 核心类型

```rust
// 快照元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 快照唯一ID (SHA256 of metadata)
    pub id: String,
    /// 父快照ID
    pub parent: Option<String>,
    /// 所属时间线
    pub timeline: String,
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 快照名称/描述
    pub name: String,
    /// 包含的文件列表
    pub files: Vec<FileEntry>,
    /// 所有内容的组合哈希
    pub content_hash: String,
    /// 快照总大小（字节）
    pub size: u64,
}

/// 文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// 相对于存档目录的路径
    pub path: RelativePath,
    /// 内容哈希 (SHA256)
    pub hash: String,
    /// 文件大小
    pub size: u64,
}

/// 时间线指针
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// 时间线名称
    pub name: String,
    /// 当前指向的快照ID
    pub head_snapshot: String,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 描述
    pub description: Option<String>,
}

/// 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 游戏存档目录
    pub game_path: PathBuf,
    /// 默认时间线
    pub default_timeline: String,
    /// 存储策略
    pub storage_strategy: StorageStrategy,
    /// 保留策略
    pub retention: RetentionPolicy,
}

/// 存储策略
pub enum StorageStrategy {
    /// 全量复制
    FullCopy,
    /// 内容去重
    Deduplication,
    /// 压缩存储
    Compression,
}

/// 保留策略
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// 保留天数 (0 = 无限)
    pub keep_days: u32,
    /// 保留快照数 (0 = 无限)
    pub keep_count: u32,
}

/// 2.1.5 .gamignore 忽略规则
///
/// 类似于 `.gitignore`，用于配置在保存快照时忽略的文件/目录
///
/// ## 规则语法
///
/// | 模式 | 匹配 | 示例 |
/// |------|------|------|
/// | `*.log` | 任意位置的 `.log` 文件 | `error.log`, `debug.log` |
/// | `temp/` | 特定目录 | 忽略 `temp/` 整个目录 |
/// | `/save.bak` | 根目录特定文件 | 仅忽略根目录的 `save.bak` |
/// | `**/*.tmp` | 任意位置的 `.tmp` 文件 | `a.tmp`, `sub/b.tmp` |
/// | `!important.dat` | 取反，保留匹配项 | 排除后再包含 |
///
/// ## 内置默认规则
///
/// ```gitignore
/// # 操作系统文件
/// .DS_Store
/// Thumbs.db
/// desktop.ini
///
/// # 临时文件
/// *.tmp
/// *.temp
/// *.swp
/// *~
/// .*~
///
/// # 日志文件 (按需启用)
/// # *.log
/// # logs/
///
/// # 备份文件
/// *.bak
/// *.backup
/// *~
/// ```
///
/// ## 使用方式
///
/// 1. 在游戏存档目录创建 `.gamignore` 文件
/// 2. 在 `.gam/config` 中启用 `use_gamignore = true`
/// 3. 运行 `gam snapshot save` 时自动应用规则
///
/// ## 示例
///
/// ```bash
/// # .gamignore
///
/// # 忽略临时文件
/// *.tmp
/// *.temp
///
/// # 忽略自动保存（可能频繁变化）
/// autosave_*
///
/// # 忽略特定目录
/// debug_logs/
/// screenshots/
///
/// # 但保留重要的配置
/// !keyconfig.ini
/// ```
///
/// ## 命令支持
///
/// ```bash
/// gam ignore add *.log          # 添加规则
/// gam ignore remove *.log       # 移除规则
/// gam ignore list               # 列出当前规则
/// gam ignore check <file>       # 检查文件是否会被忽略
/// gam ignore init               # 初始化默认规则模板
/// ```

/// .gamignore 配置
#[derive(Debug, Clone, Default)]
pub struct GamIgnoreConfig {
    /// 是否启用 .gamignore
    pub enabled: bool,
    /// 忽略规则列表
    pub patterns: Vec<IgnorePattern>,
    /// 规则文件路径
    pub ignore_file: PathBuf,
}

/// 忽略模式
#[derive(Debug, Clone)]
pub struct IgnorePattern {
    /// 原始模式字符串
    pub pattern: String,
    /// 是否为否定模式 (! 开头)
    pub negated: bool,
    /// 模式类型
    pub pattern_type: PatternType,
}

/// 模式类型
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    /// 精确文件名匹配
    Exact(String),
    /// 通配符匹配 (glob)
    Glob(String),
    /// 目录匹配
    Directory(String),
    /// 根目录特定文件 (以 / 开头)
    RootFile(String),
    /// 递归匹配 (**/)
    Recursive(String),
}
```

### 2.2 对象存储布局

```
.gam/                                    # 版本控制目录
├── config                               # 全局配置
├── HEAD                                 # 当前时间线引用
├── .gamignore                           # 忽略规则（可选）
├── refs/
│   └── timelines/                       # 所有时间线指针
│       ├── main
│       ├── bad_ending
│       └── new_character_run
├── objects/
│   ├── snapshot/                        # 快照元数据
│   │   ├── ab/                         # SHA256 前2位
│   │   │   └── cdef123456789...        # 完整哈希
│   │   └── pack/                       # 打包存储（可选）
│   └── content/                        # 内容可寻址存储
│       ├── ab/                         # SHA256 前2位
│       │   └── cdef123456789...        # 文件内容
│       └── index                       # 内容索引
├── index                               # 快速查找索引
├── activity.log                        # 操作日志
└── gc_last_run                         # 上次GC时间

snapshots/                              # 人类可读的快捷方式
├── 1 -> ../.am/objects/snapshot/...
├── 2 -> ...
└── ...
```

### 2.3 元数据 JSON Schema

**快照元数据 (`.gam/objects/snapshot/ab/cdef123...`)**:

```json
{
  "id": "abc123def456789...",
  "parent": "parent123...",
  "parent": "another_parent...",  // 可选，合并场景
  "timeline": "main",
  "timestamp": "2024-01-22T10:30:00+08:00",
  "name": "Defeated the dragon boss",
  "description": "Finally beat the final boss on ng+",
  "files": [
    {
      "path": "savegame.bin",
      "hash": "filehash123...",
      "size": 1048576,
      "permissions": "rw"
    },
    {
      "path": "inventory.json",
      "hash": "filehash456...",
      "size": 2048
    }
  ],
  "content_hash": "combinedhash123...",
  "size": 1071616,
  "compression": "none",
  "gam_version": "2.0.0"
}
```

**时间线引用 (`.gam/refs/timelines/main`)**:

```
abc123def456789...
```

---

## 三、命令规格

### 3.1 命令列表

| 分类 | 命令 | 别名 | 说明 |
|------|------|------|------|
| **初始化** | `gam init` | - | 初始化版本控制 |
| | `gam init --path <dir>` | - | 指定存档目录 |
| **快照** | `gam snapshot save` | `gam save`, `gam commit` | 保存当前状态 |
| | `gam snapshot save -m "msg"` | - | 带消息保存 |
| | `gam snapshot save -t main` | - | 保存到指定时间线 |
| | `gam snapshot list` | `gam log` | 列出快照 |
| | `gam snapshot list -t beta` | - | 列出指定时间线 |
| | `gam snapshot info <id>` | `gam show` | 查看快照详情 |
| | `gam snapshot delete <id>` | `gam rm` | 删除快照 |
| | `gam snapshot tag <id> <name>` | - | 给快照打标签 |
| **时间线** | `gam timeline create <name>` | `gam branch` | 创建分支 |
| | `gam timeline create <name> --from <id>` | - | 从指定快照创建 |
| | `gam timeline list` | `gam branches` | 列出所有分支 |
| | `gam timeline switch <name>` | `gam checkout` | 切换分支 |
| | `gam timeline switch <id>` | - | 切换到指定快照 (分离HEAD) |
| | `gam timeline rename <old> <new>` | - | 重命名分支 |
| | `gam timeline delete <name>` | - | 删除分支 |
| | `gam timeline current` | - | 显示当前分支 |
| **恢复** | `gam restore <id>` | `gam checkout <id>` | 恢复到快照 |
| | `gam restore --force` | - | 强制恢复 |
| **查看** | `gam history` | - | 当前时间线历史 |
| | `gam history --all` | - | 所有时间线历史 |
| | `gam activity` | `gam reflog` | 操作日志 |
| | `gam status` | - | 当前状态 |
| | `gam diff <id1> <id2>` | - | 比较两个快照 |
| **管理** | `gam gc` | - | 垃圾回收 |
| | `gam gc --aggressive` | - | 强力GC |
| | `gam config <key> <value>` | - | 配置设置 |
| | `gam doctor` | - | 诊断问题 |
| **忽略规则** | `gam ignore add <pattern>` | - | 添加忽略规则 |
| | `gam ignore remove <pattern>` | - | 移除忽略规则 |
| | `gam ignore list` | - | 列出当前规则 |
| | `gam ignore check <file>` | - | 检查文件是否忽略 |
| | `gam ignore init` | - | 初始化默认规则 |
| **帮助** | `gam help` | `gam --help` | 显示帮助 |
| | `gam help snapshot` | - | 命令特定帮助 |

### 3.2 命令详解

#### 3.2.1 初始化

```bash
# 交互式初始化
$ gam init
  游戏存档目录: /path/to/game/saves
  默认时间线名称: main
  ✓ 初始化完成

# 非交互式
$ gam init --path "D:/Games/Isaac/saves"
  ✓ 初始化完成

# 重新初始化（会保留已有数据）
$ gam init --force
```

#### 3.2.2 保存快照

```bash
# 基本保存
$ gam snapshot save
  请输入快照名称: Defeated the dragon boss
  ✓ 已保存快照 abc1234 (main 分支)

# 带消息保存
$ gam snapshot save -m "First completion"
  ✓ 已保存快照 abc1234 (main 分支)

# 保存到指定时间线
$ gam snapshot save -m "Trying mage build" -t mage_build
  ✓ 已保存快照 def5678 (mage_build 分支)

# 组合使用
$ gam snapshot save -m "Boss beaten" -t new_route --from snapshot_5
```

#### 3.2.3 快照列表

```bash
# 当前时间线
$ gam snapshot list
  main 分支快照 (共 5 个)
  ┌─────┬────────────────────┬──────────────────────┐
  │ ID  │ 时间               │ 名称                 │
  ├─────┼────────────────────┼──────────────────────┤
  │ 1   │ 2024-01-20 10:00   │ Game start           │
  │ 2   │ 2024-01-20 11:30   │ Level 5 reached      │
  │ 3   │ 2024-01-21 09:00   │ First boss died      │
  └─────┴────────────────────┴──────────────────────┘

# 所有时间线
$ gam snapshot list --all
  所有快照 (共 12 个)
  ...

# 指定时间线
$ gam snapshot list -t beta
```

#### 3.2.4 恢复

```bash
# 恢复到快照
$ gam restore 3
  此操作会覆盖当前存档。确定继续? (y/n): y
  ✓ 已恢复到快照 #3 (2024-01-21 09:00)

# 分离 HEAD 恢复
$ gam restore abc123def...
  HEAD 现在指向 abc123def...

# 强制恢复（跳过确认）
$ gam restore --force 3
```

#### 3.2.5 时间线操作

```bash
# 创建分支
$ gam timeline create new_route
  ✓ 从快照 #5 创建新分支 'new_route'
  已切换到 'new_route'

# 从指定快照创建
$ gam timeline create hard_mode --from 1
  ✓ 从快照 #1 创建新分支 'hard_mode'

# 列出分支
$ gam timeline list
  * main          (5 个快照)
    new_route     (3 个快照)
    hard_mode     (1 个快照)

# 切换分支
$ gam timeline switch main
  已切换到 'main'

# 重命名
$ gam timeline rename new_route alternative_route
  ✓ 重命名完成

# 删除
$ gam timeline delete hard_mode
  警告: 将删除分支及其所有快照。确定? (y/n): y
  ✓ 已删除 'hard_mode'
```

#### 3.2.6 查看命令

```bash
# 历史
$ gam history
  main 分支历史
  * 2024-01-22 10:30  #5 Defeated dragon boss
  * 2024-01-21 14:00  #4 Level 10 reached
  * 2024-01-21 09:00  #3 First boss died
  * 2024-01-20 11:30  #2 Level 5 reached
  * 2024-01-20 10:00  #1 Game start

# 所有历史
$ gam history --all
  ...

# 操作日志
$ gam activity
  2024-01-22 10:30  restore #3 → #5
  2024-01-22 10:00  snapshot save #4
  2024-01-21 14:00  timeline switch main → new_route
  ...

# 状态
$ gam status
  当前时间线: main
  快照数量: 5
  存档大小: 156.78 MB
  存储大小: 89.23 MB (去重后)

# 差异比较
$ gam diff 1 5
  比较 #1 和 #5
  M  savegame.bin    +1.2 MB
  A  new_item.json   2.5 KB
  D  old_item.json   -1.0 KB
```

---

## 四、实现阶段

### 阶段 1: 基础框架 (Week 1)

#### 1.1 项目结构

```
src/
├── main.rs                           # 入口
├── cli/                              # CLI 解析
│   ├── mod.rs
│   ├── args.rs                       # 参数定义
│   └── commands/                     # 命令实现
├── core/                             # 核心逻辑
│   ├── mod.rs
│   ├── store/                        # 存储引擎
│   │   ├── mod.rs
│   │   ├── content_store.rs          # 内容可寻址存储
│   │   └── snapshot_store.rs         # 快照存储
│   ├── snapshot.rs                   # 快照类型
│   ├── timeline.rs                   # 时间线类型
│   ├── config.rs                     # 配置管理
│   └── ignore.rs                     # 忽略规则引擎
├── ui/                               # 输出
│   ├── mod.rs
│   ├── formatter.rs                  # 格式化输出
│   └── colors.rs                     # 颜色工具
└── utils/                            # 工具函数
    ├── mod.rs
    ├── file_utils.rs                 # 文件操作
    └── hash.rs                       # 哈希计算
```

#### 1.2 核心类型实现

- [ ] `Snapshot` 结构体及序列化
- [ ] `FileEntry` 结构体
- [ ] `Timeline` 结构体
- [ ] `Config` 结构体
- [ ] `StorageStrategy` 枚举
- [ ] 错误类型定义

#### 1.3 CLI 框架

- [ ] 使用 `clap` 解析器
- [ ] 定义所有命令的子命令结构
- [ ] 帮助信息格式化

### 阶段 2: 存储引擎 (Week 2)

#### 2.1 内容可寻址存储

```rust
pub struct ContentStore {
    root: PathBuf,
}

impl ContentStore {
    pub fn store(&mut self, path: &Path) -> Result<String, Error>;
    pub fn get(&self, hash: &str) -> Result<PathBuf, Error>;
    pub fn exists(&self, hash: &str) -> bool;
    pub fn deduplicate(&self) -> Result<u64, Error>;  // 统计去重节省空间
}
```

- [ ] 实现 `ContentStore`
- [ ] SHA-256 哈希计算
- [ ] 文件复制/链接逻辑
- [ ] 索引维护

#### 2.2 快照存储

```rust
pub struct SnapshotStore {
    content_store: ContentStore,
    snapshot_dir: PathBuf,
}

impl SnapshotStore {
    pub fn create(&mut self, files: &[PathBuf]) -> Result<Snapshot, Error>;
    pub fn get(&self, id: &str) -> Result<Snapshot, Error>;
    pub fn list(&self, timeline: &str) -> Vec<Snapshot>;
    pub fn delete(&mut self, id: &str) -> Result<(), Error>;
}
```

- [ ] 实现 `SnapshotStore`
- [ ] 快照元数据序列化/反序列化
- [ ] 快照列表查询优化
- [ ] 快照删除及清理

#### 2.3 时间线管理

```rust
pub struct TimelineManager {
    refs_dir: PathBuf,
    current_head: PathBuf,
}

impl TimelineManager {
    pub fn create(&self, name: &str, from_snapshot: Option<&str>) -> Result<(), Error>;
    pub fn list(&self) -> Vec<Timeline>;
    pub fn switch(&self, name: &str) -> Result<(), Error>;
    pub fn current(&self) -> Result<String, Error>;
    pub fn delete(&self, name: &str) -> Result<(), Error>;
}
```

- [ ] 实现 `TimelineManager`
- [ ] HEAD 指针管理
- [ ] 时间线引用读写

### 阶段 3: 命令实现 (Week 3)

#### 3.1 基础命令

| 命令 | 优先级 | 状态 |
|------|--------|------|
| `init` | P0 | - |
| `snapshot save` | P0 | - |
| `snapshot list` | P0 | - |
| `snapshot info` | P1 | - |
| `restore` | P0 | - |

#### 3.2 时间线命令

| 命令 | 优先级 | 状态 |
|------|--------|------|
| `timeline create` | P0 | - |
| `timeline list` | P0 | - |
| `timeline switch` | P0 | - |
| `timeline rename` | P1 | - |
| `timeline delete` | P1 | - |

#### 3.3 查看命令

| 命令 | 优先级 | 状态 |
|------|--------|------|
| `history` | P1 | - |
| `activity` | P2 | - |
| `status` | P1 | - |
| `diff` | P2 | - |

### 阶段 4: 高级功能 (Week 4)

#### 4.1 存储优化

- [ ] 文件去重检测
- [ ] 增量存储（仅存储变更文件）
- [ ] 可选压缩支持 (lz4/zstd)

#### 4.2 生命周期管理

- [ ] `gc` 命令实现
- [ ] 保留策略检查
- [ ] 孤立对象清理

#### 3.9 忽略规则命令

```bash
# 添加忽略规则
$ gam ignore add "*.log"
  ✓ 已添加规则: *.log

$ gam ignore add "temp/"
  ✓ 已添加规则: temp/

$ gam ignore add "!important.dat"
  ✓ 已添加规则: !important.dat

# 移除规则
$ gam ignore remove "*.log"
  ✓ 已移除规则: *.log

# 列出当前规则
$ gam ignore list
  当前忽略规则 (共 5 条):
  1. *.tmp       (内置)
  2. *.temp      (内置)
  3. *.log       (用户)
  4. temp/       (用户)
  5. !key.dat    (用户)

# 检查文件是否会被忽略
$ gam ignore check save.bak
  save.bak → 忽略 (匹配 *.bak)

$ gam ignore check important.dat
  important.dat → 保留 (被 ! 规则显式包含)

# 初始化默认规则模板
$ gam ignore init
  ✓ 已创建 .gamignore 模板
  编辑 .gamignore 自定义规则

# 检查 .gamignore 语法
$ gam ignore --check
  ✓ .gamignore 语法正确
```

#### 4.3 健壮性

- [ ] 存档验证
- [ ] 损坏检测与修复
- [ ] `doctor` 命令

### 阶段 5: 用户体验 (Week 5)

#### 5.1 交互式 Shell

```bash
$ gam shell
gam v2.0.0 > status
gam v2.0.0 > snapshot save -m "Boss beaten"
gam v2.0.0 > timeline switch new_route
gam v2.0.0 > quit
```

- [ ] 交互模式实现
- [ ] Tab 补全（可选）
- [ ] 历史命令

#### 5.2 输出美化

- [ ] 表格格式化
- [ ] 进度条（大文件复制）
- [ ] 颜色主题

---

## 五、存储格式详解

### 5.1 快照对象文件

位置: `.gam/objects/snapshot/{hash_prefix}/{full_hash}`

格式: 压缩的 JSON (gzip)

```json
{
  "id": "abc123...",
  "parent": "def456...",
  "timeline": "main",
  "timestamp": "2024-01-22T10:30:00+08:00",
  "name": "Boss defeated",
  "files": [
    {
      "path": "save.bin",
      "hash": "filehash...",
      "size": 1024000,
      "compressed_size": 900000
    }
  ],
  "content_hash": "combined...",
  "size": 1024000,
  "compression": "gzip",
  "version": "2.0.0"
}
```

### 5.2 内容对象文件

位置: `.gam/objects/content/{hash_prefix}/{full_hash}`

- 原始文件内容或压缩内容
- 根据配置决定是否压缩

### 5.3 索引格式

位置: `.gam/objects/content/index`

```json
{
  "version": 1,
  "entries": [
    {
      "hash": "abc123...",
      "size": 1024000,
      "refcount": 3,
      "path_hint": ["save1.bin", "save2.bin"]
    }
  ]
}
```

### 5.4 活动日志

位置: `.gam/activity.log`

```
2024-01-22T10:30:00Z|snapshot_save|main|abc123...|Boss defeated
2024-01-22T10:25:00Z|timeline_switch|main|new_route|-
2024-01-22T10:20:00Z|restore|abc123...|def456...|-
```

格式: `timestamp|action|timeline|target|source`

---

## 六、向后兼容性

### 6.1 v1.0 迁移

原有结构:
```
Archive/
├── Archive0/
├── Archive1/
└── information.log
```

迁移到 v2.0:

```bash
$ gam migrate
  检测到 v1.0 格式
  正在导入 Archive0...
  正在导入 Archive1...
  ✓ 迁移完成 (2 个快照导入到 'main' 时间线)
  建议: 可安全删除旧的 Archive/ 文件夹
```

迁移逻辑:
1. 读取 `information.log` 获取存档信息
2. 为每个 Archive 创建对应快照
3. 移动内容到新的存储结构

### 6.2 配置兼容

```toml
# v1.0 的 path.txt 迁移
game_path = "..."

# v2.0 配置
[core]
game_path = "..."
```

---

## 七、测试计划

### 7.1 单元测试

- [ ] 快照序列化/反序列化
- [ ] 哈希计算
- [ ] 路径处理
- [ ] 配置解析

### 7.2 集成测试

| 测试场景 | 步骤 | 预期结果 |
|---------|------|---------|
| 完整工作流 | init → save → list → restore | 成功执行 |
| 多时间线 | create → switch → save → list | 分支隔离 |
| 去重检测 | 相同文件多次保存 | 存储不增长 |
| 边界条件 | 空存档、大文件、特殊字符 | 正常处理 |

### 7.3 兼容性测试

- [ ] Windows 路径处理
- [ ] Unicode 文件名
- [ ] 大文件 (>1GB)

---

## 八、任务清单

### Phase 1: 基础框架

- [ ] 1.1 创建项目结构
- [ ] 1.2 实现核心类型 (Snapshot, FileEntry, Timeline, Config)
- [ ] 1.3 实现错误类型
- [ ] 1.4 配置 CLI (clap)
- [ ] 1.5 定义命令参数结构

### Phase 2: 存储引擎

- [ ] 2.1 实现 SHA-256 哈希
- [ ] 2.2 实现 ContentStore
- [ ] 2.3 实现 SnapshotStore
- [ ] 2.4 实现 TimelineManager
- [ ] 2.5 实现配置读写
- [ ] 2.6 实现 GamIgnoreConfig 类型
- [ ] 2.7 实现忽略规则解析器 (glob pattern matching)

### Phase 3: 命令实现

- [ ] 3.1 init 命令
- [ ] 3.2 snapshot save 命令
- [ ] 3.3 snapshot list/info 命令
- [ ] 3.4 snapshot delete 命令
- [ ] 3.5 timeline create/list/switch 命令
- [ ] 3.6 timeline rename/delete 命令
- [ ] 3.7 restore 命令
- [ ] 3.8 history/status/activity 命令

### Phase 4: 高级功能

- [ ] 4.1 文件去重
- [ ] 4.2 gc 命令
- [ ] 4.3 doctor 命令
- [ ] 4.4 diff 命令
- [ ] 4.5 .gamignore 忽略规则引擎
- [ ] 4.6 ignore add/remove/list/check 命令
- [ ] 4.7 ignore init 默认模板生成

### Phase 5: 优化与测试

- [ ] 5.1 输出格式化美化
- [ ] 5.2 交互式 Shell
- [ ] 5.3 单元测试
- [ ] 5.4 集成测试
- [ ] 5.5 v1.0 迁移工具

---

## 九、参考资源

### Rust 库

- `clap` - 命令行解析
- `serde` / `serde_json` - 序列化
- `chrono` - 日期时间
- `sha2` - SHA-256
- `glob` - glob 模式匹配
- `walkdir` - 目录遍历
- `rayon` - 并行处理（可选）
- `indicatif` - 进度条（可选）

### 灵感来源

- Git 官方文档
- libgit2 实现
- zoxide (Rust CLI 最佳实践)

---

## 十、待解决问题

以下问题需要在实现过程中决定：

1. **压缩格式**: gzip, lz4, zstd?
2. **索引格式**: JSON 还是二进制?
3. **快照 ID**: 短格式长度 (5位? 6位?)
4. **并行处理**: 大文件复制是否需要多线程?
5. **冲突检测**: 恢复时检测游戏运行中?

---

> 文档版本: 1.0  
> 创建时间: 2024-01-22  
> 状态: 等待实现
