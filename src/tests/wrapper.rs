use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use crate::config::Input;
use crate::config::Program;
use crate::constants::E_MACHINE_MAPPING;
use crate::tests::create_sample_experiment;
use crate::tests::get_compiled_example;
use crate::wrapper::wrap;

const X86_64_PRE_PROGRAMMED_BINARY: &str = include_str!("resources/x86_64_pre_programmed.rs");

/// This test will generate an ARM binary and check if [crate::wrapper::wrap] rightfully rejects it.
#[cfg(target_os = "linux")]
#[test]
fn non_matching_arch() {
    use crate::tests::create_sample_experiment;

    const ARM_PRE_PROGRAMMED_BINARY: &str = include_str!("resources/arm_pre_programmed.rs");

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
