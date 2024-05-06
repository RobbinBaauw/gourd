use clap::Args;
use clap::Parser;
use clap::Subcommand;

/// Structure of main command (gourd)
#[derive(Parser, Debug)]
#[command(styles=get_styles())]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(
        short,
        long,
        help = "Disable interactive mode (For use in scripts)",
        global = true
    )]
    script: bool,

    #[arg(
        short,
        long,
        help = "Verbose mode (Displays debug info)",
        global = true
    )]
    verbose: bool,
}

/// Structure of Run subcommand
#[derive(Args, Debug)]
pub struct RunStruct {
    #[command(subcommand)]
    sub_command: RunSubcommand,
}

/// Enum for subcommands of Run subcommand
#[derive(Subcommand, Debug)]
pub enum RunSubcommand {
    /// Subcommand for running locally
    #[command(about = "Schedule run on local machine")]
    Local {
        /// Flag used to set a path to gourd configuration file
        #[arg(help = "Path to gourd configuration file", short, long)]
        config_path: Option<String>,
    },

    /// Subcommand for running on slurm
    #[command(about = "Schedule run using slurm")]
    Slurm {
        /// Flag used to set a path to gourd configuration file
        #[arg(help = "Path to gourd configuration file", short, long)]
        config_path: Option<String>,
    },
}

/// Structure of status subcommand
#[derive(Args, Debug)]
pub struct StatusStruct {
    #[arg(help = "Path to gourd configuration file")]
    config_path: String,

    #[arg(
        id = "run-failed",
        value_name = "bool",
        short,
        long,
        help = "Rerun failed jobs"
    )]
    run_failed: bool,
}

/// Structure of init subcommand
#[derive(Args, Debug)]
pub struct InitStruct {
    /// Flag used to point to directory in which to set up a new experiment
    #[arg(
        short,
        long,
        help = "Directory in which to create an experiment",
        default_value = "./"
    )]
    directory: Option<String>,
}

/// Structure of anal subcommand
#[derive(Args, Debug)]
pub struct AnalStruct {
    /// Flag used to set a path to gourd configuration file
    #[arg(help = "Path to gourd configuration file", short, long)]
    config_path: Option<String>,
}

/// Enum for subcommands of main command
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Subcommand for scheduling a run
    #[command(about = "Schedule run")]
    Run(RunStruct),

    /// Subcommand for checking status of a run
    #[command(about = "Display status of newest run")]
    Status(StatusStruct),

    /// Subcommand for analyzing results of a run
    #[command(about = "Analyze run")]
    Anal(AnalStruct),

    /// Subcommand for initializing new experiment
    #[command(about = "Initialize new experiment")]
    Init(InitStruct),
}

/// This function parses command that gourd was run with
///
///
pub fn parse_command() {
    let command = Cli::parse();

    match command.command {
        Command::Run(args) => match args.sub_command {
            RunSubcommand::Local { .. } => {
                panic!("Running locally has not been implemented yet")
            }
            RunSubcommand::Slurm { .. } => {
                panic!("Running on Slurm has not been implemented yet")
            }
        },
        Command::Status(_) => panic!("Checking status has not been implemented yet"),
        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),
        Command::Anal(_) => panic!("Analyze has not been implemented yet"),
    }
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
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
