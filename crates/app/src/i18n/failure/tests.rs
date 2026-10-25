use super::*;
use crate::i18n::{Locale, en, t};

/// Every message a failure can produce, for the leak checks.
fn all_keys() -> Vec<MessageKey> {
    vec![
        search_error_key(&SearchError::DocumentNotOpen),
        search_error_key(&SearchError::QueryTooShort),
        search_error_key(&SearchError::EngineUnavailable),
        search_error_key(&SearchError::SearchFailed("D:\\secret\\path.pdf".into())),
        render_error_key(&RenderError::DocumentNotOpen),
        render_error_key(&RenderError::PageIndexOutOfRange),
        render_error_key(&RenderError::RenderFailed("/home/u/secret.pdf".into())),
        render_error_key(&RenderError::EncodingFailed("x".into())),
        render_error_key(&RenderError::EngineUnavailable),
        text_layer_error_key(&TextLayerError::DocumentNotOpen),
        text_layer_error_key(&TextLayerError::EngineUnavailable),
        text_layer_error_key(&TextLayerError::PageOutOfBounds),
        text_layer_error_key(&TextLayerError::SegmentIndexOverflow),
        text_layer_error_key(&TextLayerError::StaleResult),
        text_layer_error_key(&TextLayerError::ExtractionFailed("x".into())),
        navigation_error_key(&NavigationError::DocumentNotOpen),
        navigation_error_key(&NavigationError::EngineUnavailable),
        navigation_error_key(&NavigationError::PageOutOfBounds),
        navigation_error_key(&NavigationError::LinkIndexOverflow),
        navigation_error_key(&NavigationError::ExtractionFailed("x".into())),
        platform_error_key(&PlatformError::NoParentDirectory),
        platform_error_key(&PlatformError::LaunchFailed(
            "/usr/bin/xdg-open: not found".into(),
        )),
    ]
}

#[test]
fn the_variant_payload_never_selects_or_leaks_into_the_message() {
    // The detail string is dropped: two failures of one variant give the same key.
    assert_eq!(
        search_error_key(&SearchError::SearchFailed("a".into())),
        search_error_key(&SearchError::SearchFailed("b".into()))
    );
    assert_eq!(
        platform_error_key(&PlatformError::LaunchFailed("one".into())),
        platform_error_key(&PlatformError::LaunchFailed("two".into()))
    );
}

#[test]
fn messages_are_localized_and_free_of_debug_text_and_paths() {
    for key in all_keys() {
        for locale in [Locale::En, Locale::Ja] {
            let text = t(locale, key);
            assert!(!text.is_empty(), "{key:?} {locale:?}");
            for bad in [
                "Err(", "Error(", "Failed(", "{", "}", "\\", "/home", "/usr", "C:", "D:",
            ] {
                assert!(
                    !text.contains(bad),
                    "{key:?} {locale:?} contains {bad:?}: {text}"
                );
            }
        }
    }
}

#[test]
fn every_failure_key_has_a_japanese_translation() {
    for key in all_keys() {
        assert!(
            crate::i18n::ja::message(key).is_some(),
            "no Japanese text for {key:?}"
        );
    }
}

#[test]
fn failure_messages_tell_the_user_what_to_do_or_what_happened() {
    assert_eq!(
        en::message(search_error_key(&SearchError::SearchFailed(String::new()))),
        "Search could not be completed. Try again."
    );
    assert_eq!(
        en::message(platform_error_key(&PlatformError::LaunchFailed(
            String::new()
        ))),
        "The file manager could not be opened."
    );
}

#[test]
fn a_too_short_query_maps_to_the_hint_not_to_a_failure_text() {
    assert_eq!(
        search_error_key(&SearchError::QueryTooShort),
        MessageKey::SearchQueryTooShort
    );
}
