//! 活动日志模块
//!
//! 提供操作日志记录和查询功能

use crate::core::error::GamResult;
use chrono::{DateTime, Local};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// 操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityAction {
    /// 初始化
    Init,
    /// 保存快照
    SnapshotSave,
    /// 删除快照
    SnapshotDelete,
    /// 恢复
    Restore,
    /// 创建时间线
    TimelineCreate,
    /// 删除时间线
    TimelineDelete,
    /// 切换时间线
    TimelineSwitch,
    /// 重命名时间线
    TimelineRename,
    /// 添加忽略规则
    IgnoreAdd,
    /// 移除忽略规则
    IgnoreRemove,
    /// GC 操作
    Gc,
    /// 未知操作
    Unknown(String),
}

impl std::fmt::Display for ActivityAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityAction::Init => write!(f, "init"),
            ActivityAction::SnapshotSave => write!(f, "snapshot save"),
            ActivityAction::SnapshotDelete => write!(f, "snapshot delete"),
            ActivityAction::Restore => write!(f, "restore"),
            ActivityAction::TimelineCreate => write!(f, "timeline create"),
            ActivityAction::TimelineDelete => write!(f, "timeline delete"),
            ActivityAction::TimelineSwitch => write!(f, "timeline switch"),
            ActivityAction::TimelineRename => write!(f, "timeline rename"),
            ActivityAction::IgnoreAdd => write!(f, "ignore add"),
            ActivityAction::IgnoreRemove => write!(f, "ignore remove"),
            ActivityAction::Gc => write!(f, "gc"),
            ActivityAction::Unknown(name) => write!(f, "{}", name),
        }
    }
}

/// 单条活动记录
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 操作类型
    pub action: ActivityAction,
    /// 时间线
    pub timeline: Option<String>,
    /// 目标（快照 ID 或时间线名称）
    pub target: Option<String>,
    /// 源（仅对切换操作有意义）
    pub source: Option<String>,
}

impl ActivityEntry {
    /// 解析日志行
    pub fn parse(line: &str) -> Option<Self> {
        // 格式: timestamp|action|timeline|target|source
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 {
            return None;
        }

        let timestamp = DateTime::parse_from_rfc3339(parts[0])
            .ok()?
            .with_timezone(&Local);

        let action = parse_action(parts[1]);
        let timeline = if parts.len() > 2 && !parts[2].is_empty() {
            Some(parts[2].to_string())
        } else {
            None
        };
        let target = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].to_string())
        } else {
            None
        };
        let source = if parts.len() > 4 && !parts[4].is_empty() {
            Some(parts[4].to_string())
        } else {
            None
        };

        Some(ActivityEntry {
            timestamp,
            action,
            timeline,
            target,
            source,
        })
    }

    /// 格式化为可读字符串
    pub fn to_display_string(&self) -> String {
        let time = self.timestamp.format("%Y-%m-%d %H:%M");
        let action = self.action.to_string();

        let details = match &self.action {
            ActivityAction::TimelineSwitch | ActivityAction::TimelineRename => {
                if let (Some(from), Some(to)) = (&self.source, &self.target) {
                    format!("{} → {}", from, to)
                } else if let Some(target) = &self.target {
                    target.clone()
                } else {
                    String::new()
                }
            }
            ActivityAction::SnapshotSave => {
                if let Some(target) = &self.target {
                    format!("#{}", target)
                } else {
                    String::new()
                }
            }
            ActivityAction::SnapshotDelete => {
                if let Some(target) = &self.target {
                    format!("#{}", target)
                } else {
                    String::new()
                }
            }
            ActivityAction::Restore => {
                if let Some(target) = &self.target {
                    format!("→ #{}", target)
                } else {
                    String::new()
                }
            }
            _ => {
                if let Some(target) = &self.target {
                    target.clone()
                } else {
                    String::new()
                }
            }
        };

        format!("{}  {}", time, action)
    }
}

/// 解析操作类型
fn parse_action(s: &str) -> ActivityAction {
    match s {
        "init" => ActivityAction::Init,
        "snapshot_save" => ActivityAction::SnapshotSave,
        "snapshot_delete" => ActivityAction::SnapshotDelete,
        "restore" => ActivityAction::Restore,
        "timeline_create" => ActivityAction::TimelineCreate,
        "timeline_delete" => ActivityAction::TimelineDelete,
        "timeline_switch" => ActivityAction::TimelineSwitch,
        "timeline_rename" => ActivityAction::TimelineRename,
        "ignore_add" => ActivityAction::IgnoreAdd,
        "ignore_remove" => ActivityAction::IgnoreRemove,
        "gc" => ActivityAction::Gc,
        _ => ActivityAction::Unknown(s.to_string()),
    }
}

/// 活动日志引擎
#[derive(Debug)]
pub struct ActivityEngine {
    /// 日志文件路径
    log_path: PathBuf,
}

impl ActivityEngine {
    /// 创建新的活动引擎
    pub fn new(gam_dir: &PathBuf) -> Self {
        ActivityEngine {
            log_path: gam_dir.join("activity.log"),
        }
    }

    /// 记录活动
    pub fn log(
        &self,
        action: ActivityAction,
        timeline: Option<&str>,
        target: Option<&str>,
        source: Option<&str>,
    ) -> GamResult<()> {
        let timestamp = Local::now().to_rfc3339();
        let timeline = timeline.unwrap_or("");
        let target = target.unwrap_or("");
        let source = source.unwrap_or("");

        let line = format!(
            "{}|{}|{}|{}|{}\n",
            timestamp, action, timeline, target, source
        );

        // 确保目录存在
        if let Some(parent) = self.log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 追加写入
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// 获取活动记录
    pub fn get_entries(&self, limit: usize) -> GamResult<Vec<ActivityEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.log_path)?;
        let mut entries: Vec<ActivityEntry> = content
            .lines()
            .filter_map(|line| ActivityEntry::parse(line))
            .rev() // 最新的在前
            .take(limit)
            .collect();

        entries.reverse(); // 按时间正序返回
        Ok(entries)
    }

    /// 获取所有活动记录
    pub fn get_all_entries(&self) -> GamResult<Vec<ActivityEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.log_path)?;
        let entries: Vec<ActivityEntry> = content
            .lines()
            .filter_map(|line| ActivityEntry::parse(line))
            .collect();

        Ok(entries)
    }

    /// 获取指定时间线的活动
    pub fn get_entries_by_timeline(
        &self,
        timeline: &str,
        limit: usize,
    ) -> GamResult<Vec<ActivityEntry>> {
        let all = self.get_all_entries()?;
        let filtered: Vec<ActivityEntry> = all
            .into_iter()
            .filter(|e| e.timeline.as_deref() == Some(timeline))
            .rev()
            .take(limit)
            .collect();

        let mut result = filtered;
        result.reverse();
        Ok(result)
    }
}
