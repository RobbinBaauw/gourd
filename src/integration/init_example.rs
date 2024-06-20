#![cfg(feature = "builtin-examples")]

use std::io::BufRead;

use crate::gourd;
use crate::init;

#[test]
fn test_first_example() {
    let env = init();
    // Run `gourd init --list-examples`
    let output = gourd!(env; "init", "--list-examples"; "listing examples");
    assert!(output.status.success());

    // Get the first line: "experiment-id" - Experiment Name
    let first_line = output
        .stdout
        .lines()
        .map_while(Result::ok)
        .find(|line| !line.is_empty())
        .expect("No non-empty lines of output from `gourd init --list-examples`");

    // Get the first experiment-id
    let experiment_id = first_line
        .split('"')
        .nth(1)
        .expect("Could not get ID from within quotation marks (\")");

    // Initialize an experiment in the directory
    // Also tests that the default behavior uses Git
    let init_dir = env.temp_dir.path().join("init_test");
    let _output = gourd!(env; "init", "-e", experiment_id, &init_dir.to_str().unwrap());
    assert!(init_dir.exists());
    assert!(init_dir.join(".git").exists());

    // Initialize the experiment again in a different directory
    // But this time, do not use Git
    let init_dir_no_git = env.temp_dir.path().join("init_test2");
    let _output =
        gourd!(env; "init", "--git=false", "-e", experiment_id, &init_dir_no_git.to_str().unwrap());
    assert!(init_dir_no_git.exists());
    assert!(!init_dir_no_git.join(".git").exists());
}
