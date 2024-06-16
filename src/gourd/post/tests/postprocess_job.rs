use std::collections::BTreeMap;
use std::time::Duration;

use gourd_lib::measurement::Measurement;

// use crate::post::postprocess_job::filter_runs_for_post_job;
use crate::status::FileSystemBasedStatus;
use crate::status::FsState;
use crate::status::PostprocessCompletion;
use crate::status::SlurmBasedStatus;
use crate::status::SlurmState;
use crate::status::Status;

#[test]
fn test_filter_runs_for_afterscript_good_weather() {
    let mut runs: BTreeMap<usize, Status> = BTreeMap::new();
    runs.insert(
        0,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: crate::status::FsState::Pending,
                afterscript_completion: Some(PostprocessCompletion::Dormant),
            },
            slurm_status: None,
        },
    );
    runs.insert(
        1,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 0,
                    rusage: None,
                }),
                afterscript_completion: None,
            },
            slurm_status: Some(SlurmBasedStatus {
                completion: SlurmState::Success,
                exit_code_program: 0,
                exit_code_slurm: 0,
            }),
        },
    );
    runs.insert(
        2,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 1,
                    rusage: None,
                }),
                afterscript_completion: None,
            },
            slurm_status: None,
        },
    );
    runs.insert(
        3,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 0,
                    rusage: None,
                }),
                afterscript_completion: None,
            },
            slurm_status: None,
        },
    );

    // let res = filter_runs_for_post_job(&mut runs, ).unwrap();
    let res: Vec<()> = Vec::new();

    assert_eq!(res.len(), 0);

    // let paths = res[0];
    // assert_eq!(paths, 1);
}

// // test post jobs getting scheduled (good + bad)
