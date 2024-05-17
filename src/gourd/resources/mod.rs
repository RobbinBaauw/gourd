use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;
use gourd_lib::ctx;
use gourd_lib::error::Ctx;
use gourd_lib::file_system::write_bytes_truncate;

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
pub fn run_script(arguments: Vec<&str>) -> Result<ExitStatus> {
    let mut command = Command::new("sh");

    command.args(&arguments);

    command
        .spawn()
        .with_context(ctx!("Could not spawn child sh {arguments:?}", ; "",))?
        .wait()
        .with_context(ctx!("Could not wait for script sh {arguments:?}", ; "",))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::remove_file;
    use std::io::Read;
    use std::path::PathBuf;

    use tempdir::TempDir;

    use super::*;

    const PREPROGRAMMED_SH_SCRIPT: &str = r#"
#!/bin/bash
cat <<EOF >filename
first line
second line
third line
EOF
"#;

    #[test]
    fn test_get_resources() {
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join("test.sh");

        let tmp_file = File::create(&file_path).unwrap();
        fs::write(&file_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

        let res = get_resources(vec![&file_path]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);

        drop(tmp_file);
        assert!(tmp_dir.close().is_ok());
    }

    #[test]
    fn test_downloading_from_url() {
        let output_name = "rustup-init.sh";
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join(output_name);

        let tmp_dir_path = PathBuf::from(tmp_dir.path());
        println!("{:?}", tmp_dir_path);

        download_from_url("https://sh.rustup.rs", &tmp_dir_path, output_name).unwrap();

        let mut file = File::open(file_path).expect("could not open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("can't read file contents");

        let text_start: String = contents.chars().take(8).collect();
        assert_eq!("#!/bin/s", text_start);

        assert!(tmp_dir.close().is_ok());
    }

    #[test]
    fn test_sh_script() {
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join("test.sh");

        let tmp_file = File::create(&file_path).unwrap();
        fs::write(&file_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

        let res = run_script(vec!["-C", &(file_path.into_os_string().to_str().unwrap())]);
        assert!(res.is_ok());

        let full_path = PathBuf::from("./filename");
        assert!(Path::exists(&full_path));

        let mut file = File::open(&full_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let text_start: String = contents.chars().take(10).collect();
        assert_eq!("first line", text_start);

        remove_file(&full_path).unwrap();

        drop(tmp_file);
        assert!(tmp_dir.close().is_ok());
    }
}
