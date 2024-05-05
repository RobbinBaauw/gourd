mod config;
mod wrapper;

use std::env;
use std::path::Path;
use std::process::Command;

use crate::local::runner::run_locally;

/// Run a naive fibonacci implementation using the local runner,
/// assert that they run correctly
/// Note: while there are no assertions for it, the outputs
/// should be in increasing order of computation time!
#[test]
fn runner_fibonacci_test() {
    // Ensure binary is built
    assert!(Command::new("cargo")
        .args(["build", "--bin", "fibonacci_example"])
        .status()
        .unwrap()
        .success());

    // get the binary we want to execute
    let fib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/fibonacci_example")
        .to_path_buf();
    assert!(
        fib_path.exists(),
        "example (fibonacci) binary not found at path: {:?}",
        fib_path
    );

    let test_cases = vec![35u128, 36u128, 38u128, 33u128, 32u128];
    // make an std::process::Command for each test case
    let mut commands: Vec<Command> = vec![];
    for value in test_cases {
        let mut cmd = Command::new(fib_path.clone());
        cmd.arg(value.to_string());
        commands.push(cmd);
    }

    let results = run_locally(commands);

    assert!(results.is_ok(), "Executing children failed");
    for r in results.unwrap() {
        assert!(r.success(), "Couldn't execute child, test failed");
    }
}
