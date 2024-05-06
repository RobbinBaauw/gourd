// The follwing code is originally from: https://docs.rs/command-rusage
// Licensed under MIT.
// It exists because we have to modify the behaviour of it.

use std::process::Child;
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::WEXITSTATUS;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::WIFEXITED;
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

/// Error type for `getrusage` failures.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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

/// Returns an empty `libc::rusage` struct.
#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe fn empty_raw_rusage() -> libc::rusage {
    std::mem::zeroed()
}

/// Converts a `libc::timeval` to a `std::time::Duration`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn duration_from_timeval(timeval: libc::timeval) -> Duration {
    Duration::new(timeval.tv_sec as u64, (timeval.tv_usec * 1000) as u32)
}

impl GetRUsage for Child {
    fn wait_for_rusage(&mut self) -> Result<RUsage, Error> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let pid = self.id() as i32;
            let mut status: i32 = 0;

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
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(Error::UnsupportedPlatform)
        }
    }
}
