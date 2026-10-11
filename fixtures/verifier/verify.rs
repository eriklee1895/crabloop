include!(env!("CRABLOOP_SOLUTION"));
#[test]
fn empty_is_none() { assert_eq!(last_index(0), None); }
#[test]
fn one_is_zero() { assert_eq!(last_index(1), Some(0)); }
#[test]
fn ten_is_nine() { assert_eq!(last_index(10), Some(9)); }
#[test]
fn largest_length_is_supported() { assert_eq!(last_index(usize::MAX), Some(usize::MAX - 1)); }
