use std::path::PathBuf;

use clap::ArgAction;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

/// Structure of the main command (gourd).
#[allow(unused)]
#[derive(Parser, Debug)]
#[command(
    about = "Gourd, an empirical evaluator",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The main command issued.
    #[command(subcommand)]
    pub command: Command,

    /// Disable interactive mode, for use in scripts.
    #[arg(short, long, global = true)]
    pub script: bool,

    /// The path to the config file.
    #[arg(short, long, default_value = "./gourd.toml", global = true)]
    pub config: PathBuf,

    /// Verbose mode, displays debug info. For even more try: -vv.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Dry run, run but don't actually affect anything.
    #[arg(short, long, global = true)]
    pub dry: bool,
}

/// Arguments supplied with the `run` command.
#[derive(Args, Debug, Clone, Copy)]
pub struct RunStruct {
    /// The run mode of this run.
    #[command(subcommand)]
    pub subcommand: RunSubcommand,
}

/// Enum for subcommands of the `run` subcommand.
#[derive(Subcommand, Debug, Copy, Clone)]
pub enum RunSubcommand {
    /// Create and run an experiment on this computer.
    #[command()]
    Local {},

    /// Create and run an experiment using Slurm.
    #[command()]
    Slurm {},
}

/// Arguments supplied with the `status` command.
#[derive(Args, Debug, Clone, Copy)]
pub struct StatusStruct {
    /// Rerun failed jobs.
    #[arg(short, long)]
    pub rerun_failed: bool,

    /// The id of the experiment for which to fetch status [default: newest
    /// experiment].
    #[arg(value_name = "EXPERIMENT")]
    pub experiment_id: Option<usize>,

    /// Get a detailed description of a run by providing its id.
    #[arg(short = 'i', long)]
    pub run_id: Option<usize>,

    /// Do not exit until all jobs are finished.
    #[arg(long)]
    pub follow: bool,
}

/// Arguments supplied with the `init` command.
#[derive(Args, Debug)]
pub struct InitStruct {
    /// Directory in which to create an experiment.
    #[arg(short = 'D', long, default_value = "./")]
    directory: Option<String>,
}

/// Arguments supplied with the `analyse` command.
#[derive(Args, Debug, Clone, Copy)]
pub struct AnalyseStruct {}

/// Enum for root-level `gourd` commands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create an experiment from configuration and run it.
    #[command()]
    Run(RunStruct),

    /// Set up a template of an experiment configuration.
    #[command()]
    Init(InitStruct),

    /// Display the status of an experiment that was run.
    #[command()]
    Status(StatusStruct),

    /// Schedule another batch of slurm jobs.
    #[command()]
    Continue,

    /// Output metrics of completed runs.
    #[command()]
    Analyse(AnalyseStruct),

    /// Print information about the version.
    #[command()]
    Version,
}
