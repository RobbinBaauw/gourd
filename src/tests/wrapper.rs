use std::fs;
use std::process::Command;

use crate::config::Config;
use crate::constants::WRAPPER;
use crate::constants::X86_64_E_MACHINE;
#[cfg(target_os = "linux")]
use crate::error::GourdError;
use crate::tests::get_compiled_example;
use crate::wrapper::wrap;
use crate::wrapper::Program;

const X86_64_PRE_PROGRAMMED_BINARY: &str = include_str!("resources/x86_64_pre_programmed.rs");
const ARM_PRE_PROGRAMMED_BINARY: &str = include_str!("resources/arm_pre_programmed.rs");

/// This test will generate an ARM binary and check if [crate::wrapper::wrap] rightfully rejects it.
#[cfg(target_os = "linux")]
#[test]
fn non_matching_arch() {
    Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("thumbv7em-none-eabihf")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let (out, _tmp) = get_compiled_example(
        ARM_PRE_PROGRAMMED_BINARY,
        Some(vec!["--target=thumbv7em-none-eabihf"]),
    );

    match wrap(
        vec![Program {
            binary: out,
            arguments: vec![],
        }],
        vec![],
        X86_64_E_MACHINE,
        &Config::default(),
    ) {
        Err(GourdError::ArchitectureMismatch {
            expected: X86_64_E_MACHINE,
            binary: 40,
        }) => {}

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

    let conf = Config {
        output_path: tmp.path().to_path_buf(),
        metrics_path: tmp.path().to_path_buf(),
    };

    let cmds = wrap(
        vec![Program {
            binary: out,
            arguments: vec![],
        }],
        vec![input.clone()],
        X86_64_E_MACHINE,
        &conf,
    )
    .unwrap();

    assert_eq!(1, cmds.len());
    assert_eq!(
        format!("{:?}", cmds[0]),
        format!(
            "{:?}",
            Command::new(WRAPPER)
                .arg(tmp.path().join("prog").canonicalize().unwrap())
                .arg(input.canonicalize().unwrap())
                .arg(conf.output_path.join("algo_0/0_output"))
                .arg(conf.metrics_path.join("algo_0/0_metrics"))
        )
    );
}
