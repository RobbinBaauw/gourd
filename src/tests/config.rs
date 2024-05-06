extern crate tempdir;

use std::fs::File;
use std::fs::Permissions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tempdir::TempDir;

use crate::config::Config;
use crate::error::GourdError;

#[test]
/// This test will fail if the semantics of the config struct are changed.
/// If this is the case, update the documentation and make sure that the
/// rest of the application reflects these changes.
fn breaking_changes_config_struct() {
    #[allow(clippy::unnecessary_operation)]
    Config {
        output_path: PathBuf::from(""),
        metrics_path: PathBuf::from(""),
    };
}

#[test]
/// This test will fail if the semantics of the config file are changed.
/// See above. Is this a valid reason for the user to update their old files?
/// If you add something to the struct, add it here too.
fn breaking_changes_config_file_all_values() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"

        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert_eq!(
        Config {
            output_path: PathBuf::from("./ginger_root/"),
            metrics_path: PathBuf::from("./vulfpeck")
        },
        Config::from_file(file_pathbuf.as_path()).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

#[test]
/// This test will fail if the semantics of all REQUIRED values in the config file are changed.
/// See above. If you add something to the struct, add it here too.
fn breaking_changes_config_file_required_values() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    let config_contents = r#"
            output_path = "./ginger_root"
            metrics_path = "./vulfpeck/"
        "#;
    let mut file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.write_all(config_contents.as_bytes())
        .expect("The test file could not be written.");

    assert_eq!(
        Config {
            output_path: PathBuf::from("./ginger_root/"),
            metrics_path: PathBuf::from("./vulfpeck")
        },
        Config::from_file(file_pathbuf.as_path()).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}

#[test]
fn config_nonexistent_file() {
    let dir = TempDir::new("config_folder").unwrap();
    let file_pathbuf = dir.path().join("file.toml");

    match Config::from_file(file_pathbuf.as_path()) {
        Ok(_) => panic!("Error expected."),
        Err(GourdError::ConfigLoadError(_, _)) => (),
        Err(_) => panic!("Incorrect error type."),
    }

    dir.close().unwrap();
}

#[test]
fn config_unreadable_file() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");
    let file = File::create(file_pathbuf.as_path()).expect("A file could not be created.");
    file.set_permissions(Permissions::from_mode(0o000))
        .expect("Could not set permissions of 'unreadable' test file to 000.");

    match Config::from_file(file_pathbuf.as_path()) {
        Ok(_) => panic!("Error expected."),
        Err(GourdError::ConfigLoadError(_, _)) => (),
        Err(_) => panic!("Incorrect error type."),
    }
    dir.close().unwrap();
}

#[test]
fn config_unparseable_file() {
    let dir = TempDir::new("config_folder").expect("A temp folder could not be created.");
    let file_pathbuf = dir.path().join("file.toml");

    File::create(file_pathbuf.as_path()).expect("A file folder could not be created.");

    match Config::from_file(file_pathbuf.as_path()) {
        Ok(_) => panic!("Error expected."),
        Err(GourdError::ConfigLoadError(_, _)) => (),
        Err(_) => panic!("Incorrect error type."),
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
        Config::from_file(file_pathbuf.as_path()).expect("Unexpected config read error.")
    );
    dir.close().unwrap();
}
