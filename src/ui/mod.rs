use std::io::{self, Write};

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
    msg_suc("==================== Game Archive Manager v1.0 ====================");
}

pub fn print_title() {
    clear_screen();
    print_version();
    msg_suc("输入 help 或 h 查看帮助，输入 q 退出\n");
}

pub fn print_help() {
    println!();
    msg_suc("【使用说明】");
    println!();
    println!("  本程序用于备份和恢复游戏存档");
    println!("  配置文件: path.txt (需放在程序同级目录，内容为存档目录的绝对路径)");
    println!();
    msg_wrn("【注意事项】");
    println!();
    println!("  1. 程序会在同级目录创建 Archive 文件夹用于存储备份");
    println!("  2. 请关闭游戏后再进行存档/读档操作");
    println!("  3. 游戏进行中请勿读取存档");
    println!("  4. 存档会随游戏进度逐渐增大，请耐心等待");
    println!();
    msg_suc("【命令列表】");
    println!();
    println!("  {:<20} {:<8} {}", "命令", "简写", "说明");
    println!("  {:-<20} {:-<8} {:-}", "─", "─", "─");

    let commands = [
        ("quit", "q", "退出程序"),
        ("help", "h", "显示帮助信息"),
        ("clearScreen", "cls", "清屏"),
        ("", "", ""),
        ("save", "s", "保存存档 (需输入名称和备注)"),
        ("qsave", "qs", "快速保存 (无需输入)"),
        ("rsave", "rs", "覆盖保存 (更新最新存档)"),
        ("", "", ""),
        ("load <id>", "l <id>", "读取指定存档"),
        ("qload", "ql", "快速读取 (最新存档)"),
        ("log", "lo", "查看所有存档"),
        ("slog", "sl", "查看最近7次存档"),
        ("", "", ""),
        ("mArchive <id>", "ma <id>", "修改存档信息"),
        ("delArch <id>", "del <id>", "删除指定存档"),
        ("qDelete", "qd", "删除最新存档"),
        ("", "", ""),
        ("usage", "use", "查看占用空间"),
    ];

    for (cmd, short, desc) in &commands {
        if cmd.is_empty() {
            println!();
        } else {
            println!("  {:<20} {:<8} {}", cmd, short, desc);
        }
    }
    println!();
    msg_suc("【示例】");
    println!();
    println!("  保存存档: 输入 s 或 save，按提示操作");
    println!("  读取存档: 输入 l 或 load，输入存档编号");
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
