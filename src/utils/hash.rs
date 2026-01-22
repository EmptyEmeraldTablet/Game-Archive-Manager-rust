use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// SHA-256 哈希工具
pub struct HashUtils;

impl HashUtils {
    /// 计算文件的 SHA-256 哈希
    pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// 计算数据的 SHA-256 哈希
    pub fn hash_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// 计算字符串的 SHA-256 哈希
    pub fn hash_string(s: &str) -> String {
        Self::hash_data(s.as_bytes())
    }

    /// 短哈希格式（取前 8 位）
    pub fn short_hash(full_hash: &str) -> &str {
        &full_hash[..8]
    }

    /// 验证哈希
    pub fn verify_hash(data: &[u8], expected: &str) -> bool {
        let computed = Self::hash_data(data);
        computed == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data_deterministic() {
        let data = b"test data";
        let hash1 = HashUtils::hash_data(data);
        let hash2 = HashUtils::hash_data(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_data_different_for_different_input() {
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        let hash1 = HashUtils::hash_data(data1);
        let hash2 = HashUtils::hash_data(data2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_data_length() {
        let data = b"test";
        let hash = HashUtils::hash_data(data);
        // SHA256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_file_consistent() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), b"test content").unwrap();
        let hash1 = HashUtils::hash_file(temp_file.path()).unwrap();
        let hash2 = HashUtils::hash_file(temp_file.path()).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_hash_valid() {
        let data = b"test data";
        let hash = HashUtils::hash_data(data);
        assert!(HashUtils::verify_hash(data, &hash));
    }

    #[test]
    fn test_verify_hash_invalid() {
        let data = b"test data";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!HashUtils::verify_hash(data, wrong_hash));
    }

    #[test]
    fn test_short_hash() {
        let full_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let short = HashUtils::short_hash(full_hash);
        assert_eq!(short, "abcdef12");
        assert_eq!(short.len(), 8);
    }
}
