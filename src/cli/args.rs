use clap::{Parser, Subcommand, ValueEnum};

/// Game Archive Manager v2.0 - 游戏存档版本控制系统
#[derive(Parser, Debug)]
#[command(name = "gam")]
#[command(author = "Game Archive Manager Contributors")]
#[command(version = "2.0.0")]
#[command(about = "游戏存档版本控制系统 - 像 Git 一样管理游戏存档", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 可用命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化版本控制
    #[command(name = "init")]
    Init(InitArgs),

    /// 快照管理
    #[command(name = "snapshot")]
    Snapshot(SnapshotArgs),

    /// 时间线管理
    #[command(name = "timeline")]
    Timeline(TimelineArgs),

    /// 恢复到快照
    #[command(name = "restore")]
    Restore(RestoreArgs),

    /// 查看历史
    #[command(name = "history")]
    History(HistoryArgs),

    /// 查看状态
    #[command(name = "status")]
    Status(StatusArgs),

    /// 查看活动日志
    #[command(name = "activity")]
    Activity(ActivityArgs),

    /// 比较快照
    #[command(name = "diff")]
    Diff(DiffArgs),

    /// 垃圾回收
    #[command(name = "gc")]
    Gc(GcArgs),

    /// 忽略规则管理
    #[command(name = "ignore")]
    Ignore(IgnoreArgs),

    /// 查看和管理配置
    #[command(name = "config")]
    Config(ConfigArgs),

    /// 诊断问题
    #[command(name = "doctor")]
    Doctor(DoctorArgs),
}

/// init 命令参数
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// 游戏存档目录路径
    #[arg(short, long)]
    pub path: Option<String>,

    /// 强制重新初始化
    #[arg(long)]
    pub force: bool,
}

/// snapshot 子命令参数
#[derive(Parser, Debug)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub command: SnapshotCommands,
}

/// snapshot 子命令
#[derive(Subcommand, Debug)]
pub enum SnapshotCommands {
    /// 保存当前状态为快照
    #[command(name = "save")]
    Save(SaveArgs),

    /// 列出快照
    #[command(name = "list")]
    List(ListArgs),

    /// 查看快照详情
    #[command(name = "info")]
    Info(InfoArgs),

    /// 删除快照
    #[command(name = "delete")]
    Delete(DeleteArgs),

    /// 为快照添加标签
    #[command(name = "tag")]
    Tag(TagArgs),

    /// 列出快照标签
    #[command(name = "tags")]
    Tags(TagsArgs),
}

/// save 命令参数
#[derive(Parser, Debug)]
pub struct SaveArgs {
    /// 快照名称
    #[arg(short, long)]
    pub message: Option<String>,

    /// 保存到指定时间线
    #[arg(short, long)]
    pub timeline: Option<String>,
}

/// list 命令参数
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// 列出所有时间线的快照
    #[arg(long)]
    pub all: bool,

    /// 指定时间线
    #[arg(short, long)]
    pub timeline: Option<String>,
}

/// info 命令参数
#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// 快照 ID（可使用短 ID）
    pub id: String,
}

/// delete 命令参数
#[derive(Parser, Debug)]
pub struct DeleteArgs {
    /// 快照 ID
    pub id: String,

    /// 强制删除（不确认）
    #[arg(long)]
    pub force: bool,
}

/// tag 命令参数
#[derive(Parser, Debug)]
pub struct TagArgs {
    /// 快照 ID
    pub id: String,

    /// 标签名称
    pub name: String,
}

/// tags 命令参数
#[derive(Parser, Debug)]
pub struct TagsArgs {
    /// 快照 ID（可选，不指定则显示所有标签）
    pub id: Option<String>,
}

/// timeline 子命令参数
#[derive(Parser, Debug)]
pub struct TimelineArgs {
    #[command(subcommand)]
    pub command: TimelineCommands,
}

/// timeline 子命令
#[derive(Subcommand, Debug)]
pub enum TimelineCommands {
    /// 创建时间线
    #[command(name = "create")]
    Create(CreateArgs),

    /// 列出时间线
    #[command(name = "list")]
    List,

    /// 切换时间线
    #[command(name = "switch")]
    Switch(SwitchArgs),

    /// 重命名时间线
    #[command(name = "rename")]
    Rename(RenameArgs),

    /// 删除时间线
    #[command(name = "delete")]
    Delete(DeleteTimelineArgs),

    /// 显示当前时间线
    #[command(name = "current")]
    Current,
}

/// create 命令参数
#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// 时间线名称
    pub name: String,

    /// 从指定快照创建
    #[arg(short, long)]
    pub from: Option<String>,
}

/// switch 命令参数
#[derive(Parser, Debug)]
pub struct SwitchArgs {
    /// 时间线名称或快照 ID
    pub target: String,
}

/// rename 命令参数
#[derive(Parser, Debug)]
pub struct RenameArgs {
    /// 旧名称
    pub old_name: String,

    /// 新名称
    pub new_name: String,
}

/// delete 命令参数
#[derive(Parser, Debug)]
pub struct DeleteTimelineArgs {
    /// 时间线名称
    pub name: String,

    /// 强制删除
    #[arg(long)]
    pub force: bool,
}

/// restore 命令参数
#[derive(Parser, Debug)]
pub struct RestoreArgs {
    /// 快照 ID
    pub id: String,

    /// 强制恢复（跳过确认）
    #[arg(long)]
    pub force: bool,
}

/// history 命令参数
#[derive(Parser, Debug)]
pub struct HistoryArgs {
    /// 显示所有时间线的历史
    #[arg(long)]
    pub all: bool,
}

/// status 命令参数
#[derive(Parser, Debug)]
pub struct StatusArgs {
    /// 显示详细信息
    #[arg(short, long)]
    pub verbose: bool,
}

/// activity 命令参数
#[derive(Parser, Debug)]
pub struct ActivityArgs {
    /// 限制显示条数
    #[arg(short, long, default_value = "20")]
    pub limit: u32,
}

/// diff 命令参数
#[derive(Parser, Debug)]
pub struct DiffArgs {
    /// 第一个快照 ID
    pub id1: String,

    /// 第二个快照 ID
    pub id2: String,
}

/// gc 命令参数
#[derive(Parser, Debug)]
pub struct GcArgs {
    /// 强力 GC（清理所有未引用对象）
    #[arg(long)]
    pub aggressive: bool,

    /// 预览模式（不实际执行）
    #[arg(long)]
    pub dry_run: bool,
}

/// ignore 子命令参数
#[derive(Parser, Debug)]
pub struct IgnoreArgs {
    #[command(subcommand)]
    pub command: IgnoreCommands,
}

/// ignore 子命令
#[derive(Subcommand, Debug)]
pub enum IgnoreCommands {
    /// 添加忽略规则
    #[command(name = "add")]
    Add(AddIgnoreArgs),

    /// 移除忽略规则
    #[command(name = "remove")]
    Remove(RemoveIgnoreArgs),

    /// 列出当前规则
    #[command(name = "list")]
    List,

    /// 检查文件是否忽略
    #[command(name = "check")]
    Check(CheckIgnoreArgs),

    /// 初始化默认规则模板
    #[command(name = "init")]
    Init(IgnoreInitArgs),
}

/// add 命令参数
#[derive(Parser, Debug)]
pub struct AddIgnoreArgs {
    /// 模式
    pub pattern: String,
}

/// remove 命令参数
#[derive(Parser, Debug)]
pub struct RemoveIgnoreArgs {
    /// 模式
    pub pattern: String,
}

/// check 命令参数
#[derive(Parser, Debug)]
pub struct CheckIgnoreArgs {
    /// 文件路径
    pub file: String,
}

/// init 命令参数
#[derive(Parser, Debug)]
pub struct IgnoreInitArgs {
    /// 覆盖已存在的文件
    #[arg(long)]
    pub force: bool,
}

/// doctor 命令参数
#[derive(Parser, Debug)]
pub struct DoctorArgs {
    /// 修复检测到的问题
    #[arg(long)]
    pub fix: bool,
}

/// config 命令参数
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// 配置项名称 (如: core.default_timeline)
    pub key: Option<String>,

    /// 配置值
    pub value: Option<String>,

    /// 列出所有配置
    #[arg(long)]
    pub list: bool,
}
