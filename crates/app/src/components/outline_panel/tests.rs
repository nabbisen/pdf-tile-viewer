use super::*;

#[test]
fn outline_title_label_uses_i18n_fallback_for_missing_empty_and_blank_truncated_titles() {
    assert_eq!(
        outline_title_label(&OutlineTitle::Missing, Locale::En),
        "Untitled"
    );
    assert_eq!(
        outline_title_label(&OutlineTitle::Empty, Locale::En),
        "Untitled"
    );
    assert_eq!(
        outline_title_label(&OutlineTitle::Truncated(String::new()), Locale::En),
        "Untitled"
    );
}

#[test]
fn outline_title_label_keeps_present_or_truncated_text() {
    assert_eq!(
        outline_title_label(&OutlineTitle::Present("Chapter".to_string()), Locale::En),
        "Chapter"
    );
    assert_eq!(
        outline_title_label(&OutlineTitle::Truncated("Chapter".to_string()), Locale::En),
        "Chapter"
    );
}
