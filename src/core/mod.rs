//! 核心模块
//!
//! 提供 GAM 的核心数据结构和管理功能

pub mod activity;
pub mod commands;
pub mod error;
pub mod ignore;
pub mod store;
pub mod tag;
pub mod types;

pub use activity::{ActivityAction, ActivityEngine, ActivityEntry};
pub use commands::{
    handle_activity, handle_gc, handle_history, handle_ignore_add, handle_ignore_check,
    handle_ignore_init, handle_ignore_list, handle_ignore_remove, handle_init, handle_restore,
    handle_snapshot_info, handle_snapshot_list, handle_snapshot_save, handle_snapshot_tag,
    handle_status, handle_timeline_create, handle_timeline_current, handle_timeline_delete,
    handle_timeline_list, handle_timeline_rename, handle_timeline_switch,
};
pub use error::{GamError, GamResult};
pub use ignore::IgnoreEngine;
pub use tag::TagStore;
pub use types::{
    Config, FileEntry, GamIgnoreConfig, IgnorePattern, PatternType, RetentionPolicy, Snapshot,
    StorageStrategy, Timeline,
};
