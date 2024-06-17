//! Full workflow integration test.

use std::collections::BTreeMap;

use gourd_lib::config::Input;
use gourd_lib::config::Label;
use gourd_lib::config::Regex;

use crate::config;
use crate::gourd;
use crate::init;
use crate::read_experiment_from_stdout;
use crate::save_gourd_toml;

#[test]
fn run_gourd_test() {
    let env = init();

    let conf = config!(env; "slow_fib", "fast_fib", "hello"; (
        "input_ten".to_string(),
        Input {
            input: Some(env.temp_dir.path().join("input_ten")),
            arguments: vec![],
        }),
        ("input_hello".to_string(),
        Input {
            input: Some(env.temp_dir.path().join("input_hello")),
            arguments: vec![],
        });
        "fast_fast_fib";
        Some(BTreeMap::from([(
            "correct".to_string(),
            Label {
                regex: Regex::from(regex_lite::Regex::new("55").unwrap()),
                priority: 0,
                rerun_by_default: false,
            },
        )]))
    );

    // write the configuration to the tempdir
    let conf_path = save_gourd_toml(&conf, &env.temp_dir);

    let output =
        gourd!(env; "-c", conf_path.to_str().unwrap(), "run", "local", "-s", "-vv"; "run local");

    // check if the output file exists
    let exp = read_experiment_from_stdout(&output).unwrap();
    let output_file = exp.runs.last().unwrap().output_path.clone();
    assert!(output_file.exists());

    // run status
    let _ = gourd!(env; "-c", conf_path.to_str().unwrap(), "status", "-s"; "status 1");
    let _o = gourd!(env; "-c", conf_path.to_str().unwrap(), "continue", "-s"; "continue");
    // let _e = read_experiment_from_stdout(&_o).unwrap();
    let _ = gourd!(env; "-c", conf_path.to_str().unwrap(), "status", "-s"; "status 2");
    let _ = gourd!(env; "-c", conf_path.to_str().unwrap(), "rerun", "-r", "0", "-s"; "rerun");

    assert!(!gourd!(env; "cancel").status.success());
}
