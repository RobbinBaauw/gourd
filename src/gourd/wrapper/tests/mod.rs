use std::collections::BTreeMap;
use std::env;
use std::fs;

use gourd_lib::config::Input;
use gourd_lib::config::Program;

use super::*;
use crate::test_utils::create_sample_experiment;
use crate::test_utils::get_compiled_example;

/// This test will generate an ARM binary and check if [crate::wrapper::wrap] rightfully rejects it.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn non_matching_arch() {
    use crate::test_utils::create_sample_experiment;

    const NO_STD_INFINITE_LOOP_RS: &str = include_str!("test_resources/panic_returner.rs");

    Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("thumbv7em-none-eabihf")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let (out, tmp) = get_compiled_example(
        NO_STD_INFINITE_LOOP_RS,
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

    match wrap(&experiment, "x86_64", &conf) {
        Err(err) => {
            assert!(format!("{}", err.root_cause()).contains("not match the expected architecture"))
        }

        e => {
            panic!(
                "Did not return the correct architecture mismatch, was: {:?}",
                e
            );
        }
    }
}

/// This test will generate a binary and of the target architecture and check if
/// [crate::wrapper::wrap] accepts it and generates correct commands.
#[test]
fn matching_arch() {
    const NUM_RETURNER_RS: &str = include_str!("test_resources/num_returner.rs");

    let (out, tmp) = get_compiled_example(NUM_RETURNER_RS, None);
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

    let cmds = wrap(&experiment, env::consts::ARCH, &conf).unwrap();

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
