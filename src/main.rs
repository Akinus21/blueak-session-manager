use clap::Parser;
use xdg::BaseDirectories;

mod app_launch;
mod ipc;
mod restore;
mod save;
mod session;

#[derive(Parser)]
enum Commands {
    Save {
        #[arg(long)]
        session_file: Option<std::path::PathBuf>,
        #[arg(long, short)]
        verbose: bool,
    },
    Restore {
        #[arg(long)]
        session_file: Option<std::path::PathBuf>,
        #[arg(long)]
        app_map: Option<std::path::PathBuf>,
        #[arg(long, default_value_t = 30)]
        timeout: u64,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, short)]
        verbose: bool,
    },
    Dump,
    #[command(name = "app-map")]
    AppMap {
        #[command(subcommand)]
        sub: AppMapCommands,
    },
}

#[derive(clap::Subcommand)]
enum AppMapCommands {
    Show,
}

fn get_session_path() -> std::path::PathBuf {
    BaseDirectories::with_prefix("niri-session")
        .map(|xdg| {
            let mut path = xdg.get_data_dir().to_path_buf();
            path.push("niri-session");
            path.push("session.json");
            path
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.local/share/niri-session/session.json"))
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cmd = Commands::parse();

    match cmd {
        Commands::Save { session_file, verbose } => {
            let path = session_file.unwrap_or_else(get_session_path);
            save::save(&path, verbose)
        }
        Commands::Restore { session_file, app_map, timeout, dry_run, verbose } => {
            let path = session_file.unwrap_or_else(get_session_path);
            restore::restore(&path, app_map.as_deref(), timeout, dry_run, verbose)
        }
        Commands::Dump => {
            let workspaces = ipc::get_workspaces()?;
            let windows = ipc::get_windows()?;

            let output = serde_json::json!({
                "workspaces": workspaces,
                "windows": windows,
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        Commands::AppMap { sub } => match sub {
            AppMapCommands::Show => {
                let _launcher = app_launch::AppLauncher::new()?;
                println!("App map loaded (resolution order: override → desktop → flatpak → binary)");
                Ok(())
            }
        },
    }
}