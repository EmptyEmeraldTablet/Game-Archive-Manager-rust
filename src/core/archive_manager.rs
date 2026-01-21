use crate::core::ArchiveInfo;
use crate::utils::file_utils::{copy_dir_all, get_dir_size, remove_dir_all_safe};
use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ArchiveManagerError {
    #[error("存档文件夹不存在: {0}")]
    ArchiveFolderNotFound(PathBuf),

    #[error("无法读取 path.txt 文件")]
    PathFileNotFound,

    #[error("path.txt 中的路径无效: {0}")]
    InvalidPath(PathBuf),

    #[error("存档不存在: {0}")]
    ArchiveNotFound(String),

    #[error("存档操作失败: {0}")]
    OperationFailed(String),

    #[error("日志文件损坏")]
    CorruptedLogFile,

    #[error("文件操作错误: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ArchiveManager {
    source_path: PathBuf,
    archive_path: PathBuf,
    info_file: PathBuf,
    archives: Vec<ArchiveInfo>,
}

impl ArchiveManager {
    pub fn new() -> Result<Self> {
        let source_path = Self::load_source_path()?;

        let exe_dir = std::env::current_dir().context("获取当前目录失败")?;

        let archive_path = exe_dir.join("Archive");
        if !archive_path.exists() {
            fs::create_dir_all(&archive_path).context("创建 Archive 文件夹失败")?;
        }

        let info_file = archive_path.join("archives.json");
        let archives = Self::load_archives(&info_file)?;

        Ok(Self {
            source_path,
            archive_path,
            info_file,
            archives,
        })
    }

    fn load_source_path() -> Result<PathBuf> {
        let exe_dir = std::env::current_dir().context("获取当前目录失败")?;

        let path_file = exe_dir.join("path.txt");

        if !path_file.exists() {
            return Err(ArchiveManagerError::PathFileNotFound.into());
        }

        let content = fs::read_to_string(&path_file).context("读取 path.txt 失败")?;

        let content = content.trim();
        if content.is_empty() {
            return Err(ArchiveManagerError::PathFileNotFound.into());
        }

        let source_path = PathBuf::from(content);

        if !source_path.exists() || !source_path.is_dir() {
            return Err(ArchiveManagerError::InvalidPath(source_path).into());
        }

        Ok(source_path)
    }

    fn load_archives(info_file: &Path) -> Result<Vec<ArchiveInfo>> {
        if !info_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(info_file).context("读取存档日志失败")?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str(&content).context("解析存档日志失败")
    }

    fn save_archives(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.archives).context("序列化存档信息失败")?;

        fs::write(&self.info_file, content).context("写入存档日志失败")?;

        Ok(())
    }

    pub fn save(&mut self, name: String, comment: String) -> Result<(), anyhow::Error> {
        if let Err(e) = ArchiveInfo::validate_name(&name) {
            return Err(anyhow::anyhow!("{}", e));
        }
        if let Err(e) = ArchiveInfo::validate_comment(&comment) {
            return Err(anyhow::anyhow!("{}", e));
        }

        let archive_dir = self
            .archive_path
            .join(format!("Archive{}", self.archives.len()));

        copy_dir_all(&self.source_path, &archive_dir)
            .with_context(|| format!("复制存档到 {} 失败", archive_dir.display()))?;

        let info = ArchiveInfo::new(name, comment);
        self.archives.push(info);

        self.save_archives()?;

        Ok(())
    }

    pub fn quick_save(&mut self) -> Result<(), anyhow::Error> {
        self.save(String::new(), String::new())
    }

    pub fn replace_save(&mut self) -> Result<(), anyhow::Error> {
        if self.archives.is_empty() {
            return self.quick_save();
        }

        let index = self.archives.len() - 1;
        let archive_dir = self.archive_path.join(format!("Archive{}", index));

        remove_dir_all_safe(&archive_dir)?;
        copy_dir_all(&self.source_path, &archive_dir)
            .with_context(|| format!("复制存档到 {} 失败", archive_dir.display()))?;

        self.archives[index].timestamp = chrono::Local::now();

        self.save_archives()?;

        Ok(())
    }

    pub fn load(&mut self, index: usize) -> Result<(), anyhow::Error> {
        if index >= self.archives.len() {
            return Err(ArchiveManagerError::ArchiveNotFound(format!("索引 {}", index)).into());
        }

        let archive_dir = self.archive_path.join(format!("Archive{}", index));

        remove_dir_all_safe(&self.source_path)?;
        std::fs::create_dir_all(&self.source_path)?;

        copy_dir_all(&archive_dir, &self.source_path)
            .with_context(|| format!("从 {} 恢复存档失败", archive_dir.display()))?;

        Ok(())
    }

    pub fn quick_load(&mut self) -> Result<(), anyhow::Error> {
        if self.archives.is_empty() {
            return Err(ArchiveManagerError::ArchiveNotFound("无存档".to_string()).into());
        }
        self.load(self.archives.len() - 1)
    }

    pub fn delete(&mut self, index: usize) -> Result<(), anyhow::Error> {
        if index >= self.archives.len() {
            return Err(ArchiveManagerError::ArchiveNotFound(format!("索引 {}", index)).into());
        }

        let archive_dir = self.archive_path.join(format!("Archive{}", index));
        remove_dir_all_safe(&archive_dir)?;

        for i in index..self.archives.len() - 1 {
            let old_path = self.archive_path.join(format!("Archive{}", i + 1));
            let new_path = self.archive_path.join(format!("Archive{}", i));

            if old_path.exists() {
                fs::rename(&old_path, &new_path)?;
            }
        }

        self.archives.remove(index);

        self.save_archives()?;

        Ok(())
    }

    pub fn quick_delete(&mut self) -> Result<(), anyhow::Error> {
        if self.archives.is_empty() {
            return Err(ArchiveManagerError::ArchiveNotFound("无存档".to_string()).into());
        }
        self.delete(self.archives.len() - 1)
    }

    pub fn modify(
        &mut self,
        index: usize,
        name: Option<String>,
        comment: Option<String>,
    ) -> Result<(), anyhow::Error> {
        if index >= self.archives.len() {
            return Err(ArchiveManagerError::ArchiveNotFound(format!("索引 {}", index)).into());
        }

        if let Some(ref n) = name {
            if let Err(e) = ArchiveInfo::validate_name(n) {
                return Err(anyhow::anyhow!("{}", e));
            }
        }
        if let Some(ref c) = comment {
            if let Err(e) = ArchiveInfo::validate_comment(c) {
                return Err(anyhow::anyhow!("{}", e));
            }
        }

        self.archives[index].modify(name, comment);
        self.save_archives()?;

        Ok(())
    }

    pub fn get_all_archives(&self) -> &[ArchiveInfo] {
        &self.archives
    }

    pub fn get_recent_archives(&self, count: usize) -> &[ArchiveInfo] {
        let start = self.archives.len().saturating_sub(count);
        &self.archives[start..]
    }

    pub fn get_usage_space(&self) -> f64 {
        let mut total_size: u64 = 0;

        for (i, _) in self.archives.iter().enumerate() {
            let archive_dir = self.archive_path.join(format!("Archive{}", i));
            if let Ok(size) = get_dir_size(&archive_dir) {
                total_size += size;
            }
        }

        total_size as f64 / (1024.0 * 1024.0)
    }

    pub fn archive_count(&self) -> usize {
        self.archives.len()
    }

    pub fn source_path(&self) -> &PathBuf {
        &self.source_path
    }
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            let exe_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let archive_path = exe_dir.join("Archive");

            Self {
                source_path: PathBuf::new(),
                archive_path: archive_path.clone(),
                info_file: archive_path.join("archives.json"),
                archives: Vec::new(),
            }
        })
    }
}
