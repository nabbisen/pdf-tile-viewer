use crate::i18n::{Locale, MessageKey, en, t};

use app_services::document_service::OpenError;
use domain::document::{DocumentError, OpenFailure};

#[test]
fn english_catalog_is_complete_and_non_empty() {
    // RFC 017 §9: the reference catalog must cover every key.
    for key in MessageKey::ALL {
        assert!(
            !en::message(*key).is_empty(),
            "empty en message for {key:?}"
        );
    }
}

#[test]
fn japanese_lookup_always_resolves_via_fallback() {
    // RFC 017 §8: a missing translation falls back to English, so lookup
    // never yields an empty string for any key.
    for key in MessageKey::ALL {
        assert!(
            !t(Locale::Ja, *key).is_empty(),
            "unresolvable ja message for {key:?}"
        );
    }
}

#[test]
fn locale_resolution_prefers_explicit_setting() {
    assert_eq!(Locale::from_tag("ja"), Locale::Ja);
    assert_eq!(Locale::from_tag("ja-JP"), Locale::Ja);
    assert_eq!(Locale::from_tag("JA"), Locale::Ja);
    assert_eq!(Locale::from_tag("en-US"), Locale::En);
    assert_eq!(Locale::from_tag("fr"), Locale::En); // unsupported → reference
    assert_eq!(Locale::resolve(Some("ja")), Locale::Ja);
}

#[test]
fn engine_open_failures_map_to_specific_keys() {
    use domain::document::OpenFailure;
    let key = |failure| crate::i18n::open_error_key(&OpenError::Engine(failure));
    assert_eq!(
        key(OpenFailure::PdfParseFailed),
        MessageKey::ErrPdfParseFailed
    );
    assert_eq!(key(OpenFailure::FileNotFound), MessageKey::ErrFileNotFound);
    assert_eq!(key(OpenFailure::UnsupportedFile), MessageKey::ErrNotAPdf);
    assert_eq!(
        key(OpenFailure::Unknown("x".into())),
        MessageKey::ErrUnknown
    );
}

#[test]
fn a_password_request_reaches_the_display_path_only_as_an_outcome() {
    // `OpenFailure` has no password variant: the conversion refuses it (RFC 037 § 5.2).
    assert!(OpenFailure::try_from(DocumentError::PasswordRequired).is_err());
    assert!(OpenFailure::try_from(DocumentError::PdfParseFailed).is_ok());
}

#[test]
fn every_catalog_key_has_a_japanese_translation() {
    // RFC 037: the catalog is complete in both languages (the fallback to English is a safety net,
    // not a way to ship an untranslated key).
    for key in MessageKey::ALL {
        assert!(
            crate::i18n::ja::message(*key).is_some(),
            "no Japanese text for {key:?}"
        );
    }
}

#[test]
fn plural_is_singular_only_for_exactly_one_in_english() {
    use crate::i18n::plural;
    assert_eq!(plural(Locale::En, 1, "match", "matches"), "match");
    assert_eq!(plural(Locale::En, 0, "match", "matches"), "matches");
    assert_eq!(plural(Locale::En, 2, "match", "matches"), "matches");
    assert_eq!(plural(Locale::En, 27692, "match", "matches"), "matches");
}

#[test]
fn japanese_has_no_plural_form() {
    use crate::i18n::plural;
    for count in [0, 1, 2, 100] {
        assert_eq!(plural(Locale::Ja, count, "一", "他"), "他");
    }
}

#[test]
fn the_summary_wordings_are_plural_aware_in_english() {
    assert_eq!(
        en::message(MessageKey::SearchSummaryMatchOne),
        "match found"
    );
    assert_eq!(
        en::message(MessageKey::SearchSummaryMatches),
        "matches found"
    );
    assert_eq!(en::message(MessageKey::SearchMatchBadgeOne), "match");
    assert_eq!(en::message(MessageKey::SearchMatchBadgeSuffix), "matches");
}

#[test]
fn the_short_query_hint_names_the_domain_minimum() {
    // The catalog text spells the number out; it must not drift from `MIN_QUERY_CHARS`.
    let digit = domain::search::MIN_QUERY_CHARS.to_string();
    for locale in [Locale::En, Locale::Ja] {
        assert!(
            t(locale, MessageKey::SearchQueryTooShort).contains(&digit),
            "{locale:?}"
        );
    }
}
