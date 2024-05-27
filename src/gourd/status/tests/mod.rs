use super::*;

#[test]
fn failure_reason_fmt_slurm_kill() {
    assert_eq!(format!("{}", FailureReason::SlurmKill), "slurm killed")
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
        format!("{}", FailureReason::ExitStatus(213)),
        "exit code 213"
    )
}

#[test]
fn completion_fmt_dormant() {
    assert_eq!(format!("{}", Completion::Dormant), "?")
}

#[test]
fn completion_fmt_pending() {
    assert_eq!(format!("{}", Completion::Pending), "pending!")
}

#[test]
fn completion_fmt_success() {
    assert_eq!(
        format!("{}", Completion::Success),
        "\u{1b}[1m\u{1b}[32msuccess\u{1b}[0m"
    )
}

#[test]
fn completion_fmt_fail() {
    assert_eq!(
        format!("{}", Completion::Fail(FailureReason::SlurmKill)),
        "\u{1b}[1m\u{1b}[5m\u{1b}[31mfailed with slurm killed\u{1b}[0m"
    )
}
