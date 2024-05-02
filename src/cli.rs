use clap::Args;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
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

#[derive(Args)]
pub struct RunStruct {
    #[command(subcommand)]
    sub_command: RunSubcommand,
}

#[derive(Subcommand)]
pub enum RunSubcommand {
    #[command(about = "Schedule run on local machine")]
    Local {
        #[arg(help = "Path to gourd configuration file")]
        config_path: String,
    },

    #[command(about = "Schedule run using slurm")]
    Slurm {
        #[arg(help = "Path to gourd configuration file")]
        config_path: String,
    },
}

#[derive(Args)]
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

#[derive(Args)]
pub struct InitStruct {}

#[derive(Args)]
pub struct AnalStruct {
    #[arg(help = "Path to gourd configuration file")]
    config_path: String,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Schedule run")]
    Run(RunStruct),

    #[command(about = "Display status of newest run")]
    Status(StatusStruct),

    #[command(about = "Analyze run")]
    Anal(AnalStruct),

    #[command(about = "Initialize new experiment")]
    Init(InitStruct),
}

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
