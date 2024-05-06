use std::fs;
use std::process::Command;

use tempdir::TempDir;

use crate::config::Config;
use crate::constants::WRAPPER;
use crate::constants::X86_64_E_MACHINE;
#[cfg(target_os = "linux")]
use crate::error::GourdError;
use crate::wrapper::wrap;
use crate::wrapper::Program;

const X86_64_PREPROGRAMED_BINARY: &str = r#"
use std::io;

fn main() {
  let mut inpt = String::new();
  io::stdin()
      .read_line(&mut inpt)
      .expect("Failed to read line");

  let num: i32 = inpt.trim().parse().unwrap();

  println!("{}", num);
}
"#;

#[cfg(target_os = "linux")]
const ARM_PREPROGRAMED_BINARY: &str = r#"
#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
"#;

/// This test will generate a ARM binary and check if [crate::wrapper::wrap] rightfully rejects it.
#[test]
#[cfg(target_os = "linux")]
fn unmatching_arch() {
    let tmp = TempDir::new("unmatch").unwrap();

    let source = tmp.path().join("prog.rs");
    let out = tmp.path().join("prog");
    fs::write(&source, ARM_PREPROGRAMED_BINARY).unwrap();

    Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("thumbv7em-none-eabihf")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("rustc")
        .arg("--target")
        .arg("thumbv7em-none-eabihf")
        .arg(source.canonicalize().unwrap())
        .arg("-o")
        .arg(out)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    match wrap(
        vec![Program {
            binary: tmp.path().join("prog"),
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
                "Did not return the correct architechure mismatch, was: {:?}",
                e
            );
        }
    }
}

/// This test will generate a X86 binary and check if [crate::wrapper::wrap]
/// accepts it and generates correct commands.
#[test]
fn matching_arch() {
    let tmp = TempDir::new("match").unwrap();

    let source = tmp.path().join("prog.rs");
    let out = tmp.path().join("prog");

    let input = tmp.path().join("test1");

    fs::write(&source, X86_64_PREPROGRAMED_BINARY).unwrap();
    fs::write(&input, "4").unwrap();

    Command::new("rustc")
        .arg(source.canonicalize().unwrap())
        .arg("-o")
        .arg(out)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let conf = Config {
        output_path: tmp.path().to_path_buf(),
        metrics_path: tmp.path().to_path_buf(),
    };

    let cmds = wrap(
        vec![Program {
            binary: tmp.path().join("prog"),
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
