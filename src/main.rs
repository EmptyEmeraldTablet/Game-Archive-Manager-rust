use anyhow::Result;
use std::panic;

mod commands;
mod core;
mod ui;
mod utils;

use commands::CommandHandler;
use ui::print_title;

fn main() -> Result<()> {
    // 设置 panic hook
    panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        eprintln!("\x1b[31m程序发生错误: {}\x1b[0m", msg);
        eprintln!(
            "如有问题请提交 issue: https://github.com/yourusername/game-archive-manager/issues"
        );
        std::process::exit(1);
    }));

    // 打印标题
    print_title();

    // 运行命令处理器
    let mut handler = CommandHandler::new();
    handler.run();

    Ok(())
}
