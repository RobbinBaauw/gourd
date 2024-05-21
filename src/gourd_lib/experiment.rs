use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Input;
use crate::config::Program;
use crate::config::ResourceLimits;
use crate::file_system::FileOperations;

/// Describes a matching between an algorithm and an input.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Run {
    /// The program to run.
    pub program: Program,

    /// The unique name of the input to run with.
    pub input: Input,

    /// The path to the stderr output.
    pub err_path: PathBuf,

    /// The path to the stdout output.
    pub output_path: PathBuf,

    /// The path to the metrics file.
    pub metrics_path: PathBuf,

    /// Slurm job id, if ran on slurm
    pub job_id: Option<usize>,

    /// The path to afterscript output, optionally.
    pub afterscript_output_path: Option<PathBuf>,

    /// The path to postprocess job output, optionally.
    pub post_job_output_path: Option<PathBuf>,
}

/// Describes one experiment.
#[derive(Serialize, Deserialize, Debug)]
pub struct Experiment {
    /// The pairings of program-input for this experiment.
    pub runs: Vec<Run>,

    /// The runtime data for running on Slurm.
    /// Present if this is a Slurm experiment, absent otherwise.
    pub slurm: Option<SlurmExperiment>,

    /// The time of creation of the experiment.
    pub creation_time: DateTime<Local>,

    /// The ID of this experiment.
    pub seq: usize,
}

impl Experiment {
    /// Save the experiment to a file with its timestamp.
    pub fn save(&self, folder: &Path, fs: &impl FileOperations) -> Result<PathBuf> {
        let saving_path = folder.join(format!("{}.toml", self.seq));

        fs.try_write_toml(&saving_path, &self)?;

        Ok(saving_path)
    }
}

/// Runtime data for running on Slurm, including scheduled chunks.
#[derive(Serialize, Deserialize, Debug)]
pub struct SlurmExperiment {
    /// Chunks scheduled for running on Slurm.
    pub chunks: Vec<Chunk>,

    /// Global resource limits that will apply to _newly created chunks_.
    pub resource_limits: ResourceLimits,
}

/// Describes one chunk: a Slurm array of scheduled runs with common resource limits.
/// Chunks are created at runtime; a run is in one chunk iff it has been scheduled.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Chunk {
    /// The runs that belong to this chunk (by RunID)
    pub runs: Vec<usize>,

    /// The resource limits of this chunk.
    pub resource_limits: ResourceLimits,
}

// this stays here as an idea to eventually implement,
// right now i realised that because SLURM_TASK_ID is an environment variable
// i cant pass it into a serialized struct
// /// serialize this and pass it to the wrapper as first argument
// #[derive(Serialize, Deserialize, Debug)]
// pub struct WrapperArgs {
//     /// Path to the experiment toml
//     pub experiment: PathBuf,
//     ///
//     pub run_id: usize,
// }
