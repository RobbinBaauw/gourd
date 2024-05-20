use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use anstyle::AnsiColor;
use anstyle::Color;
use anstyle::Style;
use chrono::Local;
use gourd_lib::config::Config;
use gourd_lib::config::Input;
use gourd_lib::config::Program;
use gourd_lib::constants::style_from_fg;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileSystemInteractor;
use tempdir::TempDir;

use crate::experiments::ExperimentExt;

pub const REAL_FS: FileSystemInteractor = FileSystemInteractor { dry_run: false };

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
        afterscript_output_folder: Some(TempDir::new("after").unwrap().into_path()),
        postprocess_job_output_folder: Some(TempDir::new("post_job").unwrap().into_path()),
    };

    (
        Experiment::from_config(&conf, Environment::Local, Local::now(), &REAL_FS).unwrap(),
        conf,
    )
}

#[test]
fn test_style() {
    assert_eq!(
        style_from_fg(AnsiColor::Red),
        Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
    );
}
