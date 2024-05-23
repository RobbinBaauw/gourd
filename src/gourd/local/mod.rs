use std::env;

use anyhow::Result;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;

use self::runner::run_locally;
use crate::wrapper::wrap;

/// The (first iteration) thread pool implementation
pub mod runner;

/// Run an experiment locally, as specified in the config file.
pub async fn run_local(experiment: &Experiment, fs: &impl FileOperations) -> Result<()> {
    let cmds = wrap(experiment, env::consts::ARCH, fs)?;

    run_locally(cmds).await?;

    Ok(())
}
