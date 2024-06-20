use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use gourd_lib::bailc;
use gourd_lib::config::Config;
use gourd_lib::config::SlurmConfig;
use gourd_lib::constants::CMD_STYLE;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::FileOperations;
use inquire::error::InquireResult;
use inquire::validator::ValueRequiredValidator;
use inquire::Confirm;
use inquire::CustomType;
use inquire::InquireError;
use inquire::Text;
use log::debug;
use log::info;

/// Correctly handles when the user cancels the operation
/// during an Inquire prompt.
pub fn ask<T>(inq: InquireResult<T>) -> Result<T> {
    match inq {
        Ok(answer) => Ok(answer),
        Err(InquireError::OperationCanceled) => bailc!(
                    "The operation was canceled using the ESC key.", ; "",; "",),
        Err(InquireError::OperationInterrupted) => {
            bailc!("The operation was interrupted using Ctrl+C.", ; "",; "",)
        }
        other => other.with_context(ctx!("Could not print a prompt", ; "", )),
    }
}

/// Initializes an experimental setup using interactive prompts.
pub fn init_interactive(
    directory: &Path,
    script_mode: bool,
    fs: &impl FileOperations,
) -> Result<()> {
    if !script_mode {
        info!("");
        info!("The following questions will help you customize a `gourd.toml` experimental setup.");
        info!("Use {CMD_STYLE}gourd init -s{CMD_STYLE:#} to skip customization.");
        info!("");
    }

    let mut config = Config {
        output_path: PathBuf::from("output"),
        metrics_path: PathBuf::from("metrics"),
        experiments_folder: PathBuf::from("experiments"),
        programs: Default::default(),
        inputs: Default::default(),
        parameters: None,
        slurm: None,
        resource_limits: None,
        postprocess_resource_limits: None,
        wrapper: Default::default(),
        postprocess_programs: None,
        labels: None,
        input_schema: None,
    };

    let slurm = if script_mode {
        false
    } else {
        ask(Confirm::new("Include options for Slurm?")
            .with_help_message("These will allow the experiment to run on a cluster computer.")
            .prompt())?
    };

    if slurm {
        let mut slurm_config = SlurmConfig {
            experiment_name: "".to_string(),
            partition: "".to_string(),
            array_count_limit: 0,
            array_size_limit: 0,
            out: None,
            account: "".to_string(),
            begin: None,
            mail_type: None,
            mail_user: None,
            additional_args: None,
        };

        slurm_config.experiment_name = if script_mode {
            "my-experiment".to_string()
        } else {
            ask(Text::new("Slurm experiment name: ")
                .with_validator(ValueRequiredValidator::default())
                .with_help_message("This will be used to name jobs submitted to Slurm.")
                .prompt())?
        };

        slurm_config.array_count_limit = if script_mode {
            10
        } else {
            ask(CustomType::new("Slurm array count limit: ")
                .with_formatter(&|num: usize| format!("{}", num))
                .with_default(10)
                .with_help_message("The number of job arrays that can be scheduled at once.")
                .prompt())?
        };

        slurm_config.array_size_limit = if script_mode {
            1000
        } else {
            ask(CustomType::new("Slurm array size limit: ")
                .with_formatter(&|num: usize| format!("{}", num))
                .with_default(1000)
                .with_help_message("The number of runs that can be scheduled in one job array.")
                .prompt())?
        };

        let enter_slurm_data_now = if script_mode {
            false
        } else {
            ask(Confirm::new("Enter Slurm credentials now?")
                .with_help_message(
                    "Choosing 'no' will leave the 'account' and 'partition' blank for now.",
                )
                .prompt())?
        };

        if enter_slurm_data_now {
            slurm_config.account = if script_mode {
                "".to_string()
            } else {
                ask(Text::new("Slurm account to use: ")
                    .with_validator(ValueRequiredValidator::default())
                    .with_help_message(
                        "This should be provided by the administrator of your supercomputer.",
                    )
                    .prompt())?
            };

            slurm_config.partition = if script_mode {
                "".to_string()
            } else {
                ask(Text::new("Slurm partition to use: ")
                    .with_help_message("Most supercomputers use this to choose types of nodes.")
                    .prompt())?
            };
        }

        config.slurm = Some(slurm_config);
    }

    write_files(directory, config, fs)
}

/// Write all files during initialization.
pub fn write_files(directory: &Path, config: Config, fs: &impl FileOperations) -> Result<()> {
    if directory.exists() {
        bailc!(
            "The path exists.", ;
            "A directory or file exists at {directory:?}.", ;
            "Choose a path that is not already taken.",
        );
    }

    let canonical_directory = fs.truncate_and_canonicalize_folder(directory)?;

    let mut toml_path = canonical_directory.to_path_buf();
    toml_path.push("gourd.toml");

    debug!("Creating `gourd.toml` file at {:?}.", &toml_path);
    fs.try_write_toml(&toml_path, &config)?;

    let dirs_to_create = vec![
        Some(config.output_path),
        Some(config.metrics_path),
        Some(config.experiments_folder),
    ];

    debug!("Creating experiment folders.");
    for dir in dirs_to_create.into_iter().flatten() {
        if dir.is_relative() {
            let mut fs_dir_path = canonical_directory.to_path_buf();
            fs_dir_path.push(dir);

            fs.truncate_and_canonicalize_folder(&fs_dir_path)?;
        } else {
            debug!("Skipping non-relative path {:?}", &dir)
        }
    }

    Ok(())
}
