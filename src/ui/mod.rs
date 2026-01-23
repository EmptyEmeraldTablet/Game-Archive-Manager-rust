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
    msg_suc("[Usage Guide]");
    println!();
    println!("  Game Archive Manager v2.0 - Version control for game saves like Git");
    println!();
    msg_wrn("[Notes]");
    println!();
    println!("  1. The program creates a .gam directory in the game saves directory");
    println!("  2. Please close the game before save/load operations");
    println!("  3. Do not load saves while the game is running");
    println!();
    msg_suc("[Command List]");
    println!();
    println!("  {:<30} {}", "Command", "Description");
    println!("  {:-<30} {:-}", "-", "-");

    let commands = [
        ("init", "Initialize version control"),
        ("snapshot save [-m msg]", "Save snapshot"),
        ("snapshot list", "List snapshots"),
        ("snapshot info <id>", "View snapshot details"),
        ("snapshot delete <id>", "Delete snapshot"),
        ("snapshot tag <id> <name>", "Add tag to snapshot"),
        ("timeline create <name>", "Create timeline"),
        ("timeline list", "List timelines"),
        ("timeline switch <name>", "Switch timeline"),
        ("timeline rename <old> <new>", "Rename timeline"),
        ("timeline current", "Show current timeline"),
        ("restore <id>", "Restore to snapshot"),
        ("history", "View history"),
        ("activity", "View activity log"),
        ("status", "View status"),
        ("diff <id1> <id2>", "Compare snapshots"),
        ("ignore <subcommand>", "Ignore rules management"),
        ("config", "Configuration management"),
        ("gc", "Garbage collection"),
        ("doctor", "Diagnose issues"),
        ("help", "Show help"),
        ("quit / q", "Quit"),
    ];

    for (cmd, desc) in &commands {
        println!("  {:<30} {}", cmd, desc);
    }
    println!();
    msg_suc("[Examples]");
    println!();
    println!("  Initialize:       gam init --path /path/to/saves");
    println!("  Save snapshot:    gam snapshot save -m \"Boss beaten\"");
    println!("  List snapshots:   gam snapshot list");
    println!("  Restore:          gam restore 1");
    println!();
}

pub fn print_prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}

pub fn print_loading_prompt() {
    let msg = crate::core::global_messages()
        .get("ui.loading")
        .cloned()
        .unwrap_or_else(|| "Loading...".to_string());
    print!("  {}", msg);
    io::stdout().flush().unwrap();
}

pub fn print_success(msg: &str) {
    let prefix = crate::core::global_messages()
        .get("ui.success")
        .map(|s| format!("  [{}] ", s))
        .unwrap_or_else(|| "  [Success] ".to_string());
    msg_suc(&format!("{}{}", prefix, msg));
}

pub fn print_error(msg: &str) {
    let prefix = crate::core::global_messages()
        .get("ui.error")
        .map(|s| format!("  [{}] ", s))
        .unwrap_or_else(|| "  [Error] ".to_string());
    msg_err(&format!("{}{}", prefix, msg));
}

pub fn print_warning(msg: &str) {
    let prefix = crate::core::global_messages()
        .get("ui.warning")
        .map(|s| format!("  [{}] ", s))
        .unwrap_or_else(|| "  [Warning] ".to_string());
    msg_wrn(&format!("{}{}", prefix, msg));
}

pub fn print_info(msg: &str) {
    let prefix = crate::core::global_messages()
        .get("ui.info")
        .map(|s| format!("  [{}] ", s))
        .unwrap_or_else(|| "  [Info] ".to_string());
    msg_log(&format!("{}{}", prefix, msg));
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
