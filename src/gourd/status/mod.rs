use std::collections::BTreeMap;

use anyhow::Result;
use gourd_lib::experiment::Experiment;

use self::fs_based::FileBasedStatus;

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

    /// The job succeeded.
    Success,

    /// The job failed with the following exit status.
    Fail(FailureReason),
}

/// This possible outcomes of a postprocessing.
#[derive(Debug, Clone)]
pub enum PostprocessCompletion {
    /// The postprocessing job has not yet started.
    Dormant,

    /// The postprocessing job is still running.
    Pending,

    /// The postprocessing job succeded.
    Success(PostprocessOutput),

    /// The postprocessing job failed with the following exit status.
    Fail(FailureReason),
}

/// The results of a postprocessing.
#[derive(Debug, Clone)]
pub struct PostprocessOutput {
    /// The shortened version of postprocessing output.
    pub short_output: String,

    /// The full output of postprocessing.
    pub long_output: String,
}

/// All possible postprocessing statuses of a job.
#[derive(Debug, Clone)]
pub enum Status {
    /// This job has no postprocessing.
    NoPostprocessing(Completion),

    /// This job runs an afterscript.
    AfterScript(Completion, PostprocessCompletion),

    /// This job has a full postprocessing.
    Postprocessed(Completion, PostprocessCompletion),
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
