use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use thiserror::Error;

use crate::config::Config;
use crate::error::ctx;
use crate::error::Ctx;
use crate::experiment::Experiment;

type MachineType = u16;

/// This function returns the commands to be run for an n x m matching of the runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(
    experiment: &Experiment,
    #[allow(unused_variables)] arch: MachineType,
    conf: &Config,
) -> Result<Vec<Command>> {
    let mut result = Vec::new();

    for run in &experiment.runs {
        let program = &conf.programs[&run.program_name];
        let input = &conf.inputs[&run.input_name];

        #[cfg(target_os = "linux")]
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

/// The architecture does not match the one we want to run on.
#[derive(Debug, Error, Clone, Copy)]
#[error("The program architecture {binary} does not match the expected architecture {expected}")]
pub struct ArchitectureMismatch {
    /// The expected architecture in `e_machine` format.
    pub expected: u16,

    /// The architecture of the binary in `e_machine` format.
    pub binary: u16,
}

/// Verify if the architecture of a `binary` matched the `expected` architecture.
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
fn verify_arch(binary: &PathBuf, expected: MachineType) -> Result<()> {
    use elf::endian::AnyEndian;
    use elf::ElfBytes;

    use crate::file_system::read_bytes;

    let elf = read_bytes(binary)?;

    let elf = ElfBytes::<AnyEndian>::minimal_parse(elf.as_slice()).with_context(ctx!(
      "Could not parse the file as ELF {binary:?}", ;
      "Are you sure this file is executable and you are using linux?",
    ))?;

    if elf.ehdr.e_machine != expected {
        Err(ArchitectureMismatch {
            binary: elf.ehdr.e_machine,
            expected,
        })
        .with_context(ctx!(
          "The architecture does not match for program {binary:?}", ;
          "Ensure that the program is compiled for the correct target",
        ))
    } else {
        Ok(())
    }
}
