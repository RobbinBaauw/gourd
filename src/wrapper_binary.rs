#![warn(missing_docs)]
//! This wrapper runs the binary and measures metrics
//!
//! Run the wrapper with:
//!   - The path to the binary
//!   - The path to the input
//!   - The path where the output of the program should be placed
//!   - The path where the metrics should be output
//! As arguments, the wrapper will then perform the experiment.

use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;
use std::time::SystemTime;

use serde::Deserialize;
use serde::Serialize;

/// This structure contains the measurements for one run of the binary.
#[derive(Serialize, Deserialize, Debug)]
pub struct Measurement {
    /// Microseconds of wall time
    pub wall_micros: u128,
    /// Microseconds of system time
    pub sys_micros: u128,
    /// Exit code
    pub code: i32,
}

#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();

    let binary_path: &String = &args[1];
    let input_path: PathBuf = args[2].parse().expect("The input path is invalid");
    let output_path: PathBuf = args[3].parse().expect("The output path is invalid");
    let result_path: PathBuf = args[4].parse().expect("The result path is invalid");

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

    let success = child
        .wait()
        .expect("Could not execute the child")
        .code()
        .expect("Could not get the exit code");

    let meas = stop_measuring(clock, success);

    fs::write(
        result_path,
        postcard::to_allocvec(&meas).expect("Could not serialize the results"),
    )
    .expect("Could not write the result file");
}

struct Clock {
    wall_time: Instant,
    sys_time: SystemTime,
}

fn start_measuring() -> Clock {
    Clock {
        wall_time: Instant::now(),
        sys_time: SystemTime::now(),
    }
}

fn stop_measuring(clk: Clock, code: i32) -> Measurement {
    Measurement {
        wall_micros: clk.wall_time.elapsed().as_micros(),
        sys_micros: clk
            .sys_time
            .elapsed()
            .expect("Could not measure system time")
            .as_micros(),
        code,
    }
}
