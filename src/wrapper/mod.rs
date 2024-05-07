use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
use elf::endian::AnyEndian;
#[cfg(target_os = "linux")]
use elf::ElfBytes;

use crate::config::Config;
use crate::config::Program;
use crate::config::Run;
use crate::error::GourdError;
use crate::error::GourdError::*;

type MachineType = u16;

/// This function returns the commands to be run for an n x m matching of the runs to tests.
///
/// The results and outputs will be located in `config.output_dir`.
pub fn wrap(
    programs: &BTreeMap<String, Program>,
    runs: &BTreeMap<String, Run>,
    #[allow(unused_variables)] arch: MachineType,
    conf: &Config,
) -> Result<Vec<Command>, GourdError> {
    let mut result = Vec::new();

    for (prog_name, program) in programs {
        #[cfg(target_os = "linux")]
        verify_arch(&program.binary, arch)?;

        for (run_name, run) in runs {
            let mut cmd = Command::new(&conf.wrapper);
            cmd.arg(
                fs::canonicalize(&program.binary)
                    .map_err(|x| FileError(program.binary.clone(), x))?,
            )
            .arg(fs::canonicalize(&run.input).map_err(|x| FileError(run.input.clone(), x))?)
            .arg(
                &conf
                    .output_path
                    .join(format!("algo_{}/{}_output", prog_name, run_name)),
            )
            .arg(
                &conf
                    .metrics_path
                    .join(format!("algo_{}/{}_metrics", prog_name, run_name)),
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
