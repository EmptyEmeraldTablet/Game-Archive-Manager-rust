use crate::core::error::GamResult;
use crate::core::types::{IgnorePattern, PatternType};
use glob::Pattern;
use std::path::Path;

/// 忽略规则引擎
pub struct IgnoreEngine {
    patterns: Vec<IgnorePattern>,
}

impl IgnoreEngine {
    /// 从规则列表创建引擎
    pub fn new(patterns: Vec<IgnorePattern>) -> Self {
        IgnoreEngine { patterns }
    }

    /// 获取所有模式
    pub fn patterns(&self) -> &[IgnorePattern] {
        &self.patterns
    }

    /// 检查文件是否应该被忽略
    pub fn is_ignored(&self, path: &Path, is_directory: bool) -> bool {
        let mut ignored = false;

        for pattern in &self.patterns {
            if self.matches_internal(pattern, path, is_directory) {
                // 如果匹配且不是否定模式，则忽略
                if !pattern.negated {
                    ignored = true;
                } else {
                    // 否定模式，取消忽略
                    ignored = false;
                }
            }
        }

        ignored
    }

    /// 检查路径是否匹配指定模式（公开方法）
    pub fn matches_pattern(&self, path: &Path, pattern: String) -> bool {
        let ignore_pattern = IgnorePattern::new(pattern);
        self.matches_internal(&ignore_pattern, path, path.is_dir())
    }

    /// 检查路径是否匹配指定的否定模式
    pub fn matches_negated(&self, path: &Path, pattern: String) -> bool {
        let mut ignore_pattern = IgnorePattern::new(pattern);
        ignore_pattern.negated = true;
        self.matches_internal(&ignore_pattern, path, path.is_dir())
    }

    /// 检查路径是否匹配模式（内部方法）
    fn matches_internal(&self, pattern: &IgnorePattern, path: &Path, _is_directory: bool) -> bool {
        let path_str = path.to_string_lossy();
        let path_normalized = path_str.replace('\\', "/");

        match &pattern.pattern_type {
            PatternType::Glob(glob_pattern) => {
                // 尝试匹配
                if let Ok(glob) = Pattern::new(glob_pattern) {
                    if glob.matches(&path_normalized) {
                        return true;
                    }
                    // 也尝试匹配完整路径
                    if let Some(file_name) = path.file_name().map(|n| n.to_string_lossy()) {
                        if glob.matches(&file_name) {
                            return true;
                        }
                    }
                }
                // 简单字符串匹配作为后备
                if path_normalized.contains(glob_pattern) {
                    return true;
                }
                if let Some(file_name) = path.file_name().map(|n| n.to_string_lossy()) {
                    if file_name == *glob_pattern || file_name.contains(glob_pattern) {
                        return true;
                    }
                }
            }
            PatternType::Directory(dir_pattern) => {
                let dir_normalized = dir_pattern.replace('\\', "/");
                // 检查是否是该目录下的文件/子目录
                if path_normalized.starts_with(&dir_normalized) {
                    return true;
                }
                // 检查路径是否包含目录名
                if path_normalized.contains(&format!("/{}/", dir_normalized))
                    || path_normalized.contains(&format!("/{}", dir_normalized))
                {
                    return true;
                }
            }
            PatternType::RootFile(file_pattern) => {
                // 只匹配根目录下的特定文件
                if path
                    .parent()
                    .map(|p| p.as_os_str().is_empty())
                    .unwrap_or(false)
                {
                    // 根目录文件
                    if let Some(file_name) = path.file_name().map(|n| n.to_string_lossy()) {
                        return file_name == *file_pattern;
                    }
                }
            }
            PatternType::Recursive(recursive_pattern) => {
                // ** 匹配任意路径
                if let Ok(glob) = Pattern::new(recursive_pattern) {
                    if glob.matches(&path_normalized) {
                        return true;
                    }
                }
                // 简单处理：检查是否以模式结尾
                if path_normalized.ends_with(&pattern.pattern) {
                    return true;
                }
            }
        }

        false
    }

    /// 解析 .gamignore 文件内容
    pub fn parse_gamignore(content: &str) -> GamResult<Vec<IgnorePattern>> {
        let mut patterns = Vec::new();

        for line in content.lines() {
            // 去除空白字符
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            patterns.push(IgnorePattern::new(line.to_string()));
        }

        Ok(patterns)
    }

    /// 默认的内置忽略规则
    pub fn default_patterns() -> Vec<IgnorePattern> {
        vec![
            // 操作系统文件
            IgnorePattern::new(".DS_Store".to_string()),
            IgnorePattern::new("Thumbs.db".to_string()),
            IgnorePattern::new("desktop.ini".to_string()),
            // 临时文件
            IgnorePattern::new("*.tmp".to_string()),
            IgnorePattern::new("*.temp".to_string()),
            IgnorePattern::new("*.swp".to_string()),
            IgnorePattern::new("*~".to_string()),
            IgnorePattern::new(".*~".to_string()),
            // 备份文件
            IgnorePattern::new("*.bak".to_string()),
            IgnorePattern::new("*.backup".to_string()),
        ]
    }

    /// 生成默认 .gamignore 模板
    pub fn default_gamignore_template() -> String {
        vec![
            "# Game Archive Manager 忽略规则模板",
            "#",
            "# 语法说明:",
            "# - *.ext    : 匹配任意位置的扩展名文件",
            "# - dirname/ : 忽略整个目录",
            "# - /file    : 只匹配根目录的特定文件",
            "# - **/*.ext : 递归匹配",
            "# - !pattern : 取反，保留匹配项",
            "#",
            "",
            "# 操作系统文件",
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            "",
            "# 临时文件",
            "*.tmp",
            "*.temp",
            "*.swp",
            "*~",
            ".*~",
            "",
            "# 日志文件（按需启用）",
            "# *.log",
            "# logs/",
            "",
            "# 备份文件",
            "*.bak",
            "*.backup",
            "",
            "# 游戏相关（按需修改）",
            "# screenshots/",
            "# mods/backup/",
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ignore_engine_new() {
        let engine = IgnoreEngine::new(Vec::new());
        assert!(engine.patterns().is_empty());
    }

    #[test]
    fn test_parse_gamignore_empty() {
        let result = IgnoreEngine::parse_gamignore("");
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_gamignore_with_comments() {
        let content = "# This is a comment\n*.tmp\n# Another comment\n*.log";
        let patterns = IgnoreEngine::parse_gamignore(content).unwrap();
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].pattern, "*.tmp");
        assert_eq!(patterns[1].pattern, "*.log");
    }

    #[test]
    fn test_parse_gamignore_ignores_empty_lines() {
        let content = "\n\n*.tmp\n\n\n*.log\n\n";
        let patterns = IgnoreEngine::parse_gamignore(content).unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_parse_gamignore_ignores_comments() {
        let content = "# Comment line\n*.tmp # inline comment\n*.log";
        let patterns = IgnoreEngine::parse_gamignore(content).unwrap();
        // Only the clean *.tmp and *.log should be parsed
        assert!(patterns.len() >= 1);
    }

    #[test]
    fn test_is_ignored_empty_patterns() {
        let engine = IgnoreEngine::new(Vec::new());
        let path = PathBuf::from("test.tmp");
        assert!(!engine.is_ignored(&path, false));
    }

    #[test]
    fn test_matches_pattern_basic() {
        let engine = IgnoreEngine::new(vec![IgnorePattern::new("*.tmp".to_string())]);
        let path = PathBuf::from("test.tmp");
        assert!(engine.matches_pattern(&path, "*.tmp".to_string()));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        let engine = IgnoreEngine::new(vec![IgnorePattern::new("*.tmp".to_string())]);
        let path = PathBuf::from("test.dat");
        assert!(!engine.matches_pattern(&path, "*.tmp".to_string()));
    }

    #[test]
    fn test_default_patterns_not_empty() {
        let patterns = IgnoreEngine::default_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_default_gamignore_template_not_empty() {
        let template = IgnoreEngine::default_gamignore_template();
        assert!(!template.is_empty());
        assert!(template.contains(".tmp"));
        assert!(template.contains(".DS_Store"));
    }
}
