use std::collections::BTreeMap;

use anyhow::Result;
use gourd_lib::experiment::Experiment;
use regex_lite::Regex;

use super::FailureReason::*;
use super::RunState;
use super::SlurmBasedStatus;
use super::SlurmKillReason::*;
use super::StatusProvider;
use crate::slurm::interactor::SlurmCli;
use crate::slurm::SacctOutput;
use crate::slurm::SlurmInteractor;

/// Provide job status information based on the files system information.
#[derive(Debug, Clone, Copy)]
pub struct SlurmBasedProvider {}

impl StatusProvider<SlurmCli, SlurmBasedStatus> for SlurmBasedProvider {
    /// Function to gather status using slurm job accounting data
    fn get_statuses(
        connection: &mut SlurmCli,
        experiment: &Experiment,
    ) -> Result<BTreeMap<usize, SlurmBasedStatus>> {
        let mut map: BTreeMap<usize, SlurmBasedStatus> = BTreeMap::new();
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
                "BOOT_FAIL" => RunState::Fail(SlurmKill(BootFail)),
                "BF" => RunState::Fail(SlurmKill(BootFail)),

                "CANCELLED" => RunState::Fail(SlurmKill(Cancelled)),
                "CA" => RunState::Fail(SlurmKill(Cancelled)),

                "COMPLETED" => RunState::Completed,
                "CO" => RunState::Completed,

                "DEADLINE" => RunState::Fail(SlurmKill(Deadline)),
                "DL" => RunState::Fail(SlurmKill(Deadline)),

                "FAILED" => RunState::Fail(SlurmKill(SlurmFail)),
                "F" => RunState::Fail(SlurmKill(SlurmFail)),

                "NODE_FAIL" => RunState::Fail(SlurmKill(NodeFail)),
                "NF" => RunState::Fail(SlurmKill(NodeFail)),

                "OUT_OF_MEMORY" => RunState::Fail(SlurmKill(OutOfMemory)),
                "OOM" => RunState::Fail(SlurmKill(OutOfMemory)),

                "PENDING" => RunState::Pending,
                "PD" => RunState::Pending,

                "PREEMPTED" => RunState::Fail(SlurmKill(Preempted)),
                "PR" => RunState::Fail(SlurmKill(Preempted)),

                "RUNNING" => RunState::Running,
                "R" => RunState::Running,

                "REQUEUED" => RunState::Pending, // For now we treat it as pending, but it may need its own label for example State::Requeued
                "RQ" => RunState::Pending,

                "RESIZING" => RunState::Running, // Needs a label, did not think of any suitable
                "RS" => RunState::Running,

                "REVOKED" => RunState::Pending, // Also will probably need a label
                "RV" => RunState::Pending,

                "SUSPENDED" => RunState::Fail(SlurmKill(Suspended)),
                "S" => RunState::Fail(SlurmKill(Suspended)),

                "TIMEOUT" => RunState::Fail(SlurmKill(Timeout)),
                "TO" => RunState::Fail(SlurmKill(Timeout)),

                // if not specified above we assume it failed
                _ => RunState::Fail(SlurmKill(SlurmFail)),
            };

            map.insert(
                slurm_map[&Some(job.job_id)],
                SlurmBasedStatus {
                    completion,
                    exit_code_program: job.program_exit_code,
                    exit_code_slurm: job.slurm_exit_code,
                },
            );
        }

        Ok(map)
    }
}

fn flatten_job_id(jobs: Vec<SacctOutput>) -> Vec<SacctOutput> {
    let mut result = vec![];

    for job in jobs {
        let range = Regex::new(r"([0-9]+_)\[([0-9]+)-([0-9]+)\]$").unwrap(); // Match job ids in form NUMBER_[NUMBER-NUMBER]
        let solo = Regex::new(r"[0-9]+_[0-9]+$").unwrap(); // Match job ids in form NUMBER_NUMBER

        if let Some(captures) = range.captures(&job.job_id) {
            let batch_id = &captures[0];
            let begin = captures[1].parse::<usize>().unwrap();
            let end = captures[2].parse::<usize>().unwrap();

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

        if solo.is_match(&job.job_id) {
            result.push(job);
        }
    }

    result
}
