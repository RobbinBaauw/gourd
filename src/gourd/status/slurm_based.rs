use std::collections::BTreeMap;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use regex_lite::Regex;

use super::SlurmBasedStatus;
use super::SlurmState::*;
use super::StatusProvider;
use crate::slurm::SlurmInteractor;

/// Structure of Sacct output.
#[derive(Debug, Clone, PartialEq)]
pub struct SacctOutput {
    /// ID of the job
    pub job_id: String,

    /// Name of the job
    pub job_name: String,

    /// Current state of the job
    pub state: String,

    /// Exit code of slurm
    pub slurm_exit_code: isize,

    /// Exit code of the program
    pub program_exit_code: isize,
}

/// Provide job status information based on the files system information.
#[derive(Debug, Clone, Copy)]
pub struct SlurmBasedProvider {}

impl<T> StatusProvider<T, SlurmBasedStatus> for SlurmBasedProvider
where
    T: SlurmInteractor,
{
    #[cfg(not(tarpaulin_include))]
    fn get_statuses(
        connection: &mut T,
        experiment: &Experiment,
    ) -> Result<BTreeMap<usize, SlurmBasedStatus>> {
        use gourd_lib::bailc;

        let mut run_id_to_status: BTreeMap<usize, SlurmBasedStatus> = BTreeMap::new();
        let mut slurm_map = BTreeMap::new();

        for (run_id, run) in experiment.runs.iter().enumerate() {
            slurm_map.insert(run.slurm_id.clone(), run_id);
        }

        let statuses: Vec<SacctOutput> = flatten_job_id(
            connection.get_accounting_data(
                experiment
                    .chunks
                    .iter()
                    .filter_map(|x| x.slurm_id.clone())
                    .collect(),
            )?,
        );

        for job in statuses {
            // Mapping of all possible job state codes https://slurm.schedmd.com/sacct.html#SECTION_JOB-STATE-CODES
            let completion = match job.state.as_str() {
                "BOOT_FAIL" | "BF" => BootFail,

                "CANCELLED" | "CA" => Cancelled,

                "COMPLETED" | "CO" => Success,

                "DEADLINE" | "DL" => Deadline,

                "FAILED" | "F" => SlurmFail,

                "NODE_FAIL" | "NF" => NodeFail,

                "OUT_OF_MEMORY" | "OOM" => OutOfMemory,

                "PENDING" | "PD" => Pending,

                "PREEMPTED" | "PR" => Preempted,

                "RUNNING" | "R" => Running,

                "REQUEUED" | "RQ" => Pending, /* For now we treat it as pending, but it may need */
                // its own label for example State::Requeued
                "RESIZING" | "RS" => Running, // Needs a label, did not think of any suitable

                "REVOKED" | "RV" => Pending, // Also will probably need a label

                "SUSPENDED" | "S" => Suspended,

                "TIMEOUT" | "TO" => Timeout,

                // if not specified above we assume it failed
                _ => bailc!("Sacct returned unexpected output", ; "", ; "",),
            };

            run_id_to_status.insert(
                slurm_map[&Some(job.job_id)],
                SlurmBasedStatus {
                    completion,
                    exit_code_program: job.program_exit_code,
                    exit_code_slurm: job.slurm_exit_code,
                },
            );
        }

        Ok(run_id_to_status)
    }
}

/// This function takes [SacctOutput] and expands job ids.
///
/// Ids like: `1234_[22-34]` will get expanded into `1234_22, 1234_23, ...,
/// 1234_34`.
fn flatten_job_id(jobs: Vec<SacctOutput>) -> Vec<SacctOutput> {
    let mut result = vec![];

    for job in jobs {
        let range = Regex::new(r"([0-9]+_)\[([0-9]+)-([0-9]+)\]$").unwrap(); // Match job ids in form NUMBER_[NUMBER-NUMBER]
        let solo = Regex::new(r"([0-9]+_)\[?([0-9]+)\]?$").unwrap(); // Match job ids in form NUMBER_NUMBER

        if let Some(captures) = range.captures(&job.job_id) {
            let batch_id = &captures[1];
            let begin = captures[2].parse::<usize>().unwrap();
            let end = captures[3].parse::<usize>().unwrap();

            for i in begin..=end {
                result.push(SacctOutput {
                    job_id: format!("{}{}", batch_id, i),
                    job_name: job.job_name.clone(),
                    state: job.state.clone(),
                    slurm_exit_code: job.slurm_exit_code,
                    program_exit_code: job.program_exit_code,
                })
            }
        }

        if let Some(captures) = solo.captures(&job.job_id) {
            let batch_id = &captures[1];
            let run_id = captures[2].parse::<usize>().unwrap();

            result.push(SacctOutput {
                job_id: format!("{}{}", batch_id, run_id),
                job_name: job.job_name.clone(),
                state: job.state.clone(),
                slurm_exit_code: job.slurm_exit_code,
                program_exit_code: job.program_exit_code,
            })
        }
    }

    result
}

#[cfg(test)]
#[path = "tests/slurm_based.rs"]
mod tests;
