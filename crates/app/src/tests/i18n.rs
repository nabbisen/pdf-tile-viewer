use crate::i18n::{Locale, MessageKey, en, t};

use app_services::document_service::OpenError;
use domain::document::DocumentError;

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
fn encrypted_pdf_open_error_uses_specific_message_key() {
    let error = OpenError::Engine(DocumentError::EncryptedUnsupported);
    assert_eq!(
        crate::i18n::open_error_key(&error),
        MessageKey::ErrEncryptedUnsupported
    );
}
