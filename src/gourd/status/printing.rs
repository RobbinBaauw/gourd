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
use log::info;

use super::ExperimentStatus;
use super::FsState;
use super::SlurmState;
use super::Status;

impl Display for SlurmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlurmState::BootFail => write!(f, "boot failed"),
            SlurmState::Cancelled => write!(f, "cancelled"),
            SlurmState::Deadline => write!(f, "deadline reached"),
            SlurmState::NodeFail => write!(f, "node failed"),
            SlurmState::OutOfMemory => write!(f, "out of memory"),
            SlurmState::Preempted => write!(f, "preempted"),
            SlurmState::Suspended => write!(f, "suspended"),
            SlurmState::Timeout => write!(f, "time out"),
            SlurmState::SlurmFail => write!(f, "{ERROR_STYLE}job failed{ERROR_STYLE:#}"),
            SlurmState::Success => write!(f, "{PRIMARY_STYLE}job finished!{PRIMARY_STYLE:#}"),
            SlurmState::Pending => write!(f, "pending.."),
            SlurmState::Running => write!(f, "running.."),
        }
    }
}

impl Display for FsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsState::Pending => write!(f, "pending?"),
            FsState::Running => write!(f, "running!"),
            FsState::Completed(metrics) => {
                if metrics.exit_code == 0 {
                    if f.alternate() {
                        write!(
                            f,
                            "{}success{:#} {NAME_STYLE}wall clock time{NAME_STYLE:#}: {}",
                            PRIMARY_STYLE,
                            PRIMARY_STYLE,
                            humantime::Duration::from(metrics.wall_micros)
                        )
                    } else {
                        write!(
                            f,
                            "{}success{:#}, took: {}",
                            PRIMARY_STYLE,
                            PRIMARY_STYLE,
                            humantime::Duration::from(metrics.wall_micros)
                        )
                    }
                } else {
                    write!(
                        f,
                        "{}failed, code: {}{:#}",
                        ERROR_STYLE, metrics.exit_code, ERROR_STYLE
                    )
                }
            }
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            // Long status.
            writeln!(
                f,
                "{NAME_STYLE}file status?{NAME_STYLE:#} {:#}",
                self.fs_status.completion
            )?;

            if let Some(slurm) = &self.slurm_status {
                writeln!(
                    f,
                    "{NAME_STYLE}slurm status?{NAME_STYLE:#} {:#} with exit code {}",
                    slurm.completion, slurm.exit_code_slurm
                )?;
            }

            if let FsState::Completed(measurement) = self.fs_status.completion {
                if let Some(rusage) = measurement.rusage {
                    write!(f, "{NAME_STYLE}metrics{NAME_STYLE:#}:\n{rusage}")?;
                }
            }
        } else {
            // Short summary.
            write!(f, "{}", self.fs_status.completion)?;

            // TODO: Incorporate slurm status here.
        }

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

            if status.fs_status.completion == FsState::Pending {
                if let Some(slurm_id) = &run.slurm_id {
                    write!(f, " scheduled on slurm as {}", slurm_id)?;
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

        writeln!(f, "{status:#}")?;

        Ok(())
    } else {
        Err(anyhow!("A run with this id does not exist")).with_context(ctx!(
            "", ;
            "You can see the run ids by running {}gourd status{:#}", PRIMARY_STYLE, PRIMARY_STYLE
        ))
    }
}
