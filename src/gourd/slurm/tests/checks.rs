use super::*;

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
