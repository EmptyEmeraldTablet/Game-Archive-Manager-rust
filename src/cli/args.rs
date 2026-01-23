use clap::{Parser, Subcommand};

/// Game Archive Manager v2.0 - Version control for game saves like Git
#[derive(Parser, Debug)]
#[command(name = "gam")]
#[command(author = "Game Archive Manager Contributors")]
#[command(version = "2.0.0")]
#[command(about = "Game Archive Manager v2.0 - Version control for game saves like Git", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize version control
    #[command(name = "init")]
    Init(InitArgs),

    /// Snapshot management
    #[command(name = "snapshot")]
    Snapshot(SnapshotArgs),

    /// Timeline management
    #[command(name = "timeline")]
    Timeline(TimelineArgs),

    /// Restore to snapshot
    #[command(name = "restore")]
    Restore(RestoreArgs),

    /// View history
    #[command(name = "history")]
    History(HistoryArgs),

    /// View status
    #[command(name = "status")]
    Status(StatusArgs),

    /// View activity log
    #[command(name = "activity")]
    Activity(ActivityArgs),

    /// Compare snapshots
    #[command(name = "diff")]
    Diff(DiffArgs),

    /// Garbage collection
    #[command(name = "gc")]
    Gc(GcArgs),

    /// Ignore rules management
    #[command(name = "ignore")]
    Ignore(IgnoreArgs),

    /// View and manage configuration
    #[command(name = "config")]
    Config(ConfigArgs),

    /// Diagnose issues
    #[command(name = "doctor")]
    Doctor(DoctorArgs),
}

/// init command arguments
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Game saves directory path
    #[arg(short, long)]
    pub path: Option<String>,

    /// Force reinitialize
    #[arg(long)]
    pub force: bool,
}

/// snapshot subcommand arguments
#[derive(Parser, Debug)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    pub command: SnapshotCommands,
}

/// snapshot subcommands
#[derive(Subcommand, Debug)]
pub enum SnapshotCommands {
    /// Save current state as snapshot
    #[command(name = "save")]
    Save(SaveArgs),

    /// List snapshots
    #[command(name = "list")]
    List(ListArgs),

    /// View snapshot details
    #[command(name = "info")]
    Info(InfoArgs),

    /// Delete snapshot
    #[command(name = "delete")]
    Delete(DeleteArgs),

    /// Add tag to snapshot
    #[command(name = "tag")]
    Tag(TagArgs),

    /// List snapshot tags
    #[command(name = "tags")]
    Tags(TagsArgs),
}

/// save command arguments
#[derive(Parser, Debug)]
pub struct SaveArgs {
    /// Snapshot name
    #[arg(short, long)]
    pub message: Option<String>,

    /// Save to specified timeline
    #[arg(short, long)]
    pub timeline: Option<String>,
}

/// list command arguments
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// List snapshots from all timelines
    #[arg(long)]
    pub all: bool,

    /// Specify timeline
    #[arg(short, long)]
    pub timeline: Option<String>,
}

/// info command arguments
#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// Snapshot ID (short ID supported)
    pub id: String,
}

/// delete command arguments
#[derive(Parser, Debug)]
pub struct DeleteArgs {
    /// Snapshot ID
    pub id: String,

    /// Force delete (no confirmation)
    #[arg(long)]
    pub force: bool,
}

/// tag command arguments
#[derive(Parser, Debug)]
pub struct TagArgs {
    /// Snapshot ID
    pub id: String,

    /// Tag name
    pub name: String,
}

/// tags command arguments
#[derive(Parser, Debug)]
pub struct TagsArgs {
    /// Snapshot ID (optional, shows all tags if not specified)
    pub id: Option<String>,
}

/// timeline subcommand arguments
#[derive(Parser, Debug)]
pub struct TimelineArgs {
    #[command(subcommand)]
    pub command: TimelineCommands,
}

/// timeline subcommands
#[derive(Subcommand, Debug)]
pub enum TimelineCommands {
    /// Create timeline
    #[command(name = "create")]
    Create(CreateArgs),

    /// List timelines
    #[command(name = "list")]
    List,

    /// Switch timeline
    #[command(name = "switch")]
    Switch(SwitchArgs),

    /// Rename timeline
    #[command(name = "rename")]
    Rename(RenameArgs),

    /// Delete timeline
    #[command(name = "delete")]
    Delete(DeleteTimelineArgs),

    /// Show current timeline
    #[command(name = "current")]
    Current,
}

/// create command arguments
#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// Timeline name
    pub name: String,

    /// Create from specified snapshot
    #[arg(short, long)]
    pub from: Option<String>,
}

/// switch command arguments
#[derive(Parser, Debug)]
pub struct SwitchArgs {
    /// Timeline name or snapshot ID
    pub target: String,
}

/// rename command arguments
#[derive(Parser, Debug)]
pub struct RenameArgs {
    /// Old name
    pub old_name: String,

    /// New name
    pub new_name: String,
}

/// delete command arguments
#[derive(Parser, Debug)]
pub struct DeleteTimelineArgs {
    /// Timeline name
    pub name: String,

    /// Force delete
    #[arg(long)]
    pub force: bool,
}

/// restore command arguments
#[derive(Parser, Debug)]
pub struct RestoreArgs {
    /// Snapshot ID
    pub id: String,

    /// Force restore (skip confirmation)
    #[arg(long)]
    pub force: bool,
}

/// history command arguments
#[derive(Parser, Debug)]
pub struct HistoryArgs {
    /// Show history from all timelines
    #[arg(long)]
    pub all: bool,
}

/// status command arguments
#[derive(Parser, Debug)]
pub struct StatusArgs {
    /// Show detailed information
    #[arg(short, long)]
    pub verbose: bool,
}

/// activity command arguments
#[derive(Parser, Debug)]
pub struct ActivityArgs {
    /// Limit number of entries
    #[arg(short, long, default_value = "20")]
    pub limit: u32,
}

/// diff command arguments
#[derive(Parser, Debug)]
pub struct DiffArgs {
    /// First snapshot ID
    pub id1: String,

    /// Second snapshot ID
    pub id2: String,
}

/// gc command arguments
#[derive(Parser, Debug)]
pub struct GcArgs {
    /// Aggressive GC (clean all unreferenced objects)
    #[arg(long)]
    pub aggressive: bool,

    /// Preview mode (do not actually execute)
    #[arg(long)]
    pub dry_run: bool,
}

/// ignore subcommand arguments
#[derive(Parser, Debug)]
pub struct IgnoreArgs {
    #[command(subcommand)]
    pub command: IgnoreCommands,
}

/// ignore subcommands
#[derive(Subcommand, Debug)]
pub enum IgnoreCommands {
    /// Add ignore rule
    #[command(name = "add")]
    Add(AddIgnoreArgs),

    /// Remove ignore rule
    #[command(name = "remove")]
    Remove(RemoveIgnoreArgs),

    /// List current rules
    #[command(name = "list")]
    List,

    /// Check if file is ignored
    #[command(name = "check")]
    Check(CheckIgnoreArgs),

    /// Initialize default rules template
    #[command(name = "init")]
    Init(IgnoreInitArgs),
}

/// add command arguments
#[derive(Parser, Debug)]
pub struct AddIgnoreArgs {
    /// Pattern
    pub pattern: String,
}

/// remove command arguments
#[derive(Parser, Debug)]
pub struct RemoveIgnoreArgs {
    /// Pattern
    pub pattern: String,
}

/// check command arguments
#[derive(Parser, Debug)]
pub struct CheckIgnoreArgs {
    /// File path
    pub file: String,
}

/// init command arguments
#[derive(Parser, Debug)]
pub struct IgnoreInitArgs {
    /// Overwrite existing file
    #[arg(long)]
    pub force: bool,
}

/// doctor command arguments
#[derive(Parser, Debug)]
pub struct DoctorArgs {
    /// Fix detected issues
    #[arg(long)]
    pub fix: bool,
}

/// config command arguments
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// Configuration key (e.g., core.default_timeline)
    pub key: Option<String>,

    /// Configuration value
    pub value: Option<String>,

    /// List all configurations
    #[arg(long)]
    pub list: bool,
}
