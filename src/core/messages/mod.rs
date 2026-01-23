//! Messages module
//!
//! Provides internationalized messages for the application.
//! Messages are defined as key-value pairs with variable interpolation support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Message key type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageKey(pub &'static str);

impl MessageKey {
    pub fn new(key: &'static str) -> Self {
        MessageKey(key)
    }
}

impl fmt::Display for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message with variable interpolation
#[derive(Debug, Clone)]
pub struct Message {
    pub key: MessageKey,
    pub text: String,
}

impl Message {
    pub fn new(key: &'static str, text: String) -> Self {
        Message {
            key: MessageKey(key),
            text,
        }
    }

    /// Interpolate variables into the message
    /// Variables are denoted by {{variable_name}}
    pub fn with(&self, vars: &[(&str, &str)]) -> String {
        let mut result = self.text.clone();
        for (name, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", name), value);
        }
        result
    }

    /// Simple interpolation with key-value pairs
    pub fn interpolate(&self, vars: &[(&str, &str)]) -> String {
        self.with(vars)
    }
}

/// Message catalog containing all messages for a locale
#[derive(Debug, Default)]
pub struct MessageCatalog {
    messages: HashMap<&'static str, String>,
}

impl MessageCatalog {
    pub fn new() -> Self {
        MessageCatalog {
            messages: HashMap::new(),
        }
    }

    /// Add a message to the catalog
    pub fn add(&mut self, key: &'static str, text: &str) {
        self.messages.insert(key, text.to_string());
    }

    /// Get a message by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.messages.get(key)
    }

    /// Get a message or return the key if not found (returns owned String)
    pub fn get_or_default(&self, key: &str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Get a message or return default (returns owned String)
    pub fn get_or_else(&self, key: &str, default: &str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Check if a message exists
    pub fn contains(&self, key: &str) -> bool {
        self.messages.contains_key(key)
    }
}

/// Message manager that handles localization
#[derive(Debug)]
pub struct MessageManager {
    catalogs: HashMap<String, MessageCatalog>,
    current_locale: String,
}

impl MessageManager {
    pub fn new() -> Self {
        let mut manager = MessageManager {
            catalogs: HashMap::new(),
            current_locale: "en".to_string(),
        };

        // Load default catalogs (English first to be primary)
        manager.load_catalog("en", english_catalog());
        manager.load_catalog("zh-CN", chinese_catalog());

        manager
    }

    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) {
        if self.catalogs.contains_key(locale) {
            self.current_locale = locale.to_string();
        }
    }

    /// Get the current locale
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Load a catalog
    pub fn load_catalog(&mut self, locale: &str, catalog: MessageCatalog) {
        self.catalogs.insert(locale.to_string(), catalog);
    }

    /// Get a message for the current locale
    pub fn get(&self, key: &str) -> Option<&String> {
        self.catalogs
            .get(&self.current_locale)
            .and_then(|c| c.get(key))
    }

    /// Get a message or return the key (returns owned String)
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.catalogs
            .get(&self.current_locale)
            .and_then(|c| c.get(key).cloned())
            .unwrap_or_else(|| default.to_string())
    }

    /// Get a message and interpolate variables
    pub fn t(&self, key: &str, vars: &[(&str, &str)]) -> String {
        self.get(key)
            .map(|msg| {
                let mut result = msg.clone();
                for (name, value) in vars {
                    result = result.replace(&format!("{{{{{}}}}}", name), value);
                }
                result
            })
            .unwrap_or_else(|| key.to_string())
    }

    /// Get success message style
    pub fn success(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let base = self.t(key, vars);
        format!("  [成功] {}", base)
    }

    /// Get error message style
    pub fn error(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let base = self.t(key, vars);
        format!("  [错误] {}", base)
    }

    /// Get warning message style
    pub fn warning(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let base = self.t(key, vars);
        format!("  [警告] {}", base)
    }

    /// Get info message style
    pub fn info(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let base = self.t(key, vars);
        format!("  [信息] {}", base)
    }
}

/// Create Chinese (Simplified) message catalog
fn chinese_catalog() -> MessageCatalog {
    let mut cat = MessageCatalog::new();

    // UI messages
    cat.add("ui.success", "成功");
    cat.add("ui.error", "错误");
    cat.add("ui.warning", "警告");
    cat.add("ui.info", "信息");

    // Init command
    cat.add(
        "init.success",
        "初始化完成！\n  游戏存档目录: {{game_path}}\n  GAM 仓库: {{gam_dir}}",
    );
    cat.add(
        "init.error.already_exists",
        "仓库已存在。使用 --force 重新初始化。",
    );
    cat.add(
        "init.error.game_path_not_found",
        "游戏存档目录不存在: {{path}}",
    );

    // Snapshot commands
    cat.add("snapshot.save.success", "已保存快照 {{short_id}} ({{name}})\n  时间线: {{timeline}}\n  文件数: {{file_count}}\n  大小: {{size}}");
    cat.add(
        "snapshot.save.warning.empty",
        "游戏存档目录为空，无文件可保存",
    );
    cat.add(
        "snapshot.save.error.timeline_not_found",
        "时间线不存在: {{timeline}}",
    );
    cat.add("snapshot.list.warning.no_snapshots", "暂无快照");
    cat.add(
        "snapshot.list.info.branch_no_snapshots",
        "{{timeline}} 分支暂无快照",
    );
    cat.add(
        "snapshot.list.info.total_count",
        "{{timeline}} 分支快照 (共 {{count}} 个)",
    );
    cat.add("snapshot.info.success", "快照 ID:    {{id}}\n  短 ID:      {{short_id}}\n  时间线:     {{timeline}}\n  创建时间:   {{timestamp}}\n  名称:       {{name}}\n  文件数量:   {{file_count}}\n  总大小:     {{size}}");
    cat.add(
        "snapshot.delete.success",
        "已删除快照 {{short_id}} ({{name}})",
    );
    cat.add(
        "snapshot.delete.warning.referenced",
        "无法删除快照 {{short_id}}，因为它被以下时间线引用:\n{{timelines}}\n使用 --force 强制删除",
    );
    cat.add("snapshot.delete.warning.confirm", "此操作将永久删除快照。\n  快照: {{short_id}} ({{name}})\n  时间线: {{timeline}}\n  文件数: {{file_count}}");
    cat.add("snapshot.delete.warning.cancelled", "操作已取消");
    cat.add("snapshot.delete.error.not_found", "快照未找到: {{id}}");
    cat.add(
        "snapshot.tag.success",
        "已为快照 {{short_id}} 添加标签 '{{tag_name}}'",
    );
    cat.add("snapshot.tag.error.not_found", "快照未找到: {{id}}");
    cat.add(
        "snapshot.tag.error.tag_exists",
        "标签 '{{tag_name}}' 已存在",
    );
    cat.add(
        "snapshot.tag.error.invalid_name",
        "无效的标签名称: {{name}}",
    );

    // Timeline commands
    cat.add("timeline.create.success", "已创建时间线 '{{name}}'");
    cat.add(
        "timeline.create.info.from_snapshot",
        "  从快照 {{short_id}} ({{snapshot_name}}) 创建",
    );
    cat.add(
        "timeline.create.error.already_exists",
        "时间线 '{{name}}' 已存在",
    );
    cat.add(
        "timeline.create.error.snapshot_not_found",
        "快照未找到: {{id}}",
    );
    cat.add("timeline.list.warning.no_timelines", "暂无时间线");
    cat.add("timeline.switch.success", "已切换到时间线 '{{name}}'");
    cat.add(
        "timeline.switch.info.detached",
        "HEAD 现在指向快照 {{short_id}} ({{name}})",
    );
    cat.add("timeline.switch.error.not_found", "时间线不存在: {{name}}");
    cat.add(
        "timeline.switch.error.invalid_snapshot",
        "无效的快照 ID: {{id}}",
    );
    cat.add(
        "timeline.rename.success",
        "已将时间线 '{{old_name}}' 重命名为 '{{new_name}}'",
    );
    cat.add("timeline.rename.error.not_found", "时间线未找到: {{name}}");
    cat.add(
        "timeline.rename.error.already_exists",
        "时间线 '{{name}}' 已存在",
    );
    cat.add(
        "timeline.rename.error.invalid_name",
        "无效的时间线名称: {{name}}",
    );
    cat.add("timeline.delete.success", "已删除时间线 '{{name}}'");
    cat.add(
        "timeline.delete.warning.current_timeline",
        "无法删除当前时间线 '{{name}}'。请先切换到其他时间线。",
    );
    cat.add(
        "timeline.delete.warning.confirm",
        "此操作将永久删除时间线 '{{name}}' 及其所有快照。",
    );
    cat.add("timeline.delete.warning.cancelled", "操作已取消");
    cat.add("timeline.delete.error.not_found", "时间线未找到: {{name}}");
    cat.add("timeline.current.success", "当前时间线: {{name}}");
    cat.add(
        "timeline.current.info.detached",
        "当前没有活动的时间线 (分离 HEAD 状态)",
    );

    // Restore command
    cat.add(
        "restore.success",
        "已恢复到快照 {{short_id}} ({{name}})\n  恢复了 {{count}} 个文件",
    );
    cat.add("restore.warning.confirm", "此操作将覆盖当前存档。\n  快照: {{short_id}} ({{name}})\n  时间线: {{timeline}}\n  文件数: {{file_count}}");
    cat.add("restore.warning.cancelled", "操作已取消");
    cat.add("restore.error.not_found", "快照未找到: {{id}}");
    cat.add("restore.error.restore_failed", "恢复失败: {{error}}");

    // History command
    cat.add("history.warning.no_history", "暂无历史记录");

    // Activity command
    cat.add("activity.warning.no_records", "暂无活动记录");

    // Status command
    cat.add("status.success", "当前状态:\n  当前时间线: {{timeline}}\n  快照数量: {{snapshot_count}}\n  存档大小: {{game_size}}\n  存储大小: {{store_size}}");
    cat.add(
        "status.info.savings",
        "节省空间:   {{saved}} ({{percent}}%)",
    );

    // GC command
    cat.add("gc.success", "垃圾回收完成:\n  清理了 {{content_count}} 个未引用的内容文件 ({{content_size}})\n  清理了 {{snapshot_count}} 个孤立快照 ({{snapshot_size}})\n  总计释放空间: {{total_freed}}");
    cat.add("gc.warning.dry_run", "预览模式 - 将清理以下内容:\n  未引用的内容文件: {{content_count}} 个 ({{content_size}})\n  孤立快照: {{snapshot_count}} 个 ({{snapshot_size}})\n  总计将释放: {{total_freed}}");
    cat.add("gc.warning.nothing_to_clean", "没有需要清理的内容");

    // Doctor command
    cat.add("doctor.success", "仓库状态良好，没有发现问题");
    cat.add("doctor.warning.issues_found", "发现 {{count}} 个问题:");
    cat.add("doctor.warning.fixed", "已修复 {{count}} 个问题");

    // Ignore commands
    cat.add("ignore.add.success", "已添加规则: {{pattern}}");
    cat.add("ignore.remove.success", "已移除规则: {{pattern}}");
    cat.add("ignore.remove.warning.not_found", "未找到规则: {{pattern}}");
    cat.add("ignore.list.warning.no_rules", "暂无忽略规则");
    cat.add("ignore.check.info.ignored", "忽略");
    cat.add("ignore.check.info.not_ignored", "不忽略");
    cat.add("ignore.check.info.reason", "原因: {{reason}}");
    cat.add("ignore.init.success", "已创建 .gamignore 模板");
    cat.add("ignore.error.file_not_exists", ".gamignore 不存在");
    cat.add(
        "ignore.error.already_exists",
        ".gamignore 已存在，使用 --force 覆盖",
    );

    // Config command
    cat.add("config.success_set", "已设置 {{key}} = {{value}}");
    cat.add("config.success_add", "已添加 {{key}} = {{value}}");
    cat.add("config.warning.not_found", "配置项 '{{key}}' 不存在");
    cat.add(
        "config.warning.section_not_found",
        "配置段 '[{{section}}]' 不存在",
    );
    cat.add(
        "config.error.invalid_format",
        "无效的配置项格式，使用 'section.key' 格式",
    );

    // Common errors
    cat.add(
        "common.error.not_initialized",
        "Not a gam repository (missing .gam directory). Run 'gam init' first.",
    );
    cat.add(
        "common.error.game_path_not_found",
        "游戏存档目录不存在: {{path}}",
    );

    cat
}

/// Create English message catalog
fn english_catalog() -> MessageCatalog {
    let mut cat = MessageCatalog::new();

    // UI messages
    cat.add("ui.success", "Success");
    cat.add("ui.error", "Error");
    cat.add("ui.warning", "Warning");
    cat.add("ui.info", "Info");

    // Init command
    cat.add("init.success", "Initialization complete!\n  Game saves directory: {{game_path}}\n  GAM repository: {{gam_dir}}");
    cat.add(
        "init.error.already_exists",
        "Repository already exists. Use --force to reinitialize.",
    );
    cat.add(
        "init.error.game_path_not_found",
        "Game saves directory not found: {{path}}",
    );

    // Snapshot commands
    cat.add("snapshot.save.success", "Snapshot saved {{short_id}} ({{name}})\n  Timeline: {{timeline}}\n  Files: {{file_count}}\n  Size: {{size}}");
    cat.add(
        "snapshot.save.warning.empty",
        "Game saves directory is empty, no files to save",
    );
    cat.add(
        "snapshot.save.error.timeline_not_found",
        "Timeline not found: {{timeline}}",
    );
    cat.add("snapshot.list.warning.no_snapshots", "No snapshots");
    cat.add(
        "snapshot.list.info.branch_no_snapshots",
        "{{timeline}} branch has no snapshots",
    );
    cat.add(
        "snapshot.list.info.total_count",
        "{{timeline}} branch snapshots ({{count}} total)",
    );
    cat.add("snapshot.info.success", "Snapshot ID:    {{id}}\n  Short ID:     {{short_id}}\n  Timeline:     {{timeline}}\n  Created:      {{timestamp}}\n  Name:         {{name}}\n  Files:        {{file_count}}\n  Size:         {{size}}");
    cat.add(
        "snapshot.delete.success",
        "Snapshot {{short_id}} ({{name}}) deleted",
    );
    cat.add("snapshot.delete.warning.referenced", "Cannot delete snapshot {{short_id}} because it is referenced by:\n{{timelines}}\nUse --force to force delete");
    cat.add("snapshot.delete.warning.confirm", "This will permanently delete the snapshot.\n  Snapshot: {{short_id}} ({{name}})\n  Timeline: {{timeline}}\n  Files: {{file_count}}");
    cat.add("snapshot.delete.warning.cancelled", "Operation cancelled");
    cat.add(
        "snapshot.delete.error.not_found",
        "Snapshot not found: {{id}}",
    );
    cat.add(
        "snapshot.tag.success",
        "Tag '{{tag_name}}' added to snapshot {{short_id}}",
    );
    cat.add("snapshot.tag.error.not_found", "Snapshot not found: {{id}}");
    cat.add(
        "snapshot.tag.error.tag_exists",
        "Tag '{{tag_name}}' already exists",
    );
    cat.add(
        "snapshot.tag.error.invalid_name",
        "Invalid tag name: {{name}}",
    );

    // Timeline commands
    cat.add("timeline.create.success", "Timeline '{{name}}' created");
    cat.add(
        "timeline.create.info.from_snapshot",
        "  From snapshot {{short_id}} ({{snapshot_name}})",
    );
    cat.add(
        "timeline.create.error.already_exists",
        "Timeline '{{name}}' already exists",
    );
    cat.add(
        "timeline.create.error.snapshot_not_found",
        "Snapshot not found: {{id}}",
    );
    cat.add("timeline.list.warning.no_timelines", "No timelines");
    cat.add("timeline.switch.success", "Switched to timeline '{{name}}'");
    cat.add(
        "timeline.switch.info.detached",
        "HEAD now points to snapshot {{short_id}} ({{name}})",
    );
    cat.add(
        "timeline.switch.error.not_found",
        "Timeline not found: {{name}}",
    );
    cat.add(
        "timeline.switch.error.invalid_snapshot",
        "Invalid snapshot ID: {{id}}",
    );
    cat.add(
        "timeline.rename.success",
        "Timeline '{{old_name}}' renamed to '{{new_name}}'",
    );
    cat.add(
        "timeline.rename.error.not_found",
        "Timeline not found: {{name}}",
    );
    cat.add(
        "timeline.rename.error.already_exists",
        "Timeline '{{name}}' already exists",
    );
    cat.add(
        "timeline.rename.error.invalid_name",
        "Invalid timeline name: {{name}}",
    );
    cat.add("timeline.delete.success", "Timeline '{{name}}' deleted");
    cat.add(
        "timeline.delete.warning.current_timeline",
        "Cannot delete current timeline '{{name}}'. Please switch to another timeline first.",
    );
    cat.add(
        "timeline.delete.warning.confirm",
        "This will permanently delete timeline '{{name}}' and all its snapshots.",
    );
    cat.add("timeline.delete.warning.cancelled", "Operation cancelled");
    cat.add(
        "timeline.delete.error.not_found",
        "Timeline not found: {{name}}",
    );
    cat.add("timeline.current.success", "Current timeline: {{name}}");
    cat.add(
        "timeline.current.info.detached",
        "No active timeline (detached HEAD state)",
    );

    // Restore command
    cat.add(
        "restore.success",
        "Restored to snapshot {{short_id}} ({{name}})\n  {{count}} files restored",
    );
    cat.add("restore.warning.confirm", "This will overwrite the current save.\n  Snapshot: {{short_id}} ({{name}})\n  Timeline: {{timeline}}\n  Files: {{file_count}}");
    cat.add("restore.warning.cancelled", "Operation cancelled");
    cat.add("restore.error.not_found", "Snapshot not found: {{id}}");
    cat.add("restore.error.restore_failed", "Restore failed: {{error}}");

    // History command
    cat.add("history.warning.no_history", "No history");

    // Activity command
    cat.add("activity.warning.no_records", "No activity records");

    // Status command
    cat.add("status.success", "Status:\n  Current timeline: {{timeline}}\n  Snapshots: {{snapshot_count}}\n  Save size: {{game_size}}\n  Storage size: {{store_size}}");
    cat.add(
        "status.info.savings",
        "Space saved:   {{saved}} ({{percent}}%)",
    );

    // GC command
    cat.add("gc.success", "Garbage collection complete:\n  Cleaned {{content_count}} unreferenced content files ({{content_size}})\n  Cleaned {{snapshot_count}} orphaned snapshots ({{snapshot_size}})\n  Total space freed: {{total_freed}}");
    cat.add("gc.warning.dry_run", "Preview mode - will clean:\n  Unreferenced content files: {{content_count}} ({{content_size}})\n  Orphaned snapshots: {{snapshot_count}} ({{snapshot_size}})\n  Total to free: {{total_freed}}");
    cat.add("gc.warning.nothing_to_clean", "Nothing to clean");

    // Doctor command
    cat.add("doctor.success", "Repository is healthy, no issues found");
    cat.add("doctor.warning.issues_found", "Found {{count}} issues:");
    cat.add("doctor.warning.fixed", "Fixed {{count}} issues");

    // Ignore commands
    cat.add("ignore.add.success", "Rule added: {{pattern}}");
    cat.add("ignore.remove.success", "Rule removed: {{pattern}}");
    cat.add(
        "ignore.remove.warning.not_found",
        "Rule not found: {{pattern}}",
    );
    cat.add("ignore.list.warning.no_rules", "No ignore rules");
    cat.add("ignore.check.info.ignored", "Ignored");
    cat.add("ignore.check.info.not_ignored", "Not ignored");
    cat.add("ignore.check.info.reason", "Reason: {{reason}}");
    cat.add("ignore.init.success", ".gamignore template created");
    cat.add("ignore.error.file_not_exists", ".gamignore does not exist");
    cat.add(
        "ignore.error.already_exists",
        ".gamignore already exists, use --force to overwrite",
    );

    // Config command
    cat.add("config.success_set", "Set {{key}} = {{value}}");
    cat.add("config.success_add", "Added {{key}} = {{value}}");
    cat.add("config.warning.not_found", "Config key '{{key}}' not found");
    cat.add(
        "config.warning.section_not_found",
        "Config section '[{{section}}]' not found",
    );
    cat.add(
        "config.error.invalid_format",
        "Invalid config format, use 'section.key' format",
    );

    // Common errors
    cat.add(
        "common.error.not_initialized",
        "Not a gam repository (missing .gam directory). Run 'gam init' first.",
    );
    cat.add(
        "common.error.game_path_not_found",
        "Game saves directory not found: {{path}}",
    );

    cat
}

/// Get the global message manager instance
pub fn global_messages() -> MessageManager {
    MessageManager::new()
}
