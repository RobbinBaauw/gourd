use std::env;
use std::io::stdout;
use std::io::BufWriter;
use std::process::exit;
use std::thread::sleep;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use clap::CommandFactory;
use clap::FromArgMatches;
use colog::default_builder;
use colog::formatter;
use gourd_lib::config::Config;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::constants::PRIMARY_STYLE;
use gourd_lib::constants::STATUS_REFRESH_RATE;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
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
use crate::cli::printing::generate_progress_bar;
use crate::cli::printing::print_version;
use crate::experiments::ExperimentExt;
use crate::local::run_local;
use crate::post::afterscript::run_afterscript;
use crate::post::postprocess_job::schedule_post_jobs;
use crate::slurm::checks::get_slurm_options_from_config;
use crate::slurm::handler::SlurmHandler;
use crate::slurm::interactor::SlurmCLI;
use crate::status::display_job;
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

            let exp_path = experiment.save(&config.experiments_folder, &file_system)?;
            debug!("Saving the experiment at {exp_path:?}");

            match args.sub_command {
                RunSubcommand::Local { .. } => {
                    if cmd.dry {
                        info!("Would have ran the experiment (dry)");
                    } else {
                        run_local(&experiment, &file_system).await?;
                        info!("Experiment started");

                        let mut complete = 0;
                        let mut message = "".to_string();

                        let bar =
                            progress.add(generate_progress_bar(experiment.runs.len() as u64)?);

                        while complete < experiment.runs.len() {
                            let mut buf = BufWriter::new(Vec::new());

                            let statuses = get_statuses(&experiment, file_system)?;
                            complete = display_statuses(&mut buf, &experiment, &statuses)?;
                            message = format!("{}\n", String::from_utf8(buf.into_inner()?)?);

                            bar.set_prefix(message.clone());
                            bar.set_position(complete as u64);

                            sleep(STATUS_REFRESH_RATE);
                        }

                        bar.finish();
                        progress.remove(&bar);
                        progress.clear()?;

                        let leftover = generate_progress_bar(experiment.runs.len() as u64)?;
                        leftover.set_position(complete as u64);
                        leftover.set_prefix(message);
                        leftover.finish();

                        info!("Experiment finished");
                        println!();
                    }
                }

                RunSubcommand::Slurm { .. } => {
                    let s: SlurmHandler<SlurmCLI> = SlurmHandler::default();
                    s.check_version()?;
                    s.check_partition(&get_slurm_options_from_config(&config)?.partition)?;

                    if cmd.dry {
                        info!("Would have scheduled the experiment on slurm (dry)");
                    } else {
                        s.run_experiment(&config, &mut experiment, exp_path)?;
                        info!("Experiment started");
                    }
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

            let statuses = get_statuses(&experiment, file_system)?;

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

                    display_statuses(&mut stdout(), &experiment, &statuses)?;
                }
            }
        }

        Command::Init(_) => panic!("Gourd Init has not been implemented yet"),

        Command::Anal(_) => panic!("Analyze has not been implemented yet"),

        Command::Version => print_version(cmd.script),

        Command::Postprocess => {
            debug!("Reading the config: {:?}", cmd.config);

            let config = Config::from_file(&cmd.config, &file_system)?;

            let mut experiment = Experiment::latest_experiment_from_folder(
                &config.experiments_folder,
                &file_system,
            )?;

            debug!("Found the newest experiment with id: {}", experiment.seq);

            let mut statuses = get_statuses(&experiment, file_system)?;

            schedule_post_jobs(&mut experiment, &mut statuses, &file_system)?;

            info!("Postprocessing scheduled for available jobs");
        }
    }

    Ok(())
}

/// Prepare the log levels for the application.
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
        return Err(anyhow!("Only two levels of verbosity supported (ie. -vv)")).context("");
    }

    LogWrapper::new(bar.clone(), log_build.build())
        .try_init()
        .with_context(ctx!(
            "Failed to initlaize the command line interface", ;
            "Make sure you are using a supported terminal",
        ))?;

    Ok(bar)
}
