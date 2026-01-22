use std::path::PathBuf;
use thiserror::Error;

/// Game Archive Manager 错误类型
#[derive(Error, Debug)]
pub enum GamError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 哈希计算错误
    #[error("Hash error: {0}")]
    Hash(String),

    /// 路径解析错误
    #[error("Path error: {0}")]
    Path(String),

    /// 文件未找到
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    /// 快照未找到
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    /// 时间线未找到
    #[error("Timeline not found: {0}")]
    TimelineNotFound(String),

    /// 时间线已存在
    #[error("Timeline already exists: {0}")]
    TimelineExists(String),

    /// 仓库已初始化
    #[error("Repository already initialized. Use --force to reinitialize.")]
    AlreadyInitialized,

    /// 快照已存在（不应该发生，但用于去重检测）
    #[error("Snapshot already exists: {0}")]
    SnapshotExists(String),

    /// 无效的快照 ID
    #[error("Invalid snapshot ID: {0}")]
    InvalidSnapshotId(String),

    /// 无效的时间线名称
    #[error("Invalid timeline name: {0}")]
    InvalidTimelineName(String),

    /// 无效的标签名称
    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),

    /// 无效的配置
    #[error("Invalid config: {0}")]
    InvalidConfig(String),

    /// 配置文件不存在
    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    /// 游戏存档目录不存在
    #[error("Game path not found: {0}")]
    GamePathNotFound(PathBuf),

    /// .gam 目录不存在（未初始化）
    #[error("Not a gam repository (missing .gam directory). Run 'gam init' first.")]
    NotInitialized,

    /// HEAD 指针无效
    #[error("Invalid HEAD reference: {0}")]
    InvalidHead(String),

    /// 恢复冲突
    #[error("Restore conflict: {0}")]
    RestoreConflict(String),

    /// 忽略规则解析错误
    #[error("Ignore pattern error: {0}")]
    IgnorePattern(String),

    /// 用户取消操作
    #[error("Operation cancelled by user")]
    Cancelled,

    /// 权限错误
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// TOML 解析错误
    #[error("TOML parse error: {0}")]
    TomlParse(String),

    /// 未知错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 结果类型别名
pub type GamResult<T> = Result<T, GamError>;
