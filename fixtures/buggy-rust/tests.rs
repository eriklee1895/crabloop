#[path = "lib.rs"]
mod solution;
#[test]
fn empty_has_no_index() { assert_eq!(solution::last_index(0), None); }
#[test]
fn single_element_index_is_zero() { assert_eq!(solution::last_index(1), Some(0)); }
#[test]
fn ten_elements_end_at_nine() { assert_eq!(solution::last_index(10), Some(9)); }
