use crate::core::ArchiveManager;
use crate::ui::{msg_err, msg_log, msg_suc, msg_wrn};
use std::io::{self, Write};

pub struct CommandHandler {
    manager: ArchiveManager,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            manager: ArchiveManager::new().unwrap_or_else(|e| {
                msg_err(&format!("初始化失败: {}", e));
                msg_wrn("请确保程序目录下存在 path.txt 文件，并正确配置游戏存档路径");
                std::process::exit(1);
            }),
        }
    }

    pub fn run(&mut self) {
        loop {
            print!(">>> ");
            io::stdout().flush().unwrap();

            let input = self.read_input();
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if let Some(cmd) = self.parse_command(input) {
                if self.execute_command(cmd).is_err() {
                    break;
                }
            }
        }
    }

    fn read_input(&self) -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        input
    }

    fn parse_command(&self, input: &str) -> Option<Command> {
        if let Ok(num) = input.parse::<usize>() {
            return Some(Command::Index(num));
        }

        match input.to_lowercase().as_str() {
            "quit" | "q" => Some(Command::Quit),
            "help" | "h" => Some(Command::Help),
            "clearscreen" | "cls" => Some(Command::ClearScreen),
            "save" | "s" => Some(Command::Save),
            "qsave" | "qs" => Some(Command::QuickSave),
            "rsave" | "rs" => Some(Command::ReplaceSave),
            "load" | "l" => Some(Command::Load),
            "qload" | "ql" => Some(Command::QuickLoad),
            "log" | "lo" => Some(Command::Log),
            "slog" | "sl" => Some(Command::ShortLog),
            "marchive" | "ma" => Some(Command::ModifyArchive),
            "delarch" | "del" => Some(Command::DeleteArchive),
            "qdelete" | "qd" => Some(Command::QuickDelete),
            "usage" | "use" => Some(Command::Usage),
            _ => {
                msg_err(&format!("未知命令: {}", input));
                None
            }
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), ()> {
        match cmd {
            Command::Quit => {
                msg_log("退出程序");
                return Err(());
            }
            Command::Help => self.do_help(),
            Command::ClearScreen => self.do_clear_screen(),
            Command::Save => self.do_save(),
            Command::QuickSave => self.do_quick_save(),
            Command::ReplaceSave => self.do_replace_save(),
            Command::Load => self.do_load(),
            Command::QuickLoad => self.do_quick_load(),
            Command::Log => self.do_log(),
            Command::ShortLog => self.do_short_log(),
            Command::ModifyArchive => self.do_modify_archive(),
            Command::DeleteArchive => self.do_delete_archive(),
            Command::QuickDelete => self.do_quick_delete(),
            Command::Usage => self.do_usage(),
            Command::Index(_) => unreachable!(),
        }
        Ok(())
    }

    fn do_help(&self) {
        crate::ui::print_help();
    }

    fn do_clear_screen(&self) {
        crate::ui::print_title();
        crate::ui::print_commands();
    }

    fn do_save(&mut self) {
        msg_log("请输入存档信息 (直接回车取消)");

        print!("存档名 (必填, 限32字): ");
        io::stdout().flush().unwrap();
        let name = self.read_input();
        let name = name.trim().to_string();

        if name.is_empty() {
            msg_log("取消保存");
            return;
        }

        if name.len() > 32 {
            msg_err("存档名超出32字符限制");
            return;
        }

        print!("存档备注 (可不填, 限1024字): ");
        io::stdout().flush().unwrap();
        let comment = self.read_input();
        let comment = comment.trim().to_string();

        if comment.len() > 1024 {
            msg_err("备注超出1024字符限制");
            return;
        }

        match self.manager.save(name, comment) {
            Ok(_) => msg_suc("保存成功"),
            Err(e) => msg_err(&format!("保存失败: {}", e)),
        }
    }

    fn do_quick_save(&mut self) {
        match self.manager.quick_save() {
            Ok(_) => msg_suc("快速保存成功"),
            Err(e) => msg_err(&format!("保存失败: {}", e)),
        }
    }

    fn do_replace_save(&mut self) {
        match self.manager.replace_save() {
            Ok(_) => msg_suc("覆盖保存成功"),
            Err(e) => msg_err(&format!("覆盖保存失败: {}", e)),
        }
    }

    fn do_load(&mut self) {
        if self.manager.archive_count() == 0 {
            msg_err("无存档可读取");
            return;
        }

        print!("需要读取的存档序号 (0取消): ");
        io::stdout().flush().unwrap();
        let input = self.read_input();
        let index: usize = match input.trim().parse::<usize>() {
            Ok(n) => n.saturating_sub(1),
            Err(_) => {
                msg_log("取消读取");
                return;
            }
        };

        if index >= self.manager.archive_count() {
            msg_log("取消读取");
            return;
        }

        msg_wrn("此过程会覆盖游戏中现有的存档, 请谨慎操作!");

        print!(
            "确定要读取存档 [{}] {} 吗 (y/n): ",
            index + 1,
            self.manager.get_all_archives()[index].name
        );
        io::stdout().flush().unwrap();

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            msg_log("取消读取");
            return;
        }

        match self.manager.load(index) {
            Ok(_) => msg_suc("读取成功"),
            Err(e) => msg_err(&format!("读取失败: {}", e)),
        }
    }

    fn do_quick_load(&mut self) {
        if self.manager.archive_count() == 0 {
            msg_err("无存档可读取");
            return;
        }

        msg_wrn("此过程会覆盖游戏中现有的存档, 请谨慎操作!");

        let index = self.manager.archive_count() - 1;
        print!(
            "确定要读取最新存档 [{}] {} 吗 (y/n): ",
            index + 1,
            self.manager.get_all_archives()[index].name
        );
        io::stdout().flush().unwrap();

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            msg_log("取消读取");
            return;
        }

        match self.manager.quick_load() {
            Ok(_) => msg_suc("读取成功"),
            Err(e) => msg_err(&format!("读取失败: {}", e)),
        }
    }

    fn do_log(&self) {
        let archives = self.manager.get_all_archives();
        msg_log(
            "------------------------------------------------------------------------------------",
        );
        msg_log(&format!("存档总数: {}", archives.len()));

        for (i, info) in archives.iter().enumerate() {
            println!(
                "{}\t{}\t{}\t{}",
                i + 1,
                info.timestamp,
                info.name,
                info.comment
            );
        }

        msg_log(
            "------------------------------------------------------------------------------------",
        );
    }

    fn do_short_log(&self) {
        let archives = self.manager.get_recent_archives(7);
        let start = self.manager.archive_count().saturating_sub(archives.len());

        msg_log(
            "------------------------------------------------------------------------------------",
        );
        msg_log(&format!("存档总数: {}", archives.len()));

        for (i, info) in archives.iter().enumerate() {
            let real_index = start + i;
            println!(
                "{}\t{}\t{}\t{}",
                real_index + 1,
                info.timestamp,
                info.name,
                info.comment
            );
        }

        msg_log(
            "------------------------------------------------------------------------------------",
        );
    }

    fn do_modify_archive(&mut self) {
        if self.manager.archive_count() == 0 {
            msg_err("无存档可修改");
            return;
        }

        print!("需要修改的存档序号 (0取消): ");
        io::stdout().flush().unwrap();
        let input = self.read_input();
        let index: usize = match input.trim().parse::<usize>() {
            Ok(n) => n.saturating_sub(1),
            Err(_) => {
                msg_log("取消修改");
                return;
            }
        };

        if index >= self.manager.archive_count() {
            msg_log("取消修改");
            return;
        }

        print!("新存档名 (直接回车保持原名): ");
        io::stdout().flush().unwrap();
        let name = self.read_input();
        let name = if name.trim().is_empty() {
            None
        } else {
            Some(name.trim().to_string())
        };

        if let Some(ref n) = name {
            if n.len() > 32 {
                msg_err("存档名超出32字符限制");
                return;
            }
        }

        print!("新备注 (直接回车保持原备注): ");
        io::stdout().flush().unwrap();
        let comment = self.read_input();
        let comment = if comment.trim().is_empty() {
            None
        } else {
            Some(comment.trim().to_string())
        };

        if let Some(ref c) = comment {
            if c.len() > 1024 {
                msg_err("备注超出1024字符限制");
                return;
            }
        }

        match self.manager.modify(index, name, comment) {
            Ok(_) => msg_suc("修改成功"),
            Err(e) => msg_err(&format!("修改失败: {}", e)),
        }
    }

    fn do_delete_archive(&mut self) {
        if self.manager.archive_count() == 0 {
            msg_err("无存档可删除");
            return;
        }

        print!("需要删除的存档序号 (0取消): ");
        io::stdout().flush().unwrap();
        let input = self.read_input();
        let index: usize = match input.trim().parse::<usize>() {
            Ok(n) => n.saturating_sub(1),
            Err(_) => {
                msg_log("取消删除");
                return;
            }
        };

        if index >= self.manager.archive_count() {
            msg_log("取消删除");
            return;
        }

        let info = &self.manager.get_all_archives()[index];
        print!("确定要删除存档 [{}] {} 吗 (y/n): ", index + 1, info.name);
        io::stdout().flush().unwrap();

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            msg_log("取消删除");
            return;
        }

        match self.manager.delete(index) {
            Ok(_) => msg_suc("删除成功"),
            Err(e) => msg_err(&format!("删除失败: {}", e)),
        }
    }

    fn do_quick_delete(&mut self) {
        if self.manager.archive_count() == 0 {
            msg_err("无存档可删除");
            return;
        }

        let index = self.manager.archive_count() - 1;
        let info = &self.manager.get_all_archives()[index];
        print!(
            "确定要删除最新存档 [{}] {} 吗 (y/n): ",
            index + 1,
            info.name
        );
        io::stdout().flush().unwrap();

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            msg_log("取消删除");
            return;
        }

        match self.manager.quick_delete() {
            Ok(_) => msg_suc("删除成功"),
            Err(e) => msg_err(&format!("删除失败: {}", e)),
        }
    }

    fn do_usage(&self) {
        let size = self.manager.get_usage_space();
        msg_log(&format!("占用空间: {:.2} MB", size));
    }
}

enum Command {
    Quit,
    Help,
    ClearScreen,
    Save,
    QuickSave,
    ReplaceSave,
    Load,
    QuickLoad,
    Log,
    ShortLog,
    ModifyArchive,
    DeleteArchive,
    QuickDelete,
    Usage,
    Index(usize),
}
