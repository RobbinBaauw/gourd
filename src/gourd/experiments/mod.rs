use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use gourd_lib::afterscript::AfterscriptInfo;
use gourd_lib::config::Config;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::truncate_and_canonicalize;
use gourd_lib::file_system::try_read_toml;

/// Extension trait for the shared `Experiment` struct.
pub trait ExperimentExt {
    /// constructor from a `config`.
    fn from_config(conf: &Config, env: Environment, time: DateTime<Local>) -> anyhow::Result<Self>
    where
        Self: Sized;
    /// find the most recent experiment in a folder
    fn latest_id_from_folder(folder: &Path) -> Result<Option<usize>>;
    /// when running gourd status without an argument it should fetch the most recent experiment
    fn latest_experiment_from_folder(folder: &Path) -> Result<Experiment>;
}

impl ExperimentExt for Experiment {
    /// Initialize a new experiment from a `config`.
    ///
    /// Creates a new experiment by matching all algorithms to all inputs.
    /// The experiment is created in the provided `env` and with `time` as the timestamp.
    fn from_config(conf: &Config, env: Environment, time: DateTime<Local>) -> Result<Self> {
        let mut runs = Vec::new();

        let seq = Self::latest_id_from_folder(&conf.experiments_folder)
            .unwrap_or(Some(0))
            .unwrap_or(0)
            + 1;

        for prog_name in conf.programs.keys() {
            for input_name in conf.inputs.keys() {
                runs.push(Run {
                    program: conf.programs[prog_name].clone(),
                    input: conf.inputs[input_name].clone(),
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
                    afterscript_info: get_afterscript_info(conf, &seq, prog_name, input_name)?,
                    job_id: None,
                });
            }
        }

        Ok(Self {
            runs,
            env,
            creation_time: time,
            seq,
        })
    }

    /// Get the filename of the newest experiment.
    fn latest_id_from_folder(folder: &Path) -> Result<Option<usize>> {
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
    fn latest_experiment_from_folder(folder: &Path) -> Result<Experiment> {
        if let Some(id) = Self::latest_id_from_folder(folder)? {
            try_read_toml(&folder.join(format!("{}.toml", id)))
        } else {
            Err(anyhow!("There are no experiments, try running some first").context(""))
        }
    }
}

/// Constructs an afterscript info struct based on values in the config.
pub fn get_afterscript_info(
    config: &Config,
    seq: &usize,
    prog_name: &String,
    input_name: &String,
) -> Result<Option<AfterscriptInfo>> {
    let afterscript = &config.programs[prog_name].afterscript;

    if let Some(afs) = afterscript {
        let afterscript_path = truncate_and_canonicalize(&afs.src.clone())?;

        let afterscript_output_path = truncate_and_canonicalize(&afs.out.clone().join(format!(
            "{}/algo_{}/{}_afterscript",
            seq, prog_name, input_name
        )))?;

        Ok(Some(AfterscriptInfo {
            afterscript_path,
            afterscript_output_path,
        }))
    } else {
        Ok(None)
    }
}
