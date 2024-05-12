use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::constants::PRIMARY_STYLE;
use crate::constants::WRAPPER_DEFAULT;
use crate::error::ctx;
use crate::error::Ctx;
use crate::file_system::read_utf8;
use crate::slurm::SlurmConfig;

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

    /// The additional cli arguments for the executable.
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
    /// The path to a folder where the experiment output will be stored.
    pub output_path: PathBuf,

    /// The path to a folder where the metrics output will be stored.
    pub metrics_path: PathBuf,

    /// The path to a folder where the experiments will be stored.
    pub experiments_folder: PathBuf,

    /// The list of tested algorithms.
    pub programs: BTreeMap<String, Program>,

    /// The list of inputs for each of them.
    pub inputs: BTreeMap<String, Input>,

    /// If running on a SLURM cluster, the job configurations
    pub slurm_config: Option<SlurmConfig>,

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
            experiments_folder: PathBuf::from("experiments"),
            wrapper: WRAPPER_DEFAULT(),
            programs: BTreeMap::new(),
            inputs: BTreeMap::new(),
            slurm_config: None,
        }
    }
}

impl Config {
    /// Load a `Config` struct instance from a TOML file at the provided path.
    /// Returns a valid `Config` or an explanatory `GourdError::ConfigLoadError`.
    pub fn from_file(path: &Path) -> Result<Config> {
        toml::from_str(&read_utf8(path)?).with_context(ctx!(
          "Could not parse {path:?}", ;
          "More help and examples can be found with {PRIMARY_STYLE}man gourd.toml{PRIMARY_STYLE:#}",
        ))
    }
}
