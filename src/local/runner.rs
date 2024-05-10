use std::process::Command;
use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;
use futures::future::join_all;
use tokio::runtime;
use tokio::task::spawn_blocking;

use crate::error::ctx;
use crate::error::Ctx;

/// # Multithreaded _local_ runner for tasks
/// (more documentation needed tbh)
#[allow(dead_code, unused)]
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
