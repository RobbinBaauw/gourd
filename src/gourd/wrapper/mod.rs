mod check_binary_linux;
mod check_binary_macos;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use std::path::PathBuf;
/// Verify if the architecture of a `binary` matched the `expected` architecture.
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;

#[cfg(target_os = "linux")]
use crate::wrapper::check_binary_linux::verify_arch;
#[cfg(target_os = "macos")]
use crate::wrapper::check_binary_macos::verify_arch;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn verify_arch(_: &PathBuf, _: &str, _: &impl FileOperations) -> Result<()> {
    Ok(())
}

/// This function returns the commands to be run for an n x m matching of the runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(
    experiment: &Experiment,
    arch: &str,
    conf: &Config,
    fs: &impl FileOperations,
) -> Result<Vec<Command>> {
    let mut result = Vec::new();

    for run in &experiment.runs {
        verify_arch(&run.program.binary, arch, fs)?;

        let mut cmd = Command::new(&conf.wrapper);
        cmd.arg(&run.program.binary.canonicalize().with_context(ctx!(
              "The executable for {:?} could not be found", run.program.binary;
              "Please ensure that all executables exist", ))?)
            // TODO: Fix this in the new version of the wrapper, assigned to Andreas
            .arg(
                run.input
                    .input
                    .clone()
                    .unwrap()
                    .canonicalize()
                    .with_context(ctx!(
              "The input file {:?} could not be found", run.input.input;
              "Please ensure that all input files exist", ))?,
            )
            .arg(run.output_path.clone())
            .arg(run.metrics_path.clone())
            .arg(run.err_path.clone())
            .args(&run.program.arguments)
            .args(&run.input.arguments);

        result.push(cmd);
    }

    Ok(result)
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
