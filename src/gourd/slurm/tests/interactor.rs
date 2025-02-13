use std::time::Duration;

use gourd_lib::constants::SLURM_VERSIONS;

use super::SlurmCli;
use crate::slurm::interactor::format_slurm_duration;

#[test]
fn duration_fmt_test() {
    assert_eq!(format_slurm_duration(Duration::from_millis(50)), "00");
    assert_eq!(format_slurm_duration(Duration::from_secs(0)), "00");
    assert_eq!(format_slurm_duration(Duration::from_secs(59)), "59");
    assert_eq!(format_slurm_duration(Duration::from_secs(30 + 60)), "01:30");

    assert_eq!(
        format_slurm_duration(Duration::from_secs(12 + 16 * 60)),
        "16:12"
    );

    assert_eq!(
        format_slurm_duration(Duration::from_secs(5 + (26 + 3 * 60) * 60)),
        "03:26:05"
    );

    assert_eq!(
        format_slurm_duration(Duration::from_secs(21 + (40 + (6 + 3 * 24) * 60) * 60)),
        "3-06:40:21"
    );
}

#[test]
fn slurm_interactor_default_test() {
    assert_eq!(SlurmCli::default().versions, SLURM_VERSIONS);
}
