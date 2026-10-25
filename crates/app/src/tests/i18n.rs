use crate::i18n::{Locale, MessageKey, choose_locale, en, t};

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

// ── RFC 043: system locale ────────────────────────────────────────────────────

fn os(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|tag| tag.to_string()).collect()
}

#[test]
fn a_real_lang_decides_alone() {
    for lang in [
        "ja_JP.UTF-8",
        "ja_JP",
        "ja",
        "ja_JP.UTF-8@euro",
        "JA_JP.utf8",
        " ja_JP.UTF-8 ",
    ] {
        assert_eq!(choose_locale(Some(lang), &[]), Locale::Ja, "{lang:?}");
        // and the OS list is not consulted
        assert_eq!(
            choose_locale(Some(lang), &os(&["en-US"])),
            Locale::Ja,
            "{lang:?}"
        );
    }
    for lang in ["en_US.UTF-8", "en_US", "en", "en_GB.UTF-8"] {
        assert_eq!(
            choose_locale(Some(lang), &os(&["ja-JP"])),
            Locale::En,
            "{lang:?}"
        );
    }
}

#[test]
fn an_unsupported_lang_means_english_without_asking_the_os() {
    // The documented behavior: LANG decides when it is a real locale (Japanese or anything else).
    assert_eq!(
        choose_locale(Some("fr_FR.UTF-8"), &os(&["ja-JP"])),
        Locale::En
    );
    assert_eq!(choose_locale(Some("de_DE"), &[]), Locale::En);
}

#[test]
fn c_posix_and_empty_lang_count_as_unset() {
    let japanese = os(&["ja-JP"]);
    for lang in [
        "C",
        "c",
        "POSIX",
        "posix",
        "C.UTF-8",
        "c.utf8",
        "POSIX.UTF-8",
        "",
        "  ",
    ] {
        assert_eq!(choose_locale(Some(lang), &japanese), Locale::Ja, "{lang:?}");
    }
    assert_eq!(choose_locale(None, &japanese), Locale::Ja);
}

#[test]
fn the_os_list_is_walked_in_order() {
    assert_eq!(choose_locale(None, &os(&["ja-JP"])), Locale::Ja);
    assert_eq!(choose_locale(None, &os(&["ja"])), Locale::Ja);
    assert_eq!(choose_locale(None, &os(&["en-US"])), Locale::En);
    assert_eq!(
        choose_locale(None, &os(&["fr", "ja"])),
        Locale::Ja,
        "first supported wins"
    );
    assert_eq!(
        choose_locale(None, &os(&["fr-FR", "en-US", "ja-JP"])),
        Locale::En
    );
    assert_eq!(choose_locale(None, &os(&["ja-JP", "en-US"])), Locale::Ja);
}

#[test]
fn only_unsupported_languages_or_an_empty_list_give_english() {
    assert_eq!(choose_locale(None, &os(&["fr"])), Locale::En);
    assert_eq!(
        choose_locale(None, &os(&["fr-FR", "de-DE", "zh-Hans-CN"])),
        Locale::En
    );
    assert_eq!(choose_locale(None, &[]), Locale::En);
    assert_eq!(choose_locale(Some("C"), &[]), Locale::En);
}

#[test]
fn the_os_list_accepts_bcp47_and_posix_style_tags() {
    for tag in [
        "ja-JP",
        "ja_JP",
        "ja_JP.UTF-8",
        "ja-JP-u-ca-japanese",
        "JA-jp",
        "ja@calendar=japanese",
    ] {
        assert_eq!(choose_locale(None, &os(&[tag])), Locale::Ja, "{tag:?}");
    }
}

#[test]
fn similar_language_codes_are_not_japanese() {
    // `jv` (Javanese), `jam`, `jap` are other languages; only the exact subtag `ja` counts.
    for tag in ["jv", "jv-ID", "jam", "jap", "japanese", "j"] {
        assert_eq!(choose_locale(None, &os(&[tag])), Locale::En, "{tag:?}");
    }
}

#[test]
fn the_explicit_setting_still_wins_over_everything() {
    // `resolve` consults the system only for `None`.
    assert_eq!(Locale::resolve(Some("en")), Locale::En);
    assert_eq!(Locale::resolve(Some("ja")), Locale::Ja);
    assert_eq!(Locale::resolve(Some("ja-JP")), Locale::Ja);
    assert_eq!(Locale::resolve(Some("fr")), Locale::En);
}

#[test]
fn lang_ja_jp_utf8_selects_japanese_where_the_old_lookup_did_not() {
    // `Locale::from_tag("ja_JP.UTF-8")` (the pre-RFC 043 `LANG` handling) gives English, because it
    // only understood `ja` and `ja-…`. The usual POSIX spelling now works.
    assert_eq!(Locale::from_tag("ja_JP.UTF-8"), Locale::En);
    assert_eq!(choose_locale(Some("ja_JP.UTF-8"), &[]), Locale::Ja);
}
