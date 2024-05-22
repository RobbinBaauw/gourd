use std::env;
use std::process::exit;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use clap::Parser;
use colog::default_builder;
use colog::formatter;
use gourd_lib::config::Config;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileSystemInteractor;
use log::debug;
use log::info;
use log::trace;
use log::LevelFilter;

use super::def::LogTokens;
use crate::cli::def::Cli;
use crate::cli::def::Command;
use crate::cli::def::RunSubcommand;
use crate::cli::printing::print_version;
use crate::experiments::ExperimentExt;
use crate::local::run_local;
use crate::post::afterscript::run_afterscript;
use crate::post::postprocess_job::schedule_post_jobs;
use crate::slurm::checks::get_slurm_options_from_config;
use crate::slurm::handler::SlurmHandler;
use crate::slurm::interactor::SlurmCLI;
use crate::status::display_statuses;
use crate::status::get_statuses;

/// An enum to distinguish the run context.
#[derive(Clone, Copy, Debug)]
pub enum Environment {
    /// Local execution.
    Local,

    /// Slurm execution.
    Slurm,
}

/// This function parses command that gourd was run with.
pub fn parse_command() {
    let command = Cli::parse();

    // https://github.com/rust-lang/rust/blob/master/library/std/src/backtrace.rs
    let backtrace_enabled = match env::var("RUST_LIB_BACKTRACE") {
        Ok(s) => s != "0",
        Err(_) => match env::var("RUST_BACKTRACE") {
            Ok(s) => s != "0",
            Err(_) => false,
        },
    };

    if backtrace_enabled {
        eprintln!("{:?}", process_command(&command));
    } else if let Err(e) = process_command(&command) {
        eprintln!("{}error:{:#} {}", ERROR_STYLE, ERROR_STYLE, e.root_cause());
        eprint!("{}", e);
        exit(1);
    }
}

/// CLAP has parsed the command, now we process it.
pub fn process_command(cmd: &Cli) -> Result<()> {
    setup_logging(cmd)?;
    let file_system = FileSystemInteractor { dry_run: cmd.dry };

    match cmd.command {
        Command::Run(args) => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            debug!("Creating a new experiment");
            trace!("The config is: {config:#?}");

            let mut experiment = Experiment::from_config(
                &config,
                match args.sub_command {
                    RunSubcommand::Local { .. } => Environment::Local,
                    RunSubcommand::Slurm { .. } => Environment::Slurm,
                },
                Local::now(),
                &file_system,
            )?;

            debug!("Trying to run the experiment");

            let exp_path = experiment.save(&config.experiments_folder, &file_system)?;

            match args.sub_command {
                RunSubcommand::Local { .. } => {
                    if cmd.dry {
                        info!("Would have ran the experiment (dry)")
                    } else {
                        run_local(&config, &experiment, &file_system)?
                    }
                }

                RunSubcommand::Slurm { .. } => {
                    let s: SlurmHandler<SlurmCLI> = SlurmHandler::default();
                    s.check_version()?;
                    s.check_partition(&get_slurm_options_from_config(&config)?.partition)?;

                    if cmd.dry {
                        info!("Would have scheduled the experiment on slurm (dry)")
                    } else {
                        s.run_experiment(&config, &mut experiment, exp_path)?;
                    }
                }
            }

            experiment.save(&config.experiments_folder, &file_system)?;

            info!("Experiment started.");
        }
        Command::Status(_) => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            let experiment = Experiment::latest_experiment_from_folder(
                &config.experiments_folder,
                &file_system,
            )?;

            debug!("Found the newest experiment with id: {}", experiment.seq);

            let statuses = get_statuses(&experiment, file_system)?;

            run_afterscript(&statuses, &experiment)?;

            display_statuses(&experiment, &statuses);
        }
        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),
        Command::Anal(_) => panic!("Analyze has not been implemented yet"),
        Command::Version => print_version(),
        Command::Postprocess => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            let mut experiment = Experiment::latest_experiment_from_folder(
                &config.experiments_folder,
                &file_system,
            )?;

            debug!("Found the newest experiment with id: {}", experiment.seq);

            let mut statuses = get_statuses(&experiment, file_system)?;

            schedule_post_jobs(&mut experiment, &config, &mut statuses, &file_system)?;

            debug!("Postprocessing scheduled for available jobs");
            display_statuses(&experiment, &statuses);
        }
    }

    Ok(())
}

/// Prepare the log levels for the application.
fn setup_logging(cmd: &Cli) -> Result<()> {
    let mut log_build = default_builder();
    log_build.format(formatter(LogTokens));

    if cmd.verbose == 2 {
        log_build.filter(None, LevelFilter::Trace);
    } else if cmd.verbose == 1 {
        log_build.filter(Some("gourd"), LevelFilter::Debug);
    } else if cmd.verbose == 0 {
        log_build.filter(Some("gourd"), LevelFilter::Info);
    } else {
        return Err(anyhow!("Only two levels of verbosity supported (ie. -vv)")).context("");
    }

    log_build.init();

    Ok(())
}
