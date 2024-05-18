use std::process::Command;
use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;
use futures::future::join_all;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use tokio::runtime;
use tokio::task::spawn_blocking;

/// # Multithreaded _local_ runner for tasks
/// (more documentation needed tbh)
pub fn run_locally(tasks: Vec<Command>) -> Result<Vec<ExitStatus>> {
    let rt = runtime::Runtime::new()
        .with_context(ctx!("Could not start the multithreaded runtime", ; "",))?;

    rt.block_on(async {
        let task_futures: Vec<_> = tasks
            .into_iter()
            .map(|mut cmd| spawn_blocking(move || cmd.status()))
            .collect();

        // Run all commands concurrently and collect their results
        let results = join_all(task_futures).await;

        let mut output = vec![];

        for result in results.into_iter() {
            output.push(
                result
                    .with_context(
                        ctx!("Could not join the child in the multithreaded runtime", ; "",),
                    )?
                    .with_context(ctx!("Could not retieve the wrappers exit status", ; "",))?,
            )
        }

        Ok(output)
    })
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::local::runner::run_locally;
    use crate::test_utils::get_compiled_example;

    /// Run a naive fibonacci implementation using the local runner,
    /// assert that they run correctly
    /// Note: while there are no assertions for it, the outputs
    /// should be in increasing order of computation time!
    #[test]
    fn runner_fibonacci_test() {
        // Ensure binary is built
        let (out, _tmp) = get_compiled_example(include_str!("test_resources/fibonacci.rs"), None);

        let test_cases = vec![38u128, 36u128, 34u128, 30u128, 24u128];
        let mut commands: Vec<Command> = vec![];
        for value in test_cases {
            let mut cmd = Command::new(&out);
            cmd.arg(value.to_string());
            commands.push(cmd);
        }

        let results = run_locally(commands);

        assert!(results.is_ok(), "Executing children failed");
        for r in results.unwrap() {
            assert!(r.success(), "Couldn't execute child, test failed");
        }
    }

    /// Test sleeping in the thread pool (don't drown tho)
    #[test]
    fn runner_sleep_test() {
        let mut commands: Vec<Command> = vec![];
        for value in [4, 3, 2, 1, 2, 3] {
            let mut cmd = Command::new("sleep");
            cmd.arg(value.to_string());
            commands.push(cmd);
        }

        let results = run_locally(commands);

        assert!(results.is_ok(), "Executing children failed");
        for r in results.unwrap() {
            assert!(r.success(), "Couldn't execute child, test failed");
        }
    }
}
