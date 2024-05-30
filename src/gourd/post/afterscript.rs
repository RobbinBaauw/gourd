use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitStatus;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;

use crate::resources::run_script;
use crate::status::ExperimentStatus;
use crate::status::FsState;
use crate::status::PostprocessCompletion;

/// Runs the afterscript on jobs that are completed and do not yet have an afterscript output.
pub fn run_afterscript(
    statuses: &ExperimentStatus,
    experiment: &Experiment,
    file_system: &impl FileOperations,
) -> Result<BTreeMap<usize, String>> {
    let runs = filter_runs_for_afterscript(statuses);

    let mut labels = BTreeMap::new();
    for run_id in runs {
        let run = &experiment.runs[*run_id];
        let after_out_path = &run.afterscript_output_path;
        let res_path = run.output_path.clone();

        if after_out_path.is_none() {
            continue;
        }

        let after_output = after_out_path
            .clone()
            .ok_or(anyhow!("Could not get the afterscript information"))
            .with_context(ctx!(
                "Could not get the afterscript information", ;
                "",
            ))?;

        let program = &experiment.config.programs[&run.program];

        let afterscript = program
            .afterscript
            .clone()
            .ok_or(anyhow!("Could not get the afterscript information"))
            .with_context(ctx!(
                "Could not get the afterscript information", ;
                "",
            ))?;
        let exit_status = run_afterscript_for_run(&afterscript, &res_path, &after_output)?;

        if !exit_status.success() {
            return Err(anyhow!(
                "Afterscript failed with exit code {}",
                exit_status
                    .code()
                    .ok_or(anyhow!("Status does not exist"))
                    .with_context(ctx!(
                        "Could not get the exit code of the execution", ;
                        "",
                    ))?
            ));
        }

        add_label_to_run(*run_id, &mut labels, experiment, after_output, file_system)?;
    }
    Ok(labels)
}

/// Find the completed jobs where afterscript did not run yet.
pub fn filter_runs_for_afterscript(runs: &ExperimentStatus) -> Vec<&usize> {
    let mut filtered = vec![];

    for (run_id, status) in runs {
        if let (&FsState::Completed(_), &Some(PostprocessCompletion::Dormant)) = (
            &status.fs_status.completion,
            &status.fs_status.afterscript_completion,
        ) {
            filtered.push(run_id);
        }
    }

    filtered
}

/// Runs the afterscript on given jobs.
pub fn run_afterscript_for_run(
    after_path: &PathBuf,
    res_path: &PathBuf,
    out_path: &PathBuf,
) -> Result<ExitStatus> {
    fs::metadata(after_path).with_context(ctx!(
        "Could not find the afterscript at {:?}", &after_path;
        "Check that the afterscript exists and the path to it is correct",
    ))?;

    fs::metadata(res_path).with_context(ctx!(
        "Could not find the job result at {:?}", &res_path;
        "Check that the job result already exists",
    ))?;

    let args = vec![
        after_path.as_os_str().to_str().with_context(ctx!(
            "Could not turn {after_path:?} into a string", ;
            "Check that the afterscript path is valid",
        ))?,
        res_path.as_os_str().to_str().with_context(ctx!(
            "Could not turn {res_path:?} into a string", ;
            "Check that the job result path is valid",
        ))?,
        out_path.as_os_str().to_str().with_context(ctx!(
            "Could not turn {out_path:?} into a string", ;
            "Check that the output path is valid",
        ))?,
    ];

    let exit_status = run_script(args).with_context(ctx!(
        "Could not run the afterscript at {after_path:?} with job results at {res_path:?}", ;
        "Check that the afterscript is correct and job results exist at {:?}", res_path,
    ))?;

    Ok(exit_status)
}

fn add_label_to_run(
    run_id: usize,
    labels: &mut BTreeMap<usize, String>,
    experiment: &Experiment,
    after_output: PathBuf,
    file_system: &impl FileOperations,
) -> Result<()> {
    if let Some(label_map) = &experiment.config.labels {
        let text = file_system.read_utf8(&after_output)?;

        let mut keys = label_map.keys().collect::<Vec<&String>>();
        keys.sort_by(|a, b| label_map[*b].priority.cmp(&label_map[*a].priority));

        for l in keys {
            let label = &label_map[l];
            if label.regex.is_match(&text) {
                labels.insert(run_id, l.clone());
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
