use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::config::Program;
use gourd_lib::config::ResourceLimits;
use gourd_lib::constants::CMD_STYLE;
use gourd_lib::constants::TERTIARY_STYLE;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::experiment::Experiment;
use log::debug;
use log::trace;

use crate::cli::printing::query_update_resource_limits;

/// Get all simple and postprocess programs to update limits.
pub fn get_setlim_programs(experiment: &Experiment) -> Result<Vec<String>> {
    let mut programs = experiment
        .config
        .programs
        .iter()
        .map(|(x, _)| x.clone())
        .collect::<Vec<String>>();

    if let Some(list) = &experiment.config.postprocess_programs {
        let mut post = list.iter().map(|(x, _)| x.clone()).collect::<Vec<String>>();

        programs.append(&mut post);
    }

    Ok(programs)
}

/// Query the user for the resource limits of the programs for a list of
/// programs.
pub fn query_changing_limits_for_all_programs(
    experiment: &mut Experiment,
    new_rss: ResourceLimits,
    old_rss: &ResourceLimits,
) -> Result<()> {
    let programs = get_setlim_programs(experiment)?;

    for name in programs {
        let program = get_program_from_name(experiment, &name)?;
        program.resource_limits = Some(new_rss);
    }

    experiment.resource_limits = Some(new_rss);

    debug!("Updating resource limits for all programs");
    trace!("Old resource limits: {:?}", old_rss);
    trace!("New resource limits: {:?}", new_rss);

    Ok(())
}

/// Query the user for the resource limits of the programs for a list of
/// programs.
pub fn query_changing_limits_for_program(
    name: &String,
    experiment: &mut Experiment,
    mem: Option<usize>,
    cpu: Option<usize>,
    time: Option<std::time::Duration>,
) -> Result<()> {
    let base_resources = experiment.resource_limits.unwrap_or_default();

    let program = get_program_from_name(experiment, name)?;

    let old_rss = match program.resource_limits {
        Some(inner) => inner,
        None => base_resources,
    };

    let new_rss = query_update_resource_limits(&old_rss, mem, cpu, time)?;

    program.resource_limits = Some(new_rss);

    debug!(
        "Updating resource limits for program {}. They will take effect next time
        {CMD_STYLE}gourd continue{CMD_STYLE:#} is called.",
        name
    );
    trace!("Old resource limits: {:?}", old_rss);
    trace!("New resource limits: {:?}", new_rss);

    Ok(())
}

/// Gets the program by checking if it is a postprocess or a regular program.
pub fn get_program_from_name<'a>(
    experiment: &'a mut Experiment,
    name: &String,
) -> Result<&'a mut Program> {
    let available_programs = get_setlim_programs(experiment)?;

    if experiment.config.programs.contains_key(name) {
        Ok(experiment.config.programs.get_mut(name).unwrap())
    } else if let Some(post) = &mut experiment.config.postprocess_programs {
        if post.contains_key(name) {
            Ok((*post).get_mut(name).unwrap())
        } else {
            bailc!(
                "No program found with the name {:?}", name;
                "", ;
                "Available programs are: {TERTIARY_STYLE}{}{TERTIARY_STYLE:#}", available_programs.join(", ")
            );
        }
    } else {
        bailc!(
            "No program found with the name {:?}", name;
            "", ;
            "Available programs are: {TERTIARY_STYLE}{}{TERTIARY_STYLE:#}", available_programs.join(", ")
        )
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
