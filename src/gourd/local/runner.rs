use std::process;
use std::process::Command;

use anyhow::Result;
use futures::future::join_all;
use log::error;
use tokio::task::spawn_blocking;

/// Run a list of tasks locally in a multithreaded way.
pub async fn run_locally(tasks: Vec<Command>) -> Result<()> {
    #[cfg(not(tarpaulin_include))] // Tarpaulin can't calculate the coverage correctly
    tokio::spawn(async {
        let task_futures: Vec<_> = tasks
            .into_iter()
            .map(|mut cmd| spawn_blocking(move || cmd.output()))
            .collect();

        // Run all commands concurrently and collect their results
        let results = join_all(task_futures).await;

        for result in results.into_iter() {
            if let Ok(join) = result {
                if let Ok(exit) = join {
                    if !exit.status.success() {
                        error!("Failed to run gourd wrapper: {:?}", exit.status);
                        error!(
                            "Wrapper returned: {}",
                            String::from_utf8(exit.stderr).unwrap()
                        );
                        process::exit(1);
                    }
                } else {
                    error!("Couldn't start the wrapper: {join:?}");
                    error!("Ensure that the wrapper is accesible. (see man gourd)");
                    process::exit(1);
                }
            } else {
                error!("Could not join the child in the multithreaded runtime");
                process::exit(1);
            }
        }

        Result::<()>::Ok(())
    });

    Ok(())
}

#[cfg(test)]
#[path = "tests/runner.rs"]
mod tests;
