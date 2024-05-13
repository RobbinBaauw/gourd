use crate::config::Config;
use crate::experiment::Experiment;
use crate::slurm::SlurmInteractor;

#[test]
fn tabling_test() {
    let data = vec![
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec!["d".to_string(), "e".to_string(), "f".to_string()],
    ];
    let expected = "a | b | c\nd | e | f";
    assert_eq!(expected, crate::cli::printing::format_table(data));
}

#[test]
fn tabling_empty_test() {
    let data = vec![];
    let expected = "";
    assert_eq!(expected, crate::cli::printing::format_table(data));
}

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

        fn run_jobs(&self, _config: &Config, _experiment: &mut Experiment) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_version_supported(&self, _v: [u64; 2]) -> bool {
            true
        }

        fn get_supported_versions(&self) -> String {
            "groff".to_string()
        }
    }
    assert!(crate::slurm::handler::check_version(&X {}).is_ok());
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

        fn run_jobs(&self, _config: &Config, _experiment: &mut Experiment) -> anyhow::Result<()> {
            Ok(())
        }

        fn is_version_supported(&self, _v: [u64; 2]) -> bool {
            false
        }

        fn get_supported_versions(&self) -> String {
            "your mom".to_string()
        }
    }
    assert!(crate::slurm::handler::check_version(&X {}).is_err());
}
