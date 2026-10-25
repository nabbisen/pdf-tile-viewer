use super::{fixed_start, parse_jump_target};

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

#[test]
fn jump_accepts_pages_one_to_last_and_is_zero_based() {
    assert_eq!(parse_jump_target("1", 50), Some(0));
    assert_eq!(parse_jump_target(" 40 ", 50), Some(39));
    assert_eq!(parse_jump_target("50", 50), Some(49));
}

#[test]
fn jump_rejects_everything_else_with_the_same_outcome() {
    for raw in ["", "0", "51", "-3", "abc", "1.5", "99999999999999999999"] {
        assert_eq!(parse_jump_target(raw, 50), None, "{raw:?}");
    }
}
