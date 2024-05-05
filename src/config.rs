use crate::error::GourdError;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output_path: PathBuf,
    pub result_path: PathBuf,
}

pub fn load(path: String) -> Result<Config, GourdError> {
    let mut file_contents = String::new();

    let mut file = File::open(&path).map_err(|e| {
        GourdError::ConfigLoadError(
            Some(e),
            format!("Error opening the file '{}'. Ensure that it exists.", path),
        )
    })?;
    file.read_to_string(&mut file_contents).map_err(|e| {
        GourdError::ConfigLoadError(Some(e), format!("Error reading the contents of '{}'", path))
    })?;
    toml::from_str(&file_contents)
        .map_err(|e| GourdError::ConfigLoadError(None, String::from(toml::de::Error::message(&e))))
}
