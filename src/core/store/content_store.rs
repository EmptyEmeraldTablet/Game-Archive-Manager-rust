use crate::core::error::{GamError, GamResult};
use crate::utils::HashUtils;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// 内容可寻址存储
///
/// 将文件内容存储在 `.gam/objects/content/{hash_prefix}/{full_hash}` 中
/// 支持去重：相同内容的文件只存储一份
pub struct ContentStore {
    /// 存储根目录
    pub root: PathBuf,
    /// 内容索引
    pub index: ContentIndex,
}

impl ContentStore {
    /// 创建新的内容存储
    pub fn new(root: PathBuf) -> GamResult<Self> {
        let index_path = root.join("index");

        let index = if index_path.exists() {
            ContentStore::load_index(&index_path)?
        } else {
            ContentIndex::new()
        };

        // 确保目录结构存在
        fs::create_dir_all(&root)?;

        Ok(ContentStore { root, index })
    }

    /// 存储文件，返回内容哈希
    pub fn store(&mut self, path: &Path) -> GamResult<String> {
        if !path.exists() {
            return Err(GamError::NotFound(path.to_path_buf()));
        }

        let hash = HashUtils::hash_file(path)?;
        self.store_with_hash(path, &hash)
    }

    /// 使用已知哈希存储文件（用于去重）
    pub fn store_with_hash(&mut self, path: &Path, hash: &str) -> GamResult<String> {
        // 检查是否已存在
        if self.exists(hash) {
            // 增加引用计数
            self.index.increment_refcount(hash);
            self.save_index()?;
            return Ok(hash.to_string());
        }

        // 创建存储路径
        let (prefix, suffix) = Self::hash_parts(hash);
        let storage_path = self.root.join(prefix);

        fs::create_dir_all(&storage_path)?;

        let full_path = storage_path.join(suffix);

        // 复制文件
        fs::copy(path, &full_path)?;

        // 更新索引
        let size = fs::metadata(&full_path)?.len();
        self.index.add_entry(hash, size);
        self.save_index()?;

        Ok(hash.to_string())
    }

    /// 获取文件路径
    pub fn get(&self, hash: &str) -> GamResult<PathBuf> {
        let (prefix, suffix) = Self::hash_parts(hash);
        let path = self.root.join(prefix).join(suffix);

        if path.exists() {
            Ok(path)
        } else {
            Err(GamError::NotFound(path))
        }
    }

    /// 检查内容是否存在
    pub fn exists(&self, hash: &str) -> bool {
        let (prefix, suffix) = Self::hash_parts(hash);
        self.root.join(prefix).join(suffix).exists()
    }

    /// 获取内容大小
    pub fn size(&self, hash: &str) -> Option<u64> {
        self.index.get_size(hash)
    }

    /// 获取引用计数
    pub fn refcount(&self, hash: &str) -> u32 {
        self.index.get_refcount(hash).unwrap_or(0)
    }

    /// 删除未引用的内容
    pub fn garbage_collect(&mut self) -> GamResult<u64> {
        let mut freed = 0u64;

        // 遍历所有存储的内容
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type().unwrap().is_dir() {
                for file in fs::read_dir(entry.path())? {
                    let file = file?;
                    if file.file_type().unwrap().is_file() {
                        let file_name = file.file_name().to_string_lossy().to_string();
                        if let Some(hash) = self.find_hash_in_directory(entry.path(), &file_name) {
                            let refcount = self.index.get_refcount(&hash).unwrap_or(0);
                            if refcount == 0 {
                                // 删除未引用的内容
                                let size = fs::metadata(file.path())?.len();
                                fs::remove_file(file.path())?;
                                self.index.remove_entry(&hash);
                                freed += size;
                            }
                        }
                    }
                }
            }
        }

        self.save_index()?;
        Ok(freed)
    }

    /// 获取根目录
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// 获取索引引用（用于 GC）
    pub fn index(&self) -> &ContentIndex {
        &self.index
    }

    /// 获取索引可变引用（用于 GC）
    pub fn index_mut(&mut self) -> &mut ContentIndex {
        &mut self.index
    }

    /// 保存索引（公开方法）
    pub fn save_index(&mut self) -> GamResult<()> {
        let index_path = self.root.join("index");
        let file = File::create(index_path)?;
        serde_json::to_writer_pretty(file, &self.index)?;
        Ok(())
    }

    /// 获取去重节省的空间
    pub fn deduplication_savings(&self) -> u64 {
        self.index.deduplication_savings()
    }

    /// 分割哈希为前缀和后缀
    fn hash_parts(hash: &str) -> (&str, &str) {
        (&hash[..2], &hash[2..])
    }

    /// 在目录中查找哈希
    fn find_hash_in_directory(&self, _dir: PathBuf, file_name: &str) -> Option<String> {
        // 遍历父目录查找匹配的哈希
        let prefix = file_name[..2].to_string();
        for entry in fs::read_dir(self.root.join(&prefix)).ok()? {
            let entry = entry.ok()?;
            if entry.file_name().to_string_lossy() == file_name {
                let full_hash = format!("{}{}", prefix, file_name);
                if self.index.contains(&full_hash) {
                    return Some(full_hash);
                }
            }
        }
        None
    }

    /// 加载索引
    fn load_index(path: &Path) -> GamResult<ContentIndex> {
        if !path.exists() {
            return Ok(ContentIndex::new());
        }

        let file = File::open(path)?;
        let index: ContentIndex = serde_json::from_reader(file)?;
        Ok(index)
    }
}

/// 内容索引
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentIndex {
    /// 内容条目映射
    pub entries: std::collections::HashMap<String, ContentEntry>,
    /// 版本号
    version: u32,
}

impl ContentIndex {
    fn new() -> Self {
        ContentIndex {
            entries: std::collections::HashMap::new(),
            version: 1,
        }
    }

    fn add_entry(&mut self, hash: &str, size: u64) {
        self.entries.insert(
            hash.to_string(),
            ContentEntry {
                hash: hash.to_string(),
                size,
                refcount: 0,
            },
        );
    }

    fn increment_refcount(&mut self, hash: &str) {
        if let Some(entry) = self.entries.get_mut(hash) {
            entry.refcount += 1;
        }
    }

    fn get_size(&self, hash: &str) -> Option<u64> {
        self.entries.get(hash).map(|e| e.size)
    }

    fn get_refcount(&self, hash: &str) -> Option<u32> {
        self.entries.get(hash).map(|e| e.refcount)
    }

    fn contains(&self, hash: &str) -> bool {
        self.entries.contains_key(hash)
    }

    fn remove_entry(&mut self, hash: &str) {
        self.entries.remove(hash);
    }

    fn deduplication_savings(&self) -> u64 {
        let mut total_size = 0u64;
        let mut total_unique_size = 0u64;

        for entry in self.entries.values() {
            total_size += entry.size * u64::from(entry.refcount.max(1));
            total_unique_size += entry.size;
        }

        total_size.saturating_sub(total_unique_size)
    }
}

/// 内容条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    /// 内容哈希
    pub hash: String,
    /// 内容大小
    pub size: u64,
    /// 引用计数
    pub refcount: u32,
}
