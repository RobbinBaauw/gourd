use std::collections::BTreeMap;

use gourd_lib::experiment::Experiment;
use regex_lite::Regex;

use super::FailureReason;
use super::SlurmBasedStatus;
use super::SlurmKillReason;
use super::State;
use crate::slurm::interactor::SlurmCli;
use crate::slurm::SlurmInteractor;
use crate::slurm::SlurmStatus;

/// Function to gather status using slurm job accounting data
pub fn get_slurm_statuses(
    experiment: &Experiment,
) -> anyhow::Result<BTreeMap<usize, SlurmBasedStatus>> {
    let mut map: BTreeMap<usize, SlurmBasedStatus> = BTreeMap::new();

    let statuses: Vec<SlurmStatus> = flatten_job_id(
        SlurmCli::default().get_accounting_data(experiment.batch_ids.as_ref().unwrap())?,
    );

    for job in statuses {
        // Mapping of all possible job state codes https://slurm.schedmd.com/sacct.html#SECTION_JOB-STATE-CODES
        let completion = match job.state.as_str() {
            "BOOT_FAIL" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::BootFail)),
            "BF" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::BootFail)),

            "CANCELLED" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Cancelled)),
            "CA" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Cancelled)),

            "COMPLETED" => State::Completed,
            "CO" => State::Completed,

            "DEADLINE" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Deadline)),
            "DL" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Deadline)),

            "FAILED" => State::Fail(FailureReason::Failed),
            "F" => State::Fail(FailureReason::Failed), // F in the chat

            "NODE_FAIL" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::NodeFail)),
            "NF" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::NodeFail)),

            "OUT_OF_MEMORY" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::OutOfMemory)),
            "OOM" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::OutOfMemory)),

            "PENDING" => State::Pending,
            "PD" => State::Pending,

            "PREEMPTED" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Preempted)),
            "PR" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Preempted)),

            "RUNNING" => State::Running,
            "R" => State::Running,

            "REQUEUED" => State::Pending, // For now we treat it as pending, but it may need its own label for example State::Requeued
            "RQ" => State::Pending,

            "RESIZING" => State::Running, // Needs a label, did not think of any suitable
            "RS" => State::Running,

            "REVOKED" => State::Pending, // Also will probably need a label
            "RV" => State::Pending,

            "SUSPENDED" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Suspended)),
            "S" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Suspended)),

            "TIMEOUT" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Timeout)),
            "TO" => State::Fail(FailureReason::SlurmKill(SlurmKillReason::Timeout)),

            // if not specified above we assume it failed
            _ => State::Fail(FailureReason::SlurmKill(SlurmKillReason::SlurmFail)),
        };

        if let Some(run_id) = experiment.id_map.as_ref().unwrap().get(&job.job_id) {
            map.insert(
                *run_id,
                SlurmBasedStatus {
                    completion,
                    exit_code_program: job.program_exit_code,
                    exit_code_slurm: job.slurm_exit_code,
                },
            );
        }
    }

    Ok(map)
}

fn flatten_job_id(jobs: Vec<SlurmStatus>) -> Vec<SlurmStatus> {
    let mut result = vec![];
    for job in jobs {
        let range = Regex::new(r"([0-9]+_)\[([0-9]+)-([0-9]+)\]$").unwrap(); // Match job ids in form NUMBER_[NUMBER-NUMBER]
        let solo = Regex::new(r"[0-9]+_[0-9]+$").unwrap(); // Match job ids in form NUMBER_NUMBER
        if let Some(captures) = range.captures(&job.job_id) {
            let batch_id = &captures[0];
            let begin = captures[1].parse::<usize>().unwrap();
            let end = captures[2].parse::<usize>().unwrap();
            for i in begin..=end {
                result.push(SlurmStatus {
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
