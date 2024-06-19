// use std::path::PathBuf;

// use gourd_lib::config::Config;
// use gourd_lib::config::ProgramMap;

// use super::*;

// #[test]
// fn test_get_program_from_name_simple() {
//     let mut config = Config::default();
//     let program = Program {
//         binary: PathBuf::from("fake_path"),
//         arguments: vec![],
//         afterscript: None,
//         postprocess_job: None,
//         resource_limits: None,
//     };
//     config.programs.insert("name".to_string(), program.clone());

//     let prog = get_program_from_name(&config, &"name".to_string());
//     assert!(prog.is_ok());
//     assert_eq!(prog.unwrap(), program);
// }

// #[test]
// fn test_get_program_from_name_post() {
//     let mut config = Config::default();
//     let program = Program {
//         binary: PathBuf::from("fake_path"),
//         arguments: vec![],
//         afterscript: None,
//         postprocess_job: None,
//         resource_limits: None,
//     };

//     let mut post_progs = ProgramMap::default();
//     post_progs.insert("name".to_string(), program.clone());
//     config.postprocess_programs = Some(post_progs);

//     let prog = get_program_from_name(&config, &"name".to_string());
//     assert!(prog.is_ok());
//     assert_eq!(prog.unwrap(), program);
// }

// #[test]
// fn test_get_program_from_name_error() {
//     assert!(get_program_from_name(&Config::default(),
// &"fake_name".to_string()).is_err()); }

// #[test]
// fn test_query_changing_limits_for_program() {}

// #[test]
// fn test_query_changing_limits_for_all_programs() {}

// #[test]
// fn test_get_setlim_programs() {
//     let mut config = Config::default();
//     let program = Program {
//         binary: PathBuf::from("fake_path"),
//         arguments: vec![],
//         afterscript: None,
//         postprocess_job: None,
//         resource_limits: None,
//     };

//     config.programs.insert("name1".to_string(), program.clone());
//     config.programs.insert("name2".to_string(), program.clone());

//     let mut post_progs = ProgramMap::default();
//     post_progs.insert("name_post".to_string(), program.clone());
//     config.postprocess_programs = Some(post_progs);

//     let prog = get_setlim_programs(&config).unwrap();
//     assert_eq!(prog.len(), 3);
// }
