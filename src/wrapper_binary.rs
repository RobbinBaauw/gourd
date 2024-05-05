#![warn(missing_docs)]
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

mod shared;

use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use libc::WEXITSTATUS;
use libc::WIFEXITED;
use serde::Deserialize;
use serde::Serialize;

use crate::shared::Measurement;
use crate::shared::RUsage;

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

// The follwing code is originally from: https://docs.rs/command-rusage
// Licensed under MIT.
// It exists because we have to modify the behaviour of it.

/// Error type for `getrusage` failures.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The process exists, but its resource usage statistics are unavailable.
    Unavailable,
    /// This platform is not supported. There is only support for linux.
    UnsupportedPlatform,
}

/// A trait for getting resource usage statistics for a process.
pub trait GetRUsage {
    /// Waits for the process to exit and returns its resource usage statistics.
    /// Works only on linux with wait4 syscall available.
    fn wait_for_rusage(&mut self) -> Result<RUsage, Error>;
}

/// Type wrapper for result.
pub type RUsageResult = std::result::Result<RUsage, Error>;

/// Returns an empty `libc::rusage` struct.
#[cfg(target_os = "linux")]
unsafe fn empty_raw_rusage() -> libc::rusage {
    std::mem::zeroed()
}

/// Converts a `libc::timeval` to a `std::time::Duration`.
fn duration_from_timeval(timeval: libc::timeval) -> Duration {
    Duration::new(timeval.tv_sec as u64, (timeval.tv_usec * 1000) as u32)
}

impl GetRUsage for Child {
    fn wait_for_rusage(&mut self) -> Result<RUsage, Error> {
        let pid = self.id() as i32;
        let mut status: i32 = 0;
        #[cfg(target_os = "linux")]
        {
            let mut rusage;
            unsafe {
                rusage = empty_raw_rusage();
                libc::wait4(
                    pid,
                    &mut status as *mut libc::c_int,
                    0i32,
                    &mut rusage as *mut libc::rusage,
                );
            }

            if WIFEXITED(status) {
                Ok(RUsage {
                    utime: duration_from_timeval(rusage.ru_utime),
                    stime: duration_from_timeval(rusage.ru_stime),
                    maxrss: rusage.ru_maxrss as usize,
                    ixrss: rusage.ru_ixrss as usize,
                    idrss: rusage.ru_idrss as usize,
                    isrss: rusage.ru_isrss as usize,
                    minflt: rusage.ru_minflt as usize,
                    majflt: rusage.ru_majflt as usize,
                    nswap: rusage.ru_nswap as usize,
                    inblock: rusage.ru_inblock as usize,
                    oublock: rusage.ru_oublock as usize,
                    msgsnd: rusage.ru_msgsnd as usize,
                    msgrcv: rusage.ru_msgrcv as usize,
                    nsignals: rusage.ru_nsignals as usize,
                    nvcsw: rusage.ru_nvcsw as usize,
                    nivcsw: rusage.ru_nivcsw as usize,
                    exit_status: WEXITSTATUS(status),
                })
            } else {
                Err(Error::Unavailable)
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(Error::UnsupportedPlatform)
        }
    }
}
