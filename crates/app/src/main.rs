//! PDF Tile Viewer — Dioxus Desktop entrypoint (RFC 001, M9/beta).
//!
//! Boot order:
//! 1. Load settings to restore window size (RFC 008 window settings).
//! 2. Resolve + bind PDFium on the engine worker thread (RFC 003).
//! 3. Launch Dioxus Desktop with the configured window.

mod app;
mod components;
mod i18n;
mod screens;
mod state;

use app_services::engine_boot;
use app_services::settings_service::SettingsStore;
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
            std::mem::forget(thread);
            Ok(handle)
        }
        Err(e) => Err(e.to_string()),
    };
    state::install_boot_state(engine);

    // Restore saved window dimensions (RFC 008 §8).
    let (settings, _) = SettingsStore::at_default_location().load();
    let win_w = settings.window.width.unwrap_or(1200).max(400) as f64;
    let win_h = settings.window.height.unwrap_or(800).max(300) as f64;

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("PDF Tile Viewer")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(win_w, win_h)),
            ),
        )
        .launch(app::App);
}
