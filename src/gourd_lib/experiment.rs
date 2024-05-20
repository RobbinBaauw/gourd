use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Input;
use crate::config::Program;
use crate::file_system::FileOperations;

/// The run location of the experiment.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    /// Local execution.
    Local,

    /// Slurm execution.
    Slurm,
}

/// Describes a matching between an algorithm and an input.
#[derive(Serialize, Deserialize, Debug)]
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

    /// The run location of this experiment.
    pub env: Environment,

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
