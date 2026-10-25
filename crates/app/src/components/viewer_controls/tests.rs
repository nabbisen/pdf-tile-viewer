use super::fixed_start;

#[test]
fn switching_to_fixed_starts_at_the_displayed_column_count() {
    assert_eq!(fixed_start(1), 1);
    assert_eq!(fixed_start(4), 4);
    assert_eq!(fixed_start(7), 7);
}

#[test]
fn the_starting_value_stays_in_the_supported_range() {
    assert_eq!(fixed_start(0), 1);
    assert_eq!(fixed_start(24), 24);
    assert_eq!(fixed_start(40), 24);
}
