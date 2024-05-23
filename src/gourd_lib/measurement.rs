// The follwing code is originally from: https://docs.rs/command-rusage
// Licensed under MIT.
// It exists because we have to modify the behaviour of it.

use std::fmt::Display;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

use crate::constants::NAME_STYLE;

/// The metrics of running a program.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(tag = "type")]
pub enum Metrics {
    /// The metrics have not been calculated yet.
    NotCompleted,

    /// The measurement has been finished.
    Done(Measurement),
}

/// This structure contains the measurements for one run of the binary.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    /// Interval of wall time.
    pub wall_micros: Duration,
    /// The exit code of the invoked program.
    pub exit_code: i32,
    /// The rusage of the invoked program.
    pub rusage: Option<RUsage>,
}

/// Resource usage statistics for a process.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct RUsage {
    /// User CPU time used.
    pub utime: Duration,
    /// System CPU time used.
    pub stime: Duration,
    /// Maximum resident set size.
    pub maxrss: usize,
    /// Integral shared memory size.
    pub ixrss: usize,
    /// Integral unshared data size.
    pub idrss: usize,
    /// Integral unshared stack size.
    pub isrss: usize,
    /// Page reclaims (soft page faults).
    pub minflt: usize,
    /// Page faults (hard page faults).
    pub majflt: usize,
    /// Swaps.
    pub nswap: usize,
    /// Block input operations.
    pub inblock: usize,
    /// Block output operations.
    pub oublock: usize,
    /// IPC messages sent.
    pub msgsnd: usize,
    /// IPC messages received.
    pub msgrcv: usize,
    /// Signals received.
    pub nsignals: usize,
    /// Voluntary context switches.
    pub nvcsw: usize,
    /// Involuntary context switches.
    pub nivcsw: usize,
}

impl Display for RUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "  {NAME_STYLE}user   cpu time{NAME_STYLE:#}: {}",
            humantime::Duration::from(self.utime)
        )?;
        writeln!(
            f,
            "  {NAME_STYLE}system cpu time{NAME_STYLE:#}: {}",
            humantime::Duration::from(self.stime)
        )?;
        writeln!(
            f,
            "  {NAME_STYLE}page faults{NAME_STYLE:#}: {}",
            self.majflt
        )?;
        writeln!(
            f,
            "  {NAME_STYLE}signals recieved{NAME_STYLE:#}: {}",
            self.nsignals
        )?;
        writeln!(
            f,
            "  {NAME_STYLE}context switches{NAME_STYLE:#}: {}",
            self.nvcsw + self.nivcsw
        )?;

        Ok(())
    }
}
