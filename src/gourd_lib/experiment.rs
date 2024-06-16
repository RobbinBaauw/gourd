use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::config::Input;
use crate::config::Label;
use crate::config::Program;
use crate::config::ResourceLimits;
use crate::file_system::FileOperations;

/// Differentiating between regular and postprocess fields.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldRef {
    /// A reference to a filed stored in a config [BTreeMap].
    Regular(String),
    /// A reference to a postprocess field.
    Postprocess(String),
}

impl Display for FieldRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            FieldRef::Regular(name) => name,
            FieldRef::Postprocess(name) => name,
        };

        write!(f, "{}", name)
    }
}

/// Describes a matching between an algorithm and an input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// The unique name of the program to run.
    pub program: FieldRef,

    /// The unique name of the input to run with.
    pub input: FieldRef,

    /// The path to the stderr output.
    pub err_path: PathBuf,

    /// The path to the stdout output.
    pub output_path: PathBuf,

    /// The path to the metrics file.
    pub metrics_path: PathBuf,

    /// The working directory of this run.
    pub work_dir: PathBuf,

    /// Slurm job id, if ran on slurm
    pub slurm_id: Option<String>,

    /// The path to afterscript output, optionally.
    pub afterscript_output_path: Option<PathBuf>,

    /// The path to postprocess job output, optionally.
    pub postprocessor: Option<usize>,

    /// If this job has been rerun, a reference to the new one.
    pub rerun: Option<usize>,
}

/// An enum to distinguish the run context.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Environment {
    /// Local execution.
    Local,

    /// Slurm execution.
    Slurm,
}

/// Describes one experiment.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
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

    /// Environment of the experiment
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

    /// Gets the program by checking if it is a postprocess or a regular
    /// program.
    pub fn get_program(&self, run: &Run) -> Result<Program> {
        match &run.program {
            FieldRef::Regular(name) => Ok(self.config.programs[name].clone()),
            FieldRef::Postprocess(name) => {
                Ok(self.config.postprocess_programs.clone().unwrap()[name].clone())
            }
        }
    }

    /// Gets the input by checking if it is a postprocess or a regular program.
    pub fn get_input(&self, run: &Run) -> Result<Input> {
        match &run.input {
            FieldRef::Regular(name) => Ok(self.config.inputs[name].clone()),
            FieldRef::Postprocess(name) => Ok(self.postprocess_inputs[name].clone()),
        }
    }

    /// Get the label by name.
    pub fn get_label(&self, name: &String) -> Result<Label> {
        self.config
            .labels
            .as_ref()
            .and_then(|labels| labels.get(name).cloned())
            .ok_or(anyhow!("Label not found"))
    }
}

/// Describes one chunk: a Slurm array of scheduled runs with common resource
/// limits. Chunks are created at runtime; a run is in one chunk iff it has been
/// scheduled.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Chunk {
    /// The runs that belong to this chunk (by RunID)
    pub runs: Vec<usize>,

    /// The resource limits of this chunk.
    pub resource_limits: Option<ResourceLimits>,

    /// Whether this chunk has been run or not.
    pub status: ChunkRunStatus,
}

/// The run status of a chunk.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ChunkRunStatus {
    /// The job hasn't started yet
    Pending,

    /// The job has started running locally.
    RanLocally,

    /// The run is scheduled on Slurm with a slurm id
    Scheduled(String),
}

impl Chunk {
    /// Get the slurm id of the chunk if it is scheduled.
    ///
    /// Returns None if it is running locally or not ran yet.
    pub fn get_slurm_id(&self) -> Option<String> {
        match self.status {
            ChunkRunStatus::Scheduled(ref id) => Some(id.clone()),
            _ => None,
        }
    }
}
