use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::constants::WRAPPER_DEFAULT;
use crate::error::GourdError;

/// A pair of a path to a binary and cli arguments.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Program {
    /// The path to the executable.
    pub binary: PathBuf,

    /// The cli arguments for the executable.
    pub arguments: Vec<String>,
}

/// A pair of a path to an input and additional cli arguments.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Input {
    /// The path to the input.
    pub input: PathBuf,

    /// The additonal cli arguments for the executable.
    pub arguments: Vec<String>,
}

/// A config struct used throughout the `gourd` application.
//
// changing the config struct? see notes in ./tests/config.rs
// 1. is the change necessary?
// 2. will it break user workflows?
// 3. update the tests
// 4. update the user documentation
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    //
    // Basic settings.
    //
    /// The path to an existing folder where the experiment output will be stored.
    pub output_path: PathBuf,

    /// The path to an existing folder where the metrics output will be stored.
    pub metrics_path: PathBuf,

    /// The list of tested algorithms.
    pub programs: BTreeMap<String, Program>,

    /// The list of inputs for each of them.
    pub runs: BTreeMap<String, Input>,

    //
    // Advanced settings.
    //
    /// The command to execute to get to the wrapper.
    #[serde(default = "WRAPPER_DEFAULT")]
    pub wrapper: String,
}

// An implementation that provides a default value of `Config`,
// which allows for the eventual addition of optional config items.
impl Default for Config {
    fn default() -> Self {
        Config {
            output_path: PathBuf::from("run-output"),
            metrics_path: PathBuf::from("run-metrics"),
            wrapper: WRAPPER_DEFAULT(),
            programs: BTreeMap::new(),
            runs: BTreeMap::new(),
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
