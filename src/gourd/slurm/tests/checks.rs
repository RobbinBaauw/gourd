use std::ops::Range;
use std::path::PathBuf;

use super::*;
#[test]
fn versioning_test() {
    struct X {}
    impl SlurmInteractor for X {
        fn get_version(&self) -> anyhow::Result<[u64; 2]> {
            Ok([21, 8])
        }
        fn get_partitions(&self) -> anyhow::Result<Vec<Vec<String>>> {
            Ok(vec![])
        }

        fn schedule_array(
            &self,
            _range: Range<usize>,
            _slurm_config: &SlurmConfig,
            _wrapper_path: &str,
            _exp_path: PathBuf,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_version_supported(&self, _v: [u64; 2]) -> bool {
            true
        }

        fn get_supported_versions(&self) -> String {
            "groff".to_string()
        }
    }
    let y = SlurmHandler { internal: X {} };
    assert!(y.check_version().is_ok());
}

#[test]
fn versioning_un_test() {
    struct X {}
    impl SlurmInteractor for X {
        fn get_version(&self) -> anyhow::Result<[u64; 2]> {
            Ok([21, 8])
        }
        fn get_partitions(&self) -> anyhow::Result<Vec<Vec<String>>> {
            Ok(vec![])
        }

        fn schedule_array(
            &self,
            _range: Range<usize>,
            _slurm_config: &SlurmConfig,
            _wrapper_path: &str,
            _exp_path: PathBuf,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_version_supported(&self, _v: [u64; 2]) -> bool {
            false
        }

        fn get_supported_versions(&self) -> String {
            "your dad".to_string()
        }
    }
    let y = SlurmHandler { internal: X {} };
    assert!(y.check_version().is_err());
}

#[test]
fn get_slurm_options_from_config_test() {
    let config = Config {
        slurm: Some(SlurmConfig {
            partition: "test".to_string(),
            time_limit: "1:00:00".to_string(),
            cpus: 1,
            mem_per_cpu: 420,
            out: None,
            experiment_name: "test".to_string(),
            account: None,
            begin: None,
            mail_type: None,
            mail_user: None,
            additional_args: None,
        }),
        ..Default::default()
    };
    assert!(get_slurm_options_from_config(&config).is_ok());
}

#[test]
fn get_slurm_options_from_config_un_test() {
    let config = Config {
        slurm: None,
        ..Default::default()
    };
    assert!(get_slurm_options_from_config(&config).is_err());
}
