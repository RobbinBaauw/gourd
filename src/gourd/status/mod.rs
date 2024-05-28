use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::BufWriter;
use std::io::Write;
use std::thread::sleep;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use fs_based::get_fs_statuses;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::constants::NAME_STYLE;
use gourd_lib::constants::PRIMARY_STYLE;
use gourd_lib::constants::SHORTEN_STATUS_CUTOFF;
use gourd_lib::constants::STATUS_REFRESH_PERIOD;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::measurement::Metrics;
use indicatif::MultiProgress;
use log::info;
use slurm_based::get_slurm_statuses;

use crate::cli::printing::generate_progress_bar;

/// File system based status information.
pub mod fs_based;

/// Slurm based status information
pub mod slurm_based;

/// The reasons for slurm to kill a job
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlurmKillReason {
    /// Job terminated due to launch failure, typically due to a hardware failure (e.g. unable to boot the node or block and the job can not be requeued).
    BootFail,

    /// Job was explicitly cancelled by the user or system administrator. The job may or may not have been initiated.
    Cancelled,

    /// Job terminated on deadline.
    Deadline,

    /// Job terminated due to failure of one or more allocated nodes.
    NodeFail,

    /// Job experienced out of memory error.
    OutOfMemory,

    /// Job terminated due to preemption.
    Preempted,

    /// Job has an allocation, but execution has been suspended and CPUs have been released for other jobs.
    Suspended,

    /// Job reached the time limit
    Timeout,

    /// Unspecified by the account reason to fail
    SlurmFail,
}

/// The reasons for a job failing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FailureReason {
    /// The job returned a non zero exit status.
    Failed,

    /// Slurm killed the job.
    SlurmKill(SlurmKillReason),

    /// User marked.
    UserForced,
}

/// This possible status of a job.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// The job has not yet started.
    Pending,

    /// The job is still running.
    Running,

    /// The job completed successfully.
    Completed,

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

/// Structure of file based status
#[derive(Debug, Clone, PartialEq)]
pub struct FileSystemBasedStatus {
    /// State of completion of the run
    pub completion: State,

    /// Metrics of the run
    pub metrics: Metrics,

    /// The completion of the afterscript, if there.
    pub afterscript_completion: Option<PostprocessCompletion>,

    /// The completion of the postprocess job, if there.
    pub postprocess_job_completion: Option<PostprocessCompletion>,
}

/// Structure of slurm based status
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct SlurmBasedStatus {
    /// State of completion of the run
    pub completion: State,

    /// Exit code of the program
    pub exit_code_program: isize,

    /// Exit code of the slurm
    pub exit_code_slurm: isize,
}

/// All possible postprocessing statuses of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// State of the run from file based status
    pub fs_state: State,

    /// State of the run from slurm based status
    pub slurm_state: Option<State>,

    /// Metrics of the run
    pub metrics: Metrics,

    /// Exit code of the program from slurm based status
    pub slurm_exit_code_slurm: Option<isize>,

    /// Exit code of the slurm
    pub slurm_exit_code_program: Option<isize>,

    /// The completion of the afterscript, if there.
    pub afterscript_completion: Option<PostprocessCompletion>,

    /// The completion of the postprocess job, if there.
    pub postprocess_job_completion: Option<PostprocessCompletion>,
}

/// This type maps between `run_id` and the [Status] of the run.
pub type ExperimentStatus = BTreeMap<usize, Option<Status>>;

/// Get the status of the provided experiment.
pub fn get_statuses(
    experiment: &Experiment,
    fs: &mut impl FileOperations,
) -> Result<ExperimentStatus> {
    // for now we do not support slurm.

    let fs = get_fs_statuses(fs, experiment)?;

    let mut slurm = None;

    if experiment.env == Environment::Slurm {
        slurm = Some(get_slurm_statuses(experiment)?);
    }

    merge_statuses(fs, slurm)
}

fn merge_statuses(
    _fs: BTreeMap<usize, FileSystemBasedStatus>,
    _slurm: Option<BTreeMap<usize, SlurmBasedStatus>>,
) -> Result<ExperimentStatus> {
    todo!()
}

impl Display for SlurmKillReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlurmKillReason::BootFail => write!(f, "boot failed"),
            SlurmKillReason::Cancelled => write!(f, "cancelled"),
            SlurmKillReason::Deadline => write!(f, "deadline reached"),
            SlurmKillReason::NodeFail => write!(f, "node failed"),
            SlurmKillReason::OutOfMemory => write!(f, "out of memory"),
            SlurmKillReason::Preempted => write!(f, "preempted"),
            SlurmKillReason::Suspended => write!(f, "suspended"),
            SlurmKillReason::Timeout => write!(f, "time out"),
            _ => write!(f, "no reason"),
        }
    }
}

impl Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::SlurmKill(reason) => write!(f, "slurm killed the job - {}", reason),
            FailureReason::UserForced => write!(f, "user killed the job"),
            FailureReason::Failed => write!(f, "failed"),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Pending => write!(f, "pending?")?,
            State::Running => write!(f, "running!")?,
            State::Completed => write!(f, "success")?,
            State::Fail(reason) => write!(f, "{}{}{:#}", ERROR_STYLE, reason, ERROR_STYLE)?,
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
            match stat.fs_state {
                State::Completed => {
                    finished += 1;
                }
                State::Fail(_) => {
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
                    status.fs_state,
                    width = longest_input
                )?;

                if let Some(state) = status.slurm_state {
                    if state == State::Pending {
                        if let Some(slurm_id) = &run.slurm_id {
                            write!(f, " scheduled on slurm as {}", slurm_id)?;
                        }
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
                "{NAME_STYLE}file status?{NAME_STYLE:#} {:#}",
                status.fs_state
            )?;

            if let Some(slurm_state) = status.slurm_state {
                writeln!(
                    f,
                    "{NAME_STYLE}slurm status?{NAME_STYLE:#} {:#}",
                    slurm_state
                )?;
            }

            if let Metrics::Done(measurement) = status.metrics {
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
