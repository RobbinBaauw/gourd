use std::path::PathBuf;
use std::process::exit;

use clap::crate_authors;
use clap::crate_name;
use clap::crate_version;
use clap::error::ErrorKind;
use clap::Args;
use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

use crate::config::Config;
use crate::constants::PRIMARY_STYLE;
use crate::constants::SECONDARY_STYLE;
use crate::constants::UNDERLINE_STYLE;
use crate::error::GourdError;
use crate::local::run_local;

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
    let mut stdio = Cli::command();

    if let Err(e) = process_command(&command) {
        stdio
            .error(ErrorKind::Format, format!("{}", e))
            .print()
            .unwrap();

        exit(1);
    }
}

fn process_command(cmd: &Cli) -> Result<(), GourdError> {
    match cmd.command {
        Command::Run(args) => match args.sub_command {
            RunSubcommand::Local { .. } => {
                let config = Config::from_file(&cmd.config)?;
                run_local(&config)?;
            }
            RunSubcommand::Slurm { .. } => {
                panic!("Running on Slurm has not been implemented yet")
            }
        },
        Command::Status(_) => panic!("Checking status has not been implemented yet"),
        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),
        Command::Anal(_) => panic!("Analyze has not been implemented yet"),
        Command::Version => print_version(),
    }

    Ok(())
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .blink()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
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
        UNDERLINE_STYLE, UNDERLINE_STYLE,
    );

    println!("Authored by:\n{}", crate_authors!("\n"));
}
