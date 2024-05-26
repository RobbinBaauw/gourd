use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::BufWriter;
use std::io::Write;
use std::thread::sleep;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::constants::NAME_STYLE;
use gourd_lib::constants::PRIMARY_STYLE;
use gourd_lib::constants::SHORTEN_STATUS_CUTOFF;
use gourd_lib::constants::STATUS_REFRESH_PERIOD;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::measurement::Measurement;
use indicatif::MultiProgress;
use log::info;

use self::fs_based::FileBasedStatus;
use crate::cli::printing::generate_progress_bar;

/// File system based status information.
pub mod fs_based;

/// The reasons for a job failing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailureReason {
    /// The job retunrned a non zero exit status.
    ExitStatus(Measurement),

    /// Slurm killed the job.
    SlurmKill,

    /// User marked.
    UserForced,
}

/// This possible outcomes of a job.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Completion {
    /// The job has not yet started.
    Dormant,

    /// The job is still running.
    Pending,

    /// The job succeeded.
    Success(Measurement),

    /// The job failed with the following exit status.
    Fail(FailureReason),
}

/// This possible outcomes of a postprocessing.
#[derive(Debug, Clone, PartialEq)]
pub enum PostprocessCompletion {
    /// The postprocessing job has not yet started.
    Dormant,

    /// The postprocessing job is still running.
    Pending,

    /// The postprocessing job succeded.
    Success(PostprocessOutput),

    /// The postprocessing job failed with the following exit status.
    Fail(FailureReason),
}

/// The results of a postprocessing.
#[derive(Debug, Clone, PartialEq)]
pub struct PostprocessOutput {
    /// The shortened version of postprocessing output.
    pub short_output: String,

    /// The full output of postprocessing.
    pub long_output: String,
}

/// All possible postprocessing statuses of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// The completion of the job.
    pub completion: Completion,

    /// The completion of the afterscript, if there.
    pub afterscript_completion: Option<PostprocessCompletion>,

    /// The completion of the postprocess job, if there.
    pub postprocess_job_completion: Option<PostprocessCompletion>,
}

/// This type maps between `run_id` and the [Status] of the run.
pub type ExperimentStatus = BTreeMap<usize, Option<Status>>;

/// A struct that can attest the statuses or some or all running jobs.
pub trait StatusProvider<T> {
    /// Try to get the statuses of jobs.
    fn get_statuses(connection: &mut T, experiment: &Experiment) -> Result<ExperimentStatus>;
}

/// Get the status of the provided experiment.
pub fn get_statuses(
    experiment: &Experiment,
    fs: &mut impl FileOperations,
) -> Result<ExperimentStatus> {
    // for now we do not support slurm.

    FileBasedStatus::get_statuses(fs, experiment)
}

impl Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::SlurmKill => write!(f, "slurm killed the job"),
            FailureReason::UserForced => write!(f, "user killed the job"),
            FailureReason::ExitStatus(exit) => write!(f, "exit code {}", exit.exit_code),
        }
    }
}

impl Display for Completion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Completion::Dormant => write!(f, "dormant?")?,
            Completion::Pending => write!(f, "pending!")?,
            Completion::Success(measurement) => {
                if f.alternate() {
                    write!(
                        f,
                        "{}success{:#} {NAME_STYLE}wall clock time{NAME_STYLE:#}: {}",
                        PRIMARY_STYLE,
                        PRIMARY_STYLE,
                        humantime::Duration::from(measurement.wall_micros)
                    )?
                } else {
                    write!(
                        f,
                        "{}success{:#}, took: {}",
                        PRIMARY_STYLE,
                        PRIMARY_STYLE,
                        humantime::Duration::from(measurement.wall_micros)
                    )?
                }
            }

            Completion::Fail(exit) => {
                write!(f, "{}failed with {}{:#}", ERROR_STYLE, exit, ERROR_STYLE)?
            }
        };

        Ok(())
    }
}

/// Display the status of an experiment in a human readable from.
///
/// Returns how many jobs are finished.
#[cfg(not(tarpaulin_include))] // We won't test stdout
pub fn display_statuses(
    f: &mut impl Write,
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> Result<usize> {
    if experiment.runs.len() <= SHORTEN_STATUS_CUTOFF {
        long_status(f, experiment, statuses)?;
    } else {
        short_status(experiment, statuses)?;
    }

    let mut finished = 0;

    for run in 0..experiment.runs.len() {
        if let Some(stat) = &statuses[&run] {
            match stat.completion {
                Completion::Success(_) => {
                    finished += 1;
                }
                Completion::Fail(_) => {
                    finished += 1;
                }
                _ => {}
            }
        }
    }

    Ok(finished)
}

#[cfg(not(tarpaulin_include))] // We can't test stdout
fn short_status(_: &Experiment, _: &ExperimentStatus) -> Result<()> {
    todo!()
}

#[cfg(not(tarpaulin_include))] // We can't test stdout
fn long_status(
    f: &mut impl Write,
    experiment: &Experiment,
    statuses: &ExperimentStatus,
) -> Result<()> {
    let runs = &experiment.runs;

    let mut by_program: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    let mut longest_input: usize = 0;

    for (run_id, run_data) in runs.iter().enumerate() {
        longest_input = max(longest_input, run_data.input.len());

        if let Some(for_this_prog) = by_program.get_mut(&run_data.program) {
            for_this_prog.push(run_id);
        } else {
            by_program.insert(run_data.program.clone(), vec![run_id]);
        }
    }

    for (prog, prog_runs) in by_program {
        writeln!(f)?;

        writeln!(f, "For program {}:", prog)?;

        for run_id in prog_runs {
            let run = &experiment.runs[run_id];

            if let Some(status) = &statuses[&run_id] {
                write!(
                    f,
                    "  {}. {:.<width$}.... {}",
                    run_id,
                    run.input,
                    status.completion,
                    width = longest_input
                )?;

                if let Completion::Dormant = status.completion {
                    if let Some(slurm_id) = &run.slurm_id {
                        write!(f, " scheduled on slurm as {}", slurm_id)?;
                    }
                }

                writeln!(f)?;
            } else {
                writeln!(
                    f,
                    "  {}. {:.<width$}.... could not retrieve status ",
                    run_id,
                    run.input,
                    width = longest_input
                )?;
            }
        }
    }

    Ok(())
}

/// Display the status of an experiment in a human readable from.
#[cfg(not(tarpaulin_include))] // We can't test stdout
pub fn display_job(
    f: &mut impl Write,
    exp: &Experiment,
    statuses: &ExperimentStatus,
    id: usize,
) -> Result<()> {
    info!(
        "Displaying the status of job {} in experiment {}",
        id, exp.seq
    );

    writeln!(f)?;

    if let Some(run) = exp.runs.get(id) {
        let program = &exp.config.programs[&run.program];
        let input = &exp.config.inputs[&run.input];

        writeln!(f, "{NAME_STYLE}program{NAME_STYLE:#}: {}", run.program)?;
        writeln!(
            f,
            "  {NAME_STYLE}binary{NAME_STYLE:#}: {:?}",
            program.binary
        )?;

        writeln!(f, "{NAME_STYLE}input{NAME_STYLE:#}: {:?}", run.input)?;
        writeln!(f, " {NAME_STYLE}file{NAME_STYLE:#}: {:?}", input.input)?;

        let mut args = vec![];
        args.append(&mut program.arguments.clone());
        args.append(&mut input.arguments.clone());

        writeln!(f, "{NAME_STYLE}arguments{NAME_STYLE:#}: {:?}\n", args)?;

        writeln!(
            f,
            "{NAME_STYLE}output path{NAME_STYLE:#}: {:?}",
            run.output_path
        )?;
        writeln!(
            f,
            "{NAME_STYLE}stderr path{NAME_STYLE:#}: {:?}",
            run.err_path
        )?;
        writeln!(
            f,
            "{NAME_STYLE}metric path{NAME_STYLE:#}: {:?}\n",
            run.metrics_path
        )?;

        if let Some(slurm_id) = &run.slurm_id {
            writeln!(f, "scheduled on slurm as {}", slurm_id)?;
        }

        if let Some(status) = &statuses[&id] {
            writeln!(
                f,
                "{NAME_STYLE}status?{NAME_STYLE:#} {:#}",
                status.completion
            )?;

            if let Completion::Success(measruement) = status.completion {
                if let Some(rusage) = measruement.rusage {
                    writeln!(f, "{NAME_STYLE}metrics{NAME_STYLE:#}:\n{rusage}")?;
                }
            } else if let Completion::Fail(FailureReason::ExitStatus(measurement)) =
                status.completion
            {
                if let Some(rusage) = measurement.rusage {
                    writeln!(f, "{NAME_STYLE}metrics{NAME_STYLE:#}:\n{rusage}")?;
                }
            }
        } else {
            writeln!(f, "no status information available")?;
        }

        Ok(())
    } else {
        Err(anyhow!("A run with this id does not exist")).with_context(ctx!(
            "", ;
            "You can see the run ids by running {}gourd status{:#}", PRIMARY_STYLE, PRIMARY_STYLE
        ))
    }
}
/// Print status until all tasks are finished.
pub fn blocking_status(
    progress: &MultiProgress,
    experiment: &Experiment,
    fs: &mut impl FileOperations,
) -> Result<()> {
    let mut complete = 0;
    let mut message = "".to_string();

    let bar = progress.add(generate_progress_bar(experiment.runs.len() as u64)?);

    while complete < experiment.runs.len() {
        let mut buf = BufWriter::new(Vec::new());

        let statuses = get_statuses(experiment, fs)?;
        complete = display_statuses(&mut buf, experiment, &statuses)?;
        message = format!("{}\n", String::from_utf8(buf.into_inner()?)?);

        bar.set_prefix(message.clone());
        bar.set_position(complete as u64);

        sleep(STATUS_REFRESH_PERIOD);
    }

    bar.finish();
    progress.remove(&bar);
    progress.clear()?;

    let leftover = generate_progress_bar(experiment.runs.len() as u64)?;
    leftover.set_position(complete as u64);
    leftover.set_prefix(message);
    leftover.finish();

    Ok(())
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
