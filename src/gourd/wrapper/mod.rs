mod check_binary_linux;
mod check_binary_macos;

use std::path::Path;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use std::path::PathBuf;
/// Verify if the architecture of a `binary` matched the `expected` architecture.
use std::process::Command;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
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
    experiment_path: &Path,
    arch: &str,
    fs: &impl FileOperations,
) -> Result<Vec<Command>> {
    let mut result = Vec::new();

    if experiment.chunks.len() != 1 {
        return Err(anyhow!("Wrapping locally requires exactly one chunk"))
            .with_context(ctx!("",;"You may be running too many programs on local",));
    }

    for (chunk_rel, run_id) in experiment.chunks[0].runs.iter().enumerate() {
        let run = &experiment.runs[*run_id];
        let program = &experiment.config.programs[&run.program];

        verify_arch(&program.binary, arch, fs)?;

        let mut cmd = Command::new(&experiment.config.wrapper);

        // The number of the first chunk is 0.
        cmd.arg("0")
            .arg(experiment_path)
            .arg(format!("{}", chunk_rel));

        result.push(cmd);
    }

    Ok(result)
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
