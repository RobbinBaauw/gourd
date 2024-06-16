use std::path::Path;

use anyhow::Result;
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

    let text = fs.read_utf8(source_file)?;
    if let Some(label_map) = &experiment.config.labels {
        let mut keys = label_map.keys().collect::<Vec<&String>>();
        keys.sort_by(|a, b| label_map[*b].priority.cmp(&label_map[*a].priority));

        for l in keys {
            let label = &label_map[l];
            if label.regex.is_match(&text) {
                return Ok(Some(l.clone()));
            }
        }
    }

    trace!("{text} does not match any labels");
    Ok(None)
}
