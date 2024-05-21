use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use gourd_lib::config::Input;
use gourd_lib::config::Program;

use super::*;
use crate::test_utils::create_sample_experiment;

#[test]
fn get_unscheduled_runs_test() {
    let prog = Program {
        binary: PathBuf::new(),
        arguments: vec![],
        afterscript: None,
        postprocess_job: None,
    };
    let input = Input {
        input: None,
        arguments: vec![],
    };
    let (mut experiment, _conf) = create_sample_experiment(
        BTreeMap::from([
            (String::from("Prog1"), prog.clone()),
            (String::from("Prog2"), prog.clone()),
            (String::from("Prog3"), prog),
        ]),
        BTreeMap::from([
            (String::from("Inp1"), input.clone()),
            (String::from("Inp2"), input.clone()),
            (String::from("Inp3"), input),
        ]),
    );
    let resource_limits = ResourceLimits {
        time_limit: Duration::new(600, 0),
        cpus: 0,
        mem_per_cpu: 0,
    };

    experiment.slurm = Some(SlurmExperiment {
        chunks: vec![Chunk {
            runs: vec![0, 1],
            resource_limits: resource_limits.clone(),
        }],
        resource_limits,
    });
    let runs = experiment.get_unscheduled_runs().unwrap();

    assert_eq!(runs, vec!(2, 3, 4, 5, 6, 7, 8))
}

#[test]
fn create_chunks_basic_test() {
    let prog = Program {
        binary: PathBuf::new(),
        arguments: vec![],
        afterscript: None,
        postprocess_job: None,
    };
    let input = Input {
        input: None,
        arguments: vec![],
    };
    let (mut experiment, _conf) = create_sample_experiment(
        BTreeMap::from([
            (String::from("Prog1"), prog.clone()),
            (String::from("Prog2"), prog.clone()),
            (String::from("Prog3"), prog),
        ]),
        BTreeMap::from([
            (String::from("Inp1"), input.clone()),
            (String::from("Inp2"), input.clone()),
            (String::from("Inp3"), input),
        ]),
    );
    let resource_limits = ResourceLimits {
        time_limit: Duration::new(600, 0),
        cpus: 0,
        mem_per_cpu: 0,
    };

    experiment.slurm = Some(SlurmExperiment {
        chunks: vec![Chunk {
            runs: vec![0, 1],
            resource_limits: resource_limits.clone(),
        }],
        resource_limits: resource_limits.clone(),
    });
    let chunks = experiment
        .create_chunks(3, 2, experiment.get_unscheduled_runs().unwrap().into_iter())
        .unwrap();

    assert_eq!(
        chunks,
        vec!(
            Chunk {
                runs: vec![2, 3, 4],
                resource_limits: resource_limits.clone(),
            },
            Chunk {
                runs: vec![5, 6, 7],
                resource_limits: resource_limits.clone(),
            }
        )
    )
}
