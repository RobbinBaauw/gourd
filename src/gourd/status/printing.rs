use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::constants::ERROR_STYLE;
use gourd_lib::constants::NAME_STYLE;
use gourd_lib::constants::PRIMARY_STYLE;
use gourd_lib::constants::SHORTEN_STATUS_CUTOFF;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::measurement::Metrics;
use log::info;

use super::ExperimentStatus;
use super::FailureReason;
use super::RunState;
use super::SlurmBasedStatus;
use super::SlurmKillReason;
use super::Status;

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
            FailureReason::SlurmKill(reason) => write!(f, "slurm killed the job, {}", reason),
            FailureReason::UserForced => write!(f, "user killed the job"),
            FailureReason::Failed(status) => write!(f, "exit status {}", status),
        }
    }
}

impl Display for SlurmBasedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.completion {
            RunState::Pending => write!(f, "reports pending")?,
            RunState::Running => write!(f, "reports running")?,
            RunState::Completed => write!(f, "reports finished")?,
            RunState::Fail(reason) => {
                write!(f, "{}failed, {}{:#}", ERROR_STYLE, reason, ERROR_STYLE)?
            }
        };

        Ok(())
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fs_status.completion {
            RunState::Pending => write!(f, "pending?")?,
            RunState::Running => write!(f, "running!")?,
            RunState::Completed => {
                if let Some(Metrics::Done(measure)) = self.fs_status.metrics {
                    if f.alternate() {
                        write!(
                            f,
                            "{}success{:#} {NAME_STYLE}wall clock time{NAME_STYLE:#}: {}",
                            PRIMARY_STYLE,
                            PRIMARY_STYLE,
                            humantime::Duration::from(measure.wall_micros)
                        )?
                    } else {
                        write!(
                            f,
                            "{}success{:#}, took: {}",
                            PRIMARY_STYLE,
                            PRIMARY_STYLE,
                            humantime::Duration::from(measure.wall_micros)
                        )?
                    }
                } else {
                    write!(f, "{}success{:#}?", PRIMARY_STYLE, PRIMARY_STYLE)?
                }
            }
            RunState::Fail(reason) => {
                write!(f, "{}failed, {}{:#}", ERROR_STYLE, reason, ERROR_STYLE)?
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
        if statuses[&run].is_completed() {
            finished += 1;
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
            let status = &statuses[&run_id];

            // TODO: introduce logic to handle all possible mismatches.

            write!(
                f,
                "  {}. {:.<width$}.... {}",
                run_id,
                run.input,
                status,
                width = longest_input
            )?;

            if let Some(state) = status.slurm_status {
                if state.completion == RunState::Pending {
                    if let Some(slurm_id) = &run.slurm_id {
                        write!(f, " scheduled on slurm as {}", slurm_id)?;
                    }
                }
            }

            writeln!(f)?;
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

        let status = &statuses[&id];

        writeln!(f, "{NAME_STYLE}file status?{NAME_STYLE:#} {:#}", status)?;

        if let Some(slurm) = status.slurm_status {
            writeln!(f, "{NAME_STYLE}slurm status?{NAME_STYLE:#} {:#}", slurm)?;
        }

        if let Some(Metrics::Done(measurement)) = status.fs_status.metrics {
            if let Some(rusage) = measurement.rusage {
                writeln!(f, "{NAME_STYLE}metrics{NAME_STYLE:#}:\n{rusage}")?;
            }
        }

        Ok(())
    } else {
        Err(anyhow!("A run with this id does not exist")).with_context(ctx!(
            "", ;
            "You can see the run ids by running {}gourd status{:#}", PRIMARY_STYLE, PRIMARY_STYLE
        ))
    }
}
