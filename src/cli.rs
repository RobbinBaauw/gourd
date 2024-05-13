use std::env;
use std::path::PathBuf;
use std::process::exit;

use anstyle::AnsiColor;
use anyhow::Result;
use chrono::Local;
use clap::crate_authors;
use clap::crate_name;
use clap::crate_version;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::config::Config;
use crate::constants::style_from_fg;
use crate::constants::ERROR_STYLE;
use crate::constants::HELP_STYLE;
use crate::constants::PRIMARY_STYLE;
use crate::constants::SECONDARY_STYLE;
use crate::constants::UNIVERSITY_STYLE;
use crate::experiment::Environment;
use crate::experiment::Experiment;
use crate::local::run_local;
use crate::status::display_statuses;
use crate::status::get_statuses;

/// Structure of the main command (gourd).
#[derive(Parser, Debug)]
#[command(styles=get_styles(), about = "Gourd, an emipirical evaluator", disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

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
    config: PathBuf,

    #[arg(short, long, help = "Verbose mode, displays debug info", global = true)]
    verbose: bool,
}

/// Structure of Run subcommand.
#[derive(Args, Debug, Clone, Copy)]
pub struct RunStruct {
    #[command(subcommand)]
    sub_command: RunSubcommand,
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

/// This function parses command that gourd was run with.
///
///
pub fn parse_command() {
    let command = Cli::parse();

    // https://github.com/rust-lang/rust/blob/master/library/std/src/backtrace.rs
    let backtace_enabled = match env::var("RUST_LIB_BACKTRACE") {
        Ok(s) => s != "0",
        Err(_) => match env::var("RUST_BACKTRACE") {
            Ok(s) => s != "0",
            Err(_) => false,
        },
    };

    if backtace_enabled {
        eprintln!("{:?}", process_command(&command));
    } else if let Err(e) = process_command(&command) {
        eprintln!("{}error:{:#} {}", ERROR_STYLE, ERROR_STYLE, e.root_cause());
        eprint!("{}", e);
        exit(1);
    }
}

fn process_command(cmd: &Cli) -> Result<()> {
    match cmd.command {
        Command::Run(args) => match args.sub_command {
            RunSubcommand::Local { .. } => {
                let config = Config::from_file(&cmd.config)?;

                let experiment =
                    Experiment::from_config(&config, Environment::Local, Local::now())?;

                run_local(&config, &experiment)?;

                experiment.save(&config.experiments_folder)?;
            }
            RunSubcommand::Slurm { .. } => {
                todo!()
            }
        },
        Command::Status(_) => {
            let config = Config::from_file(&cmd.config)?;

            let experiment = Experiment::latest_experiment_from_folder(&config.experiments_folder)?;

            let statuses = get_statuses(&experiment)?;

            display_statuses(&experiment, &statuses);
        }
        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),
        Command::Anal(_) => panic!("Analyze has not been implemented yet"),
        Command::Version => print_version(),
    }

    Ok(())
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(style_from_fg(AnsiColor::Yellow).bold())
        .header(style_from_fg(AnsiColor::Green).bold().underline())
        .literal(style_from_fg(AnsiColor::Cyan).bold())
        .invalid(style_from_fg(AnsiColor::Blue).bold())
        .error(ERROR_STYLE)
        .valid(HELP_STYLE)
        .placeholder(style_from_fg(AnsiColor::White))
}

fn print_version() {
    println!(
        "{}{}{:#} at version {}{}{:#}\n\n",
        PRIMARY_STYLE,
        crate_name!(),
        PRIMARY_STYLE,
        SECONDARY_STYLE,
        crate_version!(),
        SECONDARY_STYLE
    );

    println!(
        "{}Technische Universiteit Delft 2024{:#}\n",
        UNIVERSITY_STYLE, UNIVERSITY_STYLE,
    );

    println!("Authored by:\n{}", crate_authors!("\n"));
}
