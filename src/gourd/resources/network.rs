use std::fs::File;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::FileOperations;

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
pub fn download_from_url(
    url: &str,
    output_dir: &Path,
    output_name: &str,
    fs: &impl FileOperations,
) -> Result<()> {
    let response = reqwest::blocking::get(url).with_context(ctx!(
      "Could not access the resource at {url}", ;
      "Check that the url is correct",
    ))?;

    let body = response.bytes().with_context(ctx!(
        "Could not parse the resource at {url}", ;
        "Check that the url is not misspelled",
    ))?;

    fs.write_bytes_truncate(&output_dir.join(output_name), &body);

    Ok(())
}

#[cfg(test)]
#[path = "tests/network.rs"]
mod tests;
