use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;

use crate::error::ctx;
use crate::error::Ctx;
use crate::file_system::write_bytes_truncate;

/// Gets the files given the filepaths.
#[allow(unused)]
pub fn get_resources(filepaths: Vec<&Path>) -> Result<Vec<File>> {
    let mut files: Vec<File> = vec![];

    for path in filepaths {
        files.push(File::open(path).with_context(ctx!(
          "Could not open resource file {path:?}", ;
          "Ensure that the file exists",
        ))?);
    }

    Ok(files)
}

/// Downloads a file given a url.
#[allow(unused)]
pub fn download_from_url(url: &str, output_dir: &Path, output_name: &str) -> Result<()> {
    let response = reqwest::blocking::get(url).with_context(ctx!(
      "Could not access the resource at {url}", ;
      "Check that the url is correct",
    ))?;

    let body = response.bytes().with_context(ctx!(
        "Could not parse the resource at {url}", ;
        "Check that the url is not misspelled",
    ))?;

    write_bytes_truncate(&output_dir.join(output_name), &body);

    Ok(())
}

/// Runs a shell script.
#[allow(unused)]
pub fn run_script(arguments: Vec<&str>) -> Result<ExitStatus> {
    let mut command = Command::new("sh");

    command.args(&arguments);

    command
        .spawn()
        .with_context(ctx!("Could not spawn child sh {arguments:?}", ; "",))?
        .wait()
        .with_context(ctx!("Could not wait for script sh {arguments:?}", ; "",))
}
