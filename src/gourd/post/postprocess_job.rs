use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Input;
<<<<<<< HEAD
=======
use gourd_lib::config::Program;
use gourd_lib::constants::INTERNAL_POST;
use gourd_lib::constants::INTERNAL_PREFIX;
>>>>>>> 21f2962 (provide file for inputs)
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::experiment::Run;
use gourd_lib::file_system::FileOperations;

use crate::status::ExperimentStatus;
use crate::status::PostprocessCompletion;
use crate::status::SlurmState;

/// Schedules the postprocessing job for jobs that are completed and do not yet
/// have a postprocess job output.
pub fn schedule_post_jobs(
    experiment: &mut Experiment,
    statuses: &mut ExperimentStatus,
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
        if status.slurm_status.is_some() {
            if let (SlurmState::Success, Some(PostprocessCompletion::Dormant)) = (
                &status.slurm_status.unwrap().completion,
                &status.fs_status.postprocess_job_completion,
            ) {
                filtered.push(run_id);
            }
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
    experiment: &mut Experiment,
    fs: &impl FileOperations,
) -> Result<()> {
<<<<<<< HEAD
    experiment.postprocess_inputs.insert(
        input_name.clone(),
=======
    let prog_name = format!("{}{}{}", INTERNAL_PREFIX, INTERNAL_POST, name);
    let input_name = format!("{}{}{}", INTERNAL_PREFIX, INTERNAL_POST, name);

    experiment.config.programs.insert(
        prog_name.clone(),
        Program {
            binary: postprocess.to_path_buf(),
            arguments: vec![],
            afterscript: None,
            postprocess_job: None,
        },
    );

    experiment.config.inputs.insert(
        prog_name.clone(),
>>>>>>> 21f2962 (provide file for inputs)
        Input {
            input: Some(postprocess_input.to_path_buf()),
            arguments: vec![],
        },
    );

    experiment.save(&experiment.config.experiments_folder, fs)?;

    experiment.runs.push(Run {
        program: FieldRef::Postprocess(postprocess_name.clone()),
        input: FieldRef::Postprocess(input_name),
        err_path: fs.truncate_and_canonicalize(
            &postprocess_out.join(format!("error_{}", postprocess_name)),
        )?,
        metrics_path: fs.truncate_and_canonicalize(
            &postprocess_out.join(format!("metrics_{}", postprocess_name)),
        )?,
        output_path: fs.truncate_and_canonicalize(
            &postprocess_out.join(format!("output_{}", postprocess_name)),
        )?,
        afterscript_output_path: None,
        post_job_output_path: None, // these two can be updated to allow pipelining
        slurm_id: None,
    });

    Ok(())
}
