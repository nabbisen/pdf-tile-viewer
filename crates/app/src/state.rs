//! UI-facing state (RFC 001 §7: Rust-first state, no JS state stores).
//!
//! `BootState` is produced in `main` before the Dioxus runtime exists and
//! handed over through a `OnceLock`; reactive state lives in signals owned
//! by the root component.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use domain::document::{DocumentSession, PageIndex};
use domain::render::ScaleBucket;

use crate::i18n::MessageKey;
use pdf_engine::worker::EngineHandle;

/// Result of pre-UI engine boot. `Err` carries a user-presentable
/// diagnostic (RFC 003 §9 — never panic on a missing/broken PDFium).
pub type EngineBoot = Result<EngineHandle, String>;

static BOOT: OnceLock<EngineBoot> = OnceLock::new();

pub fn install_boot_state(engine: EngineBoot) {
    let _ = BOOT.set(engine);
}

pub fn engine() -> Option<EngineHandle> {
    BOOT.get().and_then(|b| b.as_ref().ok()).cloned()
}

pub fn engine_boot_error() -> Option<String> {
    BOOT.get().and_then(|b| b.as_ref().err()).cloned()
}

/// Render state of a single page tile (RFC 007 §11, RFC 035 § 5.4).
///
/// `Rendering(gen)` carries the layout-generation at the time the render was
/// requested; stale completions (wrong gen) are silently dropped.
///
/// `Ready` records the [`ScaleBucket`] the image was rendered at: after a scale change
/// the old image stays on screen until a render for the new bucket replaces it.
#[derive(Clone, Debug, PartialEq)]
pub enum TileImageState {
    Pending,
    Rendering(u64),
    Ready {
        /// `data:image/png;base64,…` — provisional M4 transport (RFC 005 §7).
        uri: String,
        bucket: ScaleBucket,
    },
    /// Why the render failed, as a catalog key: no formatted error ever reaches the UI (RFC 037).
    Failed(MessageKey),
}

/// Per-document tile-image map: page index → render state.
pub type TileImages = HashMap<PageIndex, TileImageState>;

/// What the viewer screen shows for the vertical slice (RFC 005 §6):
/// the opened session plus the provisional page-1 preview.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenDocumentView {
    pub session: DocumentSession,
}

/// Top-level UI phase: dashboard or viewer (router deferred; the vertical
/// slice switches on state per RFC 005 §5 "minimal shell").
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Phase {
    #[default]
    Dashboard,
    Opening,
    Viewer(OpenDocumentView),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PasswordPromptState {
    pub path: PathBuf,
    pub rejected: bool,
    pub submitting: bool,
}

// ── Search state (RFC 010/011) ────────────────────────────────────────────────

use domain::search::{SearchHighlightSet, SearchQuery, SearchResultSet};

/// Top-level search lifecycle (RFC 010 §8).
#[derive(Clone, Debug, Default)]
pub enum SearchState {
    #[default]
    Idle,
    Searching,
    Results {
        results: SearchResultSet,
        highlights: SearchHighlightSet,
    },
    NoResults {
        query: SearchQuery,
    },
    /// A submitted query was shorter than `MIN_QUERY_CHARS`: shown as a hint, nothing was searched.
    QueryTooShort,
    Failed(MessageKey),
}
