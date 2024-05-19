use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

/// Holds info required to run an afterscript on a job.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfterscriptInfo {
    /// The path to output of the job.
    pub afterscript_path: PathBuf,

    /// The path to place output of afterscript.
    pub afterscript_output_path: PathBuf,
}
