use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 递归复制目录
pub fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(&entry.path(), &dst_path)?;
        }
    }

    Ok(())
}

/// 获取目录总大小 (字节)
pub fn get_dir_size(path: &Path) -> io::Result<u64> {
    let mut total_size: u64 = 0;

    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_size += get_dir_size(&path)?;
        } else {
            total_size += fs::metadata(&path)?.len();
        }
    }

    Ok(total_size)
}

/// 安全删除目录
pub fn remove_dir_all_safe(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// 检查路径是否存在且为目录
pub fn ensure_dir_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 读取文件内容为字符串
pub fn read_file_to_string(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

/// 将字符串写入文件
pub fn write_string_to_file(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

/// 获取可执行文件所在目录
pub fn get_executable_dir() -> PathBuf {
    let exe_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    exe_path
}
