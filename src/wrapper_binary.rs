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
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::measurement::GetRUsage;
use crate::measurement::Measurement;
use crate::measurement::RUsage;

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();

    let binary_path: &String = &args[1];
    let input_path: PathBuf = args[2].parse().expect("The input path is invalid");
    let output_path: PathBuf = args[3].parse().expect("The output path is invalid");
    let result_path: PathBuf = args[4].parse().expect("The result path is invalid");

    if let Some(parent) = result_path.parent() {
        fs::create_dir_all(parent).expect("Could not create result directory");
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("Could not create output directory");
    }

    let clock = start_measuring();

    let mut child = Command::new(binary_path)
        .args(&args[5..])
        .stdin(Stdio::from(
            File::open(input_path).expect("Could not open the input"),
        ))
        .stdout(Stdio::from(
            File::create(output_path).expect("Could not truncate the output file"),
        ))
        .spawn()
        .expect("Could not spawn the binary");

    let success = child.wait_for_rusage().expect("Could not rusage the child");

    let meas = stop_measuring(clock, success);

    fs::write(
        result_path,
        toml::to_string(&meas).expect("Could not serialize the results"),
    )
    .expect("Could not write the result file");
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
