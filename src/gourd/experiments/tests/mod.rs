use std::collections::BTreeMap;
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

    let result = Experiment::from_config(&config, time, Environment::Local, &REAL_FS);
    assert!(result.is_ok());

    let runs = vec![Run {
        program: ProgramRef::Regular("a".to_string()),
        input: InputRef::Regular("b".to_string()),
        err_path: PathBuf::from(tempdir.path())
            .join("1/a/error_b")
            .canonicalize()
            .unwrap(),
        output_path: PathBuf::from(tempdir.path())
            .join("1/a/output_b")
            .canonicalize()
            .unwrap(),
        metrics_path: PathBuf::from(tempdir.path())
            .join("1/a/metrics_b")
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
        env: Environment::Local,
        postprocess_inputs: BTreeMap::new(),
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

    let result = Experiment::from_config(&config, time, Environment::Local, &REAL_FS);
    assert!(result.is_ok());

    let runs = vec![
        Run {
            program: ProgramRef::Regular("b".to_string()),
            input: InputRef::Regular("d".to_string()),
            err_path: PathBuf::from(tempdir.path())
                .join("1/b/error_d")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/b/output_d")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/b/metrics_d")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: ProgramRef::Regular("b".to_string()),
            input: InputRef::Regular("e".to_string()),
            err_path: PathBuf::from(tempdir.path())
                .join("1/b/error_e")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/b/output_e")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/b/metrics_e")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: ProgramRef::Regular("c".to_string()),
            input: InputRef::Regular("d".to_string()),
            err_path: PathBuf::from(tempdir.path())
                .join("1/c/error_d")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/c/output_d")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/c/metrics_d")
                .canonicalize()
                .unwrap(),
            slurm_id: None,
            afterscript_output_path: None,
            post_job_output_path: None,
        },
        Run {
            program: ProgramRef::Regular("c".to_string()),
            input: InputRef::Regular("e".to_string()),
            err_path: PathBuf::from(tempdir.path())
                .join("1/c/error_e")
                .canonicalize()
                .unwrap(),
            output_path: PathBuf::from(tempdir.path())
                .join("1/c/output_e")
                .canonicalize()
                .unwrap(),
            metrics_path: PathBuf::from(tempdir.path())
                .join("1/c/metrics_e")
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
        env: Environment::Local,
        postprocess_inputs: BTreeMap::new(),
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

    Experiment::from_config(&config, Local::now(), Environment::Local, &REAL_FS).unwrap();
    Experiment::from_config(&config, Local::now(), Environment::Local, &REAL_FS).unwrap();

    let id = Experiment::latest_id_from_folder(tempdir.path()).unwrap();
    assert_eq!(id, Some(2));
}
