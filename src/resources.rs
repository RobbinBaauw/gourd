use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use reqwest;

use crate::error::GourdError;

/// gets the files given the filepaths
#[allow(unused)]
pub fn get_resources(filepaths: Vec<&PathBuf>) -> Result<Vec<File>, GourdError> {
    let mut files: Vec<File> = vec![];

    for path in filepaths {
        files.push(File::open(path).map_err(|x| GourdError::FileError(path.clone(), x))?);
    }

    Ok(files)
}

/// downloads a file given a url
#[allow(unused)]
pub fn download_from_url(
    url: &str,
    output_dir: &PathBuf,
    output_name: &str,
) -> Result<(), GourdError> {
    let response = reqwest::blocking::get(url).map_err(GourdError::NetworkError)?;
    let body = response.bytes().map_err(GourdError::NetworkError)?;

    std::fs::create_dir_all(output_dir)
        .map_err(|x| GourdError::FileError(output_dir.clone(), x))?;
    std::fs::write(output_dir.join(output_name), &body)
        .map_err(|x| GourdError::FileError(output_dir.clone(), x))?;

    Ok(())
}

/// runs a shell script
#[allow(unused)]
pub fn run_script(arguments: Vec<&str>) -> Result<ExitStatus, GourdError> {
    let mut command = Command::new("sh");

    command.args(arguments);

    command
        .spawn()
        .map_err(GourdError::ChildSpawnError)?
        .wait()
        .map_err(GourdError::ChildSpawnError)
}
