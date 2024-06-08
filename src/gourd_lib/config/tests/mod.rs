extern crate tempdir;

use std::collections::BTreeMap;
use std::fs::File;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs::Permissions;
use std::fs::{self};
use std::io::Write;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tempdir::TempDir;

use crate::config::maps::InputMap;
use crate::config::maps::ProgramMap;
use crate::config::Config;
use crate::config::Input;
use crate::test_utils::REAL_FS;

/// This test will fail if the semantics of the config struct are changed.
/// If this is the case, update the documentation and make sure that the
/// rest of the application reflects these changes.
#[test]
fn breaking_changes_config_struct() {
    #[allow(clippy::unnecessary_operation)]
    Config {
        output_path: PathBuf::from(""),
        metrics_path: PathBuf::from(""),
        experiments_folder: PathBuf::from(""),
        wrapper: "".to_string(),
        inputs: InputMap::default(),
        programs: ProgramMap::default(),
        postprocess_programs: None,
        slurm: None,
        resource_limits: None,
        postprocess_resource_limits: None,
        afterscript_output_folder: None,
        postprocess_job_output_folder: None,
        labels: Some(BTreeMap::new()),
    };
}

/// This test will fail if the semantics of the config file are changed.
/// See above. Is this a valid reason for the user to update their old files?
/// If you add something to the struct, add it here too.
#[test]
fn breaking_changes_config_file_all_values() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
        output_path = "./ginger_root"
        metrics_path = "./vulfpeck/"
        experiments_folder = "./parcels/"
        wrapper = "gourd_wrapper"
        afterscript_output_folder = "./after/"
        postprocess_job_output_folder = "./post_job/"

        [programs]

        [inputs]
    "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert_eq!(
        Config {
            output_path: PathBuf::from("./ginger_root/"),
            metrics_path: PathBuf::from("./vulfpeck"),
            experiments_folder: PathBuf::from("./parcels/"),
            wrapper: "gourd_wrapper".to_string(),
            inputs: InputMap::default(),
            programs: ProgramMap::default(),
            postprocess_programs: None,
            slurm: None,
            resource_limits: None,
            postprocess_resource_limits: None,
            afterscript_output_folder: Some(PathBuf::from("./after/")),
            postprocess_job_output_folder: Some(PathBuf::from("./post_job/")),
            labels: None,
        },
        Config::from_file(file_pathbuf.as_path(), &REAL_FS).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

/// This test will fail if the semantics of all REQUIRED values in the config
/// file are changed. See above. If you add something to the struct, add it here
/// too.
#[test]
fn breaking_changes_config_file_required_values() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
        output_path = "./ginger_root"
        metrics_path = "./vulfpeck/"
        experiments_folder = ""

        [inputs]

        [programs]
    "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert_eq!(
        Config {
            output_path: PathBuf::from("./ginger_root/"),
            metrics_path: PathBuf::from("./vulfpeck"),
            experiments_folder: PathBuf::from(""),
            wrapper: "gourd_wrapper".to_string(),
            inputs: InputMap::default(),
            programs: ProgramMap::default(),
            postprocess_programs: None,
            slurm: None,
            resource_limits: None,
            postprocess_resource_limits: None,
            afterscript_output_folder: None,
            postprocess_job_output_folder: None,
            labels: None,
        },
        Config::from_file(file_pathbuf.as_path(), &REAL_FS).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

#[test]
fn config_nonexistent_file() {
    let dir = TempDir::new("config_folder").unwrap();
    let file_pathbuf = dir.path().join("file.toml");

    if Config::from_file(file_pathbuf.as_path(), &REAL_FS).is_ok() {
        panic!("Error expected.")
    }

    dir.close().unwrap();
}

// Tests on a file without read permissions.
// The test does not run on Windows because we access Unix-style permissions
// here.
#[test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn config_unreadable_file() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");
    let file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.set_permissions(Permissions::from_mode(0o000))
        .expect("Could not set permissions of 'unreadable' test file to 000.");

    if Config::from_file(file_pathbuf.as_path(), &REAL_FS).is_ok() {
        panic!("Error expected.")
    }
    dir.close().unwrap();
}

#[test]
fn config_unparseable_file() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    File::create(file_pathbuf.as_path()).expect("A file folder could not be created.");

    if Config::from_file(file_pathbuf.as_path(), &REAL_FS).is_ok() {
        panic!("Error expected.")
    }
    dir.close().unwrap();
}

#[test]
fn config_ok_file() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let conf = Config::default();
    let out =
        toml::to_string(&conf).expect("The default Config could not be serialized for testing.");

    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(out.as_bytes())
        .expect("The test file could not be written.");

    assert_eq!(
        conf,
        Config::from_file(file_pathbuf.as_path(), &REAL_FS).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

#[test]
fn disallow_glob_names() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
            experiments_folder = ""

            [inputs.test_glob_]
            arguments = []

            [programs]
        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert!(
        format!("{:?}", Config::from_file(file_pathbuf.as_path(), &REAL_FS)).contains("_glob_")
    );
    dir.close().unwrap();
}

#[test]
fn test_globs() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let in_pathbuf = dir.path().join("test.in");

    fs::write(&in_pathbuf, "asd").unwrap();

    let config_contents = format!(
        r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
            experiments_folder = ""

            [inputs.test_blob]
            arguments = ["-f", "glob|{}/*.in"]

            [programs]
        "#,
        dir.path().to_str().unwrap()
    );

    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    let mut inputs = InputMap::default();

    inputs.insert(
        "test_blob_glob_0".to_string(),
        Input {
            input: None,
            arguments: vec!["-f".to_string(), in_pathbuf.to_str().unwrap().to_string()],
        },
    );

    assert_eq!(
        Config {
            output_path: PathBuf::from("./ginger_root/"),
            metrics_path: PathBuf::from("./vulfpeck"),
            experiments_folder: PathBuf::from(""),
            wrapper: "gourd_wrapper".to_string(),
            inputs,
            programs: ProgramMap::default(),
            postprocess_programs: None,
            slurm: None,
            resource_limits: None,
            postprocess_resource_limits: None,
            afterscript_output_folder: None,
            postprocess_job_output_folder: None,
            labels: None,
        },
        Config::from_file(file_pathbuf.as_path(), &REAL_FS).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

#[test]
fn test_globs_invalid_pattern() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
            experiments_folder = ""

            [inputs.test_blob]
            arguments = ["-f", "glob|***"]

            [programs]
        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert!(
        format!("{:?}", Config::from_file(file_pathbuf.as_path(), &REAL_FS))
            .contains("could not expand")
    );
}

#[test]
fn test_regex_that_do_match() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");
    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
            experiments_folder = ""
            [label.stefan]
            regex = "[a-zA-Z0-9]+ loves stefan"
            priority = 42
            [programs]
            [inputs]
        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    let config =
        Config::from_file(file_pathbuf.as_path(), &REAL_FS).expect("Unexpected config read error.");
    assert!(
        regex_lite::Regex::new(config.labels.unwrap().get("stefan").unwrap().regex.as_str())
            .unwrap()
            .is_match("18C loves stefan")
    );
}

#[test]
fn test_invalid_regex_gives_error() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");
    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
            experiments_folder = ""
            [label.stefan]
            regex = "{{{{{{{{{{{{{{{{ i didnt pass acc"
            priority = 42
            [programs]
            [inputs]
        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert!(Config::from_file(file_pathbuf.as_path(), &REAL_FS).is_err());
}
