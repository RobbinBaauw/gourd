use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::error::ctx;
use crate::error::Ctx;
use crate::file_system::truncate_and_canonicalize;
use crate::file_system::try_read_toml;

/// The run location of the experiment.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    /// Local execution.
    Local,

    /// Slurm execution.
    Slurm,
}

/// Describes a matching between an algorithm and an input.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
}

/// Describes one experiment.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Experiment {
    /// The pairings of program-input for this experiment.
    pub runs: Vec<Run>,

    /// The run location of this experiment.
    pub env: Environment,

    /// The time of the experiment.
    pub time: DateTime<Local>,

    /// The ID of this experiment.
    pub seq: usize,
}

impl Experiment {
    /// Initialize a new experiment from a `config`.
    ///
    /// Creates a new experiment by matching all algorithms to all inputs.
    /// The experiment is created in the provided `env` and with `time` as the timestamp.
    pub fn from_config(conf: &Config, env: Environment, time: DateTime<Local>) -> Result<Self> {
        let mut runs = Vec::new();

        let seq = Self::latest_id_from_folder(&conf.experiments_folder)
            .unwrap_or(Some(0))
            .unwrap_or(0)
            + 1;

        for prog_name in conf.programs.keys() {
            for input_name in conf.inputs.keys() {
                runs.push(Run {
                    program_name: prog_name.clone(),
                    input_name: input_name.clone(),
                    err_path: truncate_and_canonicalize(
                        &conf
                            .output_path
                            .join(format!("{}/algo_{}/{}_error", seq, prog_name, input_name)),
                    )?,
                    metrics_path: truncate_and_canonicalize(
                        &conf
                            .metrics_path
                            .join(format!("{}/algo_{}/{}_metrics", seq, prog_name, input_name)),
                    )?,
                    output_path: truncate_and_canonicalize(
                        &conf
                            .output_path
                            .join(format!("{}/algo_{}/{}_output", seq, prog_name, input_name)),
                    )?,
                });
            }
        }

        Ok(Self {
            runs,
            env,
            time,
            seq,
        })
    }

    /// Get the filename of the newest experiment.
    pub fn latest_id_from_folder(folder: &PathBuf) -> Result<Option<usize>> {
        let mut highest = None;

        for file in fs::read_dir(folder).with_context(ctx!(
          "Could not access the experiments directory {folder:?}", ;
          "Run some experiments first or ensure that you have suffient permissions to read it",
        ))? {
            let actual = match file {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let seq_of_file: usize = actual
                .file_name()
                .to_str()
                .ok_or(anyhow!("Invalid filename in experiment directory"))?
                .trim_end_matches(".toml")
                .parse()
                .with_context(ctx!(
                  "Invalid name of experiment file {actual:?}", ;
                  "Do not manually modify files in the experiment directory",
                ))?;

            if highest.is_none() {
                highest = Some(seq_of_file);
            } else if let Some(num) = highest {
                if seq_of_file > num {
                    highest = Some(seq_of_file);
                }
            }
        }

        Ok(highest)
    }

    /// Provided a folder gets the most recent experiment.
    pub fn latest_experiment_from_folder(folder: &PathBuf) -> Result<Experiment> {
        if let Some(id) = Self::latest_id_from_folder(folder)? {
            try_read_toml(&folder.join(format!("{}.toml", id)))
        } else {
            Err(anyhow!("There are no experiments, try running some first").context(""))
        }
    }

    /// Save the experiment to a file with its timestamp.
    pub fn save(&self, folder: &Path) -> Result<()> {
        let saving_path = truncate_and_canonicalize(&folder.join(format!("{}.toml", self.seq)))?;

        fs::write(&saving_path, toml::to_string(&self)?).with_context(ctx!(
            "Could not save the experiment at {saving_path:?}", ;
            "Enusre that you have suffcient permissions",
        ))?;

        Ok(())
    }
}
