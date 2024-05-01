use std::fs;
use std::path::PathBuf;
use std::process::Command;

use elf::endian::AnyEndian;
use elf::ElfBytes;

use crate::error::GourdError;
use crate::error::GourdError::*;

type MachineType = u16;

const WRAPPER: &'static str = "./wrapper";

pub struct Run {
    pub binary: PathBuf,
    pub arguments: Vec<String>,
}

/// This function wraps a executable in a testing harness.
pub fn wrap(
    runs: Vec<Run>,
    tests: Vec<PathBuf>,
    arch: MachineType,
) -> Result<Vec<Command>, GourdError> {
    let this_will_be_in_the_config_output_path: PathBuf = "/tmp/gourd/".parse().unwrap();
    let this_will_be_in_the_config_result_path: PathBuf = "/tmp/gourd/".parse().unwrap();

    fs::create_dir_all(&this_will_be_in_the_config_result_path)?;
    fs::create_dir_all(&this_will_be_in_the_config_output_path)?;

    let mut result = Vec::new();

    for (run_id, run) in runs.iter().enumerate() {
        verify_arch(&run.binary, arch)?;

        for (test_id, test) in tests.iter().enumerate() {
            let mut cmd = Command::new(WRAPPER);
            cmd.arg(fs::canonicalize(&run.binary).map_err(|x| FileError(run.binary.clone(), x))?)
                .arg(fs::canonicalize(test).map_err(|x| FileError(test.clone(), x))?)
                .arg(
                    this_will_be_in_the_config_output_path
                        .join(format!("run{}_{}_output", run_id, test_id)),
                )
                .arg(
                    this_will_be_in_the_config_result_path
                        .join(format!("run{}_{}_result", run_id, test_id)),
                )
                .args(&run.arguments);

            result.push(cmd);
        }
    }

    Ok(result)
}

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
