//! The `planrun` binary (docs/architecture.md §7).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Executes structured implementation plans against a git repository, task by task.
#[derive(Debug, Parser)]
#[command(name = "planrun", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and check a plan; print diagnostics and the coverage report. Writes nothing.
    Check {
        /// The plan file.
        plan: PathBuf,
        /// Force a format id instead of detecting one.
        #[arg(long)]
        format: Option<String>,
    },
    /// Print the typed plan, or one task (`P1-T2`) or step (`P1-T2/1`).
    Show {
        /// The plan file.
        plan: PathBuf,
        /// A task or task/step.
        target: Option<String>,
        /// Force a format id.
        #[arg(long)]
        format: Option<String>,
    },
    /// Execute a plan (Phase 5).
    Run {
        /// The plan file.
        plan: PathBuf,
        /// Parse, check and validate against a virtual tree; change nothing, run nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    eprintln!(
        "planrun: {:?} is not implemented yet (docs/roadmap.md)",
        cli.command
    );
    ExitCode::from(2)
}
