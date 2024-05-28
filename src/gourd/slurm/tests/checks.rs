// use std::path::Path;

// use anyhow::Result;
// use gourd_lib::experiment::Chunk;
// use gourd_lib::experiment::Experiment;

use super::*;
// use crate::slurm::SlurmStatus;
// #[test]
// fn versioning_test() {
//     struct X {}
//     impl SlurmInteractor for X {
//         fn get_version(&self) -> Result<[u64; 2]> {
//             Ok([21, 8])
//         }
//         fn get_partitions(&self) -> Result<Vec<Vec<String>>> {
//             Ok(vec![])
//         }

//         fn schedule_chunk(
//             &self,
//             _slurm_config: &SlurmConfig,
//             _chunk: &mut Chunk,
//             _chunk_id: usize,
//             _experiment: &mut Experiment,
//             _exp_path: &Path,
//         ) -> Result<u64> {
//             Ok(123456)
//         }

//         fn is_version_supported(&self, _v: [u64; 2]) -> bool {
//             true
//         }

//         fn get_supported_versions(&self) -> String {
//             "groff".to_string()
//         }

//         fn get_accounting_data(&self, _job_id: Vec<String>) -> anyhow::Result<Vec<SlurmStatus>> {
//             Ok(vec![SlurmStatus {
//                 job_id: "123456".to_string(),
//                 job_name: "test job name".to_string(),
//                 state: "FAILED".to_string(),
//                 slurm_exit_code: 0,
//                 program_exit_code: 0,
//             }])
//         }
//     }
//     let y = SlurmHandler { internal: X {} };
//     assert!(y.check_version().is_ok());
// }

// #[test]
// fn versioning_un_test() {
//     struct X {}
//     impl SlurmInteractor for X {
//         fn get_version(&self) -> Result<[u64; 2]> {
//             Ok([21, 8])
//         }
//         fn get_partitions(&self) -> Result<Vec<Vec<String>>> {
//             Ok(vec![])
//         }

//         fn schedule_chunk(
//             &self,
//             _slurm_config: &SlurmConfig,
//             _chunk: &mut Chunk,
//             _chunk_id: usize,
//             _experiment: &mut Experiment,
//             _exp_path: &Path,
//         ) -> Result<u64> {
//             Ok(123456)
//         }

//         fn is_version_supported(&self, _v: [u64; 2]) -> bool {
//             false
//         }

//         fn get_supported_versions(&self) -> String {
//             "your dad".to_string()
//         }

//         fn get_accounting_data(&self, _job_id: Vec<String>) -> anyhow::Result<Vec<SlurmStatus>> {
//             Ok(vec![SlurmStatus {
//                 job_id: "123456".to_string(),
//                 job_name: "test job name".to_string(),
//                 state: "FAILED".to_string(),
//                 slurm_exit_code: 0,
//                 program_exit_code: 0,
//             }])
//         }
//     }
//     let y = SlurmHandler { internal: X {} };
//     assert!(y.check_version().is_err());
// }

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
