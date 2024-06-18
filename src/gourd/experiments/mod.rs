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
                runs.push(generate_new_run(
                    runs.len(),
                    FieldRef::Regular(prog_name.clone()),
                    FieldRef::Regular(input_name.clone()),
                    seq,
                    conf,
                    fs,
                )?);
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

    fn experiment_from_folder(
        seq: usize,
        folder: &Path,
        fs: &impl FileOperations,
    ) -> Result<Experiment> {
        fs.try_read_toml(&folder.join(format!("{}.lock", seq)))
    }
}

/// This function will generate a new run.
///
/// This should be used by all code paths adding runs to the experiment.
pub fn generate_new_run(
    run_id: usize,
    program: FieldRef,
    input: FieldRef,
    seq: usize,
    conf: &Config,
    fs: &impl FileOperations,
) -> Result<Run> {
    Ok(Run {
        program: program.clone(),
        input,
        err_path: fs.truncate_and_canonicalize(
            &conf
                .output_path
                .join(format!("{}/{}/{}/stderr", seq, program, run_id)),
        )?,
        metrics_path: fs.truncate_and_canonicalize(
            &conf
                .metrics_path
                .join(format!("{}/{}/{}/metrics", seq, program, run_id)),
        )?,
        output_path: fs.truncate_and_canonicalize(
            &conf
                .output_path
                .join(format!("{}/{}/{}/stdout", seq, program, run_id)),
        )?,
        work_dir: fs.truncate_and_canonicalize_folder(
            &conf
                .output_path
                .join(format!("{}/{}/{}/", seq, program, run_id)),
        )?,
        afterscript_output_path: match program {
            FieldRef::Regular(prog_name) => {
                get_afterscript_file(conf, &seq, &prog_name, run_id, fs)?
            }
            FieldRef::Postprocess(_) => None,
        },
        slurm_id: None,
        rerun: None,
        postprocessor: None,
    })
}

/// Constructs an afterscript path based on values in the config.
pub fn get_afterscript_file(
    config: &Config,
    seq: &usize,
    prog_name: &String,
    run_id: usize,
    fs: &impl FileOperations,
) -> Result<Option<PathBuf>> {
    let postprocessing = &config.programs[prog_name].afterscript;

    if postprocessing.is_some() {
        let path = &config
            .output_path
            .join(format!("{}/{}/{}/", seq, prog_name, run_id));

        let afterscript_output_path = fs.truncate_and_canonicalize_folder(path)?;

        Ok(Some(afterscript_output_path.join("afterscript")))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
