//! This wrapper runs the binary and measures metrics
//!
//! Run the wrapper with:
//!   - The path to the binary
//!   - The path to the input
//!   - The path where the output of the program should be placed
//!   - The path where the metrics should be output
//!
//! as arguments, the wrapper will then perform the experiment.

/// Measurements for unix-like systems.
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
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::file_system::FileSystemInteractor;
use gourd_lib::measurement::Measurement;
use gourd_lib::measurement::Metrics;
use gourd_lib::measurement::RUsage;

/// How to style the errors.
const ERROR_STYLE: Style = anstyle::Style::new()
    .blink()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Red)));

/// How to style the help.
const HELP_STYLE: Style = anstyle::Style::new()
    .bold()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Green)));

/// A single run configuration.
struct RunConf {
    /// The path to the binary.
    binary_path: PathBuf,
    /// The path to the input.
    input_path: Option<PathBuf>,
    /// The path to the working directory.
    work_dir: PathBuf,

    /// The path to the stdout file.
    output_path: PathBuf,
    /// The path to the result file.
    result_path: PathBuf,
    /// The path to the stderr file.
    err_path: PathBuf,
    /// Additional arguments.
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
        eprintln!("{}help:{:#} The gourd_wrapper program is internal. You should not be invoking it manually", HELP_STYLE, HELP_STYLE);
        exit(1);
    }
}

/// Internal part of the wrapper.
fn process() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let fs = FileSystemInteractor { dry_run: false };

    let rc = match args.len() {
        5 => process_args(&args, &fs)?,
        _ => bail!("gourd_wrapper needs an experiment file path, a chunk index and a task index"),
    };

    fs::write(
        &rc.result_path,
        toml::to_string(&Metrics::NotCompleted)
            .context("Could not serialize the Not Completed metrics state")?,
    )
    .context(format!(
        "Could not write to the result file {:?}",
        rc.result_path
    ))?;

    let clock = start_measuring();

    eprintln!("RUNNING {:?}", &rc.binary_path);
    eprintln!("ARGS {:?}", &rc.additional_args);
    #[allow(unused_mut)]
    let mut child = Command::new(&rc.binary_path)
        .current_dir(&rc.work_dir)
        .args(&rc.additional_args)
        .stdin(if let Some(actual_input) = rc.input_path.clone() {
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
        child
            .wait_for_rusage()
            .context("Could not rusage the child")?
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

/// Process the command line arguments passed to the wrapper.
fn process_args(args: &[String], fs: &impl FileOperations) -> Result<RunConf> {
    let exp_path: PathBuf = args[1]
        .parse()
        .context(format!("The experiment file path is invalid: {}", args[1]))?;

    let exp = fs.try_read_toml::<Experiment>(exp_path.as_path())?;

    if exp.chunks.is_empty() {
        bail!("The experiment has no chunks");
    }

    let chunk_id: usize = args[2].parse().with_context(ctx!(
        "Could not parse the run id from the arguments {:?}", args;
        "Ensure that Slurm is configured correctly",
    ))?;

    let job_id: usize = args[3].parse().with_context(ctx!(
        "Could not parse the run id from the arguments {:?}", args;
        "Ensure that Slurm is configured correctly",
    ))?;

    let run_idx: usize = args[4].parse().with_context(ctx!(
        "Could not parse the run id from the arguments {:?}", args;
        "Ensure that Slurm is configured correctly",
    ))?;

    let run = exp.runs[exp.chunks[chunk_id][job_id][run_idx]].clone();

    let program = &exp.get_program(&run)?;

    let mut additional_args = program.arguments.clone();
    additional_args.append(&mut run.input.arguments.clone());

    Ok(RunConf {
        binary_path: program.binary.clone().to_path_buf(),
        input_path: run.input.file,
        output_path: run.output_path.clone(),
        result_path: run.metrics_path.clone(),
        work_dir: run.work_dir.clone(),
        err_path: run.err_path.clone(),
        additional_args,
    })
}

/// This is an extensible structure for measuring monotonic metrics.
struct Clock {
    /// The real-world time this program took to execute.
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
