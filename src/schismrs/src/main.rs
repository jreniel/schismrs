// schismrs/src/main.rs

use anyhow::Result;
use clap::{Parser, Subcommand};
use schismrs::cli::{init_project, sync_project};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "schismrs")]
#[command(about = "Configuration management system for SCHISM ocean models", long_about = None)]
#[command(version = env!("SCHISMRS_CLI_VERSION"))]
struct Cli {
    /// Project directory (defaults to current directory)
    #[arg(short, long, value_name = "DIR", global = true)]
    project_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new SCHISM project
    Init,
    /// Synchronize configuration changes
    Sync,
}

fn entrypoint() -> Result<()> {
    pretty_env_logger::init();
    let cli = Cli::parse();

    let project_dir = cli
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let result = match cli.command {
        Commands::Init => init_project(&project_dir),
        Commands::Sync => sync_project(&project_dir),
    };
    result
}

fn main() -> ExitCode {
    match entrypoint() {
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}
