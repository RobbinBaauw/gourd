use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Formatter;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::{ResourceLimits};
use gourd_lib::constants::CMD_DOC_STYLE;
use gourd_lib::constants::CMD_HELP_STYLE;
use gourd_lib::constants::HELP_STYLE;
use gourd_lib::constants::NAME_STYLE;
use gourd_lib::constants::RERUN_LIST_PROMPT_CUTOFF;
use gourd_lib::{ctx};
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::file_system::FileOperations;
use inquire::Select;
use log::debug;
use log::trace;

use crate::cli::printing::query_update_resource_limits;
use crate::cli::printing::query_yes_no;
use crate::slurm::handler::get_limits;
use crate::status::{get_statuses, PostprocessCompletion};
use crate::status::ExperimentStatus;
use crate::status::FsState;

/// A map of run_id to the status of the run.
type RerunStatusMap = BTreeMap<usize, RerunStatus>;

/// The status of a single run
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
enum RerunStatus {
    /// A run by this id does not exist in the experiment
    DoesntExist,

    /// Run has not finished yet
    NotFinished,

    /// Finished successfully, with exit code 0
    FinishedExitZero,

    /// Finished successfully, and the assigned label has rerun_by_default set
    /// to false
    FinishedSuccessLabel(String),

    /// Failed because the assigned label has rerun_by_default set to true
    FailedErrorLabel(String),

    /// Failed with an exit code
    FailedExitCode(i32),
}

impl Display for RerunStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RerunStatus::DoesntExist => write!(f, "Does not exist"),
            RerunStatus::NotFinished => write!(f, "Not finished"),
            RerunStatus::FinishedExitZero => write!(f, "Finished with exit code 0"),
            RerunStatus::FinishedSuccessLabel(l) => write!(f, "Finished with label {}", l),
            RerunStatus::FailedErrorLabel(l) => write!(f, "Failed with label {}", l),
            RerunStatus::FailedExitCode(c) => write!(f, "Failed with exit code {}", c),
        }
    }
}

/// Get the list of runs to rerun from the rerun options.
pub fn get_runs_from_rerun_options(
    run_id: &Option<usize>,
    list: &Option<Vec<usize>>,
    experiment: &Experiment,
    file_system: &impl FileOperations,
) -> Result<Vec<usize>> {
    if run_id.is_some() && list.is_some() {
        return Err(anyhow!("You can only pick one of -l or -r for the rerun command")).with_context(ctx!(
            "Multiple runs can be reran using {CMD_DOC_STYLE} gourd rerun -l <list of run ids> {CMD_DOC_STYLE:#}, rerun just one using {CMD_DOC_STYLE} gourd rerun -r <run id> {CMD_DOC_STYLE:#}",;
            "try using {CMD_HELP_STYLE} gourd rerun -l {} {CMD_HELP_STYLE:#}", list.as_ref().unwrap().iter().chain([run_id.unwrap()].iter()).map(|x| x.to_string()).collect::<Vec<String>>().join(" ")
        ));
    }

    let statuses = get_statuses(experiment, file_system)?;
    Ok(if let Some(specific_run) = run_id {
        vec![check_single_run_failed(
            specific_run,
            experiment,
            &statuses,
        )?]
    } else if let Some(runs) = list {
        check_multiple_runs_failed(runs, experiment, &statuses)?
    } else {
        get_what_runs_to_rerun_from_experiment(experiment, statuses)?
    })
}

/// Query the user for the resource limits of the programs for a list of runs.
pub fn query_changing_limits_for_programs(
    runs: &[usize],
    experiment: &mut Experiment,
) -> Result<()> {
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

/// Find and update the resource limits for the program of a run.
fn update_program_resource_limits(
    run_id: usize,
    experiment: &mut Experiment,
    new_rss: ResourceLimits,
) {
    match &experiment.runs[run_id].program {
        FieldRef::Regular(name) => {
            experiment
                .config
                .programs
                .get_mut(name)
                .unwrap()
                .resource_limits = Some(new_rss);
        }
        FieldRef::Postprocess(name) => {
            experiment
                .config
                .postprocess_programs
                .as_mut()
                .unwrap()
                .get_mut(name)
                .unwrap()
                .resource_limits = Some(new_rss);
        }
    }
}

/// Check the status of a single run and ask the user what to rerun.
fn check_single_run_failed(
    specific_run: &usize,
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> Result<usize> {
    match status_of_single_run(specific_run, statuses, experiment)? {
        RerunStatus::DoesntExist => Err(anyhow!("Run {} does not exist", specific_run)).with_context(ctx!(
    "Experiment {} does not have run #{specific_run}", experiment.seq;
    "Experiment {} has runs 0-{}.
                            The runs that have finished are: {:?}.", experiment.seq, experiment.runs.len(), get_finished_runs_from_statuses(statuses)
    )),

        RerunStatus::NotFinished => {
            if !query_yes_no(&format!("Run {specific_run} has not completed yet, are you sure you want to schedule a re-run? You can cancel this run with {CMD_HELP_STYLE}gourd cancel {specific_run}{CMD_HELP_STYLE:#}"))? {
                // todo: confirm that this is how gourd cancel works
                Err(anyhow!("Rerun cancelled.."))
            } else {
                Ok(*specific_run)
            }
        }

        RerunStatus::FinishedExitZero => {
            if !query_yes_no(&format!("Run {specific_run} has finished successfully, are you sure you want to schedule a re-run?"))? {
                Err(anyhow!("Rerun cancelled."))
            } else {
                Ok(*specific_run)
            }
        }

        RerunStatus::FinishedSuccessLabel(l) => {
            if !query_yes_no(&format!("Run {specific_run} has finished successfully, with label {NAME_STYLE} {l} {NAME_STYLE:#}. Are you sure you want to schedule a re-run?"))? {
                Err(anyhow!("Rerun cancelled."))
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
    }
}

/// Check the status of a single run and return the status.
fn status_of_single_run(
    run_id: &usize,
    statuses: &ExperimentStatus,
    experiment: &Experiment,
) -> Result<RerunStatus> {
    // 1. check if run exists
    let runs_status = if let Some(status) = statuses.get(run_id) {
        status
    } else {
        return Ok(RerunStatus::DoesntExist);
    };

    // 2. check if run has completed
    match runs_status.fs_status.completion {
        FsState::Pending | FsState::Running => Ok(RerunStatus::NotFinished),

        FsState::Completed(m) => {
            // 3. check if the run failed
            if m.exit_code == 0 {
                // run the afterscript to get a label
                if let Some(PostprocessCompletion::Success(label_ref)) = &runs_status.fs_status.afterscript_completion {
                    if let Some(label) = label_ref {
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
                    Ok(RerunStatus::FinishedExitZero)
                }
            } else {
                Ok(RerunStatus::FailedExitCode(m.exit_code))
            }
        }
    }
}

/// Get the list of run_id(s) that have finished from an instance of
/// ExperimentStatus.
fn get_finished_runs_from_statuses(statuses: &ExperimentStatus) -> Vec<usize> {
    statuses
        .iter()
        .filter(|(_, s)| s.is_completed())
        .map(|(r, _)| *r)
        .collect()
}

/// Get the number of failed and successful runs from a table of statuses.
fn success_fails_from_table(table: BTreeMap<RerunStatus, usize>) -> (usize, usize) {
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
            .filter(|(r, _)| {
                !matches!(
                    r,
                    RerunStatus::FailedErrorLabel(_) | RerunStatus::FailedExitCode(_)
                )
            })
            .fold(0, |acc, (_, v)| acc + v),
    )
}

/// Check the statuses of a list of runs and ask the user what to rerun.
fn check_multiple_runs_failed(
    list: &[usize],
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> Result<Vec<usize>> {
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
        Ok(match Select::new(&format!("There are {HELP_STYLE}{failed}{HELP_STYLE:#} failed runs and {HELP_STYLE}{success}{HELP_STYLE:#} successful runs. What would you like to do?"), choices).prompt()? {
            "Rerun only failed" => Ok(list.iter().filter_map(|l| {
                if matches!(status_of_single_run(l, statuses, experiment), Ok(RerunStatus::FailedErrorLabel(_)) | Ok(RerunStatus::FailedExitCode(_)))
                { Some(*l) } else { None }
            }).collect()),
            "Rerun all" => Ok(list.to_vec()),
            "Cancel" => Err(anyhow!("Rerun cancelled.")),
            _ => unreachable!()
        }?)
    }
}

/// Aggregate the statuses of a list of runs.
///
/// This function will return a map of status to number of runs with this status
/// in the list.
fn aggregate_run_statuses(
    rerun_status_map: &RerunStatusMap,
) -> Result<BTreeMap<RerunStatus, usize>> {
    let mut x: BTreeMap<RerunStatus, usize> = BTreeMap::new();

    for status in rerun_status_map.values() {
        *x.entry(status.clone()).or_insert(0) += 1;
    }

    Ok(x)
}

/// Get the statuses of all runs in a list.
fn all_run_statuses(
    list: &[usize],
    experiment: &Experiment,
    statuses: ExperimentStatus,
) -> Result<RerunStatusMap> {
    let mut x = RerunStatusMap::new();
    for run in list {
        let _ = match status_of_single_run(run, &statuses, experiment)? {
            RerunStatus::DoesntExist => x.insert(*run, RerunStatus::DoesntExist),
            RerunStatus::NotFinished => x.insert(*run, RerunStatus::NotFinished),
            RerunStatus::FinishedExitZero => x.insert(*run, RerunStatus::FinishedExitZero),
            RerunStatus::FinishedSuccessLabel(_) => {
                x.insert(*run, RerunStatus::FinishedSuccessLabel(String::new()))
            }
            RerunStatus::FailedErrorLabel(_) => {
                x.insert(*run, RerunStatus::FailedErrorLabel(String::new()))
            }
            RerunStatus::FailedExitCode(_) => x.insert(*run, RerunStatus::FailedExitCode(0)),
        };
    }
    Ok(x)
}

/// Get the list of runs that have finished and ask the user what to rerun.
fn get_what_runs_to_rerun_from_experiment(
    experiment: &Experiment,
    statuses: ExperimentStatus,
) -> Result<Vec<usize>> {
    let finished_runs = get_finished_runs_from_statuses(&statuses);

    if finished_runs.is_empty() {
        return Err(anyhow!("No runs have finished yet")).with_context(ctx!(
            "Experiment {} has no runs that have finished yet", experiment.seq;
            "You can check the status of the runs with {HELP_STYLE}gourd status {}{HELP_STYLE:#}",
            experiment.seq,
        ));
    }

    let all_statuses = all_run_statuses(
        finished_runs.as_slice(),
        experiment,
        statuses.clone(),
    )?;
    let (failed, success) = success_fails_from_table(aggregate_run_statuses(&all_statuses)?);

    let choices: Vec<String> = vec![
        format!("Rerun only failed ({} runs)", failed),
        format!("Rerun all finished ({} runs)", success + failed),
    ];
    match Select::new("What would you like to do?", choices.clone())
        .prompt()?
        .as_str()
    {
        x if x == choices[1] => Ok::<Vec<usize>, anyhow::Error>(finished_runs),
        x if x == choices[0] => Ok::<Vec<usize>, anyhow::Error>(
            all_statuses
                .iter()
                .filter_map(|(r, s)| {
                    if matches!(
                        s,
                        RerunStatus::FailedErrorLabel(_) | RerunStatus::FailedExitCode(_)
                    ) {
                        Some(*r)
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        x => unreachable!("got: {:?}", x),
    }
}
