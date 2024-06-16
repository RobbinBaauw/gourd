use std::process::Command;

use clap::crate_version;

#[test]
fn test_gourd_short_version() {
    let gourd = Command::new(env!("CARGO_BIN_EXE_gourd"))
        .arg("version")
        .arg("-s")
        .output()
        .unwrap();

    assert!(gourd.status.success());
    assert!(String::from_utf8(gourd.stdout)
        .unwrap()
        .contains(crate_version!()));
}

#[test]
fn test_gourd_version() {
    let gourd = Command::new(env!("CARGO_BIN_EXE_gourd"))
        .arg("version")
        .output()
        .unwrap();

    assert!(gourd.status.success());
    assert!(String::from_utf8(gourd.stdout)
        .unwrap()
        .contains(crate_version!()));
}
