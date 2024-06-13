use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use git2::Repository;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use log::info;

pub fn init_template_repository(path: &Path) -> Result<()> {
    Repository::init(path).with_context(ctx!("Error initialising a Git repository.", ;
                            "You can use '--no-git' to skip this.", ))?;
    info!("Successfully created a Git repository");
    Ok(())
}
