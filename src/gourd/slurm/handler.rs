use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::config::ResourceLimits;
use gourd_lib::config::SlurmConfig;
use gourd_lib::constants::MAIL_TYPE_VALID_OPTIONS;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::experiment::Run;
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
    /// The way of interaction with slurm. (May be cli or library based).
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
    ///
    /// # Returns
    /// The amount of chunks that have been scheduled.
    pub fn run_experiment(
        &self,
        config: &Config,
        experiment: &mut Experiment,
        exp_path: PathBuf,
        fs: impl FileOperations,
    ) -> Result<usize> {
        let slurm_config = get_slurm_options_from_config(config)?;
        let runs = experiment.get_unscheduled_runs()?;

        let mut chunks_to_schedule = experiment.create_chunks_with_resource_limits(
            slurm_config.array_count_limit,
            // TODO: correctly handle ongoing array jobs causing a lower limit
            slurm_config.array_size_limit,
            get_limits,
            runs.into_iter(),
        )?;

        experiment.chunks.append(&mut chunks_to_schedule);
        let mut chunks_to_iterate = experiment.chunks.clone();

        experiment.save(&config.experiments_folder, &fs)?;

        let mut counter = 0;
        for (chunk_id, chunk) in chunks_to_iterate.iter_mut().enumerate() {
            if chunk.slurm_id.is_some() {
                continue;
            }

            debug!(
                "Scheduling chunk {} with {} runs",
                chunk_id,
                chunk.runs.len()
            );

            self.internal.schedule_chunk(
                slurm_config,
                chunk,
                chunk_id,
                experiment,
                &fs.canonicalize(&exp_path)?,
            )?;
            counter += 1;
        }

        experiment.chunks = chunks_to_iterate;
        experiment.save(&config.experiments_folder, &fs)?;

        Ok(counter)
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

/// Get resource limits depending on if it is a regular program or
/// postprocessing program
pub fn get_limits(run: &Run, experiment: &Experiment) -> Result<ResourceLimits> {
    let program = experiment.get_program(run)?;

    // If there are program-specific limits, those overwrite defaults
    if program.resource_limits.is_some() {
        return program
            .resource_limits
            .ok_or(anyhow!(
                "Could not get the program-specific resource limits"
            ))
            .with_context(ctx!(
                "Could not get the resource limits of the program", ;
                "Please ensure that the program resource limits are specified for the experiment",
            ));
    }

    match &run.program {
        FieldRef::Regular(_) => {
            // Defaults of regular programs
            experiment
            .config
            .resource_limits
            .ok_or(anyhow!("Could not get the default resource limits"))
            .with_context(ctx!(
                "Could not get the default resource limits", ;
                "Please ensure that the default resource limits are specified for the experiment",
            ))
        }
        FieldRef::Postprocess(_) => {
            // Defaults of postprocess programs
            experiment
            .config
            .postprocess_resource_limits
            .ok_or(anyhow!("Could not get the postprocessing resource limits"))
            .with_context(ctx!(
                "Could not get the postprocessing resource limits of the program", ;
                "Please ensure that the postprocessing resource limits are specified for the experiment",
            ))
        }
    }
}

#[cfg(test)]
#[path = "tests/handler.rs"]
mod tests;
