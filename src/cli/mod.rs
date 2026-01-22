//! CLI 模块
//!
//! 提供命令行参数解析

pub mod args;

pub use args::{
    ActivityArgs, AddIgnoreArgs, CheckIgnoreArgs, Cli, Commands, CreateArgs, DeleteArgs,
    DeleteTimelineArgs, DiffArgs, DoctorArgs, GcArgs, HistoryArgs, IgnoreArgs, IgnoreCommands,
    IgnoreInitArgs, InfoArgs, InitArgs, ListArgs, RemoveIgnoreArgs, RenameArgs, RestoreArgs,
    SaveArgs, SnapshotArgs, SnapshotCommands, StatusArgs, SwitchArgs, TimelineArgs,
    TimelineCommands,
};
