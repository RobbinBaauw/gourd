use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;

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
        let program = &run.program;
        let input = &run.input;

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

/// Verify if the architecture of a `binary` matched the `expected` architecture.
#[cfg(target_os = "linux")]
use std::path::PathBuf;

use gourd_lib::config::Config;
use gourd_lib::experiment::Experiment;

#[cfg(target_os = "linux")]
fn verify_arch(binary: &PathBuf, expected: MachineType) -> Result<()> {
    use anyhow::anyhow;
    use elf::endian::AnyEndian;
    use elf::ElfBytes;
    use gourd_lib::file_system::read_bytes;

    let elf = read_bytes(binary)?;

    let elf = ElfBytes::<AnyEndian>::minimal_parse(elf.as_slice()).with_context(ctx!(
      "Could not parse the file as ELF {binary:?}", ;
      "Are you sure this file is executable and you are using linux?",
    ))?;

    if elf.ehdr.e_machine != expected {
        Err(anyhow!(
            "The program architecture {} does not match the expected architecture {}",
            elf.ehdr.e_machine,
            expected
        ))
        .with_context(ctx!(
          "The architecture does not match for program {binary:?}", ;
          "Ensure that the program is compiled for the correct target",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use gourd_lib::config::Input;
    use gourd_lib::config::Program;
    use gourd_lib::constants::E_MACHINE_MAPPING;

    use super::*;
    use crate::test_utils::create_sample_experiment;
    use crate::test_utils::get_compiled_example;

    const X86_64_PRE_PROGRAMMED_BINARY: &str =
        include_str!("test_resources/x86_64_pre_programmed.rs");

    /// This test will generate an ARM binary and check if [crate::wrapper::wrap] rightfully rejects it.
    #[cfg(target_os = "linux")]
    #[test]
    fn non_matching_arch() {
        use crate::test_utils::create_sample_experiment;

        const ARM_PRE_PROGRAMMED_BINARY: &str =
            include_str!("test_resources/arm_pre_programmed.rs");

        Command::new("rustup")
            .arg("target")
            .arg("add")
            .arg("thumbv7em-none-eabihf")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let (out, tmp) = get_compiled_example(
            ARM_PRE_PROGRAMMED_BINARY,
            Some(vec!["--target=thumbv7em-none-eabihf"]),
        );

        let input = tmp.path().join("test1");

        fs::write(&input, "4").unwrap();

        let mut first = BTreeMap::new();

        first.insert(
            "any".to_string(),
            Program {
                binary: out,
                arguments: vec![],
                afterscript: None,
            },
        );

        let mut second = BTreeMap::new();

        second.insert(
            "test1".to_string(),
            Input {
                input: input.clone(),
                arguments: vec![],
            },
        );

        let (experiment, conf) = create_sample_experiment(first, second);

        match wrap(&experiment, E_MACHINE_MAPPING("x86_64"), &conf) {
            Err(err) => {
                assert_eq!(
                    "The program architecture 40 does not match the expected architecture 62",
                    format!("{}", err.root_cause())
                )
            }

            e => {
                panic!(
                    "Did not return the correct architecture mismatch, was: {:?}",
                    e
                );
            }
        }
    }

    /// This test will generate a X86 binary and check if [crate::wrapper::wrap]
    /// accepts it and generates correct commands.
    #[test]
    fn matching_arch() {
        let (out, tmp) = get_compiled_example(X86_64_PRE_PROGRAMMED_BINARY, None);
        let input = tmp.path().join("test1");

        fs::write(&input, "4").unwrap();

        let mut first = BTreeMap::new();

        first.insert(
            "any".to_string(),
            Program {
                binary: out.clone(),
                arguments: vec![],
                afterscript: None,
            },
        );

        let mut second = BTreeMap::new();

        second.insert(
            "test1".to_string(),
            Input {
                input: input.clone(),
                arguments: vec![],
            },
        );

        let (experiment, conf) = create_sample_experiment(first, second);

        let cmds = wrap(&experiment, E_MACHINE_MAPPING("x86_64"), &conf).unwrap();

        assert_eq!(1, cmds.len());
        assert_eq!(
            format!("{:?}", cmds[0]),
            format!(
                "{:?}",
                Command::new(&conf.wrapper)
                    .arg(tmp.path().join("prog").canonicalize().unwrap())
                    .arg(input.canonicalize().unwrap())
                    .arg(
                        conf.output_path
                            .join("1/algo_any/test1_output")
                            .canonicalize()
                            .unwrap()
                    )
                    .arg(
                        conf.metrics_path
                            .join("1/algo_any/test1_metrics")
                            .canonicalize()
                            .unwrap()
                    )
                    .arg(
                        conf.output_path
                            .join("1/algo_any/test1_error")
                            .canonicalize()
                            .unwrap()
                    )
            )
        );
    }
}
