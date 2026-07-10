//! App-specific directories (RFC 008 §7: settings live in the platform
//! config dir, not next to the executable).

use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "pdf-tile-viewer";

/// `<config_dir>/pdf-tile-viewer`
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(APP_DIR_NAME))
}

/// `<config_dir>/pdf-tile-viewer/settings.json`
pub fn settings_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

/// `<cache_dir>/pdf-tile-viewer` — reserved for extracted PDFium
/// (RFC 003 Stage 3) and future render caches.
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join(APP_DIR_NAME))
}
