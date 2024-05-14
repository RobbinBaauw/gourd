use crate::config::Config;
use crate::experiment::Experiment;
use crate::slurm::checks::get_slurm_options_from_config;
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
    /// TODO: scheduling algorithm to support multiple memory and time configurations.
    pub fn run_experiment(&self, config: &Config, experiment: &Experiment) -> anyhow::Result<()> {
        let slurm_config = get_slurm_options_from_config(config)?;

        self.internal
            .schedule_array(0..experiment.runs.len(), slurm_config, &config.wrapper)?;
        Ok(())
    }
}
