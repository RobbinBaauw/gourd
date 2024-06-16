use gourd_lib::config::Input;

use crate::config;
use crate::gourd;
use crate::init;
use crate::read_experiment_from_stdout;
use crate::save_gourd_toml;

#[test]
fn test_dry_one_run() {
    let env = init();
    let conf = config!(&env; "fibonacci"; (
        "input_ten".to_string(),
        Input {
            input: None,
            arguments: vec!["10".to_string()],
        },
    ));
    let conf_path = save_gourd_toml(&conf, &env.temp_dir);

    let output = gourd!(&env; "-c", conf_path.to_str().unwrap(), "run", "local", "-s"; "run local");
    let mut exp = read_experiment_from_stdout(&output).unwrap();
    assert_eq!(exp.runs.len(), 1);

    let _ =
        gourd!(&env; "-c", conf_path.to_str().unwrap(), "rerun", "-s", "-r", "0"; "rerun local");

    let _ = gourd!(&env; "-c", conf_path.to_str().unwrap(), "continue", "-s"; "continue");
    exp = read_experiment_from_stdout(&output).unwrap();
    assert_eq!(exp.runs.len(), 2);
}
