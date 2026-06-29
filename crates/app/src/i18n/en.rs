//! English reference catalog (RFC 017 §5). The exhaustive `match` makes
//! completeness compiler-enforced: a new `MessageKey` cannot ship without
//! an English string.

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    match key {
        MessageKey::AppTitle => "PDF Tile Viewer",
        MessageKey::DashboardHeading => "Open a PDF to get started",
        MessageKey::DashboardHint => "Pages are laid out as a tile grid for fast scanning.",
        MessageKey::OpenPdfButton => "Open PDF…",
        MessageKey::OpeningDocument => "Opening document…",
        MessageKey::RecentSessionsHeading => "This session",
        MessageKey::RecentSessionsEmpty => "Documents you open will be listed here.",
        MessageKey::ViewerBackToDashboard => "← Back",
        MessageKey::ViewerPageCountLabel => "Pages",
        MessageKey::ViewerPageOnePreview => "Page 1 preview",
        MessageKey::RenderingPage => "Rendering page…",
        MessageKey::EngineUnavailableTitle => "PDF engine unavailable",
        MessageKey::EngineUnavailableBody => {
            "The bundled PDFium library could not be loaded. Reinstalling the application usually fixes this."
        }
        MessageKey::ErrFileNotFound => "The file could not be found.",
        MessageKey::ErrNotAFile => "That item is not a file.",
        MessageKey::ErrWrongExtension => "Only .pdf files can be opened.",
        MessageKey::ErrNotAPdf => "This file is not a valid PDF document.",
        MessageKey::ErrUnreadable => "The file could not be read.",
        MessageKey::ErrEncryptedUnsupported => "Password-protected PDFs are not supported yet.",
        MessageKey::ErrPdfParseFailed => "The PDF could not be parsed.",
        MessageKey::ErrTooLarge => "This document exceeds the size limit.",
        MessageKey::ErrUnknown => "An unexpected error occurred.",
        MessageKey::ErrRenderFailed => "The page could not be rendered.",
        MessageKey::ErrMultipleFilesDropped => {
            "Only one PDF can be opened at a time. Drop a single file."
        }
        MessageKey::DropZoneHint => "Drop a PDF here or click \"Open PDF\u{2026}\"",
        MessageKey::RevealInFileManager => "Show in File Manager",
        MessageKey::ZenModeEnter => "Zen",
        MessageKey::ZenModeExit => "Exit Zen (Esc)",
    }
}
