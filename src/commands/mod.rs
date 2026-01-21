use crate::core::ArchiveManager;
use crate::ui::{
    msg_log, print_confirm, print_error, print_help, print_info, print_prompt, print_success,
    print_title, print_warning,
};
use std::io::{self, Write};

pub struct CommandHandler {
    manager: ArchiveManager,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            manager: ArchiveManager::new().unwrap_or_else(|e| {
                print_error(&format!("初始化失败: {}", e));
                print_warning("请确保程序目录下存在 path.txt 文件，并正确配置游戏存档路径");
                std::process::exit(1);
            }),
        }
    }

    pub fn run(&mut self) {
        print_title();
        self.print_welcome();

        loop {
            print_prompt();

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

    fn print_welcome(&self) {
        let count = self.manager.archive_count();
        if count > 0 {
            print_info(&format!("当前共有 {} 个存档", count));
        }
    }

    fn read_input(&self) -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or_default();
        input
    }

    fn parse_command(&self, input: &str) -> Option<Command> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "quit" | "q" => Some(Command::Quit),
            "help" | "h" => Some(Command::Help),
            "clearscreen" | "cls" | "clear" => Some(Command::ClearScreen),
            "save" | "s" => Some(Command::Save),
            "qsave" | "qs" => Some(Command::QuickSave),
            "rsave" | "rs" => Some(Command::ReplaceSave),
            "load" | "l" => Some(Command::Load(args.first().map(|s| s.to_string()))),
            "qload" | "ql" => Some(Command::QuickLoad),
            "log" | "lo" => Some(Command::Log),
            "slog" | "sl" => Some(Command::ShortLog),
            "marchive" | "ma" | "mod" | "m" => {
                Some(Command::ModifyArchive(args.first().map(|s| s.to_string())))
            }
            "delarch" | "del" | "delete" | "d" => {
                Some(Command::DeleteArchive(args.first().map(|s| s.to_string())))
            }
            "qdelete" | "qd" => Some(Command::QuickDelete),
            "usage" | "use" | "space" => Some(Command::Usage),
            _ => {
                print_error(&format!("未知命令: {}", cmd));
                print_info("输入 help 或 h 查看帮助");
                None
            }
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), ()> {
        match cmd {
            Command::Quit => {
                print_info("再见！");
                return Err(());
            }
            Command::Help => print_help(),
            Command::ClearScreen => {
                print_title();
                self.print_welcome();
            }
            Command::Save => self.do_save(),
            Command::QuickSave => self.do_quick_save(),
            Command::ReplaceSave => self.do_replace_save(),
            Command::Load(args) => self.do_load(args),
            Command::QuickLoad => self.do_quick_load(),
            Command::Log => self.do_log(),
            Command::ShortLog => self.do_short_log(),
            Command::ModifyArchive(args) => self.do_modify_archive(args),
            Command::DeleteArchive(args) => self.do_delete_archive(args),
            Command::QuickDelete => self.do_quick_delete(),
            Command::Usage => self.do_usage(),
        }
        Ok(())
    }

    fn do_save(&mut self) {
        print_info("请输入存档信息 (直接回车取消)");

        print!("  存档名 (必填, 限32字): ");
        io::stdout().flush().unwrap();
        let name = self.read_input();
        let name = name.trim().to_string();

        if name.is_empty() {
            print_info("已取消保存");
            return;
        }

        if name.len() > 32 {
            print_error("存档名超出32字符限制");
            return;
        }

        print!("  存档备注 (可不填, 限1024字): ");
        io::stdout().flush().unwrap();
        let comment = self.read_input();
        let comment = comment.trim().to_string();

        if comment.len() > 1024 {
            print_error("备注超出1024字符限制");
            return;
        }

        match self.manager.save(name, comment) {
            Ok(_) => print_success("保存成功"),
            Err(e) => print_error(&format!("保存失败: {}", e)),
        }
    }

    fn do_quick_save(&mut self) {
        match self.manager.quick_save() {
            Ok(_) => print_success("快速保存成功"),
            Err(e) => print_error(&format!("保存失败: {}", e)),
        }
    }

    fn do_replace_save(&mut self) {
        match self.manager.replace_save() {
            Ok(_) => print_success("覆盖保存成功"),
            Err(e) => print_error(&format!("覆盖保存失败: {}", e)),
        }
    }

    fn do_load(&mut self, args: Option<String>) {
        if self.manager.archive_count() == 0 {
            print_error("无存档可读取");
            return;
        }

        let index = match args {
            Some(arg) => match arg.parse::<usize>() {
                Ok(n) if n > 0 => n.saturating_sub(1),
                _ => {
                    print_error("请输入有效的存档编号");
                    return;
                }
            },
            None => {
                print_info("请输入存档编号 (0取消): ");
                let input = self.read_input();
                match input.trim().parse::<usize>() {
                    Ok(n) => n.saturating_sub(1),
                    Err(_) => {
                        print_info("已取消读取");
                        return;
                    }
                }
            }
        };

        if index >= self.manager.archive_count() {
            print_error("存档编号超出范围");
            return;
        }

        let info = &self.manager.get_all_archives()[index];
        print_warning(&format!("此操作会覆盖游戏中现有的存档"));

        print_confirm(&format!(
            "确定要读取存档 [{}] {} ({}) 吗",
            index + 1,
            info.name,
            info.timestamp
        ));

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            print_info("已取消读取");
            return;
        }

        match self.manager.load(index) {
            Ok(_) => print_success("读取成功"),
            Err(e) => print_error(&format!("读取失败: {}", e)),
        }
    }

    fn do_quick_load(&mut self) {
        if self.manager.archive_count() == 0 {
            print_error("无存档可读取");
            return;
        }

        let index = self.manager.archive_count() - 1;
        let info = &self.manager.get_all_archives()[index];
        print_warning("此操作会覆盖游戏中现有的存档");

        print_confirm(&format!(
            "确定要读取最新存档 [{}] {} ({}) 吗",
            index + 1,
            info.name,
            info.timestamp
        ));

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            print_info("已取消读取");
            return;
        }

        match self.manager.quick_load() {
            Ok(_) => print_success("读取成功"),
            Err(e) => print_error(&format!("读取失败: {}", e)),
        }
    }

    fn do_log(&self) {
        let archives = self.manager.get_all_archives();
        let count = archives.len();

        msg_log("==================================================");
        print_info(&format!("存档列表 (共 {} 个)", count));
        msg_log("==================================================");

        if count == 0 {
            print_info("暂无存档");
            return;
        }

        println!();
        println!("  {:<4} {:<20} {:<8} {}", "编号", "时间", "名称", "备注");
        println!("  {:-<4} {:-<20} {:-<8} {}", "─", "─", "─", "─");

        for (i, info) in archives.iter().enumerate() {
            let name = if info.name.len() > 8 {
                &info.name[..8]
            } else {
                &info.name
            };
            let comment = if info.comment.len() > 20 {
                &info.comment[..20]
            } else {
                &info.comment
            };
            println!(
                "  {:<4} {:<20} {:<8} {}",
                i + 1,
                info.timestamp.format("%Y-%m-%d %H:%M"),
                name,
                comment
            );
        }
        println!();
    }

    fn do_short_log(&self) {
        let archives = self.manager.get_recent_archives(7);
        let start = self.manager.archive_count().saturating_sub(archives.len());
        let count = archives.len();

        msg_log("==================================================");
        print_info(&format!("最近 {} 个存档", count));
        msg_log("==================================================");

        if count == 0 {
            print_info("暂无存档");
            return;
        }

        println!();
        println!("  {:<4} {:<20} {:<8} {}", "编号", "时间", "名称", "备注");
        println!("  {:-<4} {:-<20} {:-<8} {}", "─", "─", "─", "─");

        for (i, info) in archives.iter().enumerate() {
            let real_index = start + i;
            let name = if info.name.len() > 8 {
                &info.name[..8]
            } else {
                &info.name
            };
            let comment = if info.comment.len() > 20 {
                &info.comment[..20]
            } else {
                &info.comment
            };
            println!(
                "  {:<4} {:<20} {:<8} {}",
                real_index + 1,
                info.timestamp.format("%Y-%m-%d %H:%M"),
                name,
                comment
            );
        }
        println!();
    }

    fn do_modify_archive(&mut self, args: Option<String>) {
        if self.manager.archive_count() == 0 {
            print_error("无存档可修改");
            return;
        }

        let index = match args {
            Some(arg) => match arg.parse::<usize>() {
                Ok(n) if n > 0 => n.saturating_sub(1),
                _ => {
                    print_error("请输入有效的存档编号");
                    return;
                }
            },
            None => {
                print_info("请输入存档编号 (0取消): ");
                let input = self.read_input();
                match input.trim().parse::<usize>() {
                    Ok(n) => n.saturating_sub(1),
                    Err(_) => {
                        print_info("已取消修改");
                        return;
                    }
                }
            }
        };

        if index >= self.manager.archive_count() {
            print_error("存档编号超出范围");
            return;
        }

        let old_info = &self.manager.get_all_archives()[index];
        print_info(&format!(
            "当前名称: {}，备注: {}",
            old_info.name, old_info.comment
        ));

        print!("  新存档名 (直接回车保持原名): ");
        io::stdout().flush().unwrap();
        let name = self.read_input();
        let name = if name.trim().is_empty() {
            None
        } else {
            let n = name.trim().to_string();
            if n.len() > 32 {
                print_error("存档名超出32字符限制");
                return;
            }
            Some(n)
        };

        print!("  新备注 (直接回车保持原备注): ");
        io::stdout().flush().unwrap();
        let comment = self.read_input();
        let comment = if comment.trim().is_empty() {
            None
        } else {
            let c = comment.trim().to_string();
            if c.len() > 1024 {
                print_error("备注超出1024字符限制");
                return;
            }
            Some(c)
        };

        match self.manager.modify(index, name, comment) {
            Ok(_) => print_success("修改成功"),
            Err(e) => print_error(&format!("修改失败: {}", e)),
        }
    }

    fn do_delete_archive(&mut self, args: Option<String>) {
        if self.manager.archive_count() == 0 {
            print_error("无存档可删除");
            return;
        }

        let index = match args {
            Some(arg) => match arg.parse::<usize>() {
                Ok(n) if n > 0 => n.saturating_sub(1),
                _ => {
                    print_error("请输入有效的存档编号");
                    return;
                }
            },
            None => {
                print_info("请输入存档编号 (0取消): ");
                let input = self.read_input();
                match input.trim().parse::<usize>() {
                    Ok(n) => n.saturating_sub(1),
                    Err(_) => {
                        print_info("已取消删除");
                        return;
                    }
                }
            }
        };

        if index >= self.manager.archive_count() {
            print_error("存档编号超出范围");
            return;
        }

        let info = &self.manager.get_all_archives()[index];
        print_warning("删除后无法恢复！");

        print_confirm(&format!("确定要删除存档 [{}] {} 吗", index + 1, info.name));

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            print_info("已取消删除");
            return;
        }

        match self.manager.delete(index) {
            Ok(_) => print_success("删除成功"),
            Err(e) => print_error(&format!("删除失败: {}", e)),
        }
    }

    fn do_quick_delete(&mut self) {
        if self.manager.archive_count() == 0 {
            print_error("无存档可删除");
            return;
        }

        let index = self.manager.archive_count() - 1;
        let info = &self.manager.get_all_archives()[index];
        print_warning("删除后无法恢复！");

        print_confirm(&format!(
            "确定要删除最新存档 [{}] {} 吗",
            index + 1,
            info.name
        ));

        let confirm = self.read_input();
        if confirm.trim().to_lowercase() != "y" {
            print_info("已取消删除");
            return;
        }

        match self.manager.quick_delete() {
            Ok(_) => print_success("删除成功"),
            Err(e) => print_error(&format!("删除失败: {}", e)),
        }
    }

    fn do_usage(&self) {
        let size = self.manager.get_usage_space();
        print_info(&format!("占用空间: {:.2} MB", size));
    }
}

enum Command {
    Quit,
    Help,
    ClearScreen,
    Save,
    QuickSave,
    ReplaceSave,
    Load(Option<String>),
    QuickLoad,
    Log,
    ShortLog,
    ModifyArchive(Option<String>),
    DeleteArchive(Option<String>),
    QuickDelete,
    Usage,
}
