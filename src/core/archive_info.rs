use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 存档信息结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchiveInfo {
    /// 存档时间戳
    pub timestamp: DateTime<Local>,
    /// 存档名称 (最大32字符)
    pub name: String,
    /// 存档备注 (最大1024字符)
    pub comment: String,
}

impl ArchiveInfo {
    /// 创建新的存档信息
    pub fn new(name: String, comment: String) -> Self {
        Self {
            timestamp: Local::now(),
            name,
            comment,
        }
    }

    /// 修改存档信息
    pub fn modify(&mut self, name: Option<String>, comment: Option<String>) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(c) = comment {
            self.comment = c;
        }
    }

    /// 验证名称长度
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("存档名不能为空".to_string());
        }
        if name.len() > 32 {
            return Err("存档名超出32字符限制".to_string());
        }
        Ok(())
    }

    /// 验证备注长度
    pub fn validate_comment(comment: &str) -> Result<(), String> {
        if comment.len() > 1024 {
            return Err("备注超出1024字符限制".to_string());
        }
        Ok(())
    }
}

impl Default for ArchiveInfo {
    fn default() -> Self {
        Self {
            timestamp: Local::now(),
            name: String::new(),
            comment: String::new(),
        }
    }
}
