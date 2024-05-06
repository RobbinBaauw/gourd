use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
use elf::endian::AnyEndian;
#[cfg(target_os = "linux")]
use elf::ElfBytes;

use crate::constants::WRAPPER;
use crate::error::GourdError;
use crate::error::GourdError::*;

type MachineType = u16;

/// A pair of a path to a binary and cli arguments.
#[derive(Debug, Clone)]
pub struct Program {
    /// The path to the executable.
    pub binary: PathBuf,

    /// The cli arguments for the executable.
    pub arguments: Vec<String>,
}

/// This function returns the commands to be run for an n x m matching of the runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(
    runs: Vec<Program>,
    tests: Vec<PathBuf>,
    #[allow(unused_variables)] arch: MachineType,
) -> Result<Vec<Command>, GourdError> {
    let this_will_be_in_the_config_output_path: PathBuf = "/tmp/gourd/".parse().unwrap();
    let this_will_be_in_the_config_result_path: PathBuf = "/tmp/gourd/".parse().unwrap();

    let mut result = Vec::new();

    for (run_id, run) in runs.iter().enumerate() {
        #[cfg(target_os = "linux")]
        verify_arch(&run.binary, arch)?;

        for (test_id, test) in tests.iter().enumerate() {
            let mut cmd = Command::new(WRAPPER);
            cmd.arg(fs::canonicalize(&run.binary).map_err(|x| FileError(run.binary.clone(), x))?)
                .arg(fs::canonicalize(test).map_err(|x| FileError(test.clone(), x))?)
                .arg(
                    this_will_be_in_the_config_output_path
                        .join(format!("algo_{}/{}_output", run_id, test_id)),
                )
                .arg(
                    this_will_be_in_the_config_result_path
                        .join(format!("algo_{}/{}_result", run_id, test_id)),
                )
                .args(&run.arguments);

            result.push(cmd);
        }
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn verify_arch(binary: &PathBuf, expected: MachineType) -> Result<(), GourdError> {
    let elf = fs::read(binary).map_err(|x| FileError(binary.clone(), x))?;

    let elf = ElfBytes::<AnyEndian>::minimal_parse(elf.as_slice())?;

    if elf.ehdr.e_machine != expected {
        Err(ArchitectureMismatch {
            expected,
            binary: elf.ehdr.e_machine,
        })
    } else {
        Ok(())
    }
}
