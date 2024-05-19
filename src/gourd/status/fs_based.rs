use std::collections::BTreeMap;

use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::measurement::Metrics;

use super::Completion;
use super::ExperimentStatus;
use super::FailureReason;
use super::PostprocessCompletion;
use super::PostprocessOutput;
use super::Status;
use super::StatusProvider;

/// Provide job status information based on the files system information.
#[derive(Debug, Clone, Copy)]
pub struct FileBasedStatus {}

impl<T> StatusProvider<T> for FileBasedStatus
where
    T: FileOperations,
{
    fn get_statuses(fs: T, experiment: &Experiment) -> anyhow::Result<ExperimentStatus> {
        let mut statuses = BTreeMap::new();

        for (run_id, run) in experiment.runs.iter().enumerate() {
            let metrics = match fs.try_read_toml::<Metrics>(&run.metrics_path) {
                Ok(x) => Some(x),
                Err(_) => None,
            };

            let completion_condition = match metrics {
                Some(inner) => match inner {
                    Metrics::Done(metrics) => match metrics.exit_code {
                        0 => Completion::Success,
                        x => Completion::Fail(FailureReason::ExitStatus(x)),
                    },
                    Metrics::Pending => Completion::Pending,
                },
                None => Completion::Dormant,
            };

            if run.afterscript_info.is_some() {
                let afterpath = run
                    .afterscript_info
                    .clone()
                    .ok_or(anyhow!("Could not get the afterscript information"))
                    .with_context(ctx!(
                        "Could not get the afterscript information", ;
                        "",
                    ))?;

                let mut postprocess_completion = PostprocessCompletion::Pending;

                let is_empty = afterpath
                    .afterscript_output_path
                    .read_dir()?
                    .next()
                    .is_none();

                if !is_empty {
                    postprocess_completion = PostprocessCompletion::Success(PostprocessOutput {
                        short_output: String::from("gg"),
                        long_output: String::from("gg"),
                    });
                }

                statuses.insert(
                    run_id,
                    Some(Status::AfterScript(
                        completion_condition,
                        postprocess_completion,
                    )),
                );
            } else {
                statuses.insert(run_id, Some(Status::NoPostprocessing(completion_condition)));
            }
        }

        Ok(statuses)
    }
}
