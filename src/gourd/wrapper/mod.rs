/// Binary verification for linux.
mod check_binary_linux;
/// Binary verification for macos.
mod check_binary_macos;

use std::path::Path;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use std::path::PathBuf;
/// Verify if the architecture of a `binary` matched the `expected`
/// architecture.
use std::process::Command;

use anyhow::Result;
use gourd_lib::experiment::ChunkRunStatus;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;

#[cfg(target_os = "linux")]
use crate::wrapper::check_binary_linux::verify_arch;
#[cfg(target_os = "macos")]
use crate::wrapper::check_binary_macos::verify_arch;

/// Verify the architecture of the binary.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn verify_arch(_: &PathBuf, _: &str, _: &impl FileOperations) -> Result<()> {
    Ok(())
}

/// This function returns the commands to be run for an n x m matching of the
/// runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(
    experiment: &mut Experiment,
    experiment_path: &Path,
    arch: &str,
    fs: &impl FileOperations,
) -> Result<Vec<Command>> {
    let mut result = Vec::new();

    let mut chunks_to_iterate = experiment.chunks.clone();

    for (chunk_id, chunk) in chunks_to_iterate.iter_mut().enumerate() {
        if matches!(chunk.status, ChunkRunStatus::RanLocally) {
            continue;
        }

        for (chunk_rel, run_id) in chunk.runs.iter().enumerate() {
            let run = &experiment.runs[*run_id];
            let program = &experiment.get_program(run)?;

            verify_arch(&program.binary, arch, fs)?;

            let mut cmd = Command::new(&experiment.config.wrapper);

            cmd.arg(format!("{}", chunk_id))
                .arg(experiment_path)
                .arg(format!("{}", chunk_rel));

            result.push(cmd);
        }

        chunk.status = ChunkRunStatus::RanLocally;
    }

    experiment.chunks = chunks_to_iterate;

    Ok(result)
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
