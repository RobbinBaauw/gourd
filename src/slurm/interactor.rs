use std::ops::Range;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use tempdir::TempDir;

use crate::constants::SLURM_VERSIONS;
use crate::error::ctx;
use crate::error::Ctx;
use crate::slurm::SlurmConfig;
use crate::slurm::SlurmInteractor;

/// An implementation of the SlurmInteractor trait for interacting with SLURM via the CLI.
#[derive(Debug)]
pub struct SlurmCLI {
    /// Supported versions by this instance of the CLI interactor
    pub versions: Vec<[u64; 2]>,
}

impl Default for SlurmCLI {
    fn default() -> Self {
        Self {
            versions: SLURM_VERSIONS.to_vec(),
        }
    }
}

/// These are using functions specific to CLI version 21.8.x
///
/// we don't know if other versions are supported.
impl SlurmInteractor for SlurmCLI {
    /// Get the SLURM version from CLI output.
    fn get_version(&self) -> anyhow::Result<[u64; 2]> {
        let s_info_out = Command::new("sinfo").arg("--version").output()?;
        let version = String::from_utf8_lossy(&s_info_out.stdout)
            .to_string()
            .split_whitespace()
            .collect::<Vec<&str>>()[1]
            .split(|c: char| !c.is_numeric())
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.parse::<u64>().unwrap())
            .collect::<Vec<u64>>();
        let mut buf = [0; 2];
        buf[0] = *version.first().ok_or(anyhow!("Invalid version received"))?;
        buf[1] = *version.get(1).ok_or(anyhow!("Invalid version received"))?;
        Ok(buf)
    }

    /// Get available partitions on the cluster.
    /// returns a (space and newline delimited) table of partition name and availability.
    fn get_partitions(&self) -> anyhow::Result<Vec<Vec<String>>> {
        let s_info_out = Command::new("sinfo").arg("-o").arg("%P %a").output()?;
        let partitions = String::from_utf8_lossy(&s_info_out.stdout)
            .split('\n')
            .map(|x| x.to_string())
            .map(|y| {
                y.split_whitespace()
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|z| z.to_string())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        Ok(partitions)
    }

    /// Schedule a new job array on the cluster.
    fn schedule_array(
        &self,
        range: Range<usize>,
        slurm_config: &SlurmConfig,
        wrapper_path: &str,
    ) -> anyhow::Result<()> {
        let temp = TempDir::new("gourd-slurm")?;
        let batch_script = temp.path().join("batch.sh");

        let contents = format!(
            "#!/bin/bash
#SBATCH --job-name={}
#SBATCH --array={}-{}
#SBATCH --ntasks=1
#SBATCH --partition={}
#SBATCH --time={}
#SBATCH --cpus-per-task={}
#SBATCH --mem-per-cpu={}

{} --id=$SLURM_ARRAY_TASK_ID
",
            slurm_config.experiment_name,
            range.start,
            range.end,
            slurm_config.partition,
            slurm_config.time_limit,
            slurm_config.cpus,
            slurm_config.mem_per_cpu,
            wrapper_path,
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

    /// Get the supported SLURM versions for this CLI interactor.
    fn is_version_supported(&self, v: [u64; 2]) -> bool {
        self.versions.contains(&v)
    }

    /// Get the supported SLURM versions for this CLI interactor.
    fn get_supported_versions(&self) -> String {
        self.versions
            .iter()
            .map(|x| format!("{}.{}", x[0], x[1]))
            .collect::<Vec<String>>()
            .join(", ")
    }
}
