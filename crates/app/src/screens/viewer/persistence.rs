//! Viewer effects that load or persist state (a pure move out of `viewer.rs`): the outline loader,
//! the viewport measurement, and the two settings writers (window size, scale/columns/page numbers).

use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::navigation_service::{NavigationService, default_outline_request};
use app_services::settings_service::SettingsStore;
use domain::document::{DocumentGeneration, DocumentId};
use domain::layout::PagesPerRowMode;
use domain::settings::PagesPerRowPreference;

use crate::components::outline_panel::OutlinePanelState;
use crate::i18n::MessageKey;
use crate::i18n::failure;

/// Loads the outline when the panel is opened (RFC 026 PR2).
pub(super) fn use_outline_loader(
    navigation_service: NavigationService,
    document_id: DocumentId,
    generation: DocumentGeneration,
    show_outline: Signal<bool>,
    mut outline_state: Signal<OutlinePanelState>,
) {
    use_effect(move || {
        if !*show_outline.read() {
            return;
        }

        let request = default_outline_request(document_id, generation);
        let service = navigation_service.clone();
        outline_state.set(OutlinePanelState::Loading);
        spawn(async move {
            match service.get_or_extract_outline(request).await {
                Ok(outline)
                    if outline.document_id == request.document_id
                        && outline.generation == request.generation =>
                {
                    outline_state.set(OutlinePanelState::Ready((*outline).clone()));
                }
                Ok(_) => {
                    outline_state.set(OutlinePanelState::Unavailable(
                        MessageKey::ErrNavigationFailed,
                    ));
                }
                Err(e) => {
                    outline_state.set(OutlinePanelState::Unavailable(failure::navigation_failed(
                        &e,
                    )));
                }
            }
        });
    });
}

/// Measures the web view's inner size once on mount.
pub(super) fn use_viewport_measurement(
    mut viewport_width: Signal<f32>,
    mut viewport_height: Signal<f32>,
) {
    use_effect(move || {
        spawn(async move {
            if let Ok(w) = eval("return window.innerWidth").join::<f64>().await
                && w > 0.0
            {
                viewport_width.set(w as f32);
            }
            if let Ok(h) = eval("return window.innerHeight").join::<f64>().await
                && h > 0.0
            {
                viewport_height.set(h as f32);
            }
        });
    });
}

/// Saves the window size when the viewport changes (RFC 008 window settings).
pub(super) fn use_window_size_persistence(
    store: SettingsStore,
    viewport_width: Signal<f32>,
    viewport_height: Signal<f32>,
) {
    use_effect(move || {
        let w = *viewport_width.read();
        let h = *viewport_height.read();
        if w > 100.0 && h > 100.0 {
            let store = store.clone();
            spawn(async move {
                let (mut s, _) = store.load();
                s.window.width = Some(w.round() as u32);
                s.window.height = Some(h.round() as u32);
                let _ = store.save(&s);
            });
        }
    });
}

/// Saves scale, pages per row and the page-number toggle whenever they change.
pub(super) fn use_viewer_settings_persistence(
    store: SettingsStore,
    scale: Signal<f32>,
    mode: Signal<PagesPerRowMode>,
    show_page_numbers: Signal<bool>,
) {
    use_effect(move || {
        let (ns, nm, nsp) = (*scale.read(), *mode.read(), *show_page_numbers.read());
        let store = store.clone();
        spawn(async move {
            let (mut s, _) = store.load();
            s.viewer.default_scale = ns;
            s.viewer.pages_per_row = match nm {
                PagesPerRowMode::Auto => PagesPerRowPreference::Auto,
                PagesPerRowMode::Fixed(n) => PagesPerRowPreference::Fixed(n),
            };
            s.viewer.show_page_numbers = nsp;
            let _ = store.save(&s);
        });
    });
}
