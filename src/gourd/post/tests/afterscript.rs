use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileSystemInteractor;
use gourd_lib::measurement::Measurement;
use tempdir::TempDir;

use crate::experiments::ExperimentExt;
use crate::post::afterscript::add_label_to_run;
use crate::post::afterscript::filter_runs_for_afterscript;
use crate::post::afterscript::run_afterscript_for_run;
use crate::post::afterscript::Config;
use crate::post::afterscript::PostprocessCompletion;
use crate::status::FileSystemBasedStatus;
use crate::status::FsState;
use crate::status::PostprocessOutput;
use crate::status::SlurmBasedStatus;
use crate::status::SlurmState;
use crate::status::Status;

const PREPROGRAMMED_SH_SCRIPT: &str = r#"
#!/bin/sh
tr '[a-z]' '[A-Z]' <$1 >$2
"#;

const PREPROGRAMMED_RESULTS: &str = r#"[package]
name = "gourd"
version = "0.1.5-alpha"
edition = "2021"

[dependencies]
"#;

#[test]
fn test_filter_runs_for_afterscript_good_weather() {
    let mut runs: BTreeMap<usize, Status> = BTreeMap::new();
    runs.insert(
        0,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: crate::status::FsState::Pending,
                afterscript_completion: Some(PostprocessCompletion::Dormant),
                postprocess_job_completion: None,
            },
            slurm_status: None,
        },
    );
    runs.insert(
        1,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 0,
                    rusage: None,
                }),
                afterscript_completion: Some(PostprocessCompletion::Dormant),
                postprocess_job_completion: None,
            },
            slurm_status: Some(SlurmBasedStatus {
                completion: SlurmState::Success,
                exit_code_program: 0,
                exit_code_slurm: 0,
            }),
        },
    );
    runs.insert(
        2,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 1,
                    rusage: None,
                }),
                afterscript_completion: Some(PostprocessCompletion::Dormant),
                postprocess_job_completion: None,
            },
            slurm_status: None,
        },
    );
    runs.insert(
        3,
        Status {
            fs_status: FileSystemBasedStatus {
                completion: FsState::Completed(Measurement {
                    wall_micros: Duration::from_nanos(0),
                    exit_code: 0,
                    rusage: None,
                }),
                afterscript_completion: Some(PostprocessCompletion::Success(PostprocessOutput {
                    short_output: String::from("short"),
                    long_output: String::from("long"),
                })),
                postprocess_job_completion: None,
            },
            slurm_status: None,
        },
    );

    let res = filter_runs_for_afterscript(&runs);

    assert_eq!(res.len(), 1);

    let paths = res[0];
    assert_eq!(*paths, 1);
}

#[test]
fn test_run_afterscript_for_run_good_weather() {
    let tmp_dir = TempDir::new("testing").unwrap();

    let results_path = tmp_dir.path().join("results.toml");
    let results_file = fs::File::create(&results_path).unwrap();
    fs::write(&results_path, PREPROGRAMMED_RESULTS).unwrap();

    let afterscript_path = tmp_dir.path().join("afterscript.sh");
    let afterscript_file = fs::File::create(&afterscript_path).unwrap();
    fs::write(&afterscript_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

    let output_path = tmp_dir.path().join("afterscript_output.toml");

    assert!(run_afterscript_for_run(&afterscript_path, &results_path, &output_path).is_ok());

    let mut contents = String::new();
    assert!(fs::File::open(output_path)
        .unwrap()
        .read_to_string(&mut contents)
        .is_ok());
    assert_eq!(contents, PREPROGRAMMED_RESULTS.to_ascii_uppercase());

    drop(results_file);
    drop(afterscript_file);

    assert!(tmp_dir.close().is_ok());
}

#[test]
fn test_run_afterscript_for_run_bad_weather() {
    let tmp_dir = TempDir::new("testing").unwrap();

    let results_path = tmp_dir.path().join("results.toml");
    let results_file = File::create(&results_path).unwrap();
    fs::write(&results_path, PREPROGRAMMED_RESULTS).unwrap();

    let output_path = tmp_dir.path().join("afterscript_output.toml");

    assert!(run_afterscript_for_run(&PathBuf::from(""), &results_path, &output_path).is_err());

    drop(results_file);

    assert!(tmp_dir.close().is_ok());
}

#[test]
fn test_add_label_to_run() {
    let fs = FileSystemInteractor { dry_run: true };
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pb = dir.path().join("file.toml");
    let config_contents = r#"
             output_path = "./goose"
             metrics_path = "./🪿/"
             experiments_folder = "/tmp/gourd/experiments/"
             afterscript_output_folder = "/tmp/gourd/after/"
             [programs.a]
             binary = "/bin/sleep"
             arguments = []
             afterscript = "/bin/echo"
             [inputs.b]
             arguments = ["1"]
             [inputs.c]
             arguments = ["2"]
             [label.found_hello]
             priority = 0
             regex = "hello"
             [label.found_world]
             priority = 1
             regex = "world"
         "#;
    let mut file = File::create(file_pb.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");
    let mut after_file =
        File::create(dir.path().join("after.txt")).expect("A file could not be created.");
    after_file
        .write_all("hello".as_bytes())
        .expect("The test file could not be written.");

    let conf = Config::from_file(file_pb.as_path(), &fs).unwrap();
    let exp =
        Experiment::from_config(&conf, chrono::Local::now(), Environment::Local, &fs).unwrap();
    let mut labels = BTreeMap::new();
    assert!(conf.labels.is_some());
    add_label_to_run(0, &mut labels, &exp, dir.path().join("after.txt"), &fs)
        .expect("tested fn failed");
    assert_eq!(labels.get(&0).unwrap(), "found_hello");
    after_file
        .write_all("hello world".as_bytes())
        .expect("The test file could not be written.");
    add_label_to_run(0, &mut labels, &exp, dir.path().join("after.txt"), &fs)
        .expect("tested fn failed");
    assert_eq!(labels.get(&0).unwrap(), "found_world");
}
