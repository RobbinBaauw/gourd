use super::*;
use crate::config::SubParameter;

#[test]
fn test_expand_parameters_ok_no_expandable() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec!["nice".to_string()],
        },
    );

    let parameters = BTreeMap::new();
    let expanded = expand_parameters(InputMap(inputs.clone()), &parameters).unwrap();
    assert_eq!(expanded, InputMap(inputs));
}

#[test]
fn test_expand_parameters_ok_parameter() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec!["parameter|-e {parameter_x}".to_string()],
        },
    );
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: None,
            values: Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        },
    );
    let expanded = expand_parameters(InputMap(inputs.clone()), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0".to_string(),
        Input {
            input: None,
            arguments: vec!["-e a".to_string()],
        },
    );
    expected.insert(
        "first_x-1".to_string(),
        Input {
            input: None,
            arguments: vec!["-e b".to_string()],
        },
    );
    expected.insert(
        "first_x-2".to_string(),
        Input {
            input: None,
            arguments: vec!["-e c".to_string()],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}

#[test]
fn test_expand_parameters_ok_parameter_doubled() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x}".to_string(),
                "parameter|-f {parameter_x}".to_string(),
            ],
        },
    );
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: None,
            values: Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        },
    );
    let expanded = expand_parameters(InputMap(inputs.clone()), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0".to_string(),
        Input {
            input: None,
            arguments: vec!["-e a".to_string(), "-f a".to_string()],
        },
    );
    expected.insert(
        "first_x-1".to_string(),
        Input {
            input: None,
            arguments: vec!["-e b".to_string(), "-f b".to_string()],
        },
    );
    expected.insert(
        "first_x-2".to_string(),
        Input {
            input: None,
            arguments: vec!["-e c".to_string(), "-f c".to_string()],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}

#[test]
fn test_expand_parameters_ok_subparameter() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "parameter|-x {parameter_x_2}".to_string(),
            ],
        },
    );
    let mut sub_parameters = BTreeMap::new();
    sub_parameters.insert(
        "1".to_string(),
        SubParameter {
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        },
    );
    sub_parameters.insert(
        "2".to_string(),
        SubParameter {
            values: vec!["10".to_string(), "20".to_string(), "30".to_string()],
        },
    );
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: Some(sub_parameters),
            values: None,
        },
    );
    let expanded = expand_parameters(InputMap(inputs), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0".to_string(),
        Input {
            input: None,
            arguments: vec!["-e a".to_string(), "-x 10".to_string()],
        },
    );
    expected.insert(
        "first_x-1".to_string(),
        Input {
            input: None,
            arguments: vec!["-e b".to_string(), "-x 20".to_string()],
        },
    );
    expected.insert(
        "first_x-2".to_string(),
        Input {
            input: None,
            arguments: vec!["-e c".to_string(), "-x 30".to_string()],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}

#[test]
fn test_expand_parameters_ok_both() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "parameter|-f {parameter_y}".to_string(),
                "parameter|-x {parameter_x_2}".to_string(),
            ],
        },
    );
    let mut sub_parameters = BTreeMap::new();
    sub_parameters.insert(
        "1".to_string(),
        SubParameter {
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        },
    );
    sub_parameters.insert(
        "2".to_string(),
        SubParameter {
            values: vec!["10".to_string(), "20".to_string(), "30".to_string()],
        },
    );

    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: Some(sub_parameters),
            values: None,
        },
    );
    parameters.insert(
        "y".to_string(),
        Parameter {
            sub: None,
            values: Some(vec![
                "xxx".to_string(),
                "yyy".to_string(),
                "zzz".to_string(),
            ]),
        },
    );

    let expanded = expand_parameters(InputMap(inputs), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f xxx".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f xxx".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f xxx".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f yyy".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f yyy".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f yyy".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f zzz".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f zzz".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f zzz".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}

#[test]
fn test_expand_parameters_ok_both_with_normal_args() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "parameter|-f {parameter_y}".to_string(),
                "-v".to_string(),
                "parameter|-x {parameter_x_2}".to_string(),
            ],
        },
    );
    let mut sub_parameters = BTreeMap::new();
    sub_parameters.insert(
        "1".to_string(),
        SubParameter {
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        },
    );
    sub_parameters.insert(
        "2".to_string(),
        SubParameter {
            values: vec!["10".to_string(), "20".to_string(), "30".to_string()],
        },
    );

    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: Some(sub_parameters),
            values: None,
        },
    );
    parameters.insert(
        "y".to_string(),
        Parameter {
            sub: None,
            values: Some(vec![
                "xxx".to_string(),
                "yyy".to_string(),
                "zzz".to_string(),
            ]),
        },
    );

    let expanded = expand_parameters(InputMap(inputs), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}

#[test]
fn test_expand_parameters_fail_subparameter_not_declared() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec!["parameter|-e {parameter_x}".to_string()],
        },
    );
    let parameters = BTreeMap::new();
    assert!(expand_parameters(InputMap(inputs), &parameters).is_err());
}

#[test]
fn test_expand_parameters_fail_subparameter_size_not_match() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "parameter|-x {parameter_x_2}".to_string(),
            ],
        },
    );
    let mut sub_parameters = BTreeMap::new();
    sub_parameters.insert(
        "1".to_string(),
        SubParameter {
            values: vec!["a".to_string(), "b".to_string()],
        },
    );
    sub_parameters.insert(
        "2".to_string(),
        SubParameter {
            values: vec!["10".to_string(), "20".to_string(), "30".to_string()],
        },
    );
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: Some(sub_parameters),
            values: None,
        },
    );
    assert!(expand_parameters(InputMap(inputs), &parameters).is_err());
}

#[test]
fn test_expand_parameters_ok_ultimate() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "first".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "parameter|-f {parameter_y}".to_string(),
                "-v".to_string(),
                "parameter|-x {parameter_x_2}".to_string(),
            ],
        },
    );
    inputs.insert(
        "second".to_string(),
        Input {
            input: None,
            arguments: vec![
                "parameter|-e {parameter_x_1}".to_string(),
                "-t".to_string(),
                "parameter|-x {parameter_y}".to_string(),
                "parameter|-g {parameter_z}".to_string(),
            ],
        },
    );
    let mut sub_parameters = BTreeMap::new();
    sub_parameters.insert(
        "1".to_string(),
        SubParameter {
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        },
    );
    sub_parameters.insert(
        "2".to_string(),
        SubParameter {
            values: vec!["10".to_string(), "20".to_string(), "30".to_string()],
        },
    );

    let mut parameters = BTreeMap::new();
    parameters.insert(
        "x".to_string(),
        Parameter {
            sub: Some(sub_parameters),
            values: None,
        },
    );
    parameters.insert(
        "y".to_string(),
        Parameter {
            sub: None,
            values: Some(vec![
                "xxx".to_string(),
                "yyy".to_string(),
                "zzz".to_string(),
            ]),
        },
    );
    parameters.insert(
        "z".to_string(),
        Parameter {
            sub: None,
            values: Some(vec![
                "1000".to_string(),
                "2000".to_string(),
                "3000".to_string(),
            ]),
        },
    );

    let expanded = expand_parameters(InputMap(inputs), &parameters).unwrap();
    let mut expected = BTreeMap::new();
    expected.insert(
        "first_x-0_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f xxx".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f yyy".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-0_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 10".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-1_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 20".to_string(),
            ],
        },
    );
    expected.insert(
        "first_x-2_y-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-f zzz".to_string(),
                "-v".to_string(),
                "-x 30".to_string(),
            ],
        },
    );

    expected.insert(
        "second_x-0_y-0_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-0_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-0_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-1_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-1_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-1_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-2_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-2_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-0_y-2_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e a".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );

    expected.insert(
        "second_x-1_y-0_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-0_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-0_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-1_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-1_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-1_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-2_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-2_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-1_y-2_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e b".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );

    expected.insert(
        "second_x-2_y-0_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-0_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-0_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x xxx".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-1_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-1_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-1_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x yyy".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-2_z-0".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 1000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-2_z-1".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 2000".to_string(),
            ],
        },
    );
    expected.insert(
        "second_x-2_y-2_z-2".to_string(),
        Input {
            input: None,
            arguments: vec![
                "-e c".to_string(),
                "-t".to_string(),
                "-x zzz".to_string(),
                "-g 3000".to_string(),
            ],
        },
    );
    assert_eq!(expanded, InputMap(expected));
}
