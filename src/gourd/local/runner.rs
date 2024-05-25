use std::process::Command;

use anyhow::Result;
use futures::future::join_all;
use log::error;
use tokio::task::spawn_blocking;

/// # Multithreaded _local_ runner for tasks
/// (more documentation needed tbh)
pub async fn run_locally(tasks: Vec<Command>) -> Result<()> {
    tokio::spawn(async {
        let task_futures: Vec<_> = tasks
            .into_iter()
            .map(|mut cmd| spawn_blocking(move || cmd.status()))
            .collect();

        // Run all commands concurrently and collect their results
        let results = join_all(task_futures).await;

        for result in results.into_iter() {
            if let Ok(join) = result {
                if let Ok(exit) = join {
                    if !exit.success() {
                        error!("Failed to run gourd wrapper: {exit:?}");
                    }
                } else {
                    error!("Could not retieve the wrappers exit status {join:?}");
                    error!("Ensure that the wrapper is accesible. (see man gourd)");
                }
            } else {
                error!("Could not join the child in the multithreaded runtime");
            }
        }

        Result::<()>::Ok(())
    });

    Ok(())
}

#[cfg(test)]
#[path = "tests/runner.rs"]
mod tests;
