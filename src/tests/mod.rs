use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use chrono::Local;
use tempdir::TempDir;

use crate::config::Config;
use crate::config::Input;
use crate::config::Program;
use crate::experiment::Environment;
use crate::experiment::Experiment;

mod config;
mod resources;
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

pub fn create_sample_experiment(
    prog: BTreeMap<String, Program>,
    inputs: BTreeMap<String, Input>,
) -> (Experiment, Config) {
    let conf = Config {
        output_path: TempDir::new("output").unwrap().into_path(),
        metrics_path: TempDir::new("metrics").unwrap().into_path(),
        experiments_folder: TempDir::new("experiments").unwrap().into_path(),
        wrapper: "".to_string(),
        programs: prog,
        inputs,
        slurm: None,
    };

    (
        Experiment::from_config(&conf, Environment::Local, Local::now()).unwrap(),
        conf,
    )
}
