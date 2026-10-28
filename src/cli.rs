use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "alejandradb", version, about = "TUI/CLI for Android via ADB")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub serial: Option<String>,

    #[arg(long, global = true, env = "ADB_PATH")]
    pub adb: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Devices,
    Apps {
        #[command(subcommand)]
        action: Option<AppsAction>,
    },
    Tui,
    Scripts {
        #[command(subcommand)]
        action: ScriptsAction,
    },
    Processes {
        #[command(subcommand)]
        action: ProcessesAction,
    },
    Logcat {
        #[arg(long)]
        lines: Option<usize>,
        #[arg(long)]
        duration: Option<u64>,
        #[arg(long, value_name = "PATH")]
        output: Option<std::path::PathBuf>,
    },
    Shell {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum AppsAction {
    List(ListOpts),
    Uninstall {
        #[arg(required = true)]
        packages: Vec<String>,
        #[arg(long)]
        keep_data: bool,
        #[arg(long)]
        system: bool,
        #[arg(long)]
        force: bool,
    },
    Install {
        #[arg(required = true)]
        path: String,
    },
    InstallExisting {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Enable {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Disable {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Interactive,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct ListOpts {
    #[arg(long)]
    pub third_party: bool,
    #[arg(long)]
    pub system: bool,
    #[arg(long)]
    pub disabled: bool,
    #[arg(long)]
    pub enabled: bool,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long, default_value_t = 0)]
    pub user: i32,
}

#[derive(Subcommand, Debug)]
pub enum ScriptsAction {
    Run {
        #[arg(long)]
        folders: Vec<String>,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProcessesAction {
    List,
    Kill {
        pid: u32,
        #[arg(long)]
        force: bool,
    },
}
