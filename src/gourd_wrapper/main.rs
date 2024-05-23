// for tarpaulin cfg
#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]
//! This wrapper runs the binary and measures metrics
//!
//! Run the wrapper with:
//!   - The path to the binary
//!   - The path to the input
//!   - The path where the output of the program should be placed
//!   - The path where the metrics should be output
//! As arguments, the wrapper will then perform the experiment.

mod measurement_unix;

use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;

use anstyle::Color;
use anstyle::Style;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::file_system::FileSystemInteractor;
use gourd_lib::measurement::Measurement;
use gourd_lib::measurement::Metrics;
use gourd_lib::measurement::RUsage;

const ERROR_STYLE: Style = anstyle::Style::new()
    .blink()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Red)));
const HELP_STYLE: Style = anstyle::Style::new()
    .bold()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Green)));

struct RunConf {
    binary_path: PathBuf,
    input_path: Option<PathBuf>,
    output_path: PathBuf,
    result_path: PathBuf,
    err_path: PathBuf,
    additional_args: Vec<String>,
}

fn main() {
    if let Err(err) = process() {
        eprintln!("{}error:{:#} {}", ERROR_STYLE, ERROR_STYLE, err);
        eprintln!(
            "{}caused by:{:#} {}",
            ERROR_STYLE,
            ERROR_STYLE,
            err.root_cause()
        );
        eprintln!("{}help:{:#} The gourd-wrapper program is internal. You should not be invoking it manually", HELP_STYLE, HELP_STYLE);
        exit(1);
    }
}

fn process() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let fs = FileSystemInteractor { dry_run: false };

    let rc = match args.len() {
        0..=2 => bail!("Slurm wrapper needs an experiment file path and a job id"),
        3 => process_slurm_args(&args, &fs).with_context(ctx!(
            "Could not process arguments (assuming slurm environment) {:?}", args;
            "",
        ))?,
        _ => process_local_args(&args).with_context(ctx!(
            "Could not process arguments (assuming local environment) {:?}", args;
            "",
        ))?,
    };

    fs::write(
        &rc.result_path,
        toml::to_string(&Metrics::Pending)
            .context("Could not serialize the Pending metrics state")?,
    )
    .context(format!(
        "Could not write to the result file {:?}",
        rc.result_path
    ))?;

    let clock = start_measuring();

    #[allow(unused_mut)]
    let mut child = Command::new(&rc.binary_path)
        .args(&rc.additional_args)
        .stdin(if let Some(actual_input) = rc.input_path {
            Stdio::from(
                File::open(actual_input.clone())
                    .context(format!("Could not open the input {:?}", actual_input))?,
            )
        } else {
            Stdio::null()
        })
        .stdout(Stdio::from(File::create(rc.output_path.clone()).context(
            format!("Could not truncate the output {:?}", rc.output_path),
        )?))
        .stderr(Stdio::from(File::create(rc.err_path.clone()).context(
            format!("Could not truncate the error {:?}", rc.err_path),
        )?))
        .spawn()
        .context(format!("Could not start the binary {:?}", &rc.binary_path))?;

    #[cfg(not(unix))]
    let (rusage_output, exit_code) = (
        None,
        child
            .wait()?
            .code()
            .context("Failed to retrieve the exit code")?,
    );
    #[cfg(unix)]
    let (rusage_output, exit_code) = {
        use crate::measurement_unix::GetRUsage;
        let (r, s) = child
            .wait_for_rusage()
            .context("Could not rusage the child")?;
        (Some(r), s)
    };

    let meas = stop_measuring(clock, exit_code, rusage_output);

    fs::write(
        &rc.result_path,
        toml::to_string(&Metrics::Done(meas)).context("Could not serialize the measurement")?,
    )
    .context(format!(
        "Could not write to the result file {:?}",
        rc.result_path
    ))?;

    Ok(())
}

fn process_slurm_args(args: &[String], fs: &impl FileOperations) -> Result<RunConf> {
    let exp_path: PathBuf = args[1]
        .parse()
        .context(format!("The experiment file path is invalid: {}", args[1]))?;
    let exp = fs.try_read_toml::<Experiment>(exp_path.as_path())?;
    let id: usize = args[2]
        .parse()
        .with_context(ctx!(
            "Could not parse the run id from the arguments {:?}", args;
            "Ensure that Slurm is configured correctly",
        ))
        .unwrap();

    Ok(RunConf {
        binary_path: exp.config.programs[&exp.runs[id].program].binary.clone(),
        input_path: exp.config.inputs[&exp.runs[id].input].input.clone(),
        output_path: exp.runs[id].output_path.clone(),
        result_path: exp.runs[id].metrics_path.clone(),
        err_path: exp.runs[id].err_path.clone(),
        additional_args: vec![],
    })
}

fn process_local_args(args: &[String]) -> Result<RunConf> {
    if args.len() < 6 {
        bail!("Wrapper needs at least 6 arguments");
    }
    Ok(RunConf {
        binary_path: PathBuf::from(&args[1]),
        input_path: Some(PathBuf::from(&args[2])),
        output_path: PathBuf::from(&args[3]),
        result_path: PathBuf::from(&args[4]),
        err_path: PathBuf::from(&args[5]),
        additional_args: args[6..].to_vec(),
    })
}

/// This is an extensible structure for measuring monotonic metrics.
struct Clock {
    wall_time: Instant,
}

/// Start the measurement, returns a new instance of a [Clock].
fn start_measuring() -> Clock {
    Clock {
        wall_time: Instant::now(),
    }
}

/// Stop a measurement, returns a new instance of a [Measurement]
fn stop_measuring(clk: Clock, exit_code: i32, rusage: Option<RUsage>) -> Measurement {
    Measurement {
        wall_micros: clk.wall_time.elapsed(),
        exit_code,
        rusage,
    }
}
