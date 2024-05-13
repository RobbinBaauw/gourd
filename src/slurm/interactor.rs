use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use tempdir::TempDir;

use crate::cli::printing::format_table;
use crate::config::Config;
use crate::constants::SLURM_VERSIONS;
use crate::error::ctx;
use crate::error::Ctx;
use crate::experiment::Experiment;
use crate::slurm::info::get_partitions;
use crate::slurm::info::get_version;
use crate::slurm::SlurmInteractor;

/// An implementation of the SlurmInteractor trait for interacting with SLURM via the CLI.
#[derive(Debug)]
pub struct SlurmCLI {
    /// Supported versions by this instance of the CLI interactor
    pub versions: Vec<[u64; 2]>,
}

/// These are using functions specific to CLI version 21.8.x
///
/// we don't know if other versions are supported.
impl SlurmInteractor for SlurmCLI {
    /// Check if the SLURM version is supported.
    fn check_version(&self) -> anyhow::Result<()> {
        match get_version() {
            Ok(version) => {
                if !self.versions.contains(&version) {
                    Err(anyhow!("SLURM Version assertion failed")).with_context(
                        ctx!("Unsupported SLURM version: {:?}",
                          version.iter().map(u64::to_string).collect::<Vec<String>>().join(".");
                          "Supported versions are: {:?}",
                          SLURM_VERSIONS.map(|x| x.iter().map(u64::to_string)
                            .collect::<Vec<String>>().join(".")).to_vec()
                        ),
                    )
                } else {
                    Ok(())
                }
            }

            Err(e) => Err(anyhow!("SLURM versioning failed")).with_context(ctx!(
              "Failed to get SLURM version: {:?}", e;
              "Please make sure that SLURM is installed and available in the PATH",
            )),
        }
    }

    /// Check if the provided partition is valid.
    fn check_partition(&self, partition: &str) -> anyhow::Result<()> {
        let partitions = get_partitions()?;
        if partitions.iter().flatten().any(|x| x == partition) {
            Ok(())
        } else {
            Err(anyhow!("Invalid partition provided")).with_context(ctx!(
              "Partition `{:?}` is not available on this cluster. ", partition;
              "Present partitions are:\n{:?}", format_table(partitions)
            ))
        }
    }

    /// Run an experiment on a SLURM cluster.
    ///
    /// input: a (parsed) configuration and the experiments to run
    fn run_job(&self, config: &Config, _experiment: &mut Experiment) -> anyhow::Result<()> {
        let slurm_config = config.slurm_config.as_ref()
            .ok_or_else(|| anyhow!("No SLURM configuration found"))
            .with_context(ctx!(
              "Tried to execute on Slurm but the configuration field for the Slurm options in gourd.toml was empty", ;
              "Make sure that your gourd.toml includes the required fields under [slurm]",
            ))?;

        self.check_version()?;
        self.check_partition(&slurm_config.partition)?;

        let temp = TempDir::new("gourd-slurm")?;
        let batch_script = temp.path().join("batch.sh");

        let contents = format!(
            "
#!/bin/bash
#SBATCH --job-name={}
#SBATCH --partition={}
#SBATCH --time={}

./{} --id=$SLURM_ARRAY_TASK_ID
",
            slurm_config.experiment_name,
            slurm_config.partition,
            slurm_config.time_limit,
            config.wrapper,
        );

        std::fs::write(&batch_script, contents)?;

        let _ = Command::new("sbatch")
            .arg(batch_script)
            .output()
            .with_context(ctx!(
              "Failed to submit batch job to SLURM", ;
              "Ensure that you have permissions to submit jobs to the cluster",
            ))?;

        Ok(())
    }
}
