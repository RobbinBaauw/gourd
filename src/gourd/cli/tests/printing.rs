use super::*;

#[test]
fn tabling_test() {
    let data = vec![
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec!["d".to_string(), "e".to_string(), "f".to_string()],
    ];
    let expected = "a | b | c\nd | e | f";
    assert_eq!(expected, format_table(data));
}

#[test]
fn tabling_empty_test() {
    let data = vec![];
    let expected = "";
    assert_eq!(expected, format_table(data));
}
