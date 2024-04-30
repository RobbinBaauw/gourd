use crate::example_covered_function;

#[test]
fn test_covered() {
    assert!(example_covered_function() == 3);
}
