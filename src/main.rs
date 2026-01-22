//! Game Archive Manager v2.0
//!
//! 游戏存档版本控制系统 - 像 Git 一样管理游戏存档

use anyhow::Result;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::process;

mod cli;
mod core;
mod ui;
mod utils;

use clap::Parser;
use cli::Cli;
use core::commands::{
    handle_activity, handle_config, handle_diff, handle_doctor, handle_gc, handle_history,
    handle_ignore_add, handle_ignore_check, handle_ignore_init, handle_ignore_list,
    handle_ignore_remove, handle_init, handle_restore, handle_snapshot_delete,
    handle_snapshot_info, handle_snapshot_list, handle_snapshot_save, handle_snapshot_tag,
    handle_status, handle_timeline_create, handle_timeline_current, handle_timeline_delete,
    handle_timeline_list, handle_timeline_rename, handle_timeline_switch,
};
use ui::{print_error, print_info, print_success};

/// 全局 GAM 目录
static GAM_DIR: Lazy<std::sync::Mutex<Option<PathBuf>>> = Lazy::new(|| std::sync::Mutex::new(None));

/// 获取 GAM 目录
pub fn get_gam_dir() -> PathBuf {
    GAM_DIR
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 设置 GAM 目录
pub fn set_gam_dir(path: PathBuf) {
    *GAM_DIR.lock().unwrap() = Some(path);
}

fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 处理 init 命令（不需要 .gam 目录）
    if let cli::Commands::Init(init_args) = cli.command {
        handle_init(init_args.path, init_args.force)?;
        return Ok(());
    }

    // 检查当前目录是否为 GAM 仓库
    let current_dir = std::env::current_dir()?;
    let gam_dir = current_dir.join(".gam");

    if !gam_dir.exists() {
        print_error("当前目录不是 GAM 仓库。请先运行 'gam init --path <存档目录>' 初始化。");
        process::exit(1);
    }

    set_gam_dir(gam_dir.clone());

    // 处理命令
    let result = handle_command(gam_dir, cli.command);

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            print_error(&format!("错误: {}", e));
            Err(e.into())
        }
    }
}

fn handle_command(gam_dir: PathBuf, command: cli::Commands) -> core::GamResult<()> {
    match command {
        cli::Commands::Init(args) => handle_init(args.path, args.force),

        cli::Commands::Snapshot(args) => match args.command {
            cli::SnapshotCommands::Save(save_args) => {
                handle_snapshot_save(&gam_dir, save_args.message, save_args.timeline)
            }
            cli::SnapshotCommands::List(list_args) => {
                handle_snapshot_list(&gam_dir, list_args.all, list_args.timeline)
            }
            cli::SnapshotCommands::Info(info_args) => handle_snapshot_info(&gam_dir, &info_args.id),
            cli::SnapshotCommands::Delete(delete_args) => {
                handle_snapshot_delete(&gam_dir, &delete_args.id, delete_args.force)
            }
            cli::SnapshotCommands::Tag(tag_args) => {
                handle_snapshot_tag(&gam_dir, &tag_args.id, &tag_args.name)
            }
            cli::SnapshotCommands::Tags(_tags_args) => {
                print_info("snapshot tags 功能尚未实现");
                Ok(())
            }
        },

        cli::Commands::Timeline(args) => match args.command {
            cli::TimelineCommands::Create(create_args) => {
                handle_timeline_create(&gam_dir, &create_args.name, create_args.from)
            }
            cli::TimelineCommands::List => handle_timeline_list(&gam_dir),
            cli::TimelineCommands::Switch(switch_args) => {
                handle_timeline_switch(&gam_dir, &switch_args.target)
            }
            cli::TimelineCommands::Rename(rename_args) => {
                handle_timeline_rename(&gam_dir, &rename_args.old_name, &rename_args.new_name)
            }
            cli::TimelineCommands::Delete(delete_args) => {
                handle_timeline_delete(&gam_dir, &delete_args.name, delete_args.force)
            }
            cli::TimelineCommands::Current => handle_timeline_current(&gam_dir),
        },

        cli::Commands::Restore(restore_args) => {
            handle_restore(&gam_dir, &restore_args.id, restore_args.force)
        }

        cli::Commands::History(history_args) => handle_history(&gam_dir, history_args.all),

        cli::Commands::Status(_args) => handle_status(&gam_dir),

        cli::Commands::Activity(args) => handle_activity(&gam_dir, args.limit),

        cli::Commands::Diff(args) => handle_diff(&gam_dir, &args.id1, &args.id2),

        cli::Commands::Gc(gc_args) => handle_gc(&gam_dir, gc_args.aggressive, gc_args.dry_run),

        cli::Commands::Ignore(args) => match args.command {
            cli::IgnoreCommands::Add(add_args) => handle_ignore_add(&gam_dir, &add_args.pattern),
            cli::IgnoreCommands::Remove(remove_args) => {
                handle_ignore_remove(&gam_dir, &remove_args.pattern)
            }
            cli::IgnoreCommands::List => handle_ignore_list(&gam_dir),
            cli::IgnoreCommands::Check(check_args) => {
                handle_ignore_check(&gam_dir, &check_args.file)
            }
            cli::IgnoreCommands::Init(init_args) => handle_ignore_init(&gam_dir, init_args.force),
        },

        cli::Commands::Config(args) => handle_config(&gam_dir, args.key, args.value, args.list),

        cli::Commands::Doctor(args) => handle_doctor(&gam_dir, args.fix),
    }
}
