use std::ops::Range;
use std::path::Path;

use anyhow::Result;
use gourd_lib::config::ResourceLimits;

use super::*;
#[test]
fn versioning_test() {
    struct X {}
    impl SlurmInteractor for X {
        fn get_version(&self) -> Result<[u64; 2]> {
            Ok([21, 8])
        }
        fn get_partitions(&self) -> Result<Vec<Vec<String>>> {
            Ok(vec![])
        }

        fn schedule_chunk(
            &self,
            _range: Range<usize>,
            _slurm_config: &SlurmConfig,
            _resource_limits: &ResourceLimits,
            _wrapper_path: &str,
            _exp_path: &Path,
        ) -> Result<()> {
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
        fn get_version(&self) -> Result<[u64; 2]> {
            Ok([21, 8])
        }
        fn get_partitions(&self) -> Result<Vec<Vec<String>>> {
            Ok(vec![])
        }

        fn schedule_chunk(
            &self,
            _range: Range<usize>,
            _slurm_config: &SlurmConfig,
            _resource_limits: &ResourceLimits,
            _wrapper_path: &str,
            _exp_path: &Path,
        ) -> Result<()> {
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
            array_count_limit: 10,
            array_size_limit: 1000,
            out: None,
            experiment_name: "test".to_string(),
            account: "test-account".to_string(),
            begin: None,
            mail_type: None,
            mail_user: None,
            additional_args: None,
            post_job_time_limit: Some("1:42:00".to_string()),
            post_job_cpus: Some(1),
            post_job_mem_per_cpu: Some(2137),
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
