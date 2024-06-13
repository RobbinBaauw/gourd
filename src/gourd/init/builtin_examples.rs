#![cfg(feature = "builtin-examples")]

use anyhow::Result;

use crate::init::template::InitTemplate;

/// Retrieves a named example experiment, or [None].
///
///
/// The result type is used here for eventual support of reading examples from files.
pub fn get_example(id: &str) -> Result<Option<InitTemplate>> {
    match id {
        "a-simple-experiment" => Ok(Some(InitTemplate {
            name: "A Simple Experiment",
            description: "A comparative evaluation of two simple programs.",

            directory_tarball: include_bytes!("tarballs/a-simple-experiment.tar.gz"),
        })),

        _ => Ok(None),
    }
}
