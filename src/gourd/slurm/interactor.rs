use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::SlurmConfig;
use gourd_lib::constants::SLURM_VERSIONS;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Chunk;
use gourd_lib::experiment::Experiment;
use log::debug;
use tempdir::TempDir;

use super::handler::parse_optional_args;
use super::SlurmStatus;
use crate::slurm::SlurmInteractor;

/// Creates a Slurm duration string.
///
/// Converts a standard `std::time::Duration` to a Slurm duration in one of
/// the following formats: {ss, mm:ss, hh:mm:ss, d-hh:mm:ss}
pub fn format_slurm_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let secs_rem = secs % 60;

    if secs == secs_rem {
        return format!("{:0>2}", secs);
    }

    let mins = secs / 60;
    let mins_rem = mins % 60;

    if mins == mins_rem {
        return format!("{:0>2}:{:0>2}", mins, secs_rem);
    }

    let hours = mins / 60;
    let hours_rem = hours % 24;

    if hours == hours_rem {
        return format!("{:0>2}:{:0>2}:{:0>2}", hours, mins_rem, secs_rem);
    }

    let days = hours / 24;

    format!(
        "{}-{:0>2}:{:0>2}:{:0>2}",
        days, hours_rem, mins_rem, secs_rem
    )
}

/// An implementation of the SlurmInteractor trait for interacting with SLURM via the CLI.
#[derive(Debug)]
pub struct SlurmCli {
    /// Supported versions by this instance of the CLI interactor
    pub versions: Vec<[u64; 2]>,
}

impl Default for SlurmCli {
    fn default() -> Self {
        Self {
            versions: SLURM_VERSIONS.to_vec(),
        }
    }
}

/// These are using functions specific to CLI version 21.8.x
///
/// we don't know if other versions are supported.
impl SlurmInteractor for SlurmCli {
    /// Get the SLURM version from CLI output.
    fn get_version(&self) -> Result<[u64; 2]> {
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
    fn get_partitions(&self) -> Result<Vec<Vec<String>>> {
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
    fn schedule_chunk(
        &self,
        slurm_config: &SlurmConfig,
        chunk: &mut Chunk,
        chunk_id: usize,
        experiment: &mut Experiment,
        exp_path: &Path,
    ) -> Result<()> {
        let resource_limits = chunk
            .resource_limits
            .clone()
            .ok_or(anyhow!("Could not get slurm resource limits"))
            .with_context(
                ctx!("",;"Specyfing resource limits in the config is required for slurm runs",),
            )?;

        let temp = TempDir::new("gourd-slurm")?;
        let batch_script = temp.path().join("batch.sh");

        let optional_args = parse_optional_args(slurm_config);

        let contents = format!(
            "#!/bin/bash
#SBATCH --job-name=\"{}\"
#SBATCH --array=\"{}-{}\"
#SBATCH --ntasks=1
#SBATCH --partition=\"{}\"
#SBATCH --time=\"{}\"
#SBATCH --cpus-per-task=\"{}\"
#SBATCH --mem-per-cpu=\"{}\"
#SBATCH --account=\"{}\"
{}

{} {} {} $SLURM_ARRAY_TASK_ID
",
            slurm_config.experiment_name,
            0,
            &chunk.runs.len() - 1,
            slurm_config.partition,
            format_slurm_duration(resource_limits.time_limit),
            resource_limits.cpus,
            resource_limits.mem_per_cpu,
            slurm_config.account,
            optional_args,
            experiment.config.wrapper,
            chunk_id,
            exp_path.display()
        );

        std::fs::write(&batch_script, &contents)?;

        debug!("Sbatch file: {}", contents);

        let proc = Command::new("sbatch")
            .arg("--parsable")
            .arg(batch_script)
            .output()
            .with_context(ctx!(
              "Failed to submit batch job to SLURM", ;
              "Ensure that you have permissions to submit jobs to the cluster",
            ))?;

        if !proc.status.success() {
            return Err(anyhow!("Sbatch failed to run")).with_context(ctx!(
                "Sbatch printed: {}", String::from_utf8(proc.stderr).unwrap();
                "Please ensure that you are running on slurm",
            ));
        }

        chunk.scheduled = true;

        let batch_id = String::from_utf8(proc.stdout).with_context(ctx!(
          "Could get sbatch output", ; "",
        ))?;

        for (job_subid, run_id) in chunk.runs.iter().enumerate() {
            experiment.runs[*run_id].slurm_id = Some(format!("{}_{}", batch_id.trim(), job_subid))
        }

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

    /// Get accounting data of user's jobs
    fn get_accounting_data(&self, job_id: Vec<String>) -> anyhow::Result<Vec<SlurmStatus>> {
        let sacct = Command::new("sacct")
            .arg("-p")
            .arg("--format=jobid,jobname,state,exitcode")
            .arg(format!("--jobs={}", job_id.join(",")))
            .output()?;
        let mut result = Vec::new();
        for job in String::from_utf8_lossy(&sacct.stdout)
            .trim()
            .split('\n')
            .skip(1)
        {
            let fields = job.split('|').collect::<Vec<&str>>();
            let exit_codes = fields[3].split(':').collect::<Vec<&str>>();
            result.push(SlurmStatus {
                job_id: fields[0].to_string(),
                job_name: fields[1].to_string(),
                state: fields[2].to_string(),
                slurm_exit_code: exit_codes[0].parse().unwrap_or(0),
                program_exit_code: exit_codes[1].parse().unwrap_or(0),
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests/interactor.rs"]
mod tests;
