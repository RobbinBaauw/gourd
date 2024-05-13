#![warn(missing_docs)]
// for tarpaulin cfg
#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]
#![allow(unused_imports)]
//! This wrapper runs the binary and measures metrics
//!
//! Run the wrapper with:
//!   - The path to the binary
//!   - The path to the input
//!   - The path where the output of the program should be placed
//!   - The path where the metrics should be output
//! As arguments, the wrapper will then perform the experiment.

mod measurement;

use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use anstyle::Color;
use anstyle::Style;
use anyhow::bail;
use anyhow::Context;
use measurement::Metrics;
use serde::Deserialize;
use serde::Serialize;

use crate::measurement::GetRUsage;
use crate::measurement::Measurement;
use crate::measurement::RUsage;

const ERROR_STYLE: Style = anstyle::Style::new()
    .blink()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Red)));
const HELP_STYLE: Style = anstyle::Style::new()
    .bold()
    .fg_color(Some(Color::Ansi(anstyle::AnsiColor::Green)));

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

fn process() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        bail!("Wrapper needs at least 6 arguments");
    }

    let binary_path: &String = &args[1];
    let input_path: PathBuf = args[2]
        .parse()
        .context(format!("The input file path is invalid: {}", args[2]))?;
    let output_path: PathBuf = args[3]
        .parse()
        .context(format!("The output file path is invalid: {}", args[3]))?;
    let result_path: PathBuf = args[4]
        .parse()
        .context(format!("The result file path is invalid: {}", args[4]))?;
    let err_path: PathBuf = args[5]
        .parse()
        .context(format!("The error file path is invalid: {}", args[5]))?;

    fs::write(
        &result_path,
        toml::to_string(&Metrics::Pending)
            .context("Could not serialize the Pending metrics state")?,
    )
    .context(format!(
        "Could not write to the result file {:?}",
        result_path
    ))?;

    let clock = start_measuring();

    let mut child = Command::new(binary_path)
        .args(&args[6..])
        .stdin(Stdio::from(File::open(input_path.clone()).context(
            format!("Could not open the input {:?}", input_path),
        )?))
        .stdout(Stdio::from(File::create(output_path.clone()).context(
            format!("Could not truncate the output {:?}", output_path),
        )?))
        .stderr(Stdio::from(File::create(err_path.clone()).context(
            format!("Could not truncate the error {:?}", err_path),
        )?))
        .spawn()
        .context(format!("Could not start the binary {}", binary_path))?;

    let success = child
        .wait_for_rusage()
        .context("Could not rusage the child")?;

    let meas = stop_measuring(clock, success);

    fs::write(
        &result_path,
        toml::to_string(&Metrics::Done(meas)).context("Could not serialize the measurement")?,
    )
    .context(format!(
        "Could not write to the result file {:?}",
        result_path
    ))?;

    Ok(())
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
fn stop_measuring(clk: Clock, rusage: RUsage) -> Measurement {
    Measurement {
        wall_micros: clk.wall_time.elapsed(),
        rusage,
    }
}
