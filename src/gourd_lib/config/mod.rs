use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use self::maps::InputMap;
use self::maps::ProgramMap;
use crate::config::regex::Regex;
use crate::constants::AFTERSCRIPT_DEFAULT;
use crate::constants::AFTERSCRIPT_OUTPUT_DEFAULT;
use crate::constants::EMPTY_ARGS;
use crate::constants::POSTPROCESS_JOB_CPUS;
use crate::constants::POSTPROCESS_JOB_DEFAULT;
use crate::constants::POSTPROCESS_JOB_MEM;
use crate::constants::POSTPROCESS_JOB_OUTPUT_DEFAULT;
use crate::constants::POSTPROCESS_JOB_TIME;
use crate::constants::PRIMARY_STYLE;
use crate::constants::RERUN_LABEL_BY_DEFAULT;
use crate::constants::WRAPPER_DEFAULT;
use crate::error::ctx;
use crate::error::Ctx;
use crate::file_system::FileOperations;

pub mod maps;
pub mod regex;

/// A pair of a path to a binary and cli arguments.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct Program {
    /// The path to the executable.
    pub binary: PathBuf,

    /// The cli arguments for the executable.
    #[serde(default = "EMPTY_ARGS")]
    pub arguments: Vec<String>,

    /// The path to the afterscript, if there is one.
    #[serde(default = "AFTERSCRIPT_DEFAULT")]
    pub afterscript: Option<PathBuf>,

    /// The path to the postprocess job, if there is one.
    #[serde(default = "POSTPROCESS_JOB_DEFAULT")]
    pub postprocess_job: Option<PathBuf>,
}

/// A pair of a path to an input and additional cli arguments.
///
/// # Examples
///
/// ```toml
/// [programs.test_program]
/// binary = "test"
/// arguments = [ "a", "b" ]
///
/// [inputs.test_input]
/// arguments = [ "c" ]
/// ```
///
/// Will run `test a b c`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct Input {
    /// The path to the input.
    ///
    /// If not specified, nothing is provided on the program's input.
    pub input: Option<PathBuf>,

    /// The additional cli arguments for the executable.
    ///
    /// # Default
    /// By default these will be empty.
    #[serde(default = "EMPTY_ARGS")]
    pub arguments: Vec<String>,
}

/// A label that can be assigned to a job based on the afterscript output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct Label {
    /// The regex to run over the afterscript output. If there's a match, this label is assigned.
    pub regex: Regex,

    /// The priority of the label. Higher numbers mean higher priority, and if label is
    /// present it will override lower priority labels, even if they are also present.
    pub priority: u64,

    /// Whether using rerun failed will rerun this job- ie is this label a "failure"
    #[serde(default = "RERUN_LABEL_BY_DEFAULT")]
    pub rerun_by_default: bool,
}

/// A config struct used throughout the `gourd` application.
//
// changing the config struct? see notes in ./tests/config.rs
// 1. is the change necessary?
// 2. will it break user workflows?
// 3. update the tests
// 4. update the user documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    //
    // Basic settings.
    //
    /// The path to a folder where the experiment output will be stored.
    pub output_path: PathBuf,

    /// The path to a folder where the metrics output will be stored.
    pub metrics_path: PathBuf,

    /// The path to a folder where the experiments will be stored.
    pub experiments_folder: PathBuf,

    /// The list of tested algorithms.
    pub programs: ProgramMap,

    /// The list of inputs for each of them.
    ///
    /// The name of an input cannot contain `glob|`.
    pub inputs: InputMap,

    /// If running on a SLURM cluster, the job configurations
    pub slurm: Option<SlurmConfig>,

    /// If running on a SLURM cluster, the initial global resource limits
    pub resource_limits: Option<ResourceLimits>,

    //
    // Advanced settings.
    //
    /// The command to execute to get to the wrapper.
    #[serde(default = "WRAPPER_DEFAULT")]
    pub wrapper: String,

    /// The path to a folder where the afterscript outputs will be stored.
    #[serde(default = "AFTERSCRIPT_OUTPUT_DEFAULT")]
    pub afterscript_output_folder: Option<PathBuf>,

    /// The path to a folder where the afterscript outputs will be stored.
    #[serde(default = "POSTPROCESS_JOB_OUTPUT_DEFAULT")]
    pub postprocess_job_output_folder: Option<PathBuf>,

    /// Allow custom labels to be assigned based on the afterscript output.
    ///
    /// syntax is:
    /// ```toml
    /// [labels.<label_name>]
    /// regex = "<regex>" # the regex where if it matches then this label is assigned
    /// rerun_by_default = true # whether using rerun failed will rerun this job- ie is this label a "failure"
    /// ```
    #[serde(alias = "label")]
    pub labels: Option<BTreeMap<String, Label>>,
}

/// The config options when running through Slurm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlurmConfig {
    /// The name of the experiment. This is used (parametrically) as the job name in SLURM, and for the output directory.
    pub experiment_name: String, // not sure if we need this user-provided or generated by gourd, but it's one of the options and i couldn't tell if it's mandatory or not

    /// Which node partition to use. On DelftBlue, the options are:
    /// - "compute"
    /// - "compute-p2"
    /// - "gpu"
    /// - "gpu-a100"
    /// - "memory"
    /// - "trans"
    /// - "visual"
    pub partition: String, // technically this would be an enum, but it's different per cluster so i don't know if we should hardcode delftblue's options

    /// The maximum number of arrays to schedule at once.
    pub array_count_limit: usize,

    /// The maximum number of jobs to schedule in a Slurm array.
    pub array_size_limit: usize,

    /// Where slurm should put the stdout and stderr of the job.
    pub out: Option<PathBuf>,

    /// Account to charge for this job
    pub account: String,

    /// Delay the run of the job
    pub begin: Option<String>,

    /// Option to set notifications for user by email when a certain event types occur.
    pub mail_type: Option<String>,

    /// User to be notified by the email (When not specified it's the user that scheduled the job)
    pub mail_user: Option<String>,

    /// Custom slurm arguments
    pub additional_args: Option<BTreeMap<String, SBatchArg>>,

    /// Maximum time allowed _for each_ postprocess job.
    #[serde(default = "POSTPROCESS_JOB_TIME")]
    pub post_job_time_limit: Option<String>, // this is a string because slurm jobs can be longer than 24h, which is the largest value in toml time. format needs to be either "days-hours:minutes:seconds" or "minutes"

    /// CPUs to use per postprocess job.
    #[serde(default = "POSTPROCESS_JOB_CPUS")]
    pub post_job_cpus: Option<usize>,

    /// Memory in MB to allocate per CPU per postprocess job.
    #[serde(default = "POSTPROCESS_JOB_MEM")]
    pub post_job_mem_per_cpu: Option<usize>,
}

/// The structure for providing custom slurm arguments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SBatchArg {
    /// Name of the sbatch argument
    pub name: String,

    /// Value of the sbatch argument
    pub value: String,
}

/// The resource limits, a Slurm configuration parameter that can be changed during an experiment.
/// Contains the CPU, time, and memory bounds per run.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum time allowed _for each_ job.
    pub time_limit: Duration,

    /// CPUs to use per job
    pub cpus: usize,

    /// Memory in MB to allocate per CPU per job
    pub mem_per_cpu: usize,
}

// An implementation that provides a default value of `Config`,
// which allows for the eventual addition of optional config items.
impl Default for Config {
    fn default() -> Self {
        Config {
            output_path: PathBuf::from("run-output"),
            metrics_path: PathBuf::from("run-metrics"),
            experiments_folder: PathBuf::from("experiments"),
            wrapper: WRAPPER_DEFAULT(),
            programs: ProgramMap::default(),
            inputs: InputMap::default(),
            slurm: None,
            resource_limits: None,
            afterscript_output_folder: AFTERSCRIPT_OUTPUT_DEFAULT(),
            postprocess_job_output_folder: POSTPROCESS_JOB_OUTPUT_DEFAULT(),
            labels: Some(BTreeMap::new()),
        }
    }
}

impl Config {
    /// Load a `Config` struct instance from a TOML file at the provided path.
    /// Returns a valid `Config` or an explanatory `GourdError::ConfigLoadError`.
    pub fn from_file<F: FileOperations>(path: &Path, fs: &F) -> Result<Config> {
        toml::from_str(&fs.read_utf8(path)?).with_context(ctx!(
          "Could not parse {path:?}", ;
          "More help and examples can be found with {PRIMARY_STYLE}man gourd.toml{PRIMARY_STYLE:#}",
        ))
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
