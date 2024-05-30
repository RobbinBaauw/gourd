use std::time::Duration;

use super::*;

#[test]
fn failure_reason_fmt_slurm_kill() {
    assert_eq!(
        format!("{}", FailureReason::SlurmKill),
        "slurm killed the job"
    )
}

#[test]
fn failure_reason_fmt_user_forced() {
    assert_eq!(
        format!("{}", FailureReason::UserForced),
        "user killed the job"
    )
}

#[test]
fn failure_reason_fmt_exit_status() {
    assert_eq!(
        format!(
            "{}",
            FailureReason::ExitStatus(Measurement {
                exit_code: 213,
                wall_micros: Duration::from_secs(1),
                rusage: None
            })
        ),
        "exit code 213"
    )
}

#[test]
fn completion_fmt_dormant() {
    assert_eq!(format!("{}", Completion::Dormant), "dormant?")
}

#[test]
fn completion_fmt_pending() {
    assert_eq!(format!("{}", Completion::Pending), "pending!")
}

#[test]
fn completion_fmt_success() {
    assert_eq!(
        format!(
            "{}",
            Completion::Success(Measurement {
                exit_code: 213,
                wall_micros: Duration::from_secs(1),
                rusage: None
            })
        ),
        "\u{1b}[1m\u{1b}[32msuccess\u{1b}[0m, took: 1s"
    )
}

#[test]
fn completion_fmt_fail() {
    assert_eq!(
        format!("{}", Completion::Fail(FailureReason::SlurmKill)),
        "\u{1b}[1m\u{1b}[5m\u{1b}[31mfailed with slurm killed the job\u{1b}[0m"
    )
}
