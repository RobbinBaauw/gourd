#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![allow(clippy::redundant_static_lifetimes)]
// for tarpaulin cfg
#![allow(unexpected_cfgs)]

//! Gourd allows

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::config::Config;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

use crate::constants::X86_64_E_MACHINE;
use crate::measurement::Measurement;
use crate::wrapper::wrap;
use crate::wrapper::Program;

/// The tests validating the behaviour of `gourd`.
#[cfg(test)]
pub mod tests;

/// The error type of `gourd`.
pub mod error;

/// A struct and related methods for global configuration,
/// declaratively specifying experiments.
pub mod config;

/// The binary wrapper around run programs.
pub mod wrapper;

/// Constant values.
pub mod constants;

/// The local runner module: `gourd run local`.
pub mod local;

/// Code shared between the wrapper and `gourd`.
pub mod measurement;

/// Code for accessing and managing resouces
pub mod resources;

#[derive(Parser)]
struct Cli {
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
struct RunStruct {
    #[command(subcommand)]
    sub_command: RunSubcommand,
}

#[derive(Subcommand)]
enum RunSubcommand {
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
struct StatusStruct {
    #[arg(help = "Path to gourd configuration file")]
    config_path: String,

    #[arg(
        id = "run-failed",
        value_name = "bool",
        default_missing_value = "true",
        hide_possible_values = true,
        short,
        long,
        help = "Rerun failed jobs"
    )]
    run_failed: Option<bool>,
}

#[derive(Args)]
struct InitStruct {}

#[derive(Args)]
struct AnalStruct {
    #[arg(help = "Path to gourd configuration file")]
    config_path: String,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Schedule run")]
    Run(RunStruct),

    #[command(about = "Display status of newest run")]
    Status(StatusStruct),

    #[command(about = "Analyze run")]
    Anal(AnalStruct),

    #[command(about = "Initialize new experiment")]
    Init(InitStruct),
}

fn parse_command() {
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

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
#[cfg(not(tarpaulin_include))]
fn main() {
    parse_command();
    println!("Hello, world!");
    let config_path = String::from("gourd.toml");

    println!("Loading configuration file at '{}'", config_path);
    let config = Config::from_file(Path::new(&config_path)).unwrap();
    // Prints contents of the configuration file. Remove.
    println!("{:?}", config);

    let path = "./bin".parse::<PathBuf>().unwrap();

    let _: Vec<ExitStatus> = wrap(
        vec![Program {
            binary: path,
            arguments: vec![],
        }],
        vec!["./test1".parse().unwrap()],
        X86_64_E_MACHINE,
        &config,
    )
    .unwrap()
    .iter_mut()
    .map(|x| {
        println!("{:?}", x);
        x.spawn().unwrap().wait().unwrap()
    })
    .collect();

    let results: Measurement = toml::from_str(
        &String::from_utf8(fs::read("/tmp/gourd/algo_0/0_result").unwrap()).unwrap(),
    )
    .unwrap();

    println!("{:?}", results);
}
