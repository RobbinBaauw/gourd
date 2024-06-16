use std::collections::BTreeMap;

use gourd_lib::experiment::Experiment;

use crate::rerun::RerunStatus;
use crate::rerun::RerunStatusMap;
use crate::status::ExperimentStatus;
use crate::status::FsState;
use crate::status::PostprocessCompletion;

/// Check the status of a single run and return the status.
pub(super) fn status_of_single_run(
    run_id: &usize,
    statuses: &ExperimentStatus,
    experiment: &Experiment,
) -> anyhow::Result<RerunStatus> {
    // 1. check if run exists
    let runs_status = if let Some(status) = statuses.get(run_id) {
        status
    } else {
        return Ok(RerunStatus::DoesNotExist);
    };

    if let Some(r) = experiment.runs[*run_id].rerun {
        return Ok(RerunStatus::RerunAs(r));
    }

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

/// Get the number of failed and successful runs from a table of statuses.
pub(super) fn success_fails_from_table(table: BTreeMap<RerunStatus, usize>) -> (usize, usize) {
    (
        table
            .iter()
            .filter(|(r, _)| {
                matches!(
                    r,
                    RerunStatus::FailedErrorLabel(_) | RerunStatus::FailedExitCode(_)
                )
            })
            .fold(0, |acc, (_, v)| acc + v),
        table
            .iter()
            .filter(|(r, _)| r.is_success())
            .fold(0, |acc, (_, v)| acc + v),
    )
}

/// Aggregate the statuses of a list of runs.
///
/// This function will return a map of status to number of runs with this status
/// in the list.
pub(super) fn aggregate_run_statuses(
    rerun_status_map: &RerunStatusMap,
) -> anyhow::Result<BTreeMap<RerunStatus, usize>> {
    let mut x: BTreeMap<RerunStatus, usize> = BTreeMap::new();

    for status in rerun_status_map.values() {
        *x.entry(status.clone()).or_insert(0) += 1;
    }

    Ok(x)
}

/// Get the statuses of all runs in a list.
pub(super) fn all_run_statuses(
    list: &[usize],
    experiment: &Experiment,
    statuses: ExperimentStatus,
) -> anyhow::Result<RerunStatusMap> {
    let mut x = RerunStatusMap::new();
    for run in list {
        let _ = match status_of_single_run(run, &statuses, experiment)? {
            RerunStatus::DoesNotExist => x.insert(*run, RerunStatus::DoesNotExist),
            RerunStatus::NotFinished => x.insert(*run, RerunStatus::NotFinished),
            RerunStatus::FinishedExitZero => x.insert(*run, RerunStatus::FinishedExitZero),
            RerunStatus::FinishedSuccessLabel(_) => {
                x.insert(*run, RerunStatus::FinishedSuccessLabel(String::new()))
            }
            RerunStatus::FailedErrorLabel(_) => {
                x.insert(*run, RerunStatus::FailedErrorLabel(String::new()))
            }
            RerunStatus::FailedExitCode(_) => x.insert(*run, RerunStatus::FailedExitCode(0)),
            RerunStatus::RerunAs(_) => x.insert(*run, RerunStatus::RerunAs(0)),
        };
    }
    Ok(x)
}

/// Get the list of run_id(s) that have finished from an instance of
/// ExperimentStatus.
pub(super) fn get_finished_runs_from_statuses(statuses: &ExperimentStatus) -> Vec<usize> {
    statuses
        .iter()
        .filter(|(_, s)| s.is_completed())
        .map(|(r, _)| *r)
        .collect()
}
