use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;

use tempdir::TempDir;

use crate::afterscript;
use crate::afterscript::run_afterscript_for_run;
use crate::status::Completion;
use crate::status::FailureReason;
use crate::status::PostprocessCompletion;
use crate::status::PostprocessOutput;
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
fn test_filter_runs_for_afterscript() {
    let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
    runs.insert(0, Some(Status::NoPostprocessing(Completion::Success)));
    runs.insert(
        1,
        Some(Status::AfterScript(
            Completion::Success,
            PostprocessCompletion::Dormant,
        )),
    );
    runs.insert(
        2,
        Some(Status::AfterScript(
            Completion::Fail(FailureReason::UserForced),
            PostprocessCompletion::Dormant,
        )),
    );
    runs.insert(
        3,
        Some(Status::AfterScript(
            Completion::Success,
            PostprocessCompletion::Success(PostprocessOutput {
                short_output: String::from("short"),
                long_output: String::from("long"),
            }),
        )),
    );

    let res = afterscript::filter_runs_for_afterscript(&runs).unwrap();

    assert!(res.len() == 1);

    let paths = res[0];
    assert!(*paths == 1);
}

#[test]
fn test_run_afterscript_for_run() {
    let tmp_dir = TempDir::new("testing").unwrap();

    let results_path = tmp_dir.path().join("results.toml");
    let results_file = File::create(&results_path).unwrap();
    fs::write(&results_path, PREPROGRAMMED_RESULTS).unwrap();

    let afterscript_path = tmp_dir.path().join("afterscript.sh");
    let afterscript_file = File::create(&afterscript_path).unwrap();
    fs::write(&afterscript_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

    let output_path = tmp_dir.path().join("afterscript_output.toml");

    let info = afterscript::AfterscriptInfo {
        afterscript_path: afterscript_path.clone(),
        afterscript_output_path: output_path.clone(),
    };
    assert!(run_afterscript_for_run(&info, &results_path).is_ok());

    let mut contents = String::new();
    assert!(File::open(output_path)
        .unwrap()
        .read_to_string(&mut contents)
        .is_ok());
    assert_eq!(contents, PREPROGRAMMED_RESULTS.to_ascii_uppercase());

    drop(results_file);
    drop(afterscript_file);

    assert!(tmp_dir.close().is_ok());
}
