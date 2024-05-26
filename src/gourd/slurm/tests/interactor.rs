use std::time::Duration;

use crate::slurm::interactor::SlurmCli;

#[test]
fn duration_fmt_test() {
    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_millis(50)),
        "00"
    );
    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(0)),
        "00"
    );
    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(59)),
        "59"
    );
    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(30 + 60)),
        "01:30"
    );

    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(12 + 16 * 60)),
        "16:12"
    );

    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(5 + (26 + 3 * 60) * 60)),
        "03:26:05"
    );

    assert_eq!(
        SlurmCli::format_slurm_duration(Duration::from_secs(21 + (40 + (6 + 3 * 24) * 60) * 60)),
        "3-06:40:21"
    );
}
