use std::fs;
use std::fs::remove_file;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;

use tempdir::TempDir;

use crate::resources;

const PREPROGRAMMED_SH_SCRIPT: &str = r#"
#!/bin/bash
cat <<EOF >filename
first line
second line
third line
EOF
"#;

#[test]
fn test_get_file() {
    let tmp_dir = TempDir::new("testing").unwrap();
    let file_path = tmp_dir.path().join("test.sh");

    let tmp_file = File::create(&file_path).unwrap();
    fs::write(&file_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

    let res = resources::get_file(&file_path);
    assert!(res.is_ok());

    drop(tmp_file);
    assert!(tmp_dir.close().is_ok());
}

#[test]
fn test_get_resources() {
    let tmp_dir = TempDir::new("testing").unwrap();
    let file_path = tmp_dir.path().join("test.sh");

    let tmp_file = File::create(&file_path).unwrap();
    fs::write(&file_path, PREPROGRAMMED_SH_SCRIPT).unwrap();

    let res = resources::get_resources(vec![&file_path, &file_path.join("2")]);
    assert_eq!(res.len(), 1);

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

    resources::download_from_url("https://sh.rustup.rs", &tmp_dir_path, output_name);

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

    let res: Result<ExitStatus, std::io::Error> =
        resources::run_script(vec!["-C", &(file_path.into_os_string().to_str().unwrap())]);
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
