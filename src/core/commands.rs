//! 命令处理器模块
//!
//! 实现所有核心命令的实际逻辑

use crate::core::activity::{ActivityAction, ActivityEngine};
use crate::core::error::GamResult;
use crate::core::ignore::IgnoreEngine;
use crate::core::store::{ContentStore, SnapshotStore, TimelineManager};
use crate::core::tag::TagStore;
use crate::core::types::{FileEntry, Snapshot};
use crate::ui::{print_error, print_info, print_success, Formatter};
use crate::utils::{FileUtils, HashUtils};
use std::path::PathBuf;

/// 仓库上下文
pub struct Repository {
    gam_dir: PathBuf,
    game_path: PathBuf,
    snapshot_store: SnapshotStore,
    content_store: ContentStore,
    timeline_manager: TimelineManager,
}

impl Repository {
    /// 创建新的仓库实例
    pub fn new(gam_dir: PathBuf, game_path: PathBuf) -> GamResult<Self> {
        let snapshot_store = SnapshotStore::new(&gam_dir);
        let content_store = ContentStore::new(gam_dir.join("objects").join("content"))?;
        let timeline_manager = TimelineManager::new(&gam_dir);

        Ok(Repository {
            gam_dir,
            game_path,
            snapshot_store,
            content_store,
            timeline_manager,
        })
    }

    /// 获取当前时间线名称
    pub fn current_timeline(&self) -> GamResult<Option<String>> {
        self.timeline_manager.current()
    }

    /// 获取当前时间线名称，如果不存在则返回默认值
    pub fn get_timeline_name(&self) -> GamResult<String> {
        Ok(self
            .current_timeline()?
            .unwrap_or_else(|| "main".to_string()))
    }
}

/// 处理 init 命令
pub fn handle_init(path: Option<String>, force: bool) -> GamResult<()> {
    // First determine game_path from argument
    let game_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        // 交互式输入
        println!("请输入游戏存档目录路径:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let game_path = PathBuf::from(input.trim());

        if !game_path.exists() {
            return Err(crate::core::error::GamError::GamePathNotFound(game_path));
        }

        game_path
    };

    // gam_dir should be INSIDE game_path
    let gam_dir = game_path.join(".gam");

    // 创建 .gam 目录结构
    std::fs::create_dir_all(&gam_dir)?;
    std::fs::create_dir_all(&gam_dir.join("refs").join("timelines"))?;
    std::fs::create_dir_all(&gam_dir.join("objects").join("snapshot"))?;
    std::fs::create_dir_all(&gam_dir.join("objects").join("content"))?;

    // 创建配置
    let config_content = format!(
        r#"[core]
game_path = "{}"
default_timeline = "main"
use_gamignore = true

[storage]
strategy = "deduplication"
"#,
        game_path.to_string_lossy().replace('\\', "/")
    );

    std::fs::write(gam_dir.join("config"), config_content)?;

    // 创建 HEAD
    std::fs::write(gam_dir.join("HEAD"), "ref: refs/timelines/main\n")?;

    // 创建默认时间线
    std::fs::write(gam_dir.join("refs").join("timelines").join("main"), "")?;

    // 记录活动
    let engine = ActivityEngine::new(&gam_dir);
    engine.log(ActivityAction::Init, None, None, None)?;

    print_success(&format!(
        "初始化完成！\n  游戏存档目录: {}\n  GAM 仓库: {}",
        game_path.to_string_lossy(),
        gam_dir.to_string_lossy()
    ));

    Ok(())
}

/// 处理 snapshot save 命令
pub fn handle_snapshot_save(
    gam_dir: &PathBuf,
    message: Option<String>,
    timeline: Option<String>,
) -> GamResult<()> {
    let mut repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 确定时间线
    let timeline_name = match timeline {
        Some(tl) => tl,
        None => repo.get_timeline_name()?,
    };

    // 检查时间线是否存在
    if !repo.timeline_manager.exists(&timeline_name) {
        return Err(crate::core::error::GamError::TimelineNotFound(
            timeline_name,
        ));
    }

    // 获取时间线当前快照
    let timeline = repo.timeline_manager.get(&timeline_name)?;
    let parent_snapshot = timeline.and_then(|t| {
        if t.head_snapshot.is_empty() {
            None
        } else {
            Some(t.head_snapshot)
        }
    });

    // 确定快照名称
    let snapshot_name = message.unwrap_or_else(|| {
        let now = chrono::Local::now();
        format!("Snapshot {}", now.format("%Y-%m-%d %H:%M"))
    });

    // 扫描游戏存档目录
    let files = scan_game_directory(&repo.game_path, &repo.gam_dir)?;

    if files.is_empty() {
        print_info("游戏存档目录为空，无文件可保存");
        return Ok(());
    }

    // 存储文件内容
    let mut stored_files = Vec::new();
    let mut total_size = 0u64;

    for file in &files {
        // 计算文件哈希并存储
        let hash = HashUtils::hash_file(file)?;
        repo.content_store.store_with_hash(file, &hash)?;

        let size = std::fs::metadata(file)?.len();
        let relative_path = FileUtils::relative_to(file, &repo.game_path)
            .unwrap_or_else(|| PathBuf::from(file.file_name().unwrap()));

        stored_files.push(FileEntry::new(relative_path, hash, size));
        total_size += size;
    }

    // 创建快照
    let snapshot = repo.snapshot_store.create(
        &stored_files,
        &timeline_name,
        parent_snapshot.as_deref(),
        &snapshot_name,
        None,
        &repo.game_path,
    )?;

    // 更新时间线 HEAD
    repo.timeline_manager
        .update_head(&timeline_name, &snapshot.id)?;

    // 记录活动
    let engine = ActivityEngine::new(gam_dir);
    engine.log(
        ActivityAction::SnapshotSave,
        Some(&timeline_name),
        Some(&Formatter::short_hash(&snapshot.id)),
        None,
    )?;

    print_success(&format!(
        "已保存快照 {} ({})\n  时间线: {}\n  文件数: {}\n  大小: {}",
        Formatter::short_hash(&snapshot.id),
        snapshot_name,
        timeline_name,
        stored_files.len(),
        Formatter::format_size(total_size)
    ));

    Ok(())
}

/// 扫描游戏存档目录，获取所有文件
fn scan_game_directory(game_path: &PathBuf, gam_dir: &PathBuf) -> GamResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    // 检查是否启用 .gamignore
    let use_gamignore = check_use_gamignore(gam_dir)?;

    // 加载忽略规则
    let ignore_engine = if use_gamignore {
        load_ignore_engine(gam_dir)?
    } else {
        IgnoreEngine::new(Vec::new())
    };

    // 扫描目录
    for entry in std::fs::read_dir(game_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = FileUtils::relative_to(&path, game_path)
                .unwrap_or_else(|| PathBuf::from(path.file_name().unwrap()));

            if !ignore_engine.is_ignored(&relative_path, false) {
                files.push(path);
            }
        } else if path.is_dir() && !is_hidden_dir(&path) {
            // 递归扫描子目录
            let sub_files = scan_directory_recursive(&path, game_path, &ignore_engine)?;
            files.extend(sub_files);
        }
    }

    Ok(files)
}

/// 递归扫描目录
fn scan_directory_recursive(
    dir: &PathBuf,
    base_path: &PathBuf,
    ignore_engine: &IgnoreEngine,
) -> GamResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let relative_path = FileUtils::relative_to(&path, base_path)
            .unwrap_or_else(|| PathBuf::from(path.file_name().unwrap()));

        if path.is_file() {
            if !ignore_engine.is_ignored(&relative_path, false) {
                files.push(path);
            }
        } else if path.is_dir() && !is_hidden_dir(&path) {
            if !ignore_engine.is_ignored(&relative_path, true) {
                let sub_files = scan_directory_recursive(&path, base_path, ignore_engine)?;
                files.extend(sub_files);
            }
        }
    }

    Ok(files)
}

/// 检查是否为隐藏目录
fn is_hidden_dir(path: &PathBuf) -> bool {
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        file_name.starts_with('.')
    } else {
        false
    }
}

/// 检查是否启用 .gamignore
fn check_use_gamignore(gam_dir: &PathBuf) -> GamResult<bool> {
    let config_path = gam_dir.join("config");
    if !config_path.exists() {
        return Ok(false);
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: toml::Value = config_content
        .parse()
        .map_err(|e: toml::de::Error| crate::core::error::GamError::TomlParse(e.to_string()))?;

    Ok(config
        .get("core")
        .and_then(|c| c.get("use_gamignore"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false))
}

/// 加载忽略规则引擎
fn load_ignore_engine(gam_dir: &PathBuf) -> GamResult<IgnoreEngine> {
    let ignore_file = gam_dir.join(".gamignore");

    if !ignore_file.exists() {
        return Ok(IgnoreEngine::new(Vec::new()));
    }

    let content = std::fs::read_to_string(&ignore_file)?;
    let patterns = IgnoreEngine::parse_gamignore(&content)?;

    Ok(IgnoreEngine::new(patterns))
}

/// 获取游戏路径
fn get_game_path(gam_dir: &PathBuf) -> GamResult<PathBuf> {
    let config_path = gam_dir.join("config");

    if !config_path.exists() {
        return Err(crate::core::error::GamError::NotInitialized);
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: toml::Value = config_content
        .parse()
        .map_err(|e: toml::de::Error| crate::core::error::GamError::TomlParse(e.to_string()))?;

    let game_path_str = config
        .get("core")
        .and_then(|c: &toml::Value| c.get("game_path"))
        .and_then(|v: &toml::Value| v.as_str())
        .ok_or_else(|| {
            crate::core::error::GamError::InvalidConfig("game_path not found".to_string())
        })?;

    let game_path = PathBuf::from(game_path_str);

    if !game_path.exists() {
        return Err(crate::core::error::GamError::GamePathNotFound(game_path));
    }

    Ok(game_path)
}

/// 处理 snapshot list 命令
pub fn handle_snapshot_list(
    gam_dir: &PathBuf,
    all: bool,
    timeline: Option<String>,
) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    // 先确定要显示的时间线名称
    let timeline_name = match &timeline {
        Some(tl) => tl.clone(),
        None => repo.get_timeline_name()?,
    };

    let snapshots = if let Some(tl) = &timeline {
        snapshot_store.list_by_timeline(tl)?
    } else if all {
        snapshot_store.list_all()?
    } else {
        snapshot_store.list_by_timeline(&timeline_name)?
    };

    if snapshots.is_empty() {
        print_info("暂无快照");
        return Ok(());
    }

    let output = Formatter::format_snapshot_table(&snapshots, &timeline_name);
    println!("{}", output);

    Ok(())
}

/// 处理 snapshot info 命令
pub fn handle_snapshot_info(gam_dir: &PathBuf, id: &str) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    // 尝试按前缀查找
    let snapshot = snapshot_store.get_by_prefix(id)?;

    match snapshot {
        Some(snap) => {
            let output = Formatter::format_snapshot_detail(&snap);
            println!("{}", output);
            Ok(())
        }
        None => Err(crate::core::error::GamError::SnapshotNotFound(
            id.to_string(),
        )),
    }
}

/// 处理 snapshot tag 命令 - 为快照添加标签
pub fn handle_snapshot_tag(gam_dir: &PathBuf, id: &str, tag_name: &str) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 验证标签名称
    if tag_name.is_empty() || tag_name.contains('/') || tag_name.contains('\\') {
        return Err(crate::core::error::GamError::InvalidTagName(
            tag_name.to_string(),
        ));
    }

    // 查找快照
    let snapshot = repo.snapshot_store.get_by_prefix(id)?;

    match snapshot {
        Some(snap) => {
            // 加载标签存储
            let mut tag_store = TagStore::new(gam_dir);

            // 检查标签是否已存在
            if tag_store.exists(tag_name) {
                print_error(&format!("标签 '{}' 已存在", tag_name));
                return Ok(());
            }

            // 添加标签
            tag_store.add_tag(tag_name, &snap.id);
            tag_store.save(gam_dir)?;

            print_success(&format!(
                "已为快照 {} 添加标签 '{}'",
                Formatter::short_hash(&snap.id),
                tag_name
            ));
            Ok(())
        }
        None => Err(crate::core::error::GamError::SnapshotNotFound(
            id.to_string(),
        )),
    }
}

/// 处理 snapshot delete 命令
pub fn handle_snapshot_delete(gam_dir: &PathBuf, id: &str, force: bool) -> GamResult<()> {
    let mut repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &mut repo.snapshot_store;

    // 查找快照
    let snapshot = snapshot_store.get_by_prefix(id)?;

    match snapshot {
        Some(snap) => {
            // 检查是否被时间线引用（使用 TimelineManager）
            let is_referenced = repo.timeline_manager.is_snapshot_referenced(&snap.id)?;

            if is_referenced && !force {
                print_error(&format!(
                    "无法删除快照 {}，因为它被以下时间线引用:",
                    Formatter::short_hash(&snap.id)
                ));
                // 列出引用该快照的时间线
                let timelines = repo.timeline_manager.list()?;
                for tl in timelines {
                    if tl.head_snapshot == snap.id {
                        println!("  - {} (当前 HEAD)", tl.name);
                    }
                }
                print_info("使用 --force 强制删除");
                return Ok(());
            }

            // 确认操作
            if !force {
                println!("此操作将永久删除快照。");
                println!(
                    "  快照: {} ({})",
                    Formatter::short_hash(&snap.id),
                    snap.name
                );
                println!("  时间线: {}", snap.timeline);
                println!("  文件数: {}", snap.files.len());

                print_confirm("确定继续?");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    print_info("操作已取消");
                    return Ok(());
                }
            }

            // 删除快照文件
            snapshot_store.delete(&snap.id)?;

            // 记录活动
            let engine = ActivityEngine::new(gam_dir);
            engine.log(
                ActivityAction::SnapshotDelete,
                Some(&snap.timeline),
                Some(&Formatter::short_hash(&snap.id)),
                None,
            )?;

            print_success(&format!(
                "已删除快照 {} ({})",
                Formatter::short_hash(&snap.id),
                snap.name
            ));

            Ok(())
        }
        None => Err(crate::core::error::GamError::SnapshotNotFound(
            id.to_string(),
        )),
    }
}

/// 处理 timeline create 命令
pub fn handle_timeline_create(
    gam_dir: &PathBuf,
    name: &str,
    from: Option<String>,
) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 检查时间线是否已存在
    if repo.timeline_manager.exists(name) {
        return Err(crate::core::error::GamError::TimelineExists(
            name.to_string(),
        ));
    }

    // 确定起始快照
    let from_snapshot = match from {
        Some(ref id) => {
            let snapshot = repo.snapshot_store.get_by_prefix(id)?;
            match snapshot {
                Some(snap) => Some(snap.id),
                None => {
                    return Err(crate::core::error::GamError::SnapshotNotFound(
                        id.to_string(),
                    ))
                }
            }
        }
        None => None,
    };

    // 创建时间线
    repo.timeline_manager
        .create(name, from_snapshot.as_deref())?;

    // 记录活动
    let engine = ActivityEngine::new(gam_dir);
    engine.log(ActivityAction::TimelineCreate, Some(name), None, None)?;

    print_success(&format!("已创建时间线 '{}'", name));

    // 如果有起始快照，列出该快照的信息
    if let Some(snap_id) = from_snapshot {
        let snapshot = repo.snapshot_store.get(&snap_id)?;
        println!(
            "  从快照 {} ({}) 创建",
            Formatter::short_hash(&snap_id),
            snapshot.name
        );
    }

    Ok(())
}

/// 处理 timeline list 命令
pub fn handle_timeline_list(gam_dir: &PathBuf) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    let timelines = repo.timeline_manager.list()?;
    let current = repo.current_timeline()?;

    if timelines.is_empty() {
        print_info("暂无时间线");
        return Ok(());
    }

    let output = Formatter::format_timeline_list(&timelines, current.as_deref());
    println!("{}", output);

    Ok(())
}

/// 处理 timeline switch 命令
pub fn handle_timeline_switch(gam_dir: &PathBuf, target: &str) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 检查是否是时间线名称
    if repo.timeline_manager.exists(target) {
        // 获取之前的时间线
        let previous = repo.current_timeline()?;

        repo.timeline_manager.set_current(target)?;

        // 记录活动
        let engine = ActivityEngine::new(gam_dir);
        engine.log(
            ActivityAction::TimelineSwitch,
            Some(target),
            Some(target),
            previous.as_deref(),
        )?;

        print_success(&format!("已切换到时间线 '{}'", target));
        return Ok(());
    }

    // 尝试作为快照 ID
    let snapshot = repo.snapshot_store.get_by_prefix(target)?;
    match snapshot {
        Some(snap) => {
            // 分离 HEAD 状态
            let head_file = gam_dir.join("HEAD");
            std::fs::write(head_file, &snap.id)?;
            print_success(&format!(
                "HEAD 现在指向快照 {} ({})",
                Formatter::short_hash(&snap.id),
                snap.name
            ));
            Ok(())
        }
        None => Err(crate::core::error::GamError::InvalidSnapshotId(
            target.to_string(),
        )),
    }
}

/// 处理 timeline rename 命令
pub fn handle_timeline_rename(gam_dir: &PathBuf, old_name: &str, new_name: &str) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 验证新名称
    if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\') {
        return Err(crate::core::error::GamError::InvalidTimelineName(
            new_name.to_string(),
        ));
    }

    // 检查旧时间线是否存在
    if !repo.timeline_manager.exists(old_name) {
        return Err(crate::core::error::GamError::TimelineNotFound(
            old_name.to_string(),
        ));
    }

    // 检查新名称是否已存在
    if repo.timeline_manager.exists(new_name) {
        return Err(crate::core::error::GamError::TimelineExists(
            new_name.to_string(),
        ));
    }

    // 执行重命名
    repo.timeline_manager.rename(old_name, new_name)?;

    // 记录活动
    let engine = ActivityEngine::new(gam_dir);
    engine.log(
        ActivityAction::TimelineRename,
        Some(old_name),
        Some(new_name),
        Some(old_name),
    )?;

    print_success(&format!(
        "已将时间线 '{}' 重命名为 '{}'",
        old_name, new_name
    ));

    Ok(())
}

/// 处理 timeline current 命令
pub fn handle_timeline_current(gam_dir: &PathBuf) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    match repo.current_timeline()? {
        Some(name) => {
            print_success(&format!("当前时间线: {}", name));
        }
        None => {
            print_info("当前没有活动的时间线 (分离 HEAD 状态)");
        }
    }

    Ok(())
}

/// 处理 timeline delete 命令
pub fn handle_timeline_delete(gam_dir: &PathBuf, name: &str, force: bool) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    if !repo.timeline_manager.exists(name) {
        return Err(crate::core::error::GamError::TimelineNotFound(
            name.to_string(),
        ));
    }

    // 检查是否是当前时间线
    let current = repo.current_timeline()?;
    if current.as_ref().map(|s| s.as_str()) == Some(name) && !force {
        print_error(&format!(
            "无法删除当前时间线 '{}'。请先切换到其他时间线。",
            name
        ));
        return Ok(());
    }

    repo.timeline_manager.delete(name)?;

    // 记录活动
    let engine = ActivityEngine::new(gam_dir);
    engine.log(ActivityAction::TimelineDelete, Some(name), None, None)?;

    print_success(&format!("已删除时间线 '{}'", name));

    Ok(())
}

/// 处理 restore 命令
pub fn handle_restore(gam_dir: &PathBuf, id: &str, force: bool) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    // 查找快照
    let snapshot = snapshot_store.get_by_prefix(id)?;

    match snapshot {
        Some(snap) => {
            // 确认操作
            if !force {
                println!("此操作将覆盖当前存档。");
                println!(
                    "  快照: {} ({})",
                    Formatter::short_hash(&snap.id),
                    snap.name
                );
                println!("  时间线: {}", snap.timeline);
                println!("  文件数: {}", snap.files.len());
                println!();

                print_confirm("确定继续?");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    print_info("操作已取消");
                    return Ok(());
                }
            }

            // 恢复文件
            let mut restored_count = 0usize;
            for file_entry in &snap.files {
                let source_path = repo.content_store.get(&file_entry.hash)?;

                let target_path = repo.game_path.join(&file_entry.path);

                // 确保父目录存在
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // 复制文件
                std::fs::copy(&source_path, &target_path)?;
                restored_count += 1;
            }

            // 更新 HEAD（如果是分离状态）
            let head_content = std::fs::read_to_string(gam_dir.join("HEAD"))?;
            if !head_content.starts_with("ref:") {
                std::fs::write(gam_dir.join("HEAD"), &snap.id)?;
            }

            // 记录活动
            let engine = ActivityEngine::new(gam_dir);
            engine.log(
                ActivityAction::Restore,
                Some(&snap.timeline),
                Some(&Formatter::short_hash(&snap.id)),
                None,
            )?;

            print_success(&format!(
                "已恢复到快照 {} ({})\n  恢复了 {} 个文件",
                Formatter::short_hash(&snap.id),
                snap.name,
                restored_count
            ));

            Ok(())
        }
        None => Err(crate::core::error::GamError::SnapshotNotFound(
            id.to_string(),
        )),
    }
}

/// 处理 history 命令
pub fn handle_history(gam_dir: &PathBuf, all: bool) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    let snapshots = if all {
        snapshot_store.list_all()?
    } else {
        let current_timeline = repo.get_timeline_name()?;
        snapshot_store.list_by_timeline(&current_timeline)?
    };

    if snapshots.is_empty() {
        print_info("暂无历史记录");
        return Ok(());
    }

    println!("历史记录 (共 {} 个快照)", snapshots.len());
    println!();

    for (i, snapshot) in snapshots.iter().enumerate().take(20) {
        let marker = if i == 0 { "*" } else { " " };
        let time = Formatter::format_time(snapshot.timestamp);
        let short_id = Formatter::short_hash(&snapshot.id);

        println!("{} {}  {}  {}", marker, time, short_id, snapshot.name);
    }

    Ok(())
}

/// 处理 diff 命令 - 比较两个快照
pub fn handle_diff(gam_dir: &PathBuf, id1: &str, id2: &str) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    // 查找两个快照
    let snapshot1 = snapshot_store.get_by_prefix(id1)?;
    let snapshot2 = snapshot_store.get_by_prefix(id2)?;

    let snap1 = match snapshot1 {
        Some(s) => s,
        None => {
            return Err(crate::core::error::GamError::SnapshotNotFound(
                id1.to_string(),
            ))
        }
    };

    let snap2 = match snapshot2 {
        Some(s) => s,
        None => {
            return Err(crate::core::error::GamError::SnapshotNotFound(
                id2.to_string(),
            ))
        }
    };

    // 比较快照
    println!("比较快照");
    println!(
        "  快照1: {} ({})",
        Formatter::short_hash(&snap1.id),
        snap1.name
    );
    println!(
        "  快照2: {} ({})",
        Formatter::short_hash(&snap2.id),
        snap2.name
    );
    println!();

    // 构建文件哈希映射
    let files1: std::collections::HashMap<String, &FileEntry> = snap1
        .files
        .iter()
        .map(|f| (f.path.to_string_lossy().to_string(), f))
        .collect();

    let files2: std::collections::HashMap<String, &FileEntry> = snap2
        .files
        .iter()
        .map(|f| (f.path.to_string_lossy().to_string(), f))
        .collect();

    // 找出添加、删除、修改的文件
    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    // 检查添加和修改的文件
    for (path, file2) in &files2 {
        match files1.get(path) {
            Some(file1) => {
                if file1.hash != file2.hash {
                    modified.push((path.clone(), file1.size, file2.size));
                } else {
                    unchanged.push(path.clone());
                }
            }
            None => {
                added.push((path.clone(), file2.size));
            }
        }
    }

    // 检查删除的文件
    for (path, _) in &files1 {
        if !files2.contains_key(path) {
            deleted.push(path.clone());
        }
    }

    // 输出结果
    println!("文件变更统计:");
    println!("  新增: {} 个", added.len());
    println!("  删除: {} 个", deleted.len());
    println!("  修改: {} 个", modified.len());
    println!("  未变: {} 个", unchanged.len());
    println!();

    // 显示新增的文件
    if !added.is_empty() {
        println!("新增文件:");
        for (path, size) in &added {
            println!("  + {} ({})", path, Formatter::format_size(*size));
        }
        println!();
    }

    // 显示删除的文件
    if !deleted.is_empty() {
        println!("删除文件:");
        for path in &deleted {
            println!("  - {}", path);
        }
        println!();
    }

    // 显示修改的文件
    if !modified.is_empty() {
        println!("修改文件:");
        for (path, old_size, new_size) in &modified {
            let change = if *new_size > *old_size {
                format!("+{}", Formatter::format_size(new_size - old_size))
            } else if *new_size < *old_size {
                format!("-{}", Formatter::format_size(old_size - new_size))
            } else {
                "无变化".to_string()
            };
            println!(
                "  ~ {} ({} → {}) [{}]",
                path,
                Formatter::format_size(*old_size),
                Formatter::format_size(*new_size),
                change
            );
        }
        println!();
    }

    // 计算大小变化
    let old_total: u64 = snap1.files.iter().map(|f| f.size).sum();
    let new_total: u64 = snap2.files.iter().map(|f| f.size).sum();
    let size_change = new_total as i64 - old_total as i64;

    println!(
        "总大小变化: {} → {} ({}{})",
        Formatter::format_size(old_total),
        Formatter::format_size(new_total),
        if size_change >= 0 { "+" } else { "" },
        Formatter::format_size(size_change.abs() as u64)
    );

    Ok(())
}

/// 处理 status 命令
pub fn handle_status(gam_dir: &PathBuf) -> GamResult<()> {
    let repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;
    let snapshot_store = &repo.snapshot_store;

    // 获取当前时间线
    let current_timeline = repo.get_timeline_name()?;

    // 获取快照数量
    let snapshots = snapshot_store.list_by_timeline(&current_timeline)?;
    let snapshot_count = snapshots.len();

    // 计算大小
    let game_size = FileUtils::get_dir_size(&repo.game_path)?;
    let store_size = repo.content_store.deduplication_savings();

    let output = Formatter::format_status(
        &current_timeline,
        snapshot_count as u32,
        game_size,
        store_size,
    );
    println!("{}", output);

    Ok(())
}

/// 处理 activity 命令
pub fn handle_activity(gam_dir: &PathBuf, limit: u32) -> GamResult<()> {
    let engine = ActivityEngine::new(gam_dir);

    let entries = engine.get_entries(limit as usize)?;

    if entries.is_empty() {
        print_info("暂无活动记录");
        return Ok(());
    }

    println!("活动记录 (最近 {} 条):", entries.len());
    println!();

    for entry in &entries {
        println!("  {}", entry.to_display_string());
    }

    Ok(())
}

/// 处理 config 命令
pub fn handle_config(
    gam_dir: &PathBuf,
    key: Option<String>,
    value: Option<String>,
    list: bool,
) -> GamResult<()> {
    let config_path = gam_dir.join("config");

    if !config_path.exists() {
        print_info("配置文件不存在");
        return Ok(());
    }

    // 读取配置
    let content = std::fs::read_to_string(&config_path)?;

    // 列出所有配置
    if list {
        println!("当前配置:");
        println!();
        println!("{}", content);
        return Ok(());
    }

    // 获取配置值
    if let Some(k) = key {
        if value.is_none() {
            // 显示配置值 - 简单文本搜索
            let lines: Vec<&str> = content.lines().collect();
            let parts: Vec<&str> = k.split('.').collect();
            if parts.len() != 2 {
                print_error("无效的配置项格式，使用 'section.key' 格式");
                return Ok(());
            }

            let section = parts[0];
            let key_name = parts[1];
            let mut found = false;

            let mut in_section = false;
            for line in lines {
                let trimmed = line.trim();

                // 检查是否进入新段
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_section = &trimmed[1..trimmed.len() - 1] == section;
                }

                // 在正确段中查找配置项
                if in_section && trimmed.starts_with(key_name) && trimmed.contains('=') {
                    if let Some(pos) = trimmed.find('=') {
                        let val = &trimmed[pos + 1..].trim();
                        println!("{} = {}", k, val);
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                print_info(&format!("配置项 '{}' 不存在", k));
            }
        } else {
            // 设置配置值 - 简单文本替换
            let v = value.unwrap();
            let parts: Vec<&str> = k.split('.').collect();
            if parts.len() != 2 {
                print_error("无效的配置项格式，使用 'section.key' 格式");
                return Ok(());
            }

            let section = parts[0];
            let key_name = parts[1];

            // 简单处理：直接修改配置文件的对应行
            let new_line = format!("{} = {}", key_name, v);
            let lines: Vec<&str> = content.lines().collect();
            let mut found = false;
            let mut new_lines: Vec<String> = Vec::new();
            let mut in_section = false;

            for line in lines {
                let trimmed = line.trim();
                let mut line = line.to_string();

                // 检查是否进入新段
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_section = &trimmed[1..trimmed.len() - 1] == section;
                }

                // 在正确段中查找配置项
                if in_section && trimmed.starts_with(key_name) && trimmed.contains('=') {
                    line = new_line.clone();
                    found = true;
                }

                new_lines.push(line);
            }

            if found {
                let new_content = new_lines.join("\n");
                std::fs::write(&config_path, new_content)?;
                print_success(&format!("已设置 {} = {}", k, v));
            } else {
                // 如果没找到，追加新配置
                let mut new_content = content;
                if !new_content.ends_with('\n') {
                    new_content.push('\n');
                }
                new_content.push_str(&format!("[{}]\n{} = {}\n", section, key_name, v));
                std::fs::write(&config_path, new_content)?;
                print_success(&format!("已添加 {} = {}", k, v));
            }
        }
    } else {
        // 没有参数，显示帮助
        println!("用法:");
        println!("  gam config --list          列出所有配置");
        println!("  gam config <key>           查看配置值");
        println!("  gam config <key> <value>   设置配置值");
        println!();
        println!("示例:");
        println!("  gam config --list");
        println!("  gam config core.default_timeline");
        println!("  gam config core.default_timeline main");
    }

    Ok(())
}

/// 处理 gc 命令
pub fn handle_gc(gam_dir: &PathBuf, aggressive: bool, dry_run: bool) -> GamResult<()> {
    let mut repo = Repository::new(gam_dir.clone(), get_game_path(gam_dir)?)?;

    // 收集所有被引用的内容哈希
    let mut referenced_hashes = std::collections::HashSet::new();

    // 1. 收集所有快照中的文件哈希
    let all_snapshots = repo.snapshot_store.list_all()?;
    for snapshot in &all_snapshots {
        for file in &snapshot.files {
            referenced_hashes.insert(file.hash.clone());
        }
    }

    // 计算将被清理的空间
    let mut orphaned_content_size = 0u64;
    let mut orphaned_content_count = 0u64;

    // 遍历内容存储中的所有文件
    for prefix_dir in std::fs::read_dir(&repo.content_store.root)? {
        let prefix_dir = prefix_dir?;
        if prefix_dir.file_type()?.is_dir() {
            let prefix_name = prefix_dir.file_name().to_string_lossy().to_string();
            for file in std::fs::read_dir(prefix_dir.path())? {
                let file = file?;
                if file.file_type()?.is_file() {
                    let file_name = file.file_name().to_string_lossy().to_string();
                    // 尝试构建完整哈希
                    if file_name.len() >= 64 {
                        let full_hash = format!("{}{}", prefix_name, file_name);

                        if !referenced_hashes.contains(&full_hash) {
                            // 未引用的内容
                            let size = std::fs::metadata(file.path())?.len();
                            orphaned_content_size += size;
                            orphaned_content_count += 1;

                            if dry_run {
                                // 预览模式：只计算大小
                            } else {
                                // 实际删除
                                std::fs::remove_file(file.path())?;
                                // 从索引中移除
                                repo.content_store.index.entries.remove(&full_hash);
                            }
                        }
                    }
                }
            }
        }
    }

    // 清理空的哈希前缀目录
    if aggressive && !dry_run {
        for prefix_dir in std::fs::read_dir(&repo.content_store.root)? {
            let prefix_dir = prefix_dir?;
            if prefix_dir.file_type()?.is_dir() {
                if std::fs::read_dir(prefix_dir.path())?.next().is_none() {
                    std::fs::remove_dir(prefix_dir.path())?;
                }
            }
        }
    }

    // 保存更新后的索引
    if !dry_run {
        repo.content_store.save_index()?;
    }

    // 查找孤立快照（没有时间线引用的快照）
    let mut orphaned_snapshots = Vec::new();
    for snapshot in &all_snapshots {
        let is_referenced = repo.timeline_manager.is_snapshot_referenced(&snapshot.id)?;
        if !is_referenced {
            orphaned_snapshots.push(snapshot.clone());
        }
    }

    let orphaned_snapshot_size = if aggressive && !dry_run {
        let mut total = 0u64;
        for snapshot in &orphaned_snapshots {
            // 删除快照文件
            let snapshot_path = repo
                .snapshot_store
                .snapshot_dir
                .join(&snapshot.id[..2])
                .join(&snapshot.id);
            if snapshot_path.exists() {
                total += std::fs::metadata(&snapshot_path)?.len();
                std::fs::remove_file(&snapshot_path)?;
            }
        }
        total
    } else {
        // 只计算大小
        orphaned_snapshots.iter().map(|s| s.size).sum()
    };

    let total_freed = if dry_run {
        orphaned_content_size + orphaned_snapshot_size
    } else {
        orphaned_content_size + orphaned_snapshot_size
    };

    if dry_run {
        print_info(&format!(
            "预览模式 - 将清理以下内容:\n  未引用的内容文件: {} 个 ({})\n  孤立快照: {} 个 ({})\n  总计将释放: {}",
            orphaned_content_count,
            Formatter::format_size(orphaned_content_size),
            orphaned_snapshots.len(),
            Formatter::format_size(orphaned_snapshot_size),
            Formatter::format_size(total_freed)
        ));
    } else {
        print_success(&format!(
            "垃圾回收完成:\n  清理了 {} 个未引用的内容文件 ({})\n  清理了 {} 个孤立快照 ({})\n  总计释放空间: {}",
            orphaned_content_count,
            Formatter::format_size(orphaned_content_size),
            orphaned_snapshots.len(),
            Formatter::format_size(orphaned_snapshot_size),
            Formatter::format_size(total_freed)
        ));
    }

    // 记录活动（仅在实际执行时）
    if !dry_run {
        let engine = ActivityEngine::new(gam_dir);
        engine.log(
            ActivityAction::Gc,
            None,
            Some(&format!(
                "{} 个内容, {} 个快照",
                orphaned_content_count,
                orphaned_snapshots.len()
            )),
            None,
        )?;
    }

    Ok(())
}

/// 处理 doctor 命令 - 诊断并修复仓库问题
pub fn handle_doctor(gam_dir: &PathBuf, fix: bool) -> GamResult<()> {
    let mut issues = Vec::new();
    let mut fixes = Vec::new();

    println!("Game Archive Manager - 健康检查");
    println!("================================\n");

    // 1. 检查 .gam 目录是否存在
    let gam_exists = gam_dir.exists();
    if !gam_exists {
        issues.push("未找到 .gam 目录，仓库未初始化");
        if fix {
            // 提示用户运行 gam init
            fixes.push("请运行 'gam init --path <存档目录>' 初始化仓库");
        }
    } else {
        println!("✓ .gam 目录存在");

        // 2. 检查配置文件
        let config_path = gam_dir.join("config");
        if !config_path.exists() {
            issues.push("配置文件 config 不存在");
            if fix {
                // 尝试从 HEAD 推断
                fixes.push("配置文件损坏，请手动删除 .gam 目录后重新运行 gam init");
            }
        } else {
            println!("✓ 配置文件存在");

            // 验证配置文件格式
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if content.parse::<toml::Value>().is_ok() {
                    println!("✓ 配置文件格式正确");
                } else {
                    issues.push("配置文件格式错误");
                    if fix {
                        fixes.push("配置文件损坏，请手动删除 .gam 目录后重新运行 gam init");
                    }
                }
            }
        }

        // 3. 检查 HEAD
        let head_path = gam_dir.join("HEAD");
        if !head_path.exists() {
            issues.push("HEAD 文件不存在");
            if fix {
                std::fs::write(&head_path, "ref: refs/timelines/main\n")?;
                fixes.push("已创建 HEAD 文件并指向 main 时间线");
            }
        } else {
            let head_content = std::fs::read_to_string(&head_path)?;
            if head_content.starts_with("ref:") {
                let timeline = head_content
                    .trim_start_matches("ref: refs/timelines/")
                    .trim();
                if timeline.is_empty() {
                    issues.push("HEAD 引用了空的时间线");
                    if fix {
                        std::fs::write(&head_path, "ref: refs/timelines/main\n")?;
                        fixes.push("已修复 HEAD，指向 main 时间线");
                    }
                } else {
                    println!("✓ HEAD 指向时间线 '{}'", timeline);
                }
            } else {
                // 分离 HEAD 状态
                if head_content.trim().len() >= 8 {
                    println!("✓ HEAD 处于分离状态 (指向快照)");
                } else {
                    issues.push("HEAD 内容无效");
                    if fix {
                        std::fs::write(&head_path, "ref: refs/timelines/main\n")?;
                        fixes.push("已修复 HEAD");
                    }
                }
            }
        }

        // 4. 检查 objects 目录结构
        let objects_dir = gam_dir.join("objects");
        if !objects_dir.exists() {
            issues.push("objects 目录不存在");
            if fix {
                std::fs::create_dir_all(&objects_dir.join("snapshot"))?;
                std::fs::create_dir_all(&objects_dir.join("content"))?;
                fixes.push("已创建 objects 目录结构");
            }
        } else {
            println!("✓ objects 目录存在");

            // 检查 snapshot 子目录
            let snapshot_dir = objects_dir.join("snapshot");
            if !snapshot_dir.exists() {
                issues.push("snapshot 目录不存在");
                if fix {
                    std::fs::create_dir_all(&snapshot_dir)?;
                    fixes.push("已创建 snapshot 目录");
                }
            } else {
                println!("✓ snapshot 目录存在");
            }

            // 检查 content 子目录
            let content_dir = objects_dir.join("content");
            if !content_dir.exists() {
                issues.push("content 目录不存在");
                if fix {
                    std::fs::create_dir_all(&content_dir)?;
                    fixes.push("已创建 content 目录");
                }
            } else {
                println!("✓ content 目录存在");
            }
        }

        // 5. 检查 refs 目录
        let refs_dir = gam_dir.join("refs");
        if !refs_dir.exists() {
            issues.push("refs 目录不存在");
            if fix {
                std::fs::create_dir_all(&refs_dir.join("timelines"))?;
                fixes.push("已创建 refs 目录结构");
            }
        } else {
            let timelines_dir = refs_dir.join("timelines");
            if !timelines_dir.exists() {
                issues.push("timelines 目录不存在");
                if fix {
                    std::fs::create_dir_all(&timelines_dir)?;
                    // 创建默认 main 时间线
                    std::fs::write(&timelines_dir.join("main"), "")?;
                    fixes.push("已创建 timelines 目录并创建 main 时间线");
                }
            } else {
                println!("✓ timelines 目录存在");

                // 检查至少有一个时间线
                let timelines = std::fs::read_dir(&timelines_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().unwrap().is_file())
                    .count();

                if timelines == 0 {
                    issues.push("没有时间线存在");
                    if fix {
                        std::fs::write(&timelines_dir.join("main"), "")?;
                        fixes.push("已创建默认 main 时间线");
                    }
                } else {
                    println!("✓ 存在 {} 个时间线", timelines);
                }
            }
        }

        // 6. 检查 .gamignore 格式（如果存在）
        let ignore_file = gam_dir.join(".gamignore");
        if ignore_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&ignore_file) {
                if IgnoreEngine::parse_gamignore(&content).is_ok() {
                    println!("✓ .gamignore 格式正确");
                } else {
                    issues.push(".gamignore 格式错误");
                    if fix {
                        fixes.push(".gamignore 格式错误，请手动检查并修复");
                    }
                }
            }
        }
    }

    // 输出结果
    println!("\n================================");

    if issues.is_empty() {
        print_success("仓库状态良好，没有发现问题");
    } else {
        println!("\n发现 {} 个问题:", issues.len());
        for (i, issue) in issues.iter().enumerate() {
            println!("  {}. {}", i + 1, issue);
        }

        if !fixes.is_empty() {
            println!("\n已修复 {} 个问题:", fixes.len());
            for (i, fix_msg) in fixes.iter().enumerate() {
                println!("  {}. {}", i + 1, fix_msg);
            }
        } else if fix {
            println!("\n部分问题需要手动处理:");
            for fix_msg in fixes {
                println!("  - {}", fix_msg);
            }
        }
    }

    if !issues.is_empty() && !fix {
        println!("\n提示: 使用 --fix 选项自动尝试修复问题");
    }

    Ok(())
}

/// 处理 ignore list 命令
pub fn handle_ignore_list(gam_dir: &PathBuf) -> GamResult<()> {
    let engine = load_ignore_engine(gam_dir)?;
    let patterns = engine.patterns();

    if patterns.is_empty() {
        print_info("暂无忽略规则");
        return Ok(());
    }

    println!("当前忽略规则 (共 {} 条):", patterns.len());
    println!();

    for (i, pattern) in patterns.iter().enumerate() {
        let neg = if pattern.negated { "!" } else { "" };
        println!("  {}. {}{}", i + 1, neg, pattern.pattern);
    }

    Ok(())
}

/// 处理 ignore init 命令
pub fn handle_ignore_init(gam_dir: &PathBuf, force: bool) -> GamResult<()> {
    let ignore_file = gam_dir.join(".gamignore");

    if ignore_file.exists() && !force {
        print_info(".gamignore 已存在，使用 --force 覆盖");
        return Ok(());
    }

    let template = IgnoreEngine::default_gamignore_template();
    std::fs::write(&ignore_file, template)?;

    print_success("已创建 .gamignore 模板");

    Ok(())
}

/// 处理 ignore add 命令
pub fn handle_ignore_add(gam_dir: &PathBuf, pattern: &str) -> GamResult<()> {
    let ignore_file = gam_dir.join(".gamignore");

    // 读取现有内容
    let mut content = if ignore_file.exists() {
        std::fs::read_to_string(&ignore_file)?
    } else {
        String::new()
    };

    // 添加新规则
    content.push_str(&format!("{}\n", pattern));

    std::fs::write(&ignore_file, content)?;

    // 记录活动
    let engine = ActivityEngine::new(gam_dir);
    engine.log(ActivityAction::IgnoreAdd, None, Some(pattern), None)?;

    print_success(&format!("已添加规则: {}", pattern));

    Ok(())
}

/// 处理 ignore remove 命令
pub fn handle_ignore_remove(gam_dir: &PathBuf, pattern: &str) -> GamResult<()> {
    let ignore_file = gam_dir.join(".gamignore");

    if !ignore_file.exists() {
        print_info(".gamignore 不存在");
        return Ok(());
    }

    // 读取现有内容
    let content = std::fs::read_to_string(&ignore_file)?;
    let lines: Vec<&str> = content.lines().collect();

    // 查找并移除匹配的规则
    let mut removed = false;
    let mut new_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // 保留空行
            new_lines.push(line);
        } else if trimmed == pattern {
            // 移除匹配的规则
            removed = true;
        } else {
            new_lines.push(line);
        }
    }

    if removed {
        std::fs::write(&ignore_file, new_lines.join("\n"))?;

        // 记录活动
        let engine = ActivityEngine::new(gam_dir);
        engine.log(ActivityAction::IgnoreRemove, None, Some(pattern), None)?;

        print_success(&format!("已移除规则: {}", pattern));
    } else {
        print_info(&format!("未找到规则: {}", pattern));
    }

    Ok(())
}

/// 处理 ignore check 命令
pub fn handle_ignore_check(gam_dir: &PathBuf, file: &str) -> GamResult<()> {
    let engine = load_ignore_engine(gam_dir)?;
    let path = std::path::PathBuf::from(file);

    let is_ignored = engine.is_ignored(&path, path.is_dir());

    println!("检查文件: {}", file);
    println!("结果: {}", if is_ignored { "忽略" } else { "不忽略" });

    // 显示匹配的原因
    let patterns = engine.patterns();
    for p in patterns {
        if p.negated {
            // 否定模式
            if engine.matches_negated(&path, p.pattern.clone()) {
                println!("  匹配否定规则: !{}", p.pattern);
            }
        } else {
            // 普通忽略规则
            if engine.matches_pattern(&path, p.pattern.clone()) {
                println!("  匹配规则: {}", p.pattern);
            }
        }
    }

    Ok(())
}

/// 确认提示
fn print_confirm(msg: &str) {
    print!("  {} (y/n): ", msg);
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}
