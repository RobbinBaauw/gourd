use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::error::ctx;
use crate::error::Ctx;

/// Read a file into raw bytes.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(ctx!(
      "Could not read the file {path:?}", ;
      "Ensure that the file exists and you have permissions to access it",
    ))
}

/// Read a file into a utf8 string.
pub fn read_utf8(path: &Path) -> Result<String> {
    String::from_utf8(read_bytes(path)?).with_context(ctx!(
      "{path:?} is not valid UTF-8", ;
      "The file doesn't seem to be human readable?",
    ))
}

/// Try to deserialize a toml file into a structure `T`.
pub fn try_read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    toml::from_str::<T>(&read_utf8(path)?).with_context(ctx!(
      "Could not deserialize toml file {path:?}", ;
      "Ensure that the file is valid toml",
    ))
}

/// Try to deserialize a toml string into a structure `T`.
pub fn try_read_toml_string<T: DeserializeOwned>(s: &String) -> Result<T> {
    toml::from_str::<T>(s).with_context(ctx!(
      "Could not deserialize toml `{s:?}`", ;
      "Ensure that the text is valid toml",
    ))
}

/// Wirte all bytes to a file.
pub fn write_bytes_truncate(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(truncate_and_canonicalize(path)?, bytes).with_context(ctx!(
      "Could not write to the file {path:?}", ;
      "Ensure that you have permissions to write it",
    ))?;

    Ok(())
}

/// Create the file and all parent directories.
pub fn truncate_and_canonicalize(path: &Path) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(ctx!(
          "Could not create parent directories for {parent:?}", ;
          "Enusre that you have suffcient permissions",
        ))?;
    }

    File::create(path).with_context(ctx!(
       "Could not create {path:?}", ;
       "Enusre that you have suffcient permissions",
    ))?;

    path.canonicalize().with_context(ctx!(
      "Could not canonicalize {path:?}", ;
      "Enusre that your path is valid",
    ))
}
