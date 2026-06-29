//! PDF Tile Viewer — Dioxus Desktop entrypoint (RFC 001, RFC 005).
//!
//! Boot order: resolve + bind PDFium on the engine worker thread first
//! (RFC 003), load settings (RFC 008), then launch the UI. A failed PDFium
//! bind does NOT abort: the app starts and shows a diagnostic panel
//! (RFC 003 §9).

mod app;
mod components;
mod i18n;
mod screens;
mod state;

use app_services::engine_boot;
use packaging::pdfium_bundle::PdfiumLoadMode;

fn main() {
    let mode = if cfg!(debug_assertions) {
        PdfiumLoadMode::Development
    } else {
        PdfiumLoadMode::ProductionBundled
    };
    let config = engine_boot::loader_config(mode, engine_boot::default_resource_root());

    let engine = match engine_boot::boot_engine(&config) {
        Ok((handle, thread)) => {
            // The engine thread lives for the whole process; the OS reaps
            // it at exit. Joining on UI close is deferred to RFC 004 M3.
            std::mem::forget(thread);
            Ok(handle)
        }
        Err(e) => Err(e.to_string()),
    };

    state::install_boot_state(engine);
    dioxus::launch(app::App);
}

#[cfg(test)]
mod tests;
