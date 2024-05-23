use std::collections::BTreeMap;

use gourd_lib::experiment::Experiment;

use super::Completion;
use super::ExperimentStatus;
use super::FailureReason;
use super::SlurmKillReason;
use super::Status;
use super::StatusProvider;
use crate::slurm::interactor::SlurmCLI;
use crate::slurm::SlurmInteractor;
use crate::slurm::SlurmStatus;

/// Provide job status information based on the slurm accounting data.
#[derive(Debug, Clone, Copy)]
pub struct SlurmBasedStatus {}

impl StatusProvider<()> for SlurmBasedStatus {
    fn get_statuses(_connection: (), experiment: &Experiment) -> anyhow::Result<ExperimentStatus> {
        let mut map: BTreeMap<String, Option<Status>> = BTreeMap::new();

        let job_ids: Vec<String> = experiment
            .slurm
            .as_ref()
            .unwrap()
            .chunks
            .iter()
            .map(|chunk| chunk.batch_id.to_string())
            .collect::<Vec<String>>();

        let statuses: Vec<SlurmStatus> = SlurmCLI::default().get_accounting_data(job_ids)?;

        for job in statuses {
            let condition = match job.state.as_str() {
                "TIMEOUT" => Completion::Fail(FailureReason::SlurmKill(SlurmKillReason::Timeout)),
                "FAILED" => Completion::Fail(FailureReason::ExitStatus(job.program_exit_code)),
                "COMPLETED" => Completion::Success,
                "PENDING" => Completion::Pending,
                "RUNNING" => Completion::Running,
                _ => Completion::Fail(FailureReason::ExitStatus(job.program_exit_code)),
            };

            map.insert(
                job.job_id,
                Some(Status {
                    completion: condition,
                    postprocess_job_completion: None,
                    afterscript_completion: None,
                }),
            );
        }

        Ok(map)
    }
}
