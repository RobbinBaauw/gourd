use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::bailc;
use gourd_lib::constants::HELP_STYLE;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use inquire::Select;

use crate::rerun::checks::check_multiple_runs_failed;
use crate::rerun::status::aggregate_run_statuses;
use crate::rerun::status::all_run_statuses;
use crate::rerun::status::get_finished_runs_from_statuses;
use crate::rerun::status::success_fails_from_table;
use crate::status::get_statuses;
use crate::status::ExperimentStatus;

/// Get the list of runs to rerun from the rerun options.
pub fn get_runs_from_rerun_options(
    run_ids: &Option<Vec<usize>>,
    experiment: &Experiment,
    file_system: &mut impl FileOperations,
    script: bool,
) -> anyhow::Result<Vec<usize>> {
    let statuses = get_statuses(experiment, file_system)?;
    if let Some(runs) = run_ids {
        if script {
            Ok(runs.clone())
        } else {
            Ok(check_multiple_runs_failed(runs, experiment, &statuses)?)
        }
    } else {
        Ok(get_what_runs_to_rerun_from_experiment(
            experiment, statuses, script,
        )?)
    }
}

/// Get the list of runs that have finished and ask the user what to rerun.
pub(super) fn get_what_runs_to_rerun_from_experiment(
    experiment: &Experiment,
    statuses: ExperimentStatus,
    script: bool,
) -> anyhow::Result<Vec<usize>> {
    let finished_runs = get_finished_runs_from_statuses(&statuses);

    if finished_runs.is_empty() {
        bailc!("No runs have finished yet",;
            "Experiment {} has no runs that have finished yet", experiment.seq;
            "You can check the status of the runs with {HELP_STYLE}gourd status {}{HELP_STYLE:#}",
            experiment.seq,
        )
    }

    let all_statuses = all_run_statuses(finished_runs.as_slice(), experiment, statuses.clone())?;

    let failed_runs: Vec<usize> = all_statuses
        .iter()
        .filter_map(|(r, s)| if s.is_fail() { Some(*r) } else { None })
        .collect();

    if script {
        return Ok(failed_runs);
    }

    let (failed, success) = success_fails_from_table(aggregate_run_statuses(&all_statuses)?);

    let choices: Vec<String> = vec![
        format!("Rerun only failed ({} runs)", failed),
        format!("Rerun all finished ({} runs)", success + failed),
    ];
    match Select::new("What would you like to do?", choices.clone())
        .prompt()
        .with_context(ctx!("",;"",))?
        .as_str()
    {
        x if x == choices[1] => Ok::<Vec<usize>, anyhow::Error>(finished_runs),
        x if x == choices[0] => Ok::<Vec<usize>, anyhow::Error>(failed_runs),
        x => unreachable!("got: {:?}", x),
    }
}
