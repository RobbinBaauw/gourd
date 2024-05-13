use std::collections::BTreeMap;

use super::Completion;
use super::ExperimentStatus;
use super::FailureReason;
use super::Status;
use super::StatusProvider;
use crate::experiment::Experiment;
use crate::file_system::try_read_toml;
use crate::measurement::Metrics;

/// Provide job status information based on the files system information.
#[derive(Debug, Clone, Copy)]
pub struct FileBasedStatus {}

impl StatusProvider<()> for FileBasedStatus {
    fn get_statuses(_: (), experiment: &Experiment) -> anyhow::Result<ExperimentStatus> {
        let mut map = BTreeMap::new();

        for (run_id, run) in experiment.runs.iter().enumerate() {
            let metrics = match try_read_toml::<Metrics>(&run.metrics_path) {
                Ok(x) => Some(x),
                Err(_) => None,
            };

            let condition = match metrics {
                Some(inner) => match inner {
                    Metrics::Done(metrics) => match metrics.rusage.exit_status {
                        0 => Completion::Success,
                        x => Completion::Fail(FailureReason::ExitStatus(x)),
                    },
                    Metrics::Pending => Completion::Pending,
                },
                None => Completion::Dormant,
            };

            // For now there is *no* postprocessing step.
            map.insert(run_id, Some(Status::NoPostprocessing(condition)));
        }

        Ok(map)
    }
}
