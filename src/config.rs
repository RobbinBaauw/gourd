use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::error::GourdError;

/// A config struct used throughout the `gourd` application.
//
// changing the config struct? see notes in ./tests/config.rs
// 1. is the change necessary?
// 2. will it break user workflows?
// 3. update the tests
// 4. update the user documentation
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// The path to an existing folder where the experiment output will be stored.
    pub output_path: PathBuf,

    /// The path to an existing folder where the metrics output will be stored.
    pub metrics_path: PathBuf,
}

// An implementation that provides a default value of `Config`,
// which allows for the eventual addition of optional config items.
impl Default for Config {
    fn default() -> Self {
        Config {
            output_path: PathBuf::from("run-output"),
            metrics_path: PathBuf::from("run-metrics"),
        }
    }
}

impl Config {
    /// Load a `Config` struct instance from a TOML file at the provided path.
    /// Returns a valid `Config` or an explanatory `GourdError::ConfigLoadError`.
    pub fn from_file(path: &Path) -> Result<Config, GourdError> {
        let mut file_contents = String::new();

        let mut file = File::open(path).map_err(|e| {
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
