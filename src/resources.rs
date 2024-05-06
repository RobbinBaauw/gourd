use std::fs::File;
use std::io::Result;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

use reqwest;

/// gets the files given the filepaths
#[allow(unused)]
pub fn get_resources(filepaths: Vec<&PathBuf>) -> Vec<File> {
    let mut files: Vec<File> = vec![];

    for path in filepaths {
        match get_file(path) {
            Ok(file) => files.push(file),
            Err(_) => println!("{:?} not found :((", path),
        }
    }

    files
}

/// gets a file given the filepath
#[allow(unused)]
pub fn get_file(filepath: &PathBuf) -> Result<File> {
    File::open(filepath)
}

/// downloads a file given a url
#[allow(unused)]
pub fn download_from_url(url: &str, output_dir: &PathBuf, output_name: &str) {
    let response = reqwest::blocking::get(url).expect("failed to get the file");
    let body = response.bytes().expect("file contents are not valid");

    std::fs::create_dir_all(output_dir).expect("directory creation failed");
    println!("{:?}", output_dir.join(output_name));
    std::fs::write(output_dir.join(output_name), &body).expect("writing file failed");
}

/// runs a shell script
#[allow(unused)]
pub fn run_script(arguments: Vec<&str>) -> Result<ExitStatus> {
    let mut command = Command::new("sh");

    for arg in arguments {
        command.arg(arg);
    }

    command.spawn().expect("sh command failed to start").wait()
}
