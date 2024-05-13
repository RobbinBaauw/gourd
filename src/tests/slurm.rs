use crate::config::Config;
use crate::slurm::SlurmConfig;

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
        }),
        ..Default::default()
    };
    assert!(crate::slurm::handler::get_slurm_options_from_config(&config).is_ok());
}

#[test]
fn get_slurm_options_from_config_un_test() {
    let config = Config {
        slurm: None,
        ..Default::default()
    };
    assert!(crate::slurm::handler::get_slurm_options_from_config(&config).is_err());
}
