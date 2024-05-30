use std::collections::BTreeMap;

use gourd_lib::measurement::Metrics;

use crate::post::postprocess_job::filter_runs_for_post_job;
use crate::status::FailureReason;
use crate::status::FileSystemBasedStatus;
use crate::status::PostprocessCompletion;
use crate::status::PostprocessOutput;
use crate::status::RunState;
use crate::status::SlurmBasedStatus;
use crate::status::Status;

#[test]
fn test_filter_runs_for_post_job_good_weather() {
    let mut runs: BTreeMap<usize, Status> = BTreeMap::new();
    runs.insert(
        0,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: RunState::Pending,
                metrics: Some(Metrics::NotCompleted),
                afterscript_completion: None,
                postprocess_job_completion: Some(PostprocessCompletion::Dormant),
            },
            slurm_status: None,
        },
    );
    runs.insert(
        1,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: RunState::Completed,
                metrics: Some(Metrics::NotCompleted),
                afterscript_completion: Some(PostprocessCompletion::Dormant),
                postprocess_job_completion: Some(PostprocessCompletion::Dormant),
            },
            slurm_status: Some(SlurmBasedStatus {
                completion: RunState::Completed,
                exit_code_program: 0,
                exit_code_slurm: 0,
            }),
        },
    );
    runs.insert(
        2,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: RunState::Fail(FailureReason::UserForced),
                metrics: Some(Metrics::NotCompleted),
                afterscript_completion: None,
                postprocess_job_completion: Some(PostprocessCompletion::Dormant),
            },
            slurm_status: None,
        },
    );
    runs.insert(
        3,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: RunState::Completed,
                metrics: Some(Metrics::NotCompleted),
                afterscript_completion: None,
                postprocess_job_completion: Some(PostprocessCompletion::Success(
                    PostprocessOutput {
                        short_output: String::from("short"),
                        long_output: String::from("long"),
                    },
                )),
            },
            slurm_status: None,
        },
    );

    let res = filter_runs_for_post_job(&mut runs).unwrap();

    assert_eq!(res.len(), 1);

    let paths = res[0];
    assert_eq!(*paths, 1);
}

// // test post jobs getting scheduled (good + bad)
