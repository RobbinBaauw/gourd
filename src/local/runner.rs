use std::process::Command;
use std::process::ExitStatus;

use futures::future::join_all;
use tokio::runtime;
use tokio::task::spawn_blocking;

use crate::error::GourdError;

/// # Multithreaded _local_ runner for tasks
/// (more documentation needed tbh)
<<<<<<< HEAD
#[allow(dead_code, unused)]
pub fn run_locally(tasks: Vec<Command>) -> Result<Vec<ExitStatus>, GourdError> {
    let rt = runtime::Runtime::new().map_err(GourdError::IoError)?;
=======
pub fn run_locally(tasks: Vec<Command>) -> Result<Vec<ExitStatus>, GourdError> {
    let rt = runtime::Runtime::new().unwrap();
>>>>>>> d1aa084 (implement (correctly) the threaded local runner + 1 integration test for it)

    rt.block_on(async {
        let task_futures: Vec<_> = tasks
            .into_iter()
            .map(|mut cmd| {
                spawn_blocking(move || {
                    let status = cmd.status();
                    match status {
                        Ok(status) => status,
                        Err(err) => panic!("Could not execute the child (runner.rs): {}", err),
                    }
                })
            })
            .collect();
<<<<<<< HEAD
        // Run all commands concurrently and collect their results
        let results = join_all(task_futures).await;
        let mut output = vec![];
        for result in results.into_iter() {
=======
        // .collect::<Vec<dyn Future<Output=ExitStatus>>>();
        // Run all commands concurrently and collect their results
        let results = join_all(task_futures).await;
        let mut output = vec![];
        for (_i, result) in results.into_iter().enumerate() {
>>>>>>> d1aa084 (implement (correctly) the threaded local runner + 1 integration test for it)
            match result {
                Ok(status) => output.push(status),
                Err(err) => return Err(GourdError::ChildJoinError(err)),
            }
        }
        Ok(output)
    })
}
