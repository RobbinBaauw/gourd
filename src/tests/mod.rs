use std::fs;
use std::process::Command;

use tempdir::TempDir;

mod config;
mod runner;
mod wrapper;

pub fn get_compiled_example(
    contents: &str,
    extra_args: Option<Vec<&str>>,
) -> (std::path::PathBuf, TempDir) {
    let tmp = TempDir::new("match").unwrap();

    let source = tmp.path().join("prog.rs");
    let out = tmp.path().join("prog");

    fs::write(&source, contents).unwrap();

    let mut cmd = Command::new("rustc");
    cmd.arg(source.canonicalize().unwrap()).arg("-o").arg(&out);
    if let Some(extra_args) = extra_args {
        cmd.args(extra_args);
    }
    cmd.spawn().unwrap().wait().unwrap();

    (out, tmp)
}
