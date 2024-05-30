use std::collections::BTreeMap;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::FileOperations;
use gourd_lib::measurement::Metrics;
use log::trace;

use super::FileSystemBasedStatus;
use super::PostprocessCompletion;
use super::PostprocessOutput;
use super::StatusProvider;
use crate::status::FsState;

/// Provide job status information based on the files system information.
#[derive(Debug, Clone, Copy)]
pub struct FileBasedProvider {}

impl<T> StatusProvider<T, FileSystemBasedStatus> for FileBasedProvider
where
    T: FileOperations,
{
    /// Function to gather status using file system
    fn get_statuses(
        fs: &mut T,
        experiment: &Experiment,
    ) -> Result<BTreeMap<usize, FileSystemBasedStatus>> {
        let mut statuses = BTreeMap::new();

        for (run_id, run) in experiment.runs.iter().enumerate() {
            trace!(
                "Reading status for run {} from {:?}",
                run_id,
                run.metrics_path
            );

            let metrics = match fs.try_read_toml::<Metrics>(&run.metrics_path) {
                Ok(x) => Some(x),
                Err(e) => {
                    trace!("Failed to read metrics: {:?}", e);
                    None
                }
            };

            let completion = match metrics {
                Some(inner) => match inner {
                    Metrics::Done(metrics) => FsState::Completed(metrics),
                    Metrics::NotCompleted => FsState::Running,
                },
                None => FsState::Pending,
            };

            let mut afterscript_completion = None;
            let mut postprocess_job_completion = None;

            if run.afterscript_output_path.is_some() {
                afterscript_completion =
                    Some(Self::get_afterscript_status(run).with_context(ctx!(
                        "Could not determine the afterscript status", ;
                        "",
                    ))?);
            }

            if run.post_job_output_path.is_some() {
                postprocess_job_completion =
                    Some(Self::get_postprocess_job_status(run).with_context(ctx!(
                        "Could not determine the postprocess job status", ;
                        "",
                    ))?);
            }

            let status = FileSystemBasedStatus {
                completion,
                afterscript_completion,
                postprocess_job_completion,
            };

            statuses.insert(run_id, status);
        }

        Ok(statuses)
    }
}

impl FileBasedProvider {
    /// Get the completion of an afterscript.
    pub fn get_afterscript_status(run: &Run) -> Result<PostprocessCompletion> {
        let mut postprocess_completion = PostprocessCompletion::Dormant;
        let dir = run
            .afterscript_output_path
            .clone()
            .ok_or(anyhow!("Could not get the afterscript information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        let is_empty = dir
            .read_dir()
            .with_context(ctx!(
                "Directory {:?} not okay", dir;
                "",
            ))?
            .next()
            .is_none();

        // TODO adding meaningful postprocess outputs
        if !is_empty {
            postprocess_completion = PostprocessCompletion::Success(PostprocessOutput {
                short_output: String::from("gg"),
                long_output: String::from("gg"),
            });
        }

        Ok(postprocess_completion)
    }

    /// Get the completion of a postprocess job.
    pub fn get_postprocess_job_status(run: &Run) -> Result<PostprocessCompletion> {
        let mut postprocess_completion = PostprocessCompletion::Dormant;
        let dir = run
            .post_job_output_path
            .clone()
            .ok_or(anyhow!("Could not get the postprocess job information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        let is_empty = dir
            .read_dir()
            .with_context(ctx!(
                "Directory {:?} not okay", dir;
                "",
            ))?
            .next()
            .is_none();

        if !is_empty {
            postprocess_completion = PostprocessCompletion::Success(PostprocessOutput {
                short_output: String::from("gg"),
                long_output: String::from("gg"),
            });
        }

        Ok(postprocess_completion)
    }
}
