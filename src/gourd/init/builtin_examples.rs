#![cfg(feature = "builtin-examples")]

use std::collections::BTreeMap;

use crate::init::template::InitTemplate;

/// Retrieves all available examples.
pub fn get_examples() -> BTreeMap<&'static str, InitTemplate<'static>> {
    let mut examples = BTreeMap::new();

    examples.insert(
        "a-simple-experiment",
        InitTemplate {
            name: "A Simple Experiment",
            description: "A comparative evaluation of two simple programs.",

            directory_tarball: include_bytes!(concat!(
                env!("OUT_DIR"),
                "/../../../tarballs/a-simple-experiment.tar.gz"
            )),
        },
    );

    examples
}

/// Retrieves a named example experiment, or [None].
///
///
/// The result type is used here for eventual support of reading examples from
/// files.
pub fn get_example(id: &str) -> Option<InitTemplate<'static>> {
    get_examples().get(id).cloned()
}
