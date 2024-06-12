use std::env;
use std::path::Path;

use anyhow::Result;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;

use self::runner::run_locally;
use crate::slurm::chunk::Chunkable;
use crate::wrapper::wrap;

/// The (first iteration) thread pool implementation.
pub mod runner;

/// Run an experiment locally, as specified in the config file.
pub async fn run_local(
    experiment: &mut Experiment,
    exp_path: &Path,
    fs: &impl FileOperations,
    force: bool,
) -> Result<()> {
    experiment.chunks = experiment.create_chunks(usize::MAX, 1, 0..experiment.runs.len())?;
    experiment.save(&experiment.config.experiments_folder, fs)?;

    let cmds = wrap(experiment, exp_path, env::consts::ARCH, fs)?;

    run_locally(cmds, force).await?;

    Ok(())
}
