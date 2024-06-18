use std::path::Path;

use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use log::debug;
use log::trace;
/// Assigns a label to a run.
pub fn assign_label(
    experiment: &Experiment,
    source_file: &Path,
    fs: &impl FileOperations,
) -> Result<Option<String>> {
    debug!("Assigning label to {:?}", source_file);

    let mut result_label: Option<String> = None;

    let text = fs.read_utf8(source_file)?;
    if let Some(label_map) = &experiment.config.labels {
        let mut keys = label_map.keys().collect::<Vec<&String>>();
        keys.sort_unstable_by(|a, b| label_map[*b].priority.cmp(&label_map[*a].priority));

        for l in keys {
            let label = &label_map[l];
            if label.regex.is_match(&text) {
                if let Some(ref r) = result_label {
                    trace!("{text} matches multiple labels: {r} and {l}");

                    let overlap = experiment.config.prevent_label_overlap;
                    if let Some(prevent) = overlap {
                        if prevent {
                            bailc!(
                                "Multiple labels match",;
                                "An afterscript should only provide one label",;
                                "The source file {:?} matches multiple labels: {} and {}",
                                source_file, r, l
                            );
                        }
                    }
                } else {
                    trace!("{text} matches {l}");
                    result_label = Some(l.clone());
                }
            }
        }
    }

    Ok(result_label)
}
