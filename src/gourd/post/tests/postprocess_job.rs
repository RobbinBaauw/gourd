// #[test]
// fn test_filter_runs_for_post_job_good_weather() {
//     let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
//     runs.insert(
//         0,
//         Some(Status {
//             completion: Completion::Success,
//             afterscript_completion: None,
//             postprocess_job_completion: None,
//         }),
//     );
//     runs.insert(
//         1,
//         Some(Status {
//             completion: Completion::Success,
//             afterscript_completion: None,
//             postprocess_job_completion: Some(PostprocessCompletion::Dormant),
//         }),
//     );
//     runs.insert(
//         2,
//         Some(Status {
//             completion: Completion::Fail(FailureReason::UserForced),
//             afterscript_completion: None,
//             postprocess_job_completion: Some(PostprocessCompletion::Dormant),
//         }),
//     );
//     runs.insert(
//         3,
//         Some(Status {
//             completion: Completion::Success,
//             afterscript_completion: None,
//             postprocess_job_completion: Some(PostprocessCompletion::Success(PostprocessOutput {
//                 short_output: String::from("short"),
//                 long_output: String::from("long"),
//             })),
//         }),
//     );

//     let res = filter_runs_for_post_job(&mut runs).unwrap();

//     assert_eq!(res.len(), 1);

//     let paths = res[0];
//     assert_eq!(*paths, 1);
// }

// #[test]
// fn test_filter_runs_for_post_job_bad_weather() {
//     let mut runs: BTreeMap<usize, Option<Status>> = BTreeMap::new();
//     runs.insert(
//         0,
//         Some(Status {
//             completion: Completion::Success,
//             afterscript_completion: None,
//             postprocess_job_completion: None,
//         }),
//     );
//     runs.insert(1, None);

//     assert!(filter_runs_for_post_job(&mut runs).is_err());
// }

// // test post jobs getting scheduled (good + bad)
