// The follwing code is originally from: https://docs.rs/command-rusage
// Licensed under MIT.
// It exists because we have to modify the behaviour of it.

use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

/// This structure contains the measurements for one run of the binary.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Measurement {
    /// Interval of wall time.
    pub wall_micros: Duration,

    /// The rusage of the invoked program.
    pub rusage: RUsage,
}

/// Resource usage statistics for a process.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
    /// The exit status of the program.
    pub exit_status: i32,
}
