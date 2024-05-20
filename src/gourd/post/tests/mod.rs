use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use tempdir::TempDir;

use super::*;
use crate::post::afterscript::run_afterscript_for_run;
use crate::post::postprocess_job::filter_runs_for_post_job;
use crate::status::Completion;
use crate::status::FailureReason;
use crate::status::PostprocessOutput;

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
fn test_filter_runs_for_post_job() {
    let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
    runs.insert(
        0,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: None,
            postprocess_job_completion: None,
        }),
    );
    runs.insert(
        1,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: None,
            postprocess_job_completion: Some(PostprocessCompletion::Dormant),
        }),
    );
    runs.insert(
        2,
        Some(Status {
            completion: Completion::Fail(FailureReason::UserForced),
            afterscript_completion: None,
            postprocess_job_completion: Some(PostprocessCompletion::Dormant),
        }),
    );
    runs.insert(
        3,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: None,
            postprocess_job_completion: Some(PostprocessCompletion::Success(PostprocessOutput {
                short_output: String::from("short"),
                long_output: String::from("long"),
            })),
        }),
    );

    let res = filter_runs_for_post_job(&mut runs).unwrap();

    assert_eq!(res.len(), 1);

    let paths = res[0];
    assert_eq!(*paths, 1);
}

#[test]
fn test_filter_runs_for_afterscript_good_weather() {
    let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
    runs.insert(
        0,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: None,
            postprocess_job_completion: None,
        }),
    );
    runs.insert(
        1,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: Some(PostprocessCompletion::Dormant),
            postprocess_job_completion: None,
        }),
    );
    runs.insert(
        2,
        Some(Status {
            completion: Completion::Fail(FailureReason::UserForced),
            afterscript_completion: Some(PostprocessCompletion::Dormant),
            postprocess_job_completion: None,
        }),
    );
    runs.insert(
        3,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: Some(PostprocessCompletion::Success(PostprocessOutput {
                short_output: String::from("short"),
                long_output: String::from("long"),
            })),
            postprocess_job_completion: None,
        }),
    );

    let res = filter_runs_for_afterscript(&runs).unwrap();

    assert_eq!(res.len(), 1);

    let paths = res[0];
    assert_eq!(*paths, 1);
}

#[test]
fn test_filter_runs_for_afterscript_bad_weather() {
    let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
    runs.insert(
        0,
        Some(Status {
            completion: Completion::Success,
            afterscript_completion: None,
            postprocess_job_completion: None,
        }),
    );
    runs.insert(1, None);

    assert!(filter_runs_for_afterscript(&runs).is_err());
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
