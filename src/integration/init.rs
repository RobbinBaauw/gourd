// use crate::gourd;
// use crate::init;

// #[test]
// fn test_gourd_init() {
//     let env = init();
//
//     // run gourd init
//     let _output = gourd!(env; "init", "init_test", "-s"; "init");
//
//     // check that the directory ./init_test exists
//     let init_test_dir = env.temp_dir.path().join("init_test");
//     assert!(init_test_dir.exists());
// }
