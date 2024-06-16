use gourd_lib::file_system::FileSystemInteractor;
use tempdir::TempDir;

use super::*;
#[cfg(feature = "builtin-examples")]
use crate::init::builtin_examples::get_examples;

#[test]
#[cfg(not(feature = "builtin-examples"))]
fn examples_disabled_list_init_examples() {
    let result = list_init_examples();

    assert!(result.is_err());

    assert_eq!(
        "No examples are available.",
        result.unwrap_err().root_cause().to_string()
    );
}

#[test]
#[cfg(feature = "builtin-examples")]
fn examples_enabled_list_init_examples() {
    let result = list_init_examples();

    assert!(result.is_ok());
}

#[test]
#[cfg(not(feature = "builtin-examples"))]
fn examples_disabled_init_example() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let init_directory = parent_directory.join("init_directory");
    let fs = FileSystemInteractor { dry_run: false };

    let id = "";

    let result = init_from_example(id, &init_directory, &fs);

    assert!(result.is_err());

    assert_eq!(
        "Cannot use the -e flag.",
        result.unwrap_err().root_cause().to_string()
    );
}

#[test]
#[cfg(feature = "builtin-examples")]
fn examples_enabled_examples_exist() {
    // The fact that there is at least one valid example is
    // a prerequisite for many tests.
    assert_ne!(0, get_examples().len());
}

#[test]
fn init_experiment_setup_script_mode_no_git() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let init_directory = parent_directory.join("init_directory");

    assert!(!init_directory.exists());

    let git = false;
    let script = true;
    let dry = false;
    let template = None;

    let fs = FileSystemInteractor { dry_run: dry };

    init_experiment_setup(&init_directory, git, script, dry, &template, &fs)
        .expect("Failed to init interactively in a temporary directory.");

    assert!(init_directory.exists());
    assert!(init_directory.is_dir());
}

#[test]
fn init_experiment_setup_script_mode_git() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let mut init_directory = parent_directory.join("init_directory");

    assert!(!init_directory.exists());

    let git = true;
    let script = true;
    let dry = false;
    let template = None;

    let fs = FileSystemInteractor { dry_run: dry };

    init_experiment_setup(&init_directory, git, script, dry, &template, &fs)
        .expect("Failed to init interactively in a temporary directory.");

    assert!(init_directory.exists());
    assert!(init_directory.is_dir());

    init_directory.push(".git");

    assert!(init_directory.exists());
}

#[test]
#[cfg(feature = "builtin-examples")]
fn examples_enabled_init_from_example() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let first_example = get_examples().into_iter().next().expect(
        "There must be at least one valid example if the `builtin-examples` flag is enabled.",
    );
    let id = first_example.0;

    let fs = FileSystemInteractor { dry_run: false };

    let init_directory = parent_directory.join("init_directory");

    assert!(!init_directory.exists());

    init_from_example(id, &init_directory, &fs)
        .expect("Failed to init the example in a temporary directory.");

    assert!(init_directory.exists());
    assert!(init_directory.is_dir());
}

#[test]
fn init_experiment_setup_script_mode_git_dry() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let init_directory = parent_directory.join("init_directory");

    assert!(!init_directory.exists());

    let git = true;
    let script = true;
    let dry = true;
    let template = None;

    let fs = FileSystemInteractor { dry_run: dry };

    init_experiment_setup(&init_directory, git, script, dry, &template, &fs)
        .expect("Failed to init with dry-run in a temporary directory.");

    assert!(!init_directory.exists());
}

#[test]
#[cfg(feature = "builtin-examples")]
fn examples_enabled_init_experiment_setup_example_git() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let mut init_directory = parent_directory.join("init_directory");

    let first_example = get_examples().into_iter().next().expect(
        "There must be at least one valid example if the `builtin-examples` flag is enabled.",
    );

    assert!(!init_directory.exists());

    let git = true;
    let script = true;
    let dry = false;
    let template = Some(first_example.0.to_string());

    let fs = FileSystemInteractor { dry_run: dry };

    init_experiment_setup(&init_directory, git, script, dry, &template, &fs)
        .expect("Failed to init with dry-run in a temporary directory.");

    assert!(init_directory.exists());
    assert!(init_directory.is_dir());

    init_directory.push(".git");

    assert!(init_directory.exists());
}

#[test]
#[cfg(feature = "builtin-examples")]
fn examples_enabled_init_experiment_setup_example_git_dry() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    let init_directory = parent_directory.join("init_directory");

    let first_example = get_examples().into_iter().next().expect(
        "There must be at least one valid example if the `builtin-examples` flag is enabled.",
    );

    assert!(!init_directory.exists());

    let git = true;
    let script = true;
    let dry = true;
    let template = Some(first_example.0.to_string());

    let fs = FileSystemInteractor { dry_run: dry };

    init_experiment_setup(&init_directory, git, script, dry, &template, &fs)
        .expect("Failed to init with dry-run in a temporary directory.");

    assert!(!init_directory.exists());
}

#[test]
fn init_experiment_setup_script_mode_dry_folder_exists() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path();

    assert!(parent_directory.exists());

    let git = false;
    let script = true;
    let dry = true;
    let template = None;

    let fs = FileSystemInteractor { dry_run: dry };

    assert!(init_experiment_setup(parent_directory, git, script, dry, &template, &fs).is_err());
}

#[test]
fn init_experiment_setup_script_mode_dry_no_parent() {
    let tempdir = TempDir::new("my_directory").expect("Could not create the temporary directory.");
    let parent_directory = tempdir.path().join("nonexistent");
    let init_directory = parent_directory.join("init_directory");

    assert!(!init_directory.exists());
    assert!(!parent_directory.exists());

    let git = false;
    let script = true;
    let dry = true;
    let template = None;

    let fs = FileSystemInteractor { dry_run: dry };

    assert!(init_experiment_setup(&init_directory, git, script, dry, &template, &fs).is_err());

    assert!(!parent_directory.exists());
}
