use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Input;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::FileOperations;
use log::debug;

use crate::status::ExperimentStatus;
use crate::status::PostprocessCompletion;

/// Schedules the postprocessing job for jobs that are completed and do not yet
/// have a postprocess job output.
pub fn schedule_post_jobs(
    experiment: &mut Experiment,
    statuses: &mut ExperimentStatus,
    fs: &impl FileOperations,
) -> Result<()> {
    let runs = filter_runs_for_post_job(statuses)?;

    for run_id in runs {
        let run = &experiment.runs[*run_id];
        let post_out_path = &run.post_job_output_path;
        let res_path = run.output_path.clone();

        if post_out_path.is_none() {
            continue;
        }

        debug!("Adding postprocessing run for job {run_id}");

        let post_output = post_out_path
            .clone()
            .ok_or(anyhow!("Could not get the postprocessing information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        let program = &experiment.get_program(run)?;

        let postprocess = program
            .postprocess_job
            .clone()
            .ok_or(anyhow!("Could not get the postprocessing information"))
            .with_context(ctx!(
                "Could not get the postprocessing information", ;
                "",
            ))?;

        let prog_name = match &run.program {
            FieldRef::Regular(name) => name.clone(),
            FieldRef::Postprocess(name) => name.clone(),
        };

        post_job_for_run(
            format!("{}_{}", prog_name, run.input),
            postprocess,
            &res_path,
            &post_output,
            run.work_dir.clone(),
            experiment,
            fs,
        )?
    }

    Ok(())
}

/// Finds the completed jobs where posprocess job did not run yet.
pub fn filter_runs_for_post_job(runs: &mut ExperimentStatus) -> Result<Vec<&usize>> {
    let mut filtered = vec![];

    for (run_id, status) in runs {
        if status.fs_status.completion.has_succeded()
            && status.fs_status.completion.is_completed()
            && !matches!(
                status.fs_status.postprocess_job_completion,
                Some(PostprocessCompletion::Success(_))
            )
        {
            filtered.push(run_id);
        }
    }

    Ok(filtered)
}

/// Schedules the postprocess job for given jobs.
pub fn post_job_for_run(
    input_name: String,
    postprocess_name: String,
    postprocess_input: &Path,
    postprocess_out: &Path,
    work_dir: PathBuf,
    experiment: &mut Experiment,
    fs: &impl FileOperations,
) -> Result<()> {
    experiment.postprocess_inputs.insert(
        input_name.clone(),
        Input {
            input: Some(postprocess_input.to_path_buf()),
            arguments: vec![],
        },
    );

    experiment.save(&experiment.config.experiments_folder, fs)?;

    experiment.runs.push(Run {
        program: FieldRef::Postprocess(postprocess_name.clone()),
        input: FieldRef::Postprocess(input_name),
        err_path: fs.truncate_and_canonicalize(&postprocess_out.join("stderr"))?,
        metrics_path: fs.truncate_and_canonicalize(&postprocess_out.join("metrics"))?,
        output_path: fs.truncate_and_canonicalize(&postprocess_out.join("stdout"))?,
        work_dir: work_dir.to_path_buf(),
        afterscript_output_path: None,
        post_job_output_path: None, // these two can be updated to allow pipelining
        slurm_id: None,
    });

    Ok(())
}
