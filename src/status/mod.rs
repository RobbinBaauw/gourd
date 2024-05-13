use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use self::fs_based::FileBasedStatus;
use crate::experiment::Experiment;

/// File system based status information.
pub mod fs_based;

/// The reasons for a job failing.
#[derive(Debug, Clone, Copy)]
pub enum FailureReason {
    /// The job retunrned a non zero exit status.
    ExitStatus(i32),

    /// Slurm killed the job.
    SlurmKill,

    /// User marked.
    UserForced,
}

/// This possible outcomes of a job.
#[derive(Debug, Clone, Copy)]
pub enum Completion {
    /// The job has not yet started.
    Dormant,
    /// The job is still running.
    Pending,
    /// The job succeded.
    Success,
    /// The job failed with the following exit status.
    Fail(FailureReason),
}

/// All possible statuses of a job.
#[derive(Debug, Clone)]
pub enum Status {
    /// This job has no postprocessing.
    NoPostprocessing(Completion),

    /// This job runs an after script and the output of it is stored at this path.
    AfterScript(Completion, PathBuf),

    /// This job has full postprocessing.
    Postprocessed(Completion, Completion, PathBuf),
}

/// This type maps between `run_id` and the [Status] of the run.
pub type ExperimentStatus = BTreeMap<usize, Option<Status>>;

/// A struct that can attest the statuses or some or all running jobs.
pub trait StatusProvider<T> {
    /// Try to get the statuses of jobs.
    fn get_statuses(connection: T, experiment: &Experiment) -> Result<ExperimentStatus>;
}

/// Get the status of the provided experiment.
pub fn get_statuses(experiment: &Experiment) -> Result<ExperimentStatus> {
    // for now we do not support slurm.

    FileBasedStatus::get_statuses((), experiment)
}

/// Display the status of an experiment in a human readable from.
pub fn display_statuses(experiment: &Experiment, statuses: &ExperimentStatus) {
    for (run_id, _) in experiment.runs.iter().enumerate() {
        println!("Run {} has {:?}", run_id, statuses[&run_id]);
    }
}
