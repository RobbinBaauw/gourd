use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitStatus;

use anyhow::anyhow;
use anyhow::Context;
use gourd_lib::afterscript::AfterscriptInfo;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;

use crate::resources::run_script;
use crate::status::Completion;
use crate::status::PostprocessCompletion;
use crate::status::Status;

/// Runs the afterscript on jobs that are completed and do not yet have an afterscript output.
pub fn run_afterscript(
    runs: &BTreeMap<usize, Option<Status>>,
    experiment: &Experiment,
) -> Result<(), anyhow::Error> {
    let runs = filter_runs_for_afterscript(runs)?;

    for run_id in runs {
        let run = &experiment.runs[*run_id];
        let info = run.afterscript_info.clone();
        let res_path = run.output_path.clone();

        if info.is_none() {
            continue;
        }

        let info = info
            .ok_or(anyhow!("Could not get the afterscript information"))
            .with_context(ctx!(
                "Could not get the afterscript information", ;
                "",
            ))?;

        let exit_status = run_afterscript_for_run(&info, &res_path)?;
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
    }

    Ok(())
}

/// Find the completed jobs where afterscript did not run yet.
pub fn filter_runs_for_afterscript(
    runs: &BTreeMap<usize, Option<Status>>,
) -> Result<Vec<&usize>, anyhow::Error> {
    let mut filtered = vec![];

    for (run_id, status) in runs {
        let status = status
            .clone()
            .ok_or(anyhow!("Status does not exist"))
            .with_context(ctx!(
                "Could not find status of run {}", run_id;
                "",
            ))?;

        if let Status::AfterScript(Completion::Success, PostprocessCompletion::Dormant) = status {
            filtered.push(run_id);
        }
    }

    Ok(filtered)
}

/// Runs the afterscript on given jobs.
pub fn run_afterscript_for_run(
    info: &AfterscriptInfo,
    res_path: &PathBuf,
) -> Result<ExitStatus, anyhow::Error> {
    let (after_path, out_path) = (&info.afterscript_path, &info.afterscript_output_path);

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

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
