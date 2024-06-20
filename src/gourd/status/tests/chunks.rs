use gourd_lib::experiment::Chunk;
use gourd_lib::experiment::ChunkRunStatus;
use gourd_lib::experiment::Environment;
use gourd_lib::experiment::Experiment;
use gourd_lib::experiment::FieldRef;
use gourd_lib::experiment::Run;

use crate::status::chunks::print_scheduling;

#[test]
fn print_scheduling_test() {
    let run = Run {
        program: FieldRef::Regular(String::from("Program")),
        input: FieldRef::Regular(String::from("Input")),
        err_path: Default::default(),
        output_path: Default::default(),
        metrics_path: Default::default(),
        work_dir: Default::default(),
        slurm_id: None,
        afterscript_output_path: None,
        postprocessor: None,
        rerun: None,
    };
    let experiment = Experiment {
        runs: vec![run.clone(), run.clone(), run.clone(), run],
        chunks: vec![Chunk {
            runs: vec![0, 1],
            resource_limits: None,
            status: ChunkRunStatus::Scheduled(String::from("id")),
        }],
        resource_limits: None,
        creation_time: Default::default(),
        config: Default::default(),
        seq: 0,
        env: Environment::Local,
        postprocess_inputs: Default::default(),
    };

    print_scheduling(&experiment, true)
        .expect("Error printing status information for a starting experiment.");

    print_scheduling(&experiment, false)
        .expect("Error printing status information for a continuing experiment.");
}
