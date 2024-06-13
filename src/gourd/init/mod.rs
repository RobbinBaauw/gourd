/// Interactive configuration of an experiment setup template.
mod interactive;

/// Functionality for unpacking template archives.
pub mod template;

/// Tarballs of built-in example templates (optional feature)
pub mod builtin_examples;

use std::path::Path;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::FileOperations;
use log::debug;
use log::error;
use log::info;

use crate::init::interactive::init_interactive;
use crate::init::template::InitTemplate;

/// Initializes an experimental setup.
///
/// If no template is present, it is created interactively.
pub fn init_experiment_setup(
    directory: &Path,
    do_not_use_git: bool,
    script_mode: bool,
    dry_run: bool,
    template: Option<InitTemplate>,
    fs: &impl FileOperations,
) -> Result<()> {
    debug!("Checking the directory at {:?}", directory);

    match directory.parent() {
        // The directory is "/", the next check will fail (already exists)
        None => {}
        // Check that the parent exists
        Some(parent) => {
            debug!("Checking the parent directory at {:?}", parent);

            if !(parent.exists() || parent.eq(Path::new(""))) {
                bailc!(
                  "The parent directory, {:?}, does not exist.", parent;
                  "", ;
                  "Ensure that you are asking 'gourd init' to \
                  initialize the directory at a valid path.",
                )
            }

            debug!("The parent directory, {:?}, is valid", parent)
        }
    }

    if directory.exists() {
        bailc!(
          "The path exists.", ;
          "A file or directory exists at {:?}.", directory;
          "Run 'gourd init' for a directory that does not yet exist.",
        )
    }

    info!("Creating an experimental setup at {:?}.", directory);

    match template {
        None => init_interactive(directory, script_mode, fs)?,
        Some(t) => {
            if dry_run {
                info!("Would have unpacked the example (dry)");
            } else {
                t.unpack_to(directory, fs)?
            }
        }
    }

    if !do_not_use_git {
        debug!("Initialising a Git repository.");
        match fs.init_template_repository(directory) {
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

    // Add the CMD styling after merging `gourd cancel`
    info!(" >  cd {:?}", directory);
    info!(" >  gourd run");

    Ok(())
}
