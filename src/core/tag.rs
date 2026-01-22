//! 标签模块
//!
//! 提供快照标签的存储和管理功能

use crate::core::error::GamResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

/// 标签存储
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagStore {
    /// 标签映射: tag_name -> snapshot_id
    tags: HashMap<String, String>,
    /// 反向映射: snapshot_id -> Vec<tag_name>
    reverse: HashMap<String, Vec<String>>,
}

impl TagStore {
    /// 创建新的标签存储
    pub fn new(gam_dir: &PathBuf) -> Self {
        let tags_file = gam_dir.join("refs").join("tags.json");
        if tags_file.exists() {
            if let Ok(content) = fs::read_to_string(&tags_file) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        TagStore::default()
    }

    /// 保存标签存储
    pub fn save(&self, gam_dir: &PathBuf) -> GamResult<()> {
        let tags_file = gam_dir.join("refs").join("tags.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&tags_file, content)?;
        Ok(())
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag_name: &str, snapshot_id: &str) -> bool {
        // 检查标签是否已存在
        if self.tags.contains_key(tag_name) {
            return false;
        }

        // 添加标签
        self.tags
            .insert(tag_name.to_string(), snapshot_id.to_string());

        // 更新反向映射
        self.reverse
            .entry(snapshot_id.to_string())
            .or_insert_with(Vec::new)
            .push(tag_name.to_string());

        true
    }

    /// 删除标签
    pub fn remove_tag(&mut self, tag_name: &str) -> bool {
        if let Some(snapshot_id) = self.tags.remove(tag_name) {
            // 从反向映射中移除
            if let Some(tags) = self.reverse.get_mut(&snapshot_id) {
                tags.retain(|t| t != tag_name);
                if tags.is_empty() {
                    self.reverse.remove(&snapshot_id);
                }
            }
            return true;
        }
        false
    }

    /// 获取标签对应的快照 ID
    pub fn get_snapshot_id(&self, tag_name: &str) -> Option<&String> {
        self.tags.get(tag_name)
    }

    /// 获取快照的所有标签
    pub fn get_tags_for_snapshot(&self, snapshot_id: &str) -> Vec<&String> {
        self.reverse
            .get(snapshot_id)
            .map(|v| v.iter())
            .unwrap_or_default()
            .collect()
    }

    /// 获取所有标签
    pub fn all_tags(&self) -> Vec<(&String, &String)> {
        self.tags.iter().collect()
    }

    /// 检查标签是否存在
    pub fn exists(&self, tag_name: &str) -> bool {
        self.tags.contains_key(tag_name)
    }
}

/// 解析标签（从快照 ID 中提取标签部分）
pub fn parse_tag(s: &str) -> Option<&str> {
    // 标签格式: refs/tags/<tag_name>
    if s.starts_with("refs/tags/") {
        Some(&s[10..])
    } else {
        None
    }
}
