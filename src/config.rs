use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::error::GourdError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub output_path: PathBuf,
    pub result_path: PathBuf,
}
// changing the config struct? see notes in ./tests/config.rs

impl Default for Config {
    fn default() -> Self {
        Config {
            output_path: PathBuf::from("run-output"),
            result_path: PathBuf::from("run-result"),
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Config, GourdError> {
        let mut file_contents = String::new();

        let mut file = File::open(&path).map_err(|e| {
            GourdError::ConfigLoadError(
                Some(e),
                format!("Error opening the file {:?}. Ensure that it exists.", path),
            )
        })?;
        file.read_to_string(&mut file_contents).map_err(|e| {
            GourdError::ConfigLoadError(
                Some(e),
                format!("Error reading the contents of {:?}", path),
            )
        })?;
        toml::from_str(&file_contents).map_err(|e| {
            GourdError::ConfigLoadError(None, String::from(toml::de::Error::message(&e)))
        })
    }
}
