//! UI-facing state (RFC 001 §7: Rust-first state, no JS state stores).
//!
//! `BootState` is produced in `main` before the Dioxus runtime exists and
//! handed over through a `OnceLock`; reactive state lives in signals owned
//! by the root component.

use std::collections::HashMap;
use std::sync::OnceLock;

use domain::document::{DocumentSession, PageIndex};
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

/// Render state of a single page tile (RFC 007 §11).
///
/// `Rendering(gen)` carries the layout-generation at the time the render was
/// requested; stale completions (wrong gen) are silently dropped.
#[derive(Clone, Debug, PartialEq)]
pub enum TileImageState {
    Pending,
    Rendering(u64),
    /// `data:image/png;base64,…` — provisional M4 transport (RFC 005 §7).
    Ready(String),
    Failed(String),
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
pub enum Phase {
    #[default]
    Dashboard,
    Opening,
    Viewer(OpenDocumentView),
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
    Failed(String),
}
