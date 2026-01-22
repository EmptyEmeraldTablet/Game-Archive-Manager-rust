use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 快照元数据
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
    /// 快照描述（可选，详细说明）
    pub description: Option<String>,
    /// 包含的文件列表
    pub files: Vec<FileEntry>,
    /// 所有内容的组合哈希
    pub content_hash: String,
    /// 快照总大小（字节）
    pub size: u64,
    /// 压缩算法（none, gzip）
    pub compression: String,
    /// GAM 版本
    pub version: String,
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot {
            id: String::new(),
            parent: None,
            timeline: String::new(),
            timestamp: Local::now(),
            name: String::new(),
            description: None,
            files: Vec::new(),
            content_hash: String::new(),
            size: 0,
            compression: String::from("none"),
            version: String::from("2.0.0"),
        }
    }
}

/// 文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// 相对于存档目录的路径
    pub path: PathBuf,
    /// 内容哈希 (SHA256)
    pub hash: String,
    /// 文件大小
    pub size: u64,
    /// 压缩后大小（可选）
    pub compressed_size: Option<u64>,
}

impl FileEntry {
    pub fn new(path: PathBuf, hash: String, size: u64) -> Self {
        FileEntry {
            path,
            hash,
            size,
            compressed_size: None,
        }
    }
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

impl Timeline {
    pub fn new(name: String, head_snapshot: String, description: Option<String>) -> Self {
        Timeline {
            name,
            head_snapshot,
            created_at: Local::now(),
            description,
        }
    }
}

/// 存储策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageStrategy {
    /// 全量复制
    #[serde(rename = "full")]
    FullCopy,
    /// 内容去重
    #[serde(rename = "deduplication")]
    Deduplication,
    /// 压缩存储
    #[serde(rename = "compression")]
    Compression,
}

impl Default for StorageStrategy {
    fn default() -> Self {
        StorageStrategy::Deduplication
    }
}

/// 保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// 保留天数 (0 = 无限)
    #[serde(default = "default_keep_days")]
    pub keep_days: u32,
    /// 保留快照数 (0 = 无限)
    #[serde(default = "default_keep_count")]
    pub keep_count: u32,
}

fn default_keep_days() -> u32 {
    0
}

fn default_keep_count() -> u32 {
    0
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy {
            keep_days: 0,
            keep_count: 0,
        }
    }
}

/// 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 游戏存档目录
    #[serde(default)]
    pub game_path: PathBuf,
    /// 默认时间线名称
    #[serde(default = "default_timeline_name")]
    pub default_timeline: String,
    /// 存储策略
    #[serde(default)]
    pub storage_strategy: StorageStrategy,
    /// 保留策略
    #[serde(default)]
    pub retention: RetentionPolicy,
    /// 是否启用 .gamignore
    #[serde(default = "default_true")]
    pub use_gamignore: bool,
    /// 版本
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_timeline_name() -> String {
    String::from("main")
}

fn default_true() -> bool {
    true
}

fn default_version() -> String {
    String::from("2.0.0")
}

impl Default for Config {
    fn default() -> Self {
        Config {
            game_path: PathBuf::new(),
            default_timeline: String::from("main"),
            storage_strategy: StorageStrategy::default(),
            retention: RetentionPolicy::default(),
            use_gamignore: true,
            version: String::from("2.0.0"),
        }
    }
}

/// .gamignore 忽略规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamIgnoreConfig {
    /// 是否启用 .gamignore
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 忽略规则列表
    #[serde(default)]
    pub patterns: Vec<IgnorePattern>,
    /// 规则文件路径
    #[serde(default)]
    pub ignore_file: PathBuf,
}

impl Default for GamIgnoreConfig {
    fn default() -> Self {
        GamIgnoreConfig {
            enabled: true,
            patterns: Vec::new(),
            ignore_file: PathBuf::from(".gamignore"),
        }
    }
}

/// 忽略模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnorePattern {
    /// 原始模式字符串
    pub pattern: String,
    /// 是否为否定模式 (! 开头)
    #[serde(default)]
    pub negated: bool,
    /// 模式类型
    #[serde(default)]
    pub pattern_type: PatternType,
}

impl IgnorePattern {
    pub fn new(pattern: String) -> Self {
        let (negated, pattern) = if pattern.starts_with('!') {
            (true, pattern[1..].to_string())
        } else {
            (false, pattern)
        };

        let pattern_type = if pattern.starts_with('/') {
            PatternType::RootFile(pattern[1..].to_string())
        } else if pattern.ends_with('/') {
            PatternType::Directory(pattern[..pattern.len() - 1].to_string())
        } else if pattern.contains("**") {
            PatternType::Recursive(pattern.clone())
        } else {
            PatternType::Glob(pattern.clone())
        };

        IgnorePattern {
            pattern,
            negated,
            pattern_type,
        }
    }
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    /// 通配符匹配 (glob)
    #[serde(rename = "glob")]
    Glob(String),
    /// 目录匹配
    #[serde(rename = "directory")]
    Directory(String),
    /// 根目录特定文件 (以 / 开头)
    #[serde(rename = "root_file")]
    RootFile(String),
    /// 递归匹配 (**/)
    #[serde(rename = "recursive")]
    Recursive(String),
}

impl Default for PatternType {
    fn default() -> Self {
        PatternType::Glob(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_default() {
        let snapshot = Snapshot::default();
        assert!(snapshot.id.is_empty());
        assert!(snapshot.files.is_empty());
        assert_eq!(snapshot.compression, "none");
        assert_eq!(snapshot.version, "2.0.0");
    }

    #[test]
    fn test_file_entry_new() {
        let entry = FileEntry::new(PathBuf::from("test/save.dat"), "abc123".to_string(), 1024);
        assert_eq!(entry.path, PathBuf::from("test/save.dat"));
        assert_eq!(entry.hash, "abc123");
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_timeline_default() {
        let timeline = Timeline::new(
            "main".to_string(),
            "snapshot123".to_string(),
            Some("Test timeline".to_string()),
        );
        assert_eq!(timeline.name, "main");
        assert_eq!(timeline.head_snapshot, "snapshot123");
        assert_eq!(timeline.description, Some("Test timeline".to_string()));
    }

    #[test]
    fn test_ignore_pattern_new() {
        let pattern = IgnorePattern::new("*.tmp".to_string());
        assert_eq!(pattern.pattern, "*.tmp");
        assert!(!pattern.negated);
        assert_eq!(pattern.pattern_type, PatternType::Glob("*.tmp".to_string()));
    }

    #[test]
    fn test_ignore_pattern_negated() {
        let mut pattern = IgnorePattern::new("important.dat".to_string());
        pattern.negated = true;
        assert!(pattern.negated);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_timeline, "main");
        assert!(config.use_gamignore);
        assert_eq!(config.version, "2.0.0");
    }

    #[test]
    fn test_gam_ignore_config_default() {
        let config = GamIgnoreConfig::default();
        assert!(config.enabled);
        assert!(config.patterns.is_empty());
        assert_eq!(config.ignore_file, PathBuf::from(".gamignore"));
    }

    #[test]
    fn test_storage_strategy_default() {
        let strategy = StorageStrategy::default();
        match strategy {
            StorageStrategy::Deduplication => {}
            StorageStrategy::FullCopy => {}
            StorageStrategy::Compression => {}
        }
    }

    #[test]
    fn test_retention_policy_default() {
        let policy = RetentionPolicy::default();
        // Default values are 0 (unlimited)
        assert!(policy.keep_days >= 0);
        assert!(policy.keep_count >= 0);
    }
}
