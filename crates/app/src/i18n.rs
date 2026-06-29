//! Internationalization (RFC 017).
//!
//! - `Locale` is a closed enum; `En` is the complete reference catalog,
//!   `Ja` is the first translation.
//! - `MessageKey` is a closed enum: adding UI text means adding a key,
//!   and the `en` catalog match is compiler-enforced to stay complete.
//! - Lookup falls back to English when a translation is missing (§8).
//! - `domain` / `pdf_engine` never contain localized text; error enums are
//!   mapped to message keys here at the UI boundary (§6).

pub mod en;
pub mod ja;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Locale {
    #[default]
    En,
    Ja,
}

impl Locale {
    /// Resolve from settings (`ui.locale`), falling back to the system
    /// locale, then English (RFC 017 §7).
    pub fn resolve(preference: Option<&str>) -> Locale {
        match preference {
            Some(tag) => Locale::from_tag(tag),
            None => Locale::from_system(),
        }
    }

    pub fn from_tag(tag: &str) -> Locale {
        if tag.eq_ignore_ascii_case("ja") || tag.to_ascii_lowercase().starts_with("ja-") {
            Locale::Ja
        } else {
            Locale::En
        }
    }

    fn from_system() -> Locale {
        std::env::var("LANG")
            .ok()
            .map(|lang| Locale::from_tag(&lang))
            .unwrap_or(Locale::En)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKey {
    AppTitle,
    DashboardHeading,
    DashboardHint,
    OpenPdfButton,
    OpeningDocument,
    RecentSessionsHeading,
    RecentSessionsEmpty,
    ViewerBackToDashboard,
    ViewerPageCountLabel,
    ViewerPageOnePreview,
    RenderingPage,
    EngineUnavailableTitle,
    EngineUnavailableBody,
    ErrFileNotFound,
    ErrNotAFile,
    ErrWrongExtension,
    ErrNotAPdf,
    ErrUnreadable,
    ErrEncryptedUnsupported,
    ErrPdfParseFailed,
    ErrTooLarge,
    ErrUnknown,
    ErrMultipleFilesDropped,
    ErrRenderFailed,
    DropZoneHint,
    RevealInFileManager,
    SearchButton,
    SearchClear,
    SearchPlaceholder,
    SearchSummaryMatches, // "{n} matches on {p} pages"
    SearchNoMatches,
    Searching,
    ZenModeEnter,
    ZenModeExit,
}

impl MessageKey {
    /// All keys, for catalog completeness tests (RFC 017 §9).
    #[allow(dead_code)]
    pub const ALL: &[MessageKey] = &[
        MessageKey::AppTitle,
        MessageKey::DashboardHeading,
        MessageKey::DashboardHint,
        MessageKey::OpenPdfButton,
        MessageKey::OpeningDocument,
        MessageKey::RecentSessionsHeading,
        MessageKey::RecentSessionsEmpty,
        MessageKey::ViewerBackToDashboard,
        MessageKey::ViewerPageCountLabel,
        MessageKey::ViewerPageOnePreview,
        MessageKey::RenderingPage,
        MessageKey::EngineUnavailableTitle,
        MessageKey::EngineUnavailableBody,
        MessageKey::ErrFileNotFound,
        MessageKey::ErrNotAFile,
        MessageKey::ErrWrongExtension,
        MessageKey::ErrNotAPdf,
        MessageKey::ErrUnreadable,
        MessageKey::ErrEncryptedUnsupported,
        MessageKey::ErrPdfParseFailed,
        MessageKey::ErrTooLarge,
        MessageKey::ErrUnknown,
        MessageKey::ErrRenderFailed,
        MessageKey::ErrMultipleFilesDropped,
        MessageKey::DropZoneHint,
        MessageKey::RevealInFileManager,
        MessageKey::SearchButton,
        MessageKey::SearchClear,
        MessageKey::SearchPlaceholder,
        MessageKey::SearchSummaryMatches,
        MessageKey::SearchNoMatches,
        MessageKey::Searching,
        MessageKey::ZenModeEnter,
        MessageKey::ZenModeExit,
    ];
}

/// Translate a key for the given locale, falling back to English.
pub fn t(locale: Locale, key: MessageKey) -> &'static str {
    match locale {
        Locale::En => en::message(key),
        Locale::Ja => ja::message(key).unwrap_or_else(|| en::message(key)),
    }
}

/// Map intake/engine errors to message keys at the UI boundary (§6).
pub fn open_error_key(error: &app_services::document_service::OpenError) -> MessageKey {
    use app_services::document_service::{IntakeRejection, OpenError};
    use domain::document::DocumentError;
    match error {
        OpenError::Rejected(IntakeRejection::NotFound) => MessageKey::ErrFileNotFound,
        OpenError::Rejected(IntakeRejection::NotAFile) => MessageKey::ErrNotAFile,
        OpenError::Rejected(IntakeRejection::WrongExtension) => MessageKey::ErrWrongExtension,
        OpenError::Rejected(IntakeRejection::NotAPdf) => MessageKey::ErrNotAPdf,
        OpenError::Rejected(IntakeRejection::Unreadable) => MessageKey::ErrUnreadable,
        OpenError::Engine(DocumentError::FileNotFound) => MessageKey::ErrFileNotFound,
        OpenError::Engine(DocumentError::FileNotReadable) => MessageKey::ErrUnreadable,
        OpenError::Engine(DocumentError::UnsupportedFile) => MessageKey::ErrNotAPdf,
        OpenError::Engine(DocumentError::EncryptedUnsupported) => {
            MessageKey::ErrEncryptedUnsupported
        }
        OpenError::Engine(DocumentError::PdfParseFailed) => MessageKey::ErrPdfParseFailed,
        OpenError::Engine(DocumentError::TooLargeForPolicy) => MessageKey::ErrTooLarge,
        OpenError::Engine(DocumentError::PdfiumUnavailable) | OpenError::EngineUnavailable => {
            MessageKey::EngineUnavailableTitle
        }
        OpenError::Engine(DocumentError::Unknown(_)) => MessageKey::ErrUnknown,
    }
}
