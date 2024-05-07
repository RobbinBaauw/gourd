use self::runner::run_locally;
use crate::config::Config;
use crate::constants::X86_64_E_MACHINE;
use crate::error::GourdError;
use crate::wrapper::wrap;

/// The (first iteration) thread pool implementation
pub mod runner;

/// Run an experiment locally, as specified in the config file.
pub fn run_local(config: &Config) -> Result<(), GourdError> {
    let cmds = wrap(&config.programs, &config.runs, X86_64_E_MACHINE, config)?;

    run_locally(cmds)?;

    Ok(())
}
