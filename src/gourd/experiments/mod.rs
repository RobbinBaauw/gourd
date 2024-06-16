use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use gourd_lib::bailc;
use gourd_lib::config::Config;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::FileOperations;

/// Extension trait for the shared `Experiment` struct.
pub trait ExperimentExt {
    /// Initialize a new experiment from a `config`.
    ///
    /// Creates a new experiment by matching all algorithms to all inputs.
    /// The experiment is created in the provided `env` and with `time` as the
    /// timestamp.
    fn from_config(
        conf: &Config,
        time: DateTime<Local>,
        env: Environment,
        fs: &impl FileOperations,
    ) -> Result<Self>
    where
        Self: Sized;

    /// Get the filename of the newest experiment.
    fn latest_id_from_folder(folder: &Path) -> Result<Option<usize>>;

    /// Provided a folder gets the most recent experiment.
    fn latest_experiment_from_folder(folder: &Path, fs: &impl FileOperations)
        -> Result<Experiment>;

    /// Provided a folder gets the specified experiment.
    fn experiment_from_folder(
        seq: usize,
        folder: &Path,
        fs: &impl FileOperations,
    ) -> Result<Experiment>;
}

impl ExperimentExt for Experiment {
    fn from_config(
        conf: &Config,
        time: DateTime<Local>,
        env: Environment,
        fs: &impl FileOperations,
    ) -> Result<Self> {
        let mut runs = Vec::new();

        let seq = Self::latest_id_from_folder(&conf.experiments_folder)
            .unwrap_or(Some(0))
            .unwrap_or(0)
            + 1;

        for prog_name in conf.programs.keys() {
            for input_name in conf.inputs.keys() {
                runs.push(Run {
                    program: FieldRef::Regular(prog_name.clone()),
                    input: FieldRef::Regular(input_name.clone()),
                    err_path: fs.truncate_and_canonicalize(
                        &conf
                            .output_path
                            .join(format!("{}/{}/{}/stderr", seq, prog_name, input_name)),
                    )?,
                    metrics_path: fs.truncate_and_canonicalize(
                        &conf
                            .metrics_path
                            .join(format!("{}/{}/{}/metrics", seq, prog_name, input_name)),
                    )?,
                    output_path: fs.truncate_and_canonicalize(
                        &conf
                            .output_path
                            .join(format!("{}/{}/{}/stdout", seq, prog_name, input_name)),
                    )?,
                    work_dir: fs.truncate_and_canonicalize_folder(
                        &conf
                            .output_path
                            .join(format!("{}/{}/{}/", seq, prog_name, input_name)),
                    )?,
                    afterscript_output_path: get_afterscript_file(
                        conf, &seq, prog_name, input_name, fs,
                    )?,
                    post_job_output_path: get_postprocess_folder(
                        conf, &seq, prog_name, input_name, fs,
                    )?,
                    slurm_id: None,
                });
            }
        }

        Ok(Self {
            runs,
            creation_time: time,
            seq,
            config: conf.clone(),
            chunks: vec![],
            resource_limits: conf.resource_limits,
            env,
            postprocess_inputs: BTreeMap::new(),
        })
    }

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
                .trim_end_matches(".lock")
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

    fn experiment_from_folder(
        seq: usize,
        folder: &Path,
        fs: &impl FileOperations,
    ) -> Result<Experiment> {
        fs.try_read_toml(&folder.join(format!("{}.lock", seq)))
    }

    fn latest_experiment_from_folder(
        folder: &Path,
        fs: &impl FileOperations,
    ) -> Result<Experiment> {
        if let Some(id) = Self::latest_id_from_folder(folder)? {
            Self::experiment_from_folder(id, folder, fs)
        } else {
            bailc!(
                "There are no experiments, try running some first", ;
                "", ;
                "",
            );
        }
    }
}

/// Constructs an afterscript path based on values in the config.
pub fn get_afterscript_file(
    config: &Config,
    seq: &usize,
    prog_name: &String,
    input_name: &String,
    fs: &impl FileOperations,
) -> Result<Option<PathBuf>> {
    let postprocessing = &config.programs[prog_name].afterscript;

    if postprocessing.is_some() {
        let path = &config
            .output_path
            .join(format!("{}/{}/{}/", seq, prog_name, input_name));

        let afterscript_output_path = fs.truncate_and_canonicalize_folder(path)?;

        Ok(Some(afterscript_output_path.join("afterscript")))
    } else {
        Ok(None)
    }
}

/// Constructs a postprocess job output path based on values in the config.
pub fn get_postprocess_folder(
    config: &Config,
    seq: &usize,
    prog_name: &String,
    input_name: &String,
    fs: &impl FileOperations,
) -> Result<Option<PathBuf>> {
    let postprocessing = &config.programs[prog_name].postprocess_job;

    if postprocessing.is_some() {
        let postprocess_folder = match config.postprocess_output_folder.clone() {
            Some(postpr_path) => Ok(postpr_path),
            None => Err(anyhow!(
                "No postprocess job output folder specified, but postprocess job exists"
            ))
            .with_context(ctx!("", ; "", )),
        }?;

        let path = postprocess_folder.join(format!("{}/{}/{}/", seq, prog_name, input_name));

        let postprocess_output_path = fs.truncate_and_canonicalize_folder(&path)?;

        Ok(Some(postprocess_output_path))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
