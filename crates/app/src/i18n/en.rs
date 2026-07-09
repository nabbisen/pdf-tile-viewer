//! English reference catalog (RFC 017 §5). The exhaustive `match` makes
//! completeness compiler-enforced: a new `MessageKey` cannot ship without
//! an English string.

use super::MessageKey;

pub fn message(key: MessageKey) -> &'static str {
    match key {
        MessageKey::AppTitle => "PDF Tile Viewer",
        MessageKey::DashboardHeading => "Open a PDF to get started",
        MessageKey::OpenPdfButton => "Open PDF…",
        MessageKey::OpeningDocument => "Opening document…",
        MessageKey::RecentSessionsHeading => "This session",
        MessageKey::RecentSessionsEmpty => "Documents you open will be listed here.",
        MessageKey::ViewerBackToDashboard => "← Back",
        MessageKey::ViewerPageOnePreview => "Page 1 preview",
        MessageKey::RenderingPage => "Rendering page…",
        MessageKey::EngineUnavailableTitle => "PDF engine unavailable",
        MessageKey::EngineUnavailableBody => {
            "PDF Tile Viewer needs its bundled PDF engine library to open, render, and search PDFs, but that library was not found."
        }
        MessageKey::EngineUnavailableDevelopmentHelp => {
            "If you installed the app, download the full official release archive again, extract it into a directory, and launch it without moving `bin/` away from `resources/`. If you are building from source, see `docs/src/contributors/dev.md` for PDFium setup."
        }
        MessageKey::EngineUnavailablePackagedHelp => {
            "This install appears incomplete. Re-extract the official release archive and keep `bin/` and `resources/` together in the same directory."
        }
        MessageKey::DiagnosticDetailsLabel => "Diagnostic detail",
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
        MessageKey::PasswordPromptTitle => "Password required",
        MessageKey::PasswordPromptBody => "This PDF is password-protected.",
        MessageKey::PasswordPromptField => "Password",
        MessageKey::PasswordPromptRejected => "The password was not accepted.",
        MessageKey::PasswordPromptOpen => "Open",
        MessageKey::PasswordPromptCancel => "Cancel",
        MessageKey::RevealInFileManager => "Show in File Manager",
        MessageKey::OutlinePanelLabel => "Outline",
        MessageKey::OutlineClose => "Close outline",
        MessageKey::OutlineLoading => "Loading outline…",
        MessageKey::OutlineEmpty => "No outline",
        MessageKey::OutlineUnavailable => "Outline unavailable",
        MessageKey::OutlineUntitled => "Untitled",
        MessageKey::OutlineUnsupported => "Unsupported outline target",
        MessageKey::OutlineExpand => "Expand",
        MessageKey::OutlineCollapse => "Collapse",
        MessageKey::SearchButton => "Search",
        MessageKey::SearchClear => "Clear",
        MessageKey::SearchPlaceholder => "Find text…",
        MessageKey::SearchSummaryMatches => "matches found",
        MessageKey::SearchNoMatches => "No matches found.",
        MessageKey::Searching => "Searching…",
        MessageKey::SearchErrorPrefix => "Error",
        MessageKey::SearchPagesPrefix => "p.",
        MessageKey::SearchMatchBadgeSuffix => "matches",
        MessageKey::CloseSearch => "Close search",
        MessageKey::ZoomClose => "Close (Esc)",
        MessageKey::ZoomPrevPage => "← Previous",
        MessageKey::ZoomNextPage => "Next →",
        MessageKey::ZoomIn => "Zoom in",
        MessageKey::ZoomOut => "Zoom out",
        MessageKey::PageZoomView => "Page zoom view",
        MessageKey::PageImageAltPrefix => "Page",
        MessageKey::ZoomPageIndicator => "Page",
        MessageKey::ZoomScaleLabel => "Zoom",
        MessageKey::ZoomTextSelectionUnavailable => "Text selection is unavailable for this page.",
        MessageKey::ZoomModeEnter => "Zoom",
        MessageKey::MoreControls => "More controls",
        MessageKey::ColumnsLabel => "Columns",
        MessageKey::ColumnsAuto => "Auto",
        MessageKey::PageNumbersLabel => "Page numbers",
        MessageKey::JumpToPageLabel => "Go to page",
        MessageKey::JumpGoButton => "Go",
        MessageKey::ZenModeEnter => "Zen",
        MessageKey::ZenModeExit => "Exit Zen (Esc)",
        MessageKey::ReopenDocument => "Reopen",
        MessageKey::PageCountSuffix => "p",
    }
}
