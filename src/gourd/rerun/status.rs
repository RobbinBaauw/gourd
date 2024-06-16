use gourd_lib::experiment::Experiment;

use crate::rerun::RerunStatus;
use crate::status::ExperimentStatus;
use crate::status::FsState;
use crate::status::PostprocessCompletion;

/// Check the status of a single run and return the status.
pub(super) fn status_of_single_run(
    run_id: &usize,
    statuses: &ExperimentStatus,
    experiment: &Experiment,
) -> anyhow::Result<RerunStatus> {
    let runs_status = &statuses[run_id];

    // 2. check if run has completed
    match runs_status.fs_status.completion {
        FsState::Pending | FsState::Running => Ok(RerunStatus::NotFinished),

        FsState::Completed(m) => {
            // 3. check if the run failed
            if m.exit_code == 0 {
                // run the afterscript to get a label
                if let Some(PostprocessCompletion::Success(Some(label))) =
                    &runs_status.fs_status.afterscript_completion
                {
                    let lb = experiment.get_label(label)?;
                    if lb.rerun_by_default {
                        Ok(RerunStatus::FailedErrorLabel(label.clone()))
                    } else {
                        Ok(RerunStatus::FinishedSuccessLabel(label.clone()))
                    }
                } else {
                    Ok(RerunStatus::FinishedExitZero)
                }
            } else {
                Ok(RerunStatus::FailedExitCode(m.exit_code))
            }
        }
    }
}
