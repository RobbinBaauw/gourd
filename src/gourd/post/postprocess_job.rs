use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::config::Input;
use gourd_lib::config::Program;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::FileOperations;

use crate::status::Completion;
use crate::status::PostprocessCompletion;
use crate::status::Status;

/// Schedules the postprocessing job for jobs that are completed and do not yet have a postprocess job output.
pub fn schedule_post_jobs(
    experiment: &mut Experiment,
    config: &Config,
    statuses: &mut BTreeMap<usize, Option<Status>>,
    fs: &impl FileOperations,
) -> Result<()> {
    let runs = filter_runs_for_post_job(statuses)?;
    let _length = runs.len();

    for run_id in runs {
        let run = &experiment.runs[*run_id];
        let post_out_path = &run.post_job_output_path;
        let res_path = run.output_path.clone();

        if post_out_path.is_none() {
            continue;
        }

        let post_output = post_out_path
            .clone()
            .ok_or(anyhow!("Could not get the postprocessing information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        let postprocess = run
            .program
            .postprocess_job
            .clone()
            .ok_or(anyhow!("Could not get the postprocessing information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        post_job_for_run(
            &postprocess,
            &res_path,
            &post_output,
            config,
            experiment,
            fs,
        )?
    }

    Ok(())
}

/// Finds the completed jobs where posprocess job did not run yet.
pub fn filter_runs_for_post_job(runs: &mut BTreeMap<usize, Option<Status>>) -> Result<Vec<&usize>> {
    Ok(runs
        .iter_mut()
        .map(|(run_id, status)| {
            Ok((
                run_id,
                status
                    .clone()
                    .ok_or(anyhow!("Status does not exist"))
                    .with_context(ctx!(
                        "Could not find status of run {}", run_id;
                        "",
                    ))?,
            ))
        })
        .map(|x: Result<(&usize, Status)>| x.unwrap())
        .filter(|(_, status)| status.completion == Completion::Success)
        .filter(|(_, status)| {
            status.postprocess_job_completion == Some(PostprocessCompletion::Dormant)
        })
        .map(|(run_id, _)| run_id)
        .collect::<Vec<&usize>>())
}

/// Schedules the postprocess job for given jobs.
pub fn post_job_for_run(
    postprocess: &Path,
    postprocess_input: &PathBuf,
    postprocess_out: &Path,
    conf: &Config,
    experiment: &mut Experiment,
    fs: &impl FileOperations,
) -> Result<()> {
    experiment.runs.push(Run {
        program: Program {
            binary: postprocess.to_path_buf(),
            arguments: vec![],
            afterscript: None,
            postprocess_job: None,
        },
        input: Input {
            input: Some(postprocess_input.clone()),
            arguments: vec![],
        },
        err_path: fs.truncate_and_canonicalize(
            &postprocess_out.join(format!("error_{:?}", postprocess_input)),
        )?,
        metrics_path: fs.truncate_and_canonicalize(
            &conf
                .metrics_path
                .join(format!("metrics_{:?}", postprocess_input)),
        )?,
        output_path: fs.truncate_and_canonicalize(
            &postprocess_out.join(format!("output_{:?}", postprocess_input)),
        )?,
        afterscript_output_path: None,
        post_job_output_path: None,
        job_id: None,
    });

    Ok(())
}
