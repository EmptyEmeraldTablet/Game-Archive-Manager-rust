//! UI 模块
//!
//! 提供命令行输出格式化功能

use std::io::{self, Write};

pub mod formatter;

pub use formatter::Formatter;

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Blue,
    Green,
    Yellow,
    Red,
    Reset,
}

pub fn set_color(color: Color) {
    let escape_code = match color {
        Color::Blue => "\x1b[34m",
        Color::Green => "\x1b[32m",
        Color::Yellow => "\x1b[33m",
        Color::Red => "\x1b[31m",
        Color::Reset => "\x1b[0m",
    };

    print!("{}", escape_code);
    io::stdout().flush().unwrap();
}

pub fn msg_log(msg: &str) {
    set_color(Color::Blue);
    println!("{}", msg);
    set_color(Color::Reset);
}

pub fn msg_suc(msg: &str) {
    set_color(Color::Green);
    println!("{}", msg);
    set_color(Color::Reset);
}

pub fn msg_wrn(msg: &str) {
    set_color(Color::Yellow);
    println!("{}", msg);
    set_color(Color::Reset);
}

pub fn msg_err(msg: &str) {
    set_color(Color::Red);
    println!("{}", msg);
    set_color(Color::Reset);
}

pub fn clear_screen() {
    print!("\x1b[2J\x1b[3J\x1b[H");
    io::stdout().flush().unwrap();
}

pub fn print_version() {
    msg_suc("==================== Game Archive Manager v2.0 ====================");
}

pub fn print_title() {
    clear_screen();
    print_version();
    msg_suc("输入 help 或 h 查看帮助，输入 q 退出\n");
}

/// Get the global message manager
pub fn messages() -> crate::core::MessageManager {
    crate::core::global_messages()
}

pub fn print_help() {
    println!();
    msg_suc("【使用说明】");
    println!();
    println!("  游戏存档版本控制系统 - 像 Git 一样管理游戏存档");
    println!();
    msg_wrn("【注意事项】");
    println!();
    println!("  1. 程序会在游戏存档目录创建 .gam 目录");
    println!("  2. 请关闭游戏后再进行存档/读档操作");
    println!("  3. 游戏进行中请勿读取存档");
    println!();
    msg_suc("【命令列表】");
    println!();
    println!("  {:<30} {}", "命令", "说明");
    println!("  {:-<30} {:-}", "─", "─");

    let commands = [
        ("init", "初始化版本控制"),
        ("snapshot save [-m msg]", "保存快照"),
        ("snapshot list", "列出快照"),
        ("snapshot info <id>", "查看快照详情"),
        ("snapshot delete <id>", "删除快照"),
        ("snapshot tag <id> <name>", "为快照添加标签"),
        ("timeline create <name>", "创建时间线"),
        ("timeline list", "列出时间线"),
        ("timeline switch <name>", "切换时间线"),
        ("timeline rename <old> <new>", "重命名时间线"),
        ("timeline current", "显示当前时间线"),
        ("restore <id>", "恢复到快照"),
        ("history", "查看历史"),
        ("activity", "查看活动日志"),
        ("status", "查看状态"),
        ("diff <id1> <id2>", "比较快照"),
        ("ignore <subcommand>", "忽略规则管理"),
        ("config", "配置管理"),
        ("gc", "垃圾回收"),
        ("doctor", "诊断问题"),
        ("help", "显示帮助"),
        ("quit / q", "退出"),
    ];

    for (cmd, desc) in &commands {
        println!("  {:<30} {}", cmd, desc);
    }
    println!();
    msg_suc("【示例】");
    println!();
    println!("  初始化:        gam init --path /path/to/saves");
    println!("  保存快照:      gam snapshot save -m \"Boss beaten\"");
    println!("  列出快照:      gam snapshot list");
    println!("  恢复到快照:    gam restore 1");
    println!();
}

pub fn print_prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}

pub fn print_loading_prompt() {
    print!("  正在加载...");
    io::stdout().flush().unwrap();
}

pub fn print_success(msg: &str) {
    msg_suc(&format!("  [成功] {}", msg));
}

pub fn print_error(msg: &str) {
    msg_err(&format!("  [错误] {}", msg));
}

pub fn print_warning(msg: &str) {
    msg_wrn(&format!("  [警告] {}", msg));
}

pub fn print_info(msg: &str) {
    msg_log(&format!("  [信息] {}", msg));
}

pub fn print_confirm(msg: &str) {
    set_color(Color::Yellow);
    print!("  {} (y/n): ", msg);
    io::stdout().flush().unwrap();
    set_color(Color::Reset);
}

/// Print success message using message key
#[macro_export]
macro_rules! t_success {
    ($key:expr, $($vars:expr),*) => {
        print_success(&crate::core::global_messages().t($key, &[$($vars),*]))
    };
}

/// Print error message using message key
#[macro_export]
macro_rules! t_error {
    ($key:expr, $($vars:expr),*) => {
        print_error(&crate::core::global_messages().t($key, &[$($vars),*]))
    };
}

/// Print warning message using message key
#[macro_export]
macro_rules! t_warning {
    ($key:expr, $($vars:expr),*) => {
        print_warning(&crate::core::global_messages().t($key, &[$($vars),*]))
    };
}

/// Print info message using message key
#[macro_export]
macro_rules! t_info {
    ($key:expr, $($vars:expr),*) => {
        print_info(&crate::core::global_messages().t($key, &[$($vars),*]))
    };
}

/// Get translated message
#[macro_export]
macro_rules! t {
    ($key:expr, $($vars:expr),*) => {
        crate::core::global_messages().t($key, &[$($vars),*])
    };
}
