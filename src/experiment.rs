use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::afterscript::AfterscriptInfo;
use crate::error::ctx;
use crate::error::Ctx;
use crate::file_system::truncate_and_canonicalize;

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
    /// The unique name of the program to run.
    pub program_name: String,

    /// The unique name of the input to run with.
    pub input_name: String,

    /// The path to the stderr output.
    pub err_path: PathBuf,

    /// The path to the stdout output.
    pub output_path: PathBuf,

    /// The path to the metrics file.
    pub metrics_path: PathBuf,

    /// Slurm job id, if ran on slurm
    pub job_id: Option<usize>,

    /// The paths to afterscript output, optionally.
    pub afterscript_info: Option<AfterscriptInfo>,
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
    pub fn save(&self, folder: &Path) -> anyhow::Result<()> {
        let saving_path = truncate_and_canonicalize(&folder.join(format!("{}.toml", self.seq)))?;

        fs::write(&saving_path, toml::to_string(&self)?).with_context(ctx!(
            "Could not save the experiment at {saving_path:?}", ;
            "Enusre that you have suffcient permissions",
        ))?;

        Ok(())
    }
}
