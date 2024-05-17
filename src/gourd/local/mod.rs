use std::env;

use anyhow::Result;
use gourd_lib::config::Config;
use gourd_lib::constants::E_MACHINE_MAPPING;
use gourd_lib::experiment::Experiment;

use self::runner::run_locally;
use crate::wrapper::wrap;

/// The (first iteration) thread pool implementation
pub mod runner;

/// Run an experiment locally, as specified in the config file.
pub fn run_local(config: &Config, experiment: &Experiment) -> Result<()> {
    let cmds = wrap(experiment, E_MACHINE_MAPPING(env::consts::ARCH), config)?;

    run_locally(cmds)?;

    Ok(())
}
