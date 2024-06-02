use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::config::Input;
use crate::config::Program;
use crate::config::ResourceLimits;
use crate::file_system::FileOperations;

/// Differentiating between regular and postprocess programs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProgramRef {
    Regular(String),
    Postprocess(String),
}

impl Display for ProgramRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ProgramRef::Regular(name) => name,
            ProgramRef::Postprocess(name) => name,
        };

        write!(f, "{}", name)
    }
}

/// Differentiating between regular and postprocess inputs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InputRef {
    Regular(String),
    Postprocess(String),
}

impl Display for InputRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            InputRef::Regular(name) => name,
            InputRef::Postprocess(name) => name,
        };

        write!(f, "{}", name)
    }
}

/// Describes a matching between an algorithm and an input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Run {
    /// The unique name of the program to run.
    pub program: ProgramRef,

    /// The unique name of the input to run with.
    pub input: InputRef,

    /// The path to the stderr output.
    pub err_path: PathBuf,

    /// The path to the stdout output.
    pub output_path: PathBuf,

    /// The path to the metrics file.
    pub metrics_path: PathBuf,

    /// Slurm job id, if ran on slurm
    pub slurm_id: Option<String>,

    /// The path to afterscript output, optionally.
    pub afterscript_output_path: Option<PathBuf>,

    /// The path to postprocess job output, optionally.
    pub post_job_output_path: Option<PathBuf>,
}

/// An enum to distinguish the run context.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Environment {
    /// Local execution.
    Local,

    /// Slurm execution.
    Slurm,
}

/// Describes one experiment.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Experiment {
    /// The pairings of program-input for this experiment.
    pub runs: Vec<Run>,

    /// Chunks scheduled for running on Slurm.
    pub chunks: Vec<Chunk>,

    /// Global resource limits that will apply to _newly created chunks_.
    pub resource_limits: Option<ResourceLimits>,

    /// The time of creation of the experiment.
    pub creation_time: DateTime<Local>,

    /// Experiment internal, specific config.
    pub config: Config,

    /// The ID of this experiment.
    pub seq: usize,

    /// Enviroment of the experiment
    pub env: Environment,

    /// The inputs for postprocessing programs.
    pub postprocess_inputs: BTreeMap<String, Input>,
}

impl Experiment {
    /// Save the experiment to a file with its timestamp.
    pub fn save(&self, folder: &Path, fs: &impl FileOperations) -> Result<PathBuf> {
        let saving_path = folder.join(format!("{}.lock", self.seq));

        fs.try_write_toml(&saving_path, &self)?;

        Ok(saving_path)
    }

    /// Gets the program by checking of it is a postprocess or a regular program.
    pub fn get_program(&self, run: &Run) -> Result<Program> {
        match &run.program {
            ProgramRef::Regular(name) => Ok(self.config.programs[name].clone()),
            ProgramRef::Postprocess(name) => {
                Ok(self.config.postprocess_programs.clone().unwrap()[name].clone())
            }
        }
    }

    pub fn get_input(&self, run: &Run) -> Result<Input> {
        match &run.input {
            InputRef::Regular(name) => Ok(self.config.inputs[name].clone()),
            InputRef::Postprocess(name) => Ok(self.postprocess_inputs[name].clone()),
        }
    }
}

/// Describes one chunk: a Slurm array of scheduled runs with common resource limits.
/// Chunks are created at runtime; a run is in one chunk iff it has been scheduled.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Chunk {
    /// The runs that belong to this chunk (by RunID)
    pub runs: Vec<usize>,

    /// The resource limits of this chunk.
    pub resource_limits: Option<ResourceLimits>,

    /// The slurm job id of this chunk.
    pub slurm_id: Option<String>,
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
