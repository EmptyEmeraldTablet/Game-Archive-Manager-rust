use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 文件工具函数
pub struct FileUtils;

impl FileUtils {
    /// 获取目录总大小
    pub fn get_dir_size(path: &Path) -> io::Result<u64> {
        let mut total_size: u64 = 0;

        if path.is_file() {
            return Ok(fs::metadata(path)?.len());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total_size += Self::get_dir_size(&path)?;
            } else {
                total_size += fs::metadata(&path)?.len();
            }
        }

        Ok(total_size)
    }

    /// 格式化字节大小为可读格式
    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }

    /// 递归复制目录
    pub fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<u64> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }

        let mut total_copied = 0u64;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;

            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                total_copied += Self::copy_dir_all(&entry.path(), &dst_path)?;
            } else {
                fs::copy(&entry.path(), &dst_path)?;
                total_copied += dst_path.metadata()?.len();
            }
        }

        Ok(total_copied)
    }

    /// 获取目录下所有文件（递归）
    pub fn get_all_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        files
    }

    /// 检查路径是否为绝对路径
    pub fn is_absolute_path(path: &Path) -> bool {
        path.is_absolute()
    }

    /// 创建相对路径（相对于基准目录）
    pub fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
        path.strip_prefix(base).ok().map(|p| p.to_path_buf())
    }

    /// 规范化路径（移除 .. 和 .）
    pub fn normalize_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                std::path::Component::Normal(c) => normalized.push(c),
                std::path::Component::ParentDir => {
                    if normalized.pop() {
                        // 成功 pop
                    }
                }
                std::path::Component::CurDir => {
                    // 忽略 .
                }
                _ => normalized.push(component.as_os_str()),
            }
        }

        normalized
    }

    /// 安全删除文件（不返回错误如果不存在）
    pub fn safe_remove_file(path: &Path) -> bool {
        match fs::remove_file(path) {
            Ok(_) => true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => true,
            Err(_) => false,
        }
    }

    /// 安全删除目录（递归）
    pub fn safe_remove_dir_all(path: &Path) -> bool {
        match fs::remove_dir_all(path) {
            Ok(_) => true,
            Err(e) if e.kind() == io::ErrorKind::NotFound => true,
            Err(_) => false,
        }
    }

    /// 检查目录是否为空
    pub fn is_dir_empty(path: &Path) -> bool {
        match fs::read_dir(path) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => true,
        }
    }

    /// 读取文件内容为字符串
    pub fn read_file_to_string(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    /// 将字符串写入文件
    pub fn write_string_to_file(path: &Path, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }

    /// 检查路径是否存在且为目录
    pub fn ensure_dir_exists(path: &Path) -> io::Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// 获取可执行文件所在目录
    pub fn get_executable_dir() -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_utils_new() {
        let _utils = FileUtils;
        // Just verify it can be created
        let size = FileUtils::get_dir_size(PathBuf::from(".").as_path());
        assert!(size.is_ok());
    }

    #[test]
    fn test_read_file_to_string_nonexistent() {
        let result =
            FileUtils::read_file_to_string(PathBuf::from("/nonexistent/file.txt").as_path());
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_remove_file_nonexistent() {
        // Returns true because file doesn't exist (NotFound is treated as success)
        let result = FileUtils::safe_remove_file(PathBuf::from("/nonexistent/file.txt").as_path());
        assert!(result);
    }

    #[test]
    fn test_is_dir_empty_nonexistent() {
        // Returns true for non-existent directories (error case returns true)
        let result = FileUtils::is_dir_empty(PathBuf::from("/nonexistent").as_path());
        assert!(result);
    }

    #[test]
    fn test_ensure_dir_exists_creates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_subdir");
        assert!(!new_dir.exists());
        let result = FileUtils::ensure_dir_exists(&new_dir);
        assert!(result.is_ok());
        assert!(new_dir.exists());
    }

    #[test]
    fn test_ensure_dir_exists_already_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = FileUtils::ensure_dir_exists(temp_dir.path());
        assert!(result.is_ok());
    }
}
