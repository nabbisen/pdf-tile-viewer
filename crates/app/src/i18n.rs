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

    /// The only place that reads the environment and the operating system for the interface
    /// language (RFC 043); every decision is made by the pure [`choose_locale`].
    fn from_system() -> Locale {
        let lang = std::env::var("LANG").ok();
        let os_locales: Vec<String> = sys_locale::get_locales().collect();
        choose_locale(lang.as_deref(), &os_locales)
    }
}

/// The language subtag of a locale tag, lowercase: BCP 47 (`ja-JP`) and POSIX (`ja_JP.UTF-8@euro`)
/// spellings alike are split on `-`, `_`, `.` and `@`; the first part is the language.
fn language_subtag(tag: &str) -> String {
    tag.trim()
        .split(['-', '_', '.', '@'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// The supported language named by a locale tag; `None` for an unsupported language and for
/// `C` / `POSIX` / empty input.
fn supported_language(tag: &str) -> Option<Locale> {
    match language_subtag(tag).as_str() {
        "ja" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        _ => None,
    }
}

/// `LANG` counts as unset when it is empty or the `C` / `POSIX` locale (any suffix, any case).
fn lang_is_unset(lang: &str) -> bool {
    matches!(language_subtag(lang).as_str(), "" | "c" | "posix")
}

/// The interface language when the user has not forced one (RFC 043 § 4). Pure.
///
/// 1. `LANG` set to a real locale decides alone: Japanese for `ja…`, English for anything else
///    (the documented Linux behavior).
/// 2. Otherwise the first language of `os_locales` (most preferred first) that is supported;
///    a list with only unsupported languages, or an empty list, gives English.
pub fn choose_locale(lang_env: Option<&str>, os_locales: &[String]) -> Locale {
    if let Some(lang) = lang_env.filter(|lang| !lang_is_unset(lang)) {
        return supported_language(lang).unwrap_or(Locale::En);
    }
    os_locales
        .iter()
        .find_map(|tag| supported_language(tag))
        .unwrap_or(Locale::En)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKey {
    AppTitle,
    DashboardHeading,
    OpenPdfButton,
    OpeningDocument,
    RecentSessionsHeading,
    RecentSessionsEmpty,
    ViewerBackToDashboard,
    ViewerPageOnePreview,
    RenderingPage,
    EngineUnavailableTitle,
    EngineUnavailableBody,
    EngineUnavailableDevelopmentHelp,
    EngineUnavailablePackagedHelp,
    DiagnosticDetailsLabel,
    ErrFileNotFound,
    ErrNotAFile,
    ErrWrongExtension,
    ErrNotAPdf,
    ErrUnreadable,
    ErrPdfParseFailed,
    ErrTooLarge,
    ErrUnknown,
    ErrMultipleFilesDropped,
    ErrRenderFailed,
    DropZoneHint,
    PasswordPromptTitle,
    PasswordPromptBody,
    PasswordPromptField,
    PasswordPromptRejected,
    PasswordPromptOpen,
    PasswordPromptCancel,
    RevealInFileManager,
    OutlinePanelLabel,
    OutlineClose,
    OutlineLoading,
    OutlineEmpty,
    OutlineUntitled,
    OutlineUnsupported,
    OutlineExpand,
    OutlineCollapse,
    SearchButton,
    SearchClear,
    SearchPlaceholder,
    SearchSummaryMatches, // summary text; distinct from badge text for context-specific copy.
    SearchNoMatches,
    Searching,
    SearchPagesPrefix,
    SearchMatchBadgeSuffix,
    CloseSearch,
    ZoomClose,
    ZoomPrevPage,
    ZoomNextPage,
    ZoomIn,
    ZoomOut,
    PageZoomView,
    PageImageAltPrefix, // image alt text; distinct from the visible page indicator.
    ZoomPageIndicator,
    ZoomScaleLabel,
    ZoomTextSelectionUnavailable,
    ZoomLinkInternal,
    ZoomLinkExternalDisabled,
    ZoomLinkDisabled,
    ExternalUriDialogTitle,
    ExternalUriCopyOnlyBody,
    ExternalUriRejectedBody,
    ExternalUriOpenUnavailable,
    ExternalUriTargetLabel,
    ExternalUriCopy,
    ExternalUriCopyDone,
    ExternalUriCopyFailed,
    ExternalUriCancel,
    ZoomModeEnter,
    MoreControls,
    ColumnsLabel,
    ColumnsAuto,
    ColumnsFixed,
    PageNumbersLabel,
    JumpToPageLabel,
    JumpGoButton,
    ZenModeEnter,
    ZenModeExit,
    ReopenDocument,
    PageCountSuffix,
    ErrDocumentNotOpen,
    ErrSearchFailed,
    ErrNavigationFailed,
    ErrRevealNoFolder,
    ErrRevealFailed,
    SearchQueryTooShort,
    JumpInvalidPage,
    SearchToggleLabel,
    TileSizeLabel,
    SearchSummaryMatchOne,
    SearchMatchBadgeOne,
}

impl MessageKey {
    /// All keys, for catalog completeness tests (RFC 017 §9).
    #[allow(dead_code)]
    pub const ALL: &[MessageKey] = &[
        MessageKey::AppTitle,
        MessageKey::DashboardHeading,
        MessageKey::OpenPdfButton,
        MessageKey::OpeningDocument,
        MessageKey::RecentSessionsHeading,
        MessageKey::RecentSessionsEmpty,
        MessageKey::ViewerBackToDashboard,
        MessageKey::ViewerPageOnePreview,
        MessageKey::RenderingPage,
        MessageKey::EngineUnavailableTitle,
        MessageKey::EngineUnavailableBody,
        MessageKey::EngineUnavailableDevelopmentHelp,
        MessageKey::EngineUnavailablePackagedHelp,
        MessageKey::DiagnosticDetailsLabel,
        MessageKey::ErrFileNotFound,
        MessageKey::ErrNotAFile,
        MessageKey::ErrWrongExtension,
        MessageKey::ErrNotAPdf,
        MessageKey::ErrUnreadable,
        MessageKey::ErrPdfParseFailed,
        MessageKey::ErrTooLarge,
        MessageKey::ErrUnknown,
        MessageKey::ErrRenderFailed,
        MessageKey::ErrMultipleFilesDropped,
        MessageKey::DropZoneHint,
        MessageKey::PasswordPromptTitle,
        MessageKey::PasswordPromptBody,
        MessageKey::PasswordPromptField,
        MessageKey::PasswordPromptRejected,
        MessageKey::PasswordPromptOpen,
        MessageKey::PasswordPromptCancel,
        MessageKey::RevealInFileManager,
        MessageKey::OutlinePanelLabel,
        MessageKey::OutlineClose,
        MessageKey::OutlineLoading,
        MessageKey::OutlineEmpty,
        MessageKey::OutlineUntitled,
        MessageKey::OutlineUnsupported,
        MessageKey::OutlineExpand,
        MessageKey::OutlineCollapse,
        MessageKey::SearchButton,
        MessageKey::SearchClear,
        MessageKey::SearchPlaceholder,
        MessageKey::SearchSummaryMatches,
        MessageKey::SearchNoMatches,
        MessageKey::Searching,
        MessageKey::SearchPagesPrefix,
        MessageKey::SearchMatchBadgeSuffix,
        MessageKey::CloseSearch,
        MessageKey::ZoomClose,
        MessageKey::ZoomPrevPage,
        MessageKey::ZoomNextPage,
        MessageKey::ZoomIn,
        MessageKey::ZoomOut,
        MessageKey::PageZoomView,
        MessageKey::PageImageAltPrefix,
        MessageKey::ZoomPageIndicator,
        MessageKey::ZoomScaleLabel,
        MessageKey::ZoomTextSelectionUnavailable,
        MessageKey::ZoomLinkInternal,
        MessageKey::ZoomLinkExternalDisabled,
        MessageKey::ZoomLinkDisabled,
        MessageKey::ExternalUriDialogTitle,
        MessageKey::ExternalUriCopyOnlyBody,
        MessageKey::ExternalUriRejectedBody,
        MessageKey::ExternalUriOpenUnavailable,
        MessageKey::ExternalUriTargetLabel,
        MessageKey::ExternalUriCopy,
        MessageKey::ExternalUriCopyDone,
        MessageKey::ExternalUriCopyFailed,
        MessageKey::ExternalUriCancel,
        MessageKey::ZoomModeEnter,
        MessageKey::MoreControls,
        MessageKey::ColumnsLabel,
        MessageKey::ColumnsAuto,
        MessageKey::ColumnsFixed,
        MessageKey::PageNumbersLabel,
        MessageKey::JumpToPageLabel,
        MessageKey::JumpGoButton,
        MessageKey::ZenModeEnter,
        MessageKey::ZenModeExit,
        MessageKey::ReopenDocument,
        MessageKey::PageCountSuffix,
        MessageKey::ErrDocumentNotOpen,
        MessageKey::ErrSearchFailed,
        MessageKey::ErrNavigationFailed,
        MessageKey::ErrRevealNoFolder,
        MessageKey::ErrRevealFailed,
        MessageKey::SearchQueryTooShort,
        MessageKey::JumpInvalidPage,
        MessageKey::SearchToggleLabel,
        MessageKey::TileSizeLabel,
        MessageKey::SearchSummaryMatchOne,
        MessageKey::SearchMatchBadgeOne,
    ];
}

/// Translate a key for the given locale, falling back to English.
pub fn t(locale: Locale, key: MessageKey) -> &'static str {
    match locale {
        Locale::En => en::message(key),
        Locale::Ja => ja::message(key).unwrap_or_else(|| en::message(key)),
    }
}

/// Map intake/engine errors to message keys at the UI boundary (§6). Exhaustive: a new
/// `OpenFailure` variant fails compilation here. `PasswordRequired` cannot arrive (RFC 037 § 5.2).
pub fn open_error_key(error: &app_services::document_service::OpenError) -> MessageKey {
    use app_services::document_service::{IntakeRejection, OpenError};
    use domain::document::OpenFailure;
    match error {
        OpenError::Rejected(IntakeRejection::NotFound) => MessageKey::ErrFileNotFound,
        OpenError::Rejected(IntakeRejection::NotAFile) => MessageKey::ErrNotAFile,
        OpenError::Rejected(IntakeRejection::WrongExtension) => MessageKey::ErrWrongExtension,
        OpenError::Rejected(IntakeRejection::NotAPdf) => MessageKey::ErrNotAPdf,
        OpenError::Rejected(IntakeRejection::Unreadable) => MessageKey::ErrUnreadable,
        OpenError::Engine(OpenFailure::FileNotFound) => MessageKey::ErrFileNotFound,
        OpenError::Engine(OpenFailure::FileNotReadable) => MessageKey::ErrUnreadable,
        OpenError::Engine(OpenFailure::UnsupportedFile) => MessageKey::ErrNotAPdf,
        OpenError::Engine(OpenFailure::PdfParseFailed) => MessageKey::ErrPdfParseFailed,
        OpenError::Engine(OpenFailure::TooLargeForPolicy) => MessageKey::ErrTooLarge,
        OpenError::Engine(OpenFailure::PdfiumUnavailable) | OpenError::EngineUnavailable => {
            MessageKey::EngineUnavailableTitle
        }
        OpenError::Engine(OpenFailure::Unknown(_)) => MessageKey::ErrUnknown,
    }
}

/// Plural-aware choice between two catalog strings (RFC 037 § 5.6.5). English has one singular
/// (exactly 1); Japanese has no plural form, so `other` is always used.
pub fn plural(
    locale: Locale,
    count: usize,
    one: &'static str,
    other: &'static str,
) -> &'static str {
    match locale {
        Locale::En if count == 1 => one,
        Locale::En | Locale::Ja => other,
    }
}

pub mod failure;
