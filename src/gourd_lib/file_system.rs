use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use log::debug;
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ctx;
use crate::error::Ctx;

/// Interactor with the actual physical file system.
#[derive(Clone, Copy, Debug)]
pub struct FileSystemInteractor {
    /// If true this will not write nor store any state to the file system.
    pub dry_run: bool,
}

/// This defines all interactions of gourd with the filesystem.
pub trait FileOperations {
    /// Read a file into raw bytes.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;

    /// Read a file into a utf8 string.
    fn read_utf8(&self, path: &Path) -> Result<String>;

    /// Try to deserialize a toml file into a struture `T`.
    fn try_read_toml<T: DeserializeOwned>(&self, path: &Path) -> Result<T>;

    /// Try to serialize a struct `T` into a toml file.
    fn try_write_toml<T: Serialize>(&self, path: &Path, data: &T) -> Result<()>;

    /// Wirte all bytes to a file.
    fn write_bytes_truncate(&self, path: &Path, bytes: &[u8]) -> Result<()>;

    /// Wirte a [String] to a file.
    fn write_utf8_truncate(&self, path: &Path, data: &str) -> Result<()>;

    /// Create the file and all parent directories.
    fn truncate_and_canonicalize(&self, path: &Path) -> Result<PathBuf>;
}

impl FileOperations for FileSystemInteractor {
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).with_context(ctx!(
          "Could not read the file {path:?}", ;
          "Ensure that the file exists and you have permissions to access it",
        ))
    }

    fn read_utf8(&self, path: &Path) -> Result<String> {
        String::from_utf8(self.read_bytes(path)?).with_context(ctx!(
          "{path:?} is not valid UTF-8", ;
          "The file doesn't seem to be human readable?",
        ))
    }

    fn try_read_toml<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        toml::from_str::<T>(&self.read_utf8(path)?).with_context(ctx!(
          "Could not deserialize toml file {path:?}", ;
          "Ensure that the file is valid toml",
        ))
    }

    fn try_write_toml<T: Serialize>(&self, path: &Path, data: &T) -> Result<()> {
        self.write_utf8_truncate(
            path,
            &toml::to_string::<T>(data).with_context(ctx!(
              "Could not serialize toml file {path:?}", ;
              "Ensure that the struct is valid toml",
            ))?,
        )
    }

    fn write_utf8_truncate(&self, path: &Path, data: &str) -> Result<()> {
        self.write_bytes_truncate(path, data.as_bytes())
    }

    fn write_bytes_truncate(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        if self.dry_run {
            info!("Would have written to {path:?} (dry)");
            return Ok(());
        }

        fs::write(self.truncate_and_canonicalize(path)?, bytes).with_context(ctx!(
          "Could not write to the file {path:?}", ;
          "Ensure that you have permissions to write it",
        ))?;

        Ok(())
    }

    fn truncate_and_canonicalize(&self, path: &Path) -> Result<PathBuf> {
        if self.dry_run {
            if let Some(parent) = path.parent() {
                debug!("Would have created {parent:?} (dry)");
            }

            debug!("Would have created {path:?} (dry)");
            return Ok(path.to_path_buf());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(ctx!(
              "Could not create parent directories for {parent:?}", ;
              "Ensure that you have sufficient permissions",
            ))?;
        }

        File::create(path).with_context(ctx!(
           "Could not create {path:?}", ;
           "Ensure that you have sufficient permissions",
        ))?;

        path.canonicalize().with_context(ctx!(
          "Could not canonicalize {path:?}", ;
          "Ensure that your path is valid",
        ))
    }
}
