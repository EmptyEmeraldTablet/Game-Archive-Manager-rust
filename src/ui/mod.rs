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

pub fn print_title() {
    clear_screen();
    msg_suc("==================== Game Archive Manager ====================");
    msg_suc("================= 初次使用请使用 help 或 h 查看帮助 =================\n");
}

pub fn print_help() {
    println!();
    msg_wrn("---------------------------------------------------------------");
    msg_wrn("1. 本程序运行时会在程序所在目录下建立一个 Archive 文件夹用于存储日志文件和保存的存档");
    msg_wrn("   请勿删除 (若删除, 则相当于恢复程序第一次运行的状态)");
    msg_wrn("2. 建议将程序放在磁盘中的某个文件夹下, 再发送到桌面快捷方式使用");
    msg_wrn("   (避免程序在桌面创建 Archive 文件夹后被误删)");
    msg_wrn("3. 建议关闭 Steam 云存档");
    msg_wrn("4. 请在正常保存、关闭后再进行存档");
    msg_wrn("   (游戏进行中存档的话保存的是自动存档, 可能是几分钟前的存档, 并非保存时的存档)");
    msg_wrn("5. 游戏进行中请勿读取存档");
    msg_wrn("6. 随着游戏的进行, 每次存档所用的时间和占用的空间也会不断增大, 请耐心等待");
    msg_suc("7. 不要把程序放在 C 盘或桌面, 不然程序没有权限往那里复制文件");
    msg_wrn("---------------------------------------------------------------");
    msg_suc("项目地址: https://github.com/yourusername/game-archive-manager");
    println!();
}

pub fn print_commands() {
    set_color(Color::Blue);
    println!("输入操作: (数字/命令/简写)");

    let commands = [
        ("quit", "q", "退出程序"),
        ("help", "h", "帮助信息"),
        ("clearScreen", "cls", "清屏"),
        ("", "", ""),
        ("save", "s", "保存存档"),
        ("qsave", "qs", "快速保存"),
        ("rsave", "rs", "覆盖式保存"),
        ("", "", ""),
        ("load", "l", "读取存档"),
        ("qload", "ql", "快速读取"),
        ("log", "lo", "查看所有存档"),
        ("slog", "sl", "近七次存档"),
        ("", "", ""),
        ("mArchive", "ma", "修改存档信息"),
        ("delArch", "del", "删除指定存档"),
        ("qDelete", "qd", "删除最新存档"),
        ("", "", ""),
        ("usage", "use", "查看占用空间"),
    ];

    for (i, (cmd, short, desc)) in commands.iter().enumerate() {
        if cmd.is_empty() {
            println!();
        } else {
            println!("{}. {:15} ({:3})  {}", i + 1, cmd, short, desc);
        }
    }

    set_color(Color::Reset);
}
