//! Failure → message mapping (RFC 037 § 5.1).
//!
//! One exhaustive `match` per failure family (no wildcard arms: a new variant fails compilation
//! here, so "wrong message for this failure" cannot be written by accident). UI state stores the
//! resulting [`MessageKey`], never a formatted error. Messages are written for users and carry no
//! path (RFC 016 § 8). The internal detail of an error is not shown in the UI: debug builds print
//! it to stderr from [`log_detail`], release builds drop it.

use app_services::platform::PlatformError;
use domain::navigation::NavigationError;
use domain::render::RenderError;
use domain::search::SearchError;
use domain::text::TextLayerError;

use super::MessageKey;

pub fn search_error_key(error: &SearchError) -> MessageKey {
    match error {
        SearchError::DocumentNotOpen => MessageKey::ErrDocumentNotOpen,
        SearchError::QueryTooShort => MessageKey::SearchQueryTooShort,
        SearchError::EngineUnavailable => MessageKey::EngineUnavailableTitle,
        SearchError::SearchFailed(_) => MessageKey::ErrSearchFailed,
    }
}

pub fn render_error_key(error: &RenderError) -> MessageKey {
    match error {
        RenderError::DocumentNotOpen => MessageKey::ErrDocumentNotOpen,
        RenderError::PageIndexOutOfRange
        | RenderError::RenderFailed(_)
        | RenderError::EncodingFailed(_) => MessageKey::ErrRenderFailed,
        RenderError::EngineUnavailable => MessageKey::EngineUnavailableTitle,
    }
}

pub fn text_layer_error_key(error: &TextLayerError) -> MessageKey {
    match error {
        TextLayerError::DocumentNotOpen => MessageKey::ErrDocumentNotOpen,
        TextLayerError::EngineUnavailable => MessageKey::EngineUnavailableTitle,
        TextLayerError::PageOutOfBounds
        | TextLayerError::SegmentIndexOverflow
        | TextLayerError::StaleResult
        | TextLayerError::ExtractionFailed(_) => MessageKey::ZoomTextSelectionUnavailable,
    }
}

pub fn navigation_error_key(error: &NavigationError) -> MessageKey {
    match error {
        NavigationError::DocumentNotOpen => MessageKey::ErrDocumentNotOpen,
        NavigationError::EngineUnavailable => MessageKey::EngineUnavailableTitle,
        NavigationError::PageOutOfBounds
        | NavigationError::LinkIndexOverflow
        | NavigationError::ExtractionFailed(_) => MessageKey::ErrNavigationFailed,
    }
}

pub fn platform_error_key(error: &PlatformError) -> MessageKey {
    match error {
        PlatformError::NoParentDirectory => MessageKey::ErrRevealNoFolder,
        PlatformError::LaunchFailed(_) => MessageKey::ErrRevealFailed,
    }
}

/// Debug builds only: the technical detail of a failure goes to stderr (never into the UI, never in
/// release builds). The single place in the app that formats an error with `Debug`.
#[allow(clippy::use_debug)]
pub fn log_detail<E: std::fmt::Debug>(what: &str, error: &E) {
    if cfg!(debug_assertions) {
        eprintln!("[error] {what}: {error:?}");
    }
}

pub fn search_failed(error: &SearchError) -> MessageKey {
    log_detail("search", error);
    search_error_key(error)
}

pub fn render_failed(error: &RenderError) -> MessageKey {
    log_detail("render", error);
    render_error_key(error)
}

pub fn text_layer_failed(error: &TextLayerError) -> MessageKey {
    log_detail("text layer", error);
    text_layer_error_key(error)
}

pub fn navigation_failed(error: &NavigationError) -> MessageKey {
    log_detail("navigation", error);
    navigation_error_key(error)
}

pub fn platform_failed(error: &PlatformError) -> MessageKey {
    log_detail("platform", error);
    platform_error_key(error)
}

#[cfg(test)]
mod tests;
