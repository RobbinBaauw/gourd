use std::path::PathBuf;

use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::config::Config;
use gourd_lib::config::Program;
use gourd_lib::config::ResourceLimits;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use gourd_lib::file_system::FileOperations;
use gourd_lib::file_system::FileSystemInteractor;
use log::debug;
use log::trace;

use crate::cli::printing::query_update_resource_limits;

/// Get all simple and postprocess programs to update limits.
pub fn get_setlim_programs(config: &Config) -> Result<Vec<String>> {
    let mut programs = config
        .programs
        .iter()
        .map(|(x, _)| x.clone())
        .collect::<Vec<String>>();

    if let Some(list) = &config.postprocess_programs {
        let mut post = list.iter().map(|(x, _)| x.clone()).collect::<Vec<String>>();

        programs.append(&mut post);
    }

    Ok(programs)
}

/// Query the user for the resource limits of the programs for a list of
/// programs.
pub fn query_changing_limits_for_all_programs(
    experiment: &mut Experiment,
    limits: ResourceLimits,
) -> Result<()> {
    let programs = get_setlim_programs(&experiment.config)?;

    for name in programs {
        let mut program = get_program_from_name(experiment, &name)?;
        program.resource_limits = Some(limits);
    }

    Ok(())
}

/// Query the user for the resource limits of the programs for a list of
/// programs.
pub fn query_changing_limits_for_program(
    name: &String,
    experiment: &mut Experiment,
    mem: &Option<usize>,
    cpu: &Option<usize>,
    time: &Option<std::time::Duration>,
) -> Result<()> {
    let mut program = get_program_from_name(experiment, name)?;

    let old_rss = program.resource_limits.unwrap_or_default();
    let new_rss = query_update_resource_limits(&old_rss, mem, cpu, time)?;

    program.resource_limits = Some(new_rss);

    debug!("Updating resource limits for program {}", name);
    trace!("Old resource limits: {:?}", program.resource_limits);
    trace!("New resource limits: {:?}", new_rss);

    Ok(())
}

/// Gets the program by checking if it is a postprocess or a regular program.
pub fn get_program_from_name(experiment: &Experiment, name: &String) -> Result<Program> {
    if experiment.config.programs.contains_key(name) {
        Ok(experiment.config.programs[name].clone())
    } else {
        let post = &experiment.config.postprocess_programs;

        if post.is_some() && post.clone().unwrap().contains_key(name) {
            Ok(post.clone().unwrap()[name].clone())
        } else {
            bailc!("No program found with the name {:?}", name);
        }
    }
}

/// Get resource limits from a toml file.
pub fn get_limits_from_toml(path: PathBuf, fs: FileSystemInteractor) -> Result<ResourceLimits> {
    let data = fs.try_read_toml::<ResourceLimits>(&path)?;

    Ok(data)
}
