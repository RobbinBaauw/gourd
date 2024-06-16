use std::collections::BTreeMap;

use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::bailc;
use gourd_lib::constants::CMD_STYLE;
use gourd_lib::constants::HELP_STYLE;
use gourd_lib::constants::NAME_STYLE;
use gourd_lib::constants::RERUN_LIST_PROMPT_CUTOFF;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use inquire::Select;
use log::debug;
use log::trace;

use crate::cli::printing::query_update_resource_limits;
use crate::cli::printing::query_yes_no;
use crate::rerun::status::aggregate_run_statuses;
use crate::rerun::status::all_run_statuses;
use crate::rerun::status::get_finished_runs_from_statuses;
use crate::rerun::status::status_of_single_run;
use crate::rerun::status::success_fails_from_table;
use crate::rerun::update_program_resource_limits;
use crate::rerun::RerunStatus;
use crate::slurm::handler::get_limits;
use crate::status::ExperimentStatus;

/// Check the status of a single run and ask the user what to rerun.
pub(super) fn check_single_run_failed(
    specific_run: &usize,
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> anyhow::Result<usize> {
    match status_of_single_run(specific_run, statuses, experiment)? {
        RerunStatus::DoesNotExist => bailc!("Run {} does not exist", specific_run;
            "Experiment {} does not have run #{specific_run}", experiment.seq;
            "Experiment {} has runs 0-{}.
            The runs that have finished are: {:?}.",
            experiment.seq, experiment.runs.len(), get_finished_runs_from_statuses(statuses)
        ),

        RerunStatus::NotFinished => {
            if !query_yes_no(&format!("Run {specific_run} has not completed yet, \
                are you sure you want to schedule a re-run? \
                You can cancel this run with \
                {CMD_STYLE}gourd cancel {specific_run}{CMD_STYLE:#}"
            ))? {
                // todo: confirm that this is how gourd cancel works
                bailc!("Rerun cancelled..")
            } else {
                Ok(*specific_run)
            }
        }

        RerunStatus::FinishedExitZero => {
            if !query_yes_no(&format!("Run {specific_run} has finished successfully, \
                are you sure you want to schedule a re-run?"
            ))? {
                bailc!("Rerun cancelled.")
            } else {
                Ok(*specific_run)
            }
        }

        RerunStatus::FinishedSuccessLabel(l) => {
            if !query_yes_no(&format!("Run {specific_run} has finished successfully, \
                with label {NAME_STYLE} {l} {NAME_STYLE:#}. Are you sure you want to schedule a re-run?"
            ))? {
                bailc!("Rerun cancelled.")
            } else {
                Ok(*specific_run)
            }
        }

        RerunStatus::FailedExitCode(c) => {
            debug!("Scheduling rerun for run #{} that failed with exit code {}", specific_run, c);
            Ok(*specific_run)
        }

        RerunStatus::FailedErrorLabel(l) => {
            debug!("Scheduling rerun for run #{} that failed with label {}", specific_run, l);
            Ok(*specific_run)
        }

        RerunStatus::RerunAs(r) => {
            if !query_yes_no(&format!("Run {specific_run} has been rerun as {r}, \
                are you sure you want to schedule a re-run?"
            ))? {
                bailc!("Rerun cancelled.")
            } else {
                Ok(*specific_run)
            }
        }
    }
}

/// Check the statuses of a list of runs and ask the user what to rerun.
pub(super) fn check_multiple_runs_failed(
    list: &[usize],
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> anyhow::Result<Vec<usize>> {
    if list.len() < RERUN_LIST_PROMPT_CUTOFF {
        Ok(list
            .iter()
            .filter_map(|r| check_single_run_failed(r, experiment, statuses).ok())
            .collect())
    } else {
        let all_statuses = all_run_statuses(list, experiment, statuses.clone())?;
        let table = aggregate_run_statuses(&all_statuses)?;

        let (failed, success) = success_fails_from_table(table);

        let choices = vec!["Rerun only failed", "Rerun all", "Cancel"];
        match Select::new(
            &format!(
                "There are {HELP_STYLE}{failed}{HELP_STYLE:#} failed runs \
                and {HELP_STYLE}{success}{HELP_STYLE:#} successful runs. What would you like to do?"
            ),
            choices,
        )
        .prompt()?
        {
            "Rerun only failed" => Ok(list
                .iter()
                .filter_map(|l| {
                    if status_of_single_run(l, statuses, experiment).is_ok_and(|s| s.is_fail()) {
                        Some(*l)
                    } else {
                        None
                    }
                })
                .collect()),
            "Rerun all" => Ok(list.to_vec()),
            "Cancel" => Err(anyhow!("Rerun cancelled.")),
            _ => unreachable!(),
        }
    }
}

/// Query the user for the resource limits of the programs for a list of runs.
pub fn query_changing_limits_for_programs(
    runs: &[usize],
    experiment: &mut Experiment,
) -> anyhow::Result<()> {
    loop {
        let mut run_programs = BTreeMap::new();

        for run in runs {
            run_programs.insert(*run, experiment.get_program(&experiment.runs[*run])?);
        }

        let mut choices = run_programs
            .keys()
            .map(|p| format!("{NAME_STYLE}{}{NAME_STYLE:#}", experiment.runs[*p].program))
            .collect::<Vec<String>>();
        choices.dedup();
        choices.push("Done".to_string());

        match Select::new("Update resource limits?", choices)
            .prompt()?
            .as_str()
        {
            "Done" => break,
            x => {
                let mut changed = vec![];
                for (run_id, program) in run_programs.iter() {
                    if x == format!(
                        "{NAME_STYLE}{}{NAME_STYLE:#}",
                        experiment.runs[*run_id].program
                    ) {
                        if changed.contains(&program) {
                            continue;
                        }
                        let new_rss = query_update_resource_limits(&get_limits(
                            &experiment.runs[*run_id],
                            experiment,
                        )?)?;

                        debug!("Updating resource limits for run {}", run_id);
                        trace!("Old resource limits: {:?}", program.resource_limits);
                        trace!("New resource limits: {:?}", new_rss);

                        update_program_resource_limits(*run_id, experiment, new_rss);
                        changed.push(program);
                    }
                }
            }
        }
    }
    Ok(())
}
