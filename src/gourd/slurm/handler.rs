use std::path::PathBuf;

use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::config::SlurmConfig;
use gourd_lib::constants::MAIL_TYPE_VALID_OPTIONS;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use log::debug;

use crate::slurm::checks::get_slurm_options_from_config;
use crate::slurm::chunk::Chunkable;
use crate::slurm::SlurmInteractor;

/// Functionality associated with running on slurm
#[derive(Debug, Clone, Copy)]
pub struct SlurmHandler<T>
where
    T: SlurmInteractor,
{
    pub(crate) internal: T,
}

impl<T> Default for SlurmHandler<T>
where
    T: SlurmInteractor + Default,
{
    fn default() -> Self {
        Self {
            internal: T::default(),
        }
    }
}

impl<T> SlurmHandler<T>
where
    T: SlurmInteractor,
{
    /// Run an experiment on delftblue.
    pub fn run_experiment(
        &self,
        config: &Config,
        experiment: &mut Experiment,
        exp_path: PathBuf,
        fs: impl FileOperations,
    ) -> Result<()> {
        let slurm_config = get_slurm_options_from_config(config)?;
        let runs = experiment.get_unscheduled_runs()?;

        let mut chunks_to_schedule = experiment.create_chunks(
            slurm_config.array_count_limit,
            // TODO: correctly handle ongoing array jobs causing a lower limit
            slurm_config.array_size_limit,
            runs.into_iter(),
        )?;

        experiment
            .slurm
            .as_mut()
            // TODO:
            .unwrap()
            .chunks
            .append(&mut chunks_to_schedule);

        experiment.save(&config.experiments_folder, &fs)?;

        for (chunk_id, chunk) in experiment
            .slurm
            .as_mut()
            // TODO:
            .unwrap()
            .chunks
            .iter_mut()
            .enumerate()
        {
            if chunk.scheduled {
                continue;
            }

            debug!("Scheduling chunk with {}", chunk.runs.len());

            self.internal.schedule_chunk(
                slurm_config,
                chunk,
                chunk_id,
                &config.wrapper,
                &fs.canonicalize(&exp_path)?,
            )?;
        }

        experiment.save(&config.experiments_folder, &fs)?;

        Ok(())
    }
}

/// Helper function to create string with optional args for slurm
pub fn parse_optional_args(slurm_config: &SlurmConfig) -> String {
    let mut result = "".to_string();

    if let Some(val) = &slurm_config.begin {
        result.push_str(&format!("#SBATCH --begin={}\n", val));
    }

    if let Some(val) = &slurm_config.mail_type {
        assert!(MAIL_TYPE_VALID_OPTIONS.contains(&val.as_str()));
        result.push_str(&format!("#SBATCH --mail-type={}\n", val))
    }

    if let Some(val) = &slurm_config.mail_user {
        result.push_str(&format!("#SBATCH --mail-user={}\n", val))
    }

    if let Some(args) = &slurm_config.additional_args {
        for arg in args.values() {
            result.push_str(&format!("#SBATCH --{}={}\n", arg.name, arg.value))
        }
    }

    result
}

#[cfg(test)]
#[path = "tests/handler.rs"]
mod tests;
