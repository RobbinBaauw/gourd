mod check_binary_linux;
mod check_binary_macos;

/// Verify if the architecture of a `binary` matched the `expected` architecture.
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;

#[cfg(target_os = "linux")]
use crate::wrapper::check_binary_linux::verify_arch;
#[cfg(target_os = "macos")]
use crate::wrapper::check_binary_macos::verify_arch;

/// This function returns the commands to be run for an n x m matching of the runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(experiment: &Experiment, arch: &str, conf: &Config) -> Result<Vec<Command>> {
    let mut result = Vec::new();

    for run in &experiment.runs {
        let program = &run.program;
        let input = &run.input;

        verify_arch(&program.binary, arch)?;

        let mut cmd = Command::new(&conf.wrapper);
        cmd.arg(&program.binary.canonicalize().with_context(ctx!(
              "The executable for {:?} could not be found", program.binary;
              "Please ensure that all executables exist", ))?)
            .arg(&input.input.canonicalize().with_context(ctx!(
              "The input file {:?} could not be found", input.input;
              "Please ensure that all input files exist", ))?)
            .arg(run.output_path.clone())
            .arg(run.metrics_path.clone())
            .arg(run.err_path.clone())
            .args(&program.arguments)
            .args(&input.arguments);

        result.push(cmd);
    }

    Ok(result)
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
