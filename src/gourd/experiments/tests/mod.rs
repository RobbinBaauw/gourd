use std::time::Duration;

use gourd_lib::config::ResourceLimits;
use tempdir::TempDir;

use super::*;
use crate::test_utils::REAL_FS;

#[test]
fn config_correct_slurm() {
    let tempdir = TempDir::new("tests").unwrap();
    let mut config: Config = Config::from_file(
        Path::new("src/gourd/experiments/tests/test_resources/config_correct_slurm.toml"),
        &REAL_FS,
    )
    .unwrap();
    config.output_path = PathBuf::from(tempdir.path());
    config.metrics_path = PathBuf::from(tempdir.path());
    config.experiments_folder = PathBuf::from(tempdir.path());

    let time = Local::now();

    let result = Experiment::from_config(&config, time, &REAL_FS);
    assert!(result.is_ok());

    let runs = vec![Run {
        program: "a".to_string(),
        input: "b".to_string(),
        err_path: PathBuf::from(tempdir.path())
            .join("1/a/b_error")
            .canonicalize()
            .unwrap(),
        output_path: PathBuf::from(tempdir.path())
            .join("1/a/b_output")
            .canonicalize()
            .unwrap(),
        metrics_path: PathBuf::from(tempdir.path())
            .join("1/a/b_metrics")
            .canonicalize()
            .unwrap(),
        slurm_id: None,
        afterscript_output_path: None,
        post_job_output_path: None,
    }];

    let test_experiment = Experiment {
        runs,
        chunks: vec![],
        resource_limits: Some(ResourceLimits {
            time_limit: Duration::new(60, 0),
            cpus: 1,
            mem_per_cpu: 512,
        }),
        creation_time: time,
        config,
        seq: 1,
    };

    assert_eq!(result.unwrap(), test_experiment);
}

#[test]
fn config_correct_local() {
    let tempdir = TempDir::new("tests").unwrap();
    let mut config: Config = Config::from_file(
        Path::new("src/gourd/experiments/tests/test_resources/config_correct_local.toml"),
        &REAL_FS,
    )
    .unwrap();
    config.output_path = PathBuf::from(tempdir.path());
    config.metrics_path = PathBuf::from(tempdir.path());
    config.experiments_folder = PathBuf::from(tempdir.path());

    let time = Local::now();

    let result = Experiment::from_config(&config, time, &REAL_FS);
    assert!(result.is_ok());

    let runs = vec![
        Run {
            program: "b".to_string(),
            input: "d".to_string(),
            err_path: PathBuf::from(tempdir.path())
                .join("1/b/d_error")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/b/d_output")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/b/d_metrics")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: "b".to_string(),
            input: "e".to_string(),
            err_path: PathBuf::from(tempdir.path())
                .join("1/b/e_error")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/b/e_output")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/b/e_metrics")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: "c".to_string(),
            input: "d".to_string(),
            err_path: PathBuf::from(tempdir.path())
                .join("1/c/d_error")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/c/d_output")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/c/d_metrics")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: "c".to_string(),
            input: "e".to_string(),
            err_path: PathBuf::from(tempdir.path())
                .join("1/c/e_error")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/c/e_output")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/c/e_metrics")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
    ];

    let test_experiment = Experiment {
        runs,
        chunks: vec![],
        resource_limits: None,
        creation_time: time,
        config,
        seq: 1,
    };

    assert_eq!(result.unwrap(), test_experiment);
}

#[test]
fn latest_id_invalid_folder() {
    let error = Experiment::latest_id_from_folder(Path::new("INVALID PATH :)"));
    assert!(error.is_err());
    assert_eq!(
        format!("{}", error.unwrap_err().root_cause()),
        "No such file or directory (os error 2)"
    );
}

#[test]
fn latest_id_correct() {
    let tempdir = TempDir::new("tests").unwrap();
    let mut config: Config = Config::from_file(
        Path::new("src/gourd/experiments/tests/test_resources/config_id_testing.toml"),
        &REAL_FS,
    )
    .unwrap();
    config.output_path = PathBuf::from(tempdir.path());
    config.metrics_path = PathBuf::from(tempdir.path());
    config.experiments_folder = PathBuf::from(tempdir.path());

    Experiment::from_config(&config, Local::now(), &REAL_FS).unwrap();
    Experiment::from_config(&config, Local::now(), &REAL_FS).unwrap();

    let id = Experiment::latest_id_from_folder(tempdir.path()).unwrap();
    assert_eq!(id, Some(2));
}
