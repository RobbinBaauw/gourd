use std::path::Path;

use anyhow::Result;
use flate2::read::GzDecoder;
use gourd_lib::config::Config;
use gourd_lib::file_system::FileOperations;
use log::debug;
use log::warn;
use tar::Archive;

/// Stores a template for `gourd init`: a named set of directory contents.
#[derive(Debug, Clone)]
pub struct InitTemplate<'a> {
    /// The template name.
    pub name: &'a str,

    /// The template description.
    pub description: &'a str,

    /// A tar-gz encoded version of the directory structure
    /// of the given template/example, containing all files
    /// within the example directory excluding `gourd.toml`.
    pub directory_tarball: &'a [u8],
}

impl InitTemplate<'_> {
    /// Extracts the template's tarball in the provided directory.
    ///
    /// The directory must have a valid parent, but may not exist.
    /// This is to be enforced by the caller method.
    pub fn unpack_to(&self, directory: &Path, file_system: &impl FileOperations) -> Result<()> {
        let tar = GzDecoder::new(self.directory_tarball);
        let mut archive = Archive::new(tar);

        // Do not preserve the creation time, etc. when unpacking the template.
        archive.set_preserve_mtime(false);

        debug!("Unpacking the example archive");
        file_system.write_archive(directory, archive)?;

        let mut config_path = directory.to_owned();
        config_path.push("gourd.toml");

        debug!("Checking for a \"gourd.toml\" at {:?}.", config_path);
        match Config::from_file(&config_path, file_system) {
            Err(e) => {
                debug!("Configuration check failed: {}", e.root_cause());
                warn!("The \"gourd.toml\" configuration in this example is missing or invalid.");
                warn!("You may have to make some changes.");
            }
            Ok(_) => debug!("A valid \"gourd.toml\" is present."),
        }

        Ok(())
    }
}
