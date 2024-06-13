/// Tarballs of built-in example templates (optional feature)
pub mod builtin_examples;

/// Interactive configuration of an experiment setup template.
mod interactive;

/// Functionality for unpacking template archives.
pub mod template;

/// Functionality for initializing `git` repositories.
mod git;

use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::FileSystemInteractor;
use log::debug;
use log::error;
use log::info;

use crate::init::git::init_template_repository;
use crate::init::interactive::init_interactive;
use crate::init::template::InitTemplate;

/// Initializes an experimental setup.
///
/// If no template is present, it is created interactively.
pub fn init_experiment_setup(
    directory: &Path,
    do_not_use_git: bool,
    script_mode: bool,
    template: Option<InitTemplate>,
    fs: &mut FileSystemInteractor,
) -> Result<()> {
    debug!("Checking the directory at {:?}", directory);

    match directory.parent() {
        // The directory is "/", the next check will fail (already exists)
        None => {}
        // Check that the parent exists
        Some(parent) => {
            debug!("Checking the parent directory at {:?}", parent);

            if !(parent.exists() || parent.eq(Path::new(""))) {
                return Err(anyhow!(format!(
                    "The parent directory, {:?}, does not exist.",
                    parent
                )))
                .with_context(ctx!("", ;
                        "Ensure that you are asking 'gourd init' to \
                        initialize the directory at a valid path.", ));
            }

            debug!("The parent directory, {:?}, is valid", parent)
        }
    }

    if directory.exists() {
        return Err(anyhow!("The path exists."))
            .with_context(ctx!("A file or directory exists at {:?}.", directory ;
                        "Run 'gourd init' for a directory that does not yet exist.", ));
    }

    info!("");
    info!("Creating an experimental setup at {:?}.", directory);

    match template {
        None => init_interactive(directory, script_mode, fs)?,
        Some(t) => t.unpack_to(directory, fs)?,
    }

    if !do_not_use_git {
        debug!("Initialising a Git repository.");
        match init_template_repository(directory) {
            Ok(_) => (),
            Err(e) => {
                error!("Error initialising a Git repository. The experimental setup was still created.");
                return Err(e);
            }
        };
    }

    info!("");
    info!("The experimental setup is ready!");
    info!("To create an experiment, use these commands:");
    info!(" >  cd {:?}", directory);
    info!(" >  gourd run");

    Ok(())
}
