use std::env;
use std::io::stdout;
use std::process::exit;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use clap::CommandFactory;
use clap::FromArgMatches;
use colog::default_builder;
use colog::formatter;
use gourd_lib::bailc;
use gourd_lib::config::Config;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::constants::PRIMARY_STYLE;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileSystemInteractor;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::debug;
use log::info;
use log::trace;
use log::LevelFilter;

use super::log::LogTokens;
use super::printing::get_styles;
use crate::cli::def::Cli;
use crate::cli::def::Command;
use crate::cli::def::RunSubcommand;
use crate::cli::def::StatusStruct;
use crate::cli::printing::print_version;
use crate::experiments::ExperimentExt;
use crate::local::run_local;
use crate::post::afterscript::run_afterscript;
use crate::post::postprocess_job::schedule_post_jobs;
use crate::slurm::checks::get_slurm_options_from_config;
use crate::slurm::handler::SlurmHandler;
use crate::slurm::interactor::SlurmCli;
use crate::status::blocking_status;
use crate::status::get_statuses;
use crate::status::printing::display_job;
use crate::status::printing::display_statuses;

/// This function parses command that gourd was run with.
pub async fn parse_command() {
    let styled = Cli::command().styles(get_styles()).get_matches();

    // This unwrap will print the error if the command is wrong.
    let command = Cli::from_arg_matches(&styled).unwrap();

    // https://github.com/rust-lang/rust/blob/master/library/std/src/backtrace.rs
    let backtrace_enabled = match env::var("RUST_LIB_BACKTRACE") {
        Ok(s) => s != "0",
        Err(_) => match env::var("RUST_BACKTRACE") {
            Ok(s) => s != "0",
            Err(_) => false,
        },
    };

    if backtrace_enabled {
        eprintln!("{:?}", process_command(&command).await);
    } else if let Err(e) = process_command(&command).await {
        eprintln!("{}error:{:#} {}", ERROR_STYLE, ERROR_STYLE, e.root_cause());
        eprint!("{}", e);
        exit(1);
    }
}

/// CLAP has parsed the command, now we process it.
pub async fn process_command(cmd: &Cli) -> Result<()> {
    let progress = setup_logging(cmd)?;

    let mut file_system = FileSystemInteractor { dry_run: cmd.dry };

    match cmd.command {
        Command::Run(args) => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            debug!("Creating a new experiment");
            trace!("The config is: {config:#?}");

            let mut experiment = Experiment::from_config(
                &config,
                Local::now(),
                match args.subcommand {
                    RunSubcommand::Local { .. } => Environment::Local,
                    RunSubcommand::Slurm { .. } => Environment::Slurm,
                },
                &file_system,
            )?;

            let exp_path = experiment.save(&config.experiments_folder, &file_system)?;
            debug!("Saved the experiment at {exp_path:?}");

            match args.subcommand {
                RunSubcommand::Local { .. } => {
                    if cmd.dry {
                        info!("Would have ran the experiment (dry)");
                    } else {
                        run_local(&mut experiment, &exp_path, &file_system).await?;

                        info!("Experiment started");

                        blocking_status(&progress, &experiment, &mut file_system)?;

                        info!("Experiment finished");
                        println!();
                    }
                }

                RunSubcommand::Slurm { .. } => {
                    let s: SlurmHandler<SlurmCli> = SlurmHandler::default();
                    s.check_version()?;
                    s.check_partition(&get_slurm_options_from_config(&config)?.partition)?;

                    if cmd.dry {
                        info!("Would have scheduled the experiment on slurm (dry)");
                    } else {
                        s.run_experiment(&config, &mut experiment, exp_path, file_system)?;
                        info!("Experiment started");
                    }

                    experiment.save(&config.experiments_folder, &file_system)?;
                }
            }

            if cmd.dry {
                info!(
                    "This was a dry run, {PRIMARY_STYLE}gourd status {}{PRIMARY_STYLE:#} \
                    will not display anything",
                    experiment.seq
                );
            } else {
                info!(
                    "Run {PRIMARY_STYLE}gourd status {}{PRIMARY_STYLE:#} to check on this experiment",
                    experiment.seq
                );
            }
        }

        Command::Status(StatusStruct {
            experiment_id,
            run_id,
            follow: blocking,
            ..
        }) => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            let experiment = match experiment_id {
                Some(id) => Experiment::experiment_from_folder(
                    id,
                    &config.experiments_folder,
                    &file_system,
                )?,

                None => Experiment::latest_experiment_from_folder(
                    &config.experiments_folder,
                    &file_system,
                )?,
            };

            debug!("Found the newest experiment with id: {}", experiment.seq);

            let statuses = get_statuses(&experiment, &mut file_system)?;

            // TODO: status should do something with these labels
            let _labels = run_afterscript(&statuses, &experiment, &file_system)?;

            match run_id {
                Some(id) => {
                    display_job(&mut stdout(), &experiment, &statuses, id)?;
                }
                None => {
                    info!(
                        "Displaying the status of jobs for experiment {}",
                        experiment.seq
                    );

                    if blocking {
                        blocking_status(&progress, &experiment, &mut file_system)?;
                    } else {
                        display_statuses(&mut stdout(), &experiment, &statuses)?;
                    }
                }
            }
        }

        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),

        Command::Analyse(_) => panic!("Analyse has not been implemented yet"),

        Command::Version => print_version(cmd.script),

        Command::Continue => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            let mut experiment = Experiment::latest_experiment_from_folder(
                &config.experiments_folder,
                &file_system,
            )?;

            debug!("Found the newest experiment with id: {}", experiment.seq);

            // Scheduling postprocessing jobs
            if let Environment::Slurm = experiment.env {
                debug!("Checking for postprocess jobs to be run");

                let mut statuses = get_statuses(&experiment, &mut file_system)?;
                schedule_post_jobs(&mut experiment, &mut statuses, &file_system)?;

                info!("Postprocessing scheduled for available jobs");
            } else {
                bailc!(
                    "Continue is only available for a Slurm experiment, not for a local one", ;
                    "",;
                    "",
                );
            }

            // Continuing the experiment
            let exp_path = experiment.save(&config.experiments_folder, &file_system)?;

            let s: SlurmHandler<SlurmCli> = SlurmHandler::default();
            s.check_version()?;
            s.check_partition(&get_slurm_options_from_config(&config)?.partition)?;

            if cmd.dry {
                info!("Would have continued the experiment on slurm (dry)");
            } else {
                s.run_experiment(&config, &mut experiment, exp_path, file_system)?;
                info!("Experiment continued");
            }

            experiment.save(&config.experiments_folder, &file_system)?;
        }
    }

    Ok(())
}

/// Prepare the log levels for the application.
///
/// Sets up a Colog logger with verbosity based on the flags provided by the
/// user. Valid verbosities are 0, 1, or 2 (for example, 2 is denoted by "-vv").
fn setup_logging(cmd: &Cli) -> Result<MultiProgress> {
    let mut log_build = default_builder();
    log_build.format(formatter(LogTokens));

    let bar = MultiProgress::new();

    if cmd.verbose == 2 {
        log_build.filter(None, LevelFilter::Trace);
    } else if cmd.verbose == 1 {
        log_build.filter(None, LevelFilter::Debug);
    } else if cmd.verbose == 0 {
        log_build.filter(None, LevelFilter::Info);
    } else {
        bailc!(
            "Only two levels of verbosity supported (ie. -vv)", ;
            "", ;
            "",
        )
    }

    LogWrapper::new(bar.clone(), log_build.build())
        .try_init()
        .with_context(ctx!(
            "Failed to initlaize the command line interface", ;
            "Make sure you are using a supported terminal",
        ))?;

    Ok(bar)
}
