use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::cli::printing::get_styles;

/// Structure of the main command (gourd).
#[allow(unused)]
#[derive(Parser, Debug)]
#[command(styles=get_styles(), about = "Gourd, an emipirical evaluator", disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    #[arg(
        short,
        long,
        help = "Disable interactive mode, for use in scripts",
        global = true
    )]
    script: bool,

    #[arg(
        short,
        long,
        help = "The path to the config file",
        default_value = "./gourd.toml",
        global = true
    )]
    pub(crate) config: PathBuf,

    #[arg(short, long, help = "Verbose mode, displays debug info", global = true)]
    verbose: bool,
}

/// Structure of Run subcommand.
#[derive(Args, Debug, Clone, Copy)]
pub struct RunStruct {
    #[command(subcommand)]
    pub(crate) sub_command: RunSubcommand,
}

/// Enum for subcommands of Run subcommand.
#[derive(Subcommand, Debug, Copy, Clone)]
pub enum RunSubcommand {
    /// Subcommand for running locally.
    #[command(about = "Schedule a run on the local machine")]
    Local {},

    /// Subcommand for running on slurm.
    #[command(about = "Schedule a run using slurm")]
    Slurm {},
}

/// Structure of status subcommand.
#[derive(Args, Debug, Clone, Copy)]
pub struct StatusStruct {
    #[arg(
        id = "run-failed",
        value_name = "bool",
        short,
        long,
        help = "Rerun failed jobs"
    )]
    run_failed: bool,
}

/// Structure of init subcommand.
#[derive(Args, Debug)]
pub struct InitStruct {
    /// Flag used to point to directory in which to set up a new experiment.
    #[arg(
        short,
        long,
        help = "Directory in which to create an experiment",
        default_value = "./"
    )]
    directory: Option<String>,
}

/// Structure of anal subcommand.
#[derive(Args, Debug, Clone, Copy)]
pub struct AnalStruct {}

/// Enum for subcommands of main command.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Subcommand for scheduling a run.
    #[command(about = "Schedule a run")]
    Run(RunStruct),

    /// Subcommand for checking status of a run.
    #[command(about = "Display the status of a run")]
    Status(StatusStruct),

    /// Subcommand for analyzing results of a run.
    #[command(about = "Analyze a run")]
    Anal(AnalStruct),

    /// Subcommand for initializing new experiment.
    #[command(about = "Initialize a new experiment")]
    Init(InitStruct),

    /// Subcommand for getting the version.
    #[command(about = "Display and about page with the program version")]
    Version,
}
