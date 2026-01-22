use crate::core::error::{GamError, GamResult};
use crate::core::types::{Config, FileEntry, Snapshot, Timeline};
use crate::utils::FileUtils;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// 快照存储管理
///
/// 管理快照元数据的存储和查询
pub struct SnapshotStore {
    /// 快照目录
    pub snapshot_dir: PathBuf,
    /// 内容目录
    content_dir: PathBuf,
}

impl SnapshotStore {
    /// 创建新的快照存储
    pub fn new(gam_dir: &Path) -> Self {
        let snapshot_dir = gam_dir.join("objects").join("snapshot");
        let content_dir = gam_dir.join("objects").join("content");

        SnapshotStore {
            snapshot_dir,
            content_dir,
        }
    }

    /// 获取快照目录路径
    pub fn snapshot_dir(&self) -> &PathBuf {
        &self.snapshot_dir
    }

    /// 获取内容目录路径
    pub fn content_dir(&self) -> &PathBuf {
        &self.content_dir
    }

    /// 创建快照
    pub fn create(
        &self,
        files: &[FileEntry],
        timeline: &str,
        parent: Option<&str>,
        name: &str,
        description: Option<&str>,
        game_path: &Path,
    ) -> GamResult<Snapshot> {
        // 确保目录存在
        fs::create_dir_all(&self.snapshot_dir)?;

        // 计算所有文件的总大小
        let total_size: u64 = files.iter().map(|f| f.size).sum();

        // 创建快照 ID（基于元数据）
        let timestamp = Local::now();
        let parent_str = parent.as_deref().unwrap_or("");
        let metadata = format!(
            "{:?}|{}|{}|{:?}|{}",
            files, timeline, parent_str, timestamp, name
        );
        let id = crate::utils::HashUtils::hash_string(&metadata);

        // 构建快照
        let snapshot = Snapshot {
            id: id.clone(),
            parent: parent.map(|s| s.to_string()),
            timeline: timeline.to_string(),
            timestamp,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            files: files.to_vec(),
            content_hash: String::new(), // TODO: 计算组合哈希
            size: total_size,
            compression: String::from("none"),
            version: String::from("2.0.0"),
        };

        // 保存快照
        self.save(&snapshot)?;

        Ok(snapshot)
    }

    /// 保存快照到文件
    pub fn save(&self, snapshot: &Snapshot) -> GamResult<()> {
        let (prefix, suffix) = Self::hash_parts(&snapshot.id);
        let storage_path = self.snapshot_dir.join(prefix);

        fs::create_dir_all(&storage_path)?;

        let file_path = storage_path.join(suffix);
        let file = File::create(&file_path)?;
        serde_json::to_writer_pretty(file, snapshot)?;

        Ok(())
    }

    /// 获取快照
    pub fn get(&self, id: &str) -> GamResult<Snapshot> {
        let (prefix, suffix) = Self::hash_parts(id);
        let file_path = self.snapshot_dir.join(prefix).join(suffix);

        if !file_path.exists() {
            return Err(GamError::SnapshotNotFound(id.to_string()));
        }

        let file = File::open(&file_path)?;
        let snapshot: Snapshot = serde_json::from_reader(file)?;

        Ok(snapshot)
    }

    /// 按 ID 前缀查找快照
    pub fn get_by_prefix(&self, prefix: &str) -> GamResult<Option<Snapshot>> {
        // 查找匹配的快照
        let matches = self.find_by_prefix(prefix)?;
        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches[0].clone())),
            _ => Err(GamError::InvalidSnapshotId(format!(
                "Multiple snapshots match prefix '{}': {}",
                prefix,
                matches
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    /// 按前缀查找所有匹配的快照
    fn find_by_prefix(&self, prefix: &str) -> GamResult<Vec<Snapshot>> {
        let mut matches = Vec::new();

        if self.snapshot_dir.exists() {
            for entry in fs::read_dir(&self.snapshot_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();

                    // If prefix length > 2, check directory name first
                    if prefix.len() > 2 {
                        let dir_prefix = &prefix[..2];
                        if dir_name.starts_with(dir_prefix) {
                            // Directory matches, check file names
                            let file_prefix = &prefix[2..];
                            for file in fs::read_dir(entry.path())? {
                                let file = file?;
                                if file.file_type()?.is_file() {
                                    let file_name = file.file_name().to_string_lossy().to_string();
                                    if file_name.starts_with(file_prefix) {
                                        match SnapshotStore::load_snapshot_file(file.path()) {
                                            Ok(snapshot) => matches.push(snapshot),
                                            Err(_) => continue,
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Prefix length <= 2, just check directory name
                        if dir_name.starts_with(prefix) {
                            // Directory matches, load all files
                            for file in fs::read_dir(entry.path())? {
                                let file = file?;
                                if file.file_type()?.is_file() {
                                    match SnapshotStore::load_snapshot_file(file.path()) {
                                        Ok(snapshot) => matches.push(snapshot),
                                        Err(_) => continue,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    /// 加载快照文件
    pub fn load_snapshot_file(path: PathBuf) -> GamResult<Snapshot> {
        let file = File::open(&path)?;
        let snapshot: Snapshot = serde_json::from_reader(file)?;
        Ok(snapshot)
    }

    /// 删除快照
    pub fn delete(&mut self, id: &str) -> GamResult<()> {
        let (prefix, suffix) = Self::hash_parts(id);
        let file_path = self.snapshot_dir.join(prefix).join(suffix);

        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        Ok(())
    }

    /// 获取所有快照
    pub fn list_all(&self) -> GamResult<Vec<Snapshot>> {
        let mut snapshots = Vec::new();

        if self.snapshot_dir.exists() {
            for entry in fs::read_dir(&self.snapshot_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    for file in fs::read_dir(entry.path())? {
                        let file = file?;
                        if file.file_type()?.is_file() {
                            match Self::load_snapshot_file(file.path()) {
                                Ok(snapshot) => snapshots.push(snapshot),
                                Err(_) => continue,
                            }
                        }
                    }
                }
            }
        }

        // 按时间戳排序
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    /// 获取指定时间线的所有快照
    pub fn list_by_timeline(&self, timeline: &str) -> GamResult<Vec<Snapshot>> {
        let all = self.list_all()?;
        Ok(all.into_iter().filter(|s| s.timeline == timeline).collect())
    }

    /// 获取时间线上的最新快照
    pub fn latest_on_timeline(&self, timeline: &str) -> GamResult<Option<Snapshot>> {
        let timeline_snapshots = self.list_by_timeline(timeline)?;
        Ok(timeline_snapshots.first().cloned())
    }

    /// 分割哈希为前缀和后缀
    fn hash_parts(hash: &str) -> (&str, &str) {
        (&hash[..2], &hash[2..])
    }
}

/// 时间线管理器
pub struct TimelineManager {
    refs_dir: PathBuf,
    head_file: PathBuf,
}

impl TimelineManager {
    /// 创建新的时间线管理器
    pub fn new(gam_dir: &Path) -> Self {
        let refs_dir = gam_dir.join("refs").join("timelines");
        let head_file = gam_dir.join("HEAD");

        TimelineManager {
            refs_dir,
            head_file,
        }
    }

    /// 获取 refs 目录
    pub fn refs_dir(&self) -> &PathBuf {
        &self.refs_dir
    }

    /// 创建时间线
    pub fn create(&self, name: &str, from_snapshot: Option<&str>) -> GamResult<Timeline> {
        // 验证时间线名称
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            return Err(GamError::InvalidTimelineName(name.to_string()));
        }

        // 检查是否已存在
        if self.exists(name) {
            return Err(GamError::TimelineExists(name.to_string()));
        }

        // 获取起始快照
        let head_snapshot = from_snapshot
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from(""));

        // 创建时间线
        let timeline = Timeline {
            name: name.to_string(),
            head_snapshot,
            created_at: Local::now(),
            description: None,
        };

        // 保存时间线引用
        self.save(timeline.clone())?;

        Ok(timeline)
    }

    /// 保存时间线
    fn save(&self, timeline: Timeline) -> GamResult<()> {
        fs::create_dir_all(&self.refs_dir)?;

        let file_path = self.refs_dir.join(&timeline.name);
        fs::write(file_path, &timeline.head_snapshot)?;

        Ok(())
    }

    /// 检查时间线是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.refs_dir.join(name).exists()
    }

    /// 获取时间线
    pub fn get(&self, name: &str) -> GamResult<Option<Timeline>> {
        let file_path = self.refs_dir.join(name);

        if !file_path.exists() {
            return Ok(None);
        }

        let head_snapshot = fs::read_to_string(file_path)?.trim().to_string();

        Ok(Some(Timeline {
            name: name.to_string(),
            head_snapshot,
            created_at: Local::now(), // TODO: 存储创建时间
            description: None,
        }))
    }

    /// 获取所有时间线
    pub fn list(&self) -> GamResult<Vec<Timeline>> {
        let mut timelines = Vec::new();

        if self.refs_dir.exists() {
            for entry in fs::read_dir(&self.refs_dir)? {
                let entry = entry?;
                if entry.file_type().unwrap().is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(Some(timeline)) = self.get(&name) {
                        timelines.push(timeline);
                    }
                }
            }
        }

        Ok(timelines)
    }

    /// 设置当前时间线（HEAD）
    pub fn set_current(&self, name: &str) -> GamResult<()> {
        fs::write(&self.head_file, format!("ref: refs/timelines/{}\n", name))?;
        Ok(())
    }

    /// 获取当前时间线名称
    pub fn current(&self) -> GamResult<Option<String>> {
        if !self.head_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.head_file)?;

        if content.starts_with("ref: refs/timelines/") {
            let name = content.trim_start_matches("ref: refs/timelines/").trim();
            Ok(Some(name.to_string()))
        } else {
            // 分离 HEAD 状态
            Ok(None)
        }
    }

    /// 删除时间线
    pub fn delete(&self, name: &str) -> GamResult<()> {
        let file_path = self.refs_dir.join(name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    /// 重命名时间线
    pub fn rename(&self, old_name: &str, new_name: &str) -> GamResult<()> {
        let old_path = self.refs_dir.join(old_name);
        let new_path = self.refs_dir.join(new_name);

        if !old_path.exists() {
            return Err(GamError::TimelineNotFound(old_name.to_string()));
        }

        if new_path.exists() {
            return Err(GamError::TimelineExists(new_name.to_string()));
        }

        // 如果当前 HEAD 指向旧时间线，需要更新 HEAD
        if self.head_file.exists() {
            let head_content = fs::read_to_string(&self.head_file)?;
            if head_content.trim() == format!("ref: refs/timelines/{}", old_name) {
                fs::write(
                    &self.head_file,
                    format!("ref: refs/timelines/{}\n", new_name),
                )?;
            }
        }

        fs::rename(old_path, new_path)?;
        Ok(())
    }

    /// 更新时间线 HEAD
    pub fn update_head(&self, name: &str, snapshot_id: &str) -> GamResult<()> {
        let file_path = self.refs_dir.join(name);
        fs::write(file_path, snapshot_id)?;
        Ok(())
    }

    /// 检查快照是否被任何时间线引用
    pub fn is_snapshot_referenced(&self, snapshot_id: &str) -> GamResult<bool> {
        // 检查 HEAD 是否直接指向此快照（分离状态）
        if self.head_file.exists() {
            let head_content = fs::read_to_string(&self.head_file)?;
            if !head_content.starts_with("ref:") && head_content.trim() == snapshot_id {
                return Ok(true);
            }
        }

        // 检查所有时间线
        let timelines = self.list()?;
        for timeline in timelines {
            if timeline.head_snapshot == snapshot_id {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
