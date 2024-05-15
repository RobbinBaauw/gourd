use std::env;
use std::process::exit;

use chrono::Local;
use clap::Parser;

use crate::afterscript;
use crate::cli::def::Cli;
use crate::cli::def::Command;
use crate::cli::def::RunSubcommand;
use crate::cli::printing::print_version;
use crate::config::Config;
use crate::constants::ERROR_STYLE;
use crate::experiment::Environment;
use crate::experiment::Experiment;
use crate::local::run_local;
use crate::slurm::checks::get_slurm_options_from_config;
use crate::slurm::handler::SlurmHandler;
use crate::slurm::interactor::SlurmCLI;
use crate::status::display_statuses;
use crate::status::get_statuses;

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

/// CLAP has parsed the command, now we process it.
pub fn process_command(cmd: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        Command::Run(args) => {
            let config = Config::from_file(&cmd.config)?;
            let mut experiment = match args.sub_command {
                RunSubcommand::Local { .. } => {
                    Experiment::from_config(&config, Environment::Local, Local::now())?
                }
                RunSubcommand::Slurm { .. } => {
                    Experiment::from_config(&config, Environment::Slurm, Local::now())?
                }
            };

            match args.sub_command {
                RunSubcommand::Local { .. } => run_local(&config, &experiment)?,
                RunSubcommand::Slurm { .. } => {
                    let s: SlurmHandler<SlurmCLI> = SlurmHandler::default();
                    s.check_version()?;
                    s.check_partition(&get_slurm_options_from_config(&config)?.partition)?;
                    #[allow(clippy::unnecessary_mut_passed)] // in the near future we will update
                    // the experiment when running it, for example to include job ids in the runs
                    s.run_experiment(&config, &mut experiment)?;
                }
            }
            experiment.save(&config.experiments_folder)?;
        }
        Command::Status(_) => {
            let config = Config::from_file(&cmd.config)?;

            let experiment = Experiment::latest_experiment_from_folder(&config.experiments_folder)?;

            let statuses = get_statuses(&experiment)?;

            afterscript::run_afterscript(&statuses, &experiment)?;

            display_statuses(&experiment, &statuses);
        }
        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),
        Command::Anal(_) => panic!("Analyze has not been implemented yet"),
        Command::Version => print_version(),
    }

    Ok(())
}
