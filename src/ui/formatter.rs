//! UI 模块
//!
//! 提供命令行输出格式化功能

use crate::core::types::{Snapshot, Timeline};
use crate::utils::FileUtils;
use chrono::{DateTime, Local};
use std::path::PathBuf;

/// 格式化输出工具
pub struct Formatter;

impl Formatter {
    /// 格式化时间戳为可读格式
    pub fn format_time(dt: DateTime<Local>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// 格式化时间戳为简短格式
    pub fn format_time_short(dt: DateTime<Local>) -> String {
        dt.format("%m-%d %H:%M").to_string()
    }

    /// 格式化文件大小
    pub fn format_size(size: u64) -> String {
        FileUtils::format_size(size)
    }

    /// 格式化快照列表表格
    pub fn format_snapshot_table(snapshots: &[Snapshot], timeline_name: &str) -> String {
        if snapshots.is_empty() {
            return format!("{} 分支暂无快照", timeline_name);
        }

        let mut output = format!("{} 分支快照 (共 {} 个)\n", timeline_name, snapshots.len());
        output.push_str("┌───────┬──────────┬────────────────────┬──────────────────────┐\n");
        output.push_str("│ 序号  │   ID     │ 时间               │ 名称                 │\n");
        output.push_str("├───────┼──────────┼────────────────────┼──────────────────────┤\n");

        for (i, snapshot) in snapshots.iter().enumerate() {
            let num = if i < 9 {
                format!("  {}   ", i + 1)
            } else {
                format!("  {:<3}", i + 1)
            };
            let short_id = Self::short_hash(&snapshot.id);
            let time = Self::format_time_short(snapshot.timestamp);
            let name = Self::truncate(&snapshot.name, 20);

            output.push_str(&format!("│{}│ {} │ {} │ {} │\n", num, short_id, time, name));
        }

        output.push_str("└───────┴──────────┴────────────────────┴──────────────────────┘");

        output
    }

    /// 格式化时间线列表
    pub fn format_timeline_list(timelines: &[Timeline], current: Option<&str>) -> String {
        if timelines.is_empty() {
            return String::from("暂无时间线");
        }

        let mut output = String::new();

        for timeline in timelines {
            let marker = if current == Some(&timeline.name) {
                "*"
            } else {
                " "
            };
            output.push_str(&format!(
                "{} {}  (HEAD: {})\n",
                marker,
                timeline.name,
                Self::short_hash(&timeline.head_snapshot)
            ));
        }

        output
    }

    /// 格式化快照详情
    pub fn format_snapshot_detail(snapshot: &Snapshot) -> String {
        let mut output = String::new();

        output.push_str(&format!("快照 ID:    {}\n", snapshot.id));
        output.push_str(&format!("短 ID:      {}\n", Self::short_hash(&snapshot.id)));
        output.push_str(&format!("时间线:     {}\n", snapshot.timeline));
        output.push_str(&format!(
            "创建时间:   {}\n",
            Self::format_time(snapshot.timestamp)
        ));
        output.push_str(&format!("名称:       {}\n", snapshot.name));

        if let Some(desc) = &snapshot.description {
            output.push_str(&format!("描述:       {}\n", desc));
        }

        output.push_str(&format!("文件数量:   {}\n", snapshot.files.len()));
        output.push_str(&format!(
            "总大小:     {}\n",
            Self::format_size(snapshot.size)
        ));

        if let Some(parent) = &snapshot.parent {
            output.push_str(&format!("父快照:     {}\n", Self::short_hash(parent)));
        }

        output.push('\n');
        output.push_str("包含的文件:\n");

        for (i, file) in snapshot.files.iter().enumerate() {
            let path = file.path.to_string_lossy();
            output.push_str(&format!(
                "  {}. {}  ({})\n",
                i + 1,
                path,
                Self::format_size(file.size)
            ));
        }

        output
    }

    /// 格式化状态信息
    pub fn format_status(
        timeline: &str,
        snapshot_count: u32,
        game_size: u64,
        store_size: u64,
    ) -> String {
        let mut output = String::new();

        output.push_str("当前状态:\n");
        output.push_str(&format!("  当前时间线: {}\n", timeline));
        output.push_str(&format!("  快照数量:   {}\n", snapshot_count));
        output.push_str(&format!(
            "  存档大小:   {}\n",
            FileUtils::format_size(game_size)
        ));
        output.push_str(&format!(
            "  存储大小:   {}\n",
            FileUtils::format_size(store_size)
        ));

        if store_size > 0 && game_size > store_size {
            let saved = game_size - store_size;
            let percent = (saved as f64 / game_size as f64 * 100.0) as u32;
            output.push_str(&format!(
                "  节省空间:   {} ({}%)\n",
                FileUtils::format_size(saved),
                percent
            ));
        }

        output
    }

    /// 短哈希格式
    pub fn short_hash(full_hash: &str) -> String {
        if full_hash.len() > 8 {
            (&full_hash[..8]).to_string()
        } else {
            full_hash.to_string()
        }
    }

    /// 截断字符串（正确处理 UTF-8）
    fn truncate(s: &str, max_len: usize) -> String {
        if s.chars().count() <= max_len {
            s.to_string()
        } else {
            let truncated: String = s.chars().take(max_len - 3).collect();
            format!("{}...", truncated)
        }
    }
}

/// 颜色样式
pub struct Colors;

impl Colors {
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}
