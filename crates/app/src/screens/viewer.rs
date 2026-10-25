//! Document viewer screen (M9: viewport-aware rendering, scroll tracking).

mod render;

use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::navigation_service::{NavigationService, default_outline_request};
use app_services::platform::reveal_in_file_manager;
use app_services::render_service::RenderService;
use app_services::session_service::SessionService;
use app_services::settings_service::SettingsStore;
use domain::document::PageIndex;
use domain::layout::{
    LayoutGeneration, PagesPerRowMode, ScaleChange, TileLayoutInput, ViewerScale, compute_layout,
};
use domain::search::{PageHighlightSet, PageSearchSummary};
use domain::settings::{AppSettingsV1, PagesPerRowPreference};
use domain::shortcuts::{Dismissal, Layer, ShortcutAction, ShortcutContext, resolve};

use crate::components::modal;
use crate::components::outline_panel::{OutlinePanel, OutlinePanelState};
use crate::components::search_panel::SearchPanel;
use crate::components::tile_grid::TileGrid;
use crate::components::viewer_controls::ViewerControls;
use crate::components::zoom_overlay::ZoomOverlay;
use crate::i18n::{Locale, MessageKey, t};
use crate::keys::key_descriptor;
use crate::state::{OpenDocumentView, Phase, SearchState, TileImageState, TileImages};

const TILE_GAP_PX: f32 = 12.0;
const TILE_PADDING_PX: f32 = 16.0;
const TILE_LABEL_HEIGHT_PX: f32 = 20.0;
const DEFAULT_VIEWPORT_PX: f32 = 1200.0;
const DEFAULT_VIEWPORT_HEIGHT_PX: f32 = 800.0;

#[component]
pub fn Viewer(view: OpenDocumentView, mut phase: Signal<Phase>) -> Element {
    let locale: Memo<Locale> = use_context();
    let settings: Signal<AppSettingsV1> = use_context();
    let settings_store: SettingsStore = use_context();
    let navigation_service: NavigationService = use_context();
    let render_service: RenderService = use_context();
    let session_service: SessionService = use_context();

    let session = view.session.clone();

    // The viewer owns the open session: leaving it (back to the dashboard) closes the
    // session in every cache and in the engine (RFC 035). Plain values only, no `spawn`.
    use_drop({
        let (id, generation) = (session.id, session.generation);
        move || session_service.close(id, generation)
    });
    let page_count = session.pages.len();
    let display_name = session.display_name.clone();
    let doc_path = match &session.source {
        domain::document::DocumentSource::LocalFile { path, .. } => Some(path.clone()),
    };

    // ── Viewer signals ────────────────────────────────────────────────────
    let mut scale = use_signal(|| settings.read().viewer.default_scale);
    let mode = use_signal(|| match settings.read().viewer.pages_per_row {
        PagesPerRowPreference::Auto => PagesPerRowMode::Auto,
        PagesPerRowPreference::Fixed(n) => PagesPerRowMode::Fixed(n),
    });
    let show_page_numbers = use_signal(|| settings.read().viewer.show_page_numbers);
    let mut zen_mode: Signal<bool> = use_signal(|| false);
    let mut show_outline: Signal<bool> = use_signal(|| false);
    let mut show_search: Signal<bool> = use_signal(|| false);
    let mut viewport_width: Signal<f32> = use_signal(|| DEFAULT_VIEWPORT_PX);
    let mut viewport_height: Signal<f32> = use_signal(|| DEFAULT_VIEWPORT_HEIGHT_PX);
    let mut scroll_y: Signal<f32> = use_signal(|| 0.0);
    let mut tile_images: Signal<TileImages> = use_signal(|| {
        session
            .pages
            .iter()
            .map(|p| (p.page_index, TileImageState::Pending))
            .collect()
    });
    let mut render_gen: Signal<u64> = use_signal(|| 0u64);
    let mut reveal_error: Signal<Option<String>> = use_signal(|| None);
    let search_state: Signal<SearchState> = use_signal(SearchState::default);
    let mut outline_state: Signal<OutlinePanelState> = use_signal(|| OutlinePanelState::Idle);
    let mut zoom_page: Signal<Option<PageIndex>> = use_signal(|| None);
    let page_descriptors = session.pages.clone();
    let rs_for_search = render_service.clone();

    // ── Outline loading (RFC 026 PR2) ─────────────────────────────────────
    let nav_for_outline = navigation_service.clone();
    let outline_session = session.clone();
    use_effect(move || {
        if !*show_outline.read() {
            return;
        }

        let request = default_outline_request(outline_session.id, outline_session.generation);
        let service = nav_for_outline.clone();
        outline_state.set(OutlinePanelState::Loading);
        spawn(async move {
            match service.get_or_extract_outline(request).await {
                Ok(outline)
                    if outline.document_id == request.document_id
                        && outline.generation == request.generation =>
                {
                    outline_state.set(OutlinePanelState::Ready((*outline).clone()));
                }
                Ok(_) | Err(_) => {
                    outline_state.set(OutlinePanelState::Unavailable);
                }
            }
        });
    });

    // ── Measure viewport on mount ─────────────────────────────────────────
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

    // ── Save window size on viewport change (RFC 008 window settings) ────────
    let store_for_size = settings_store.clone();
    use_effect(move || {
        let w = *viewport_width.read();
        let h = *viewport_height.read();
        if w > 100.0 && h > 100.0 {
            let store = store_for_size.clone();
            spawn(async move {
                let (mut s, _) = store.load();
                s.window.width = Some(w.round() as u32);
                s.window.height = Some(h.round() as u32);
                let _ = store.save(&s);
            });
        }
    });

    // ── Layout memo ────────────────────────────────────────────────────────
    let session_pages = session.pages.clone();
    let layout = use_memo(move || {
        compute_layout(
            &TileLayoutInput {
                pages: session_pages.clone(),
                viewport_width_px: *viewport_width.read(),
                scale: ViewerScale::clamped(*scale.read()),
                mode: *mode.read(),
                gap_px: TILE_GAP_PX,
                padding_px: TILE_PADDING_PX,
                label_height_px: if *show_page_numbers.read() {
                    TILE_LABEL_HEIGHT_PX
                } else {
                    0.0
                },
            },
            LayoutGeneration(0),
        )
    });

    // ── Viewport-aware render scheduling (M9 RFC 007 §8) ──────────────────
    let session_eff = view.session.clone();
    use_effect(move || {
        let current_gen = *render_gen.read();
        render::schedule_visible_renders(
            layout.read().clone(),
            *scroll_y.read(),
            *viewport_height.read(),
            scale,
            current_gen,
            session_eff.id,
            session_eff.generation,
            tile_images,
            render_gen,
            render_service.clone(),
        );
    });

    // ── Settings persistence ──────────────────────────────────────────────
    let store_clone = settings_store.clone();
    use_effect(move || {
        let (ns, nm, nsp) = (*scale.read(), *mode.read(), *show_page_numbers.read());
        let store = store_clone.clone();
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

    // ── Helpers ────────────────────────────────────────────────────────────
    // After a scale change: tiles that have no image yet go back to `Pending`; tiles that
    // are `Ready` keep their (now stale-scale) image on screen and are re-rendered by the
    // scheduler (`tile_needs_render`). Resetting them would flash blank tiles.
    let mut invalidate = move || {
        let mut images = tile_images.write();
        for state in images.values_mut() {
            if !matches!(state, TileImageState::Ready { .. }) {
                *state = TileImageState::Pending;
            }
        }
        drop(images);
        let next = *render_gen.peek() + 1;
        render_gen.set(next);
    };
    let mut change_scale = move |change: ScaleChange| {
        // Read in its own statement: no guard may be alive when `set` runs (RFC 039).
        let current = *scale.peek();
        if let Some(next) = ViewerScale::next(current, change) {
            scale.set(next);
            invalidate();
        }
    };
    let default_scale = settings.read().viewer.default_scale;

    // Closing the transient panels returns focus to the button that opened them (RFC 033 § 5.5).
    let mut close_search = move || {
        show_search.set(false);
        modal::focus_by_id("viewer-search-button");
    };
    let mut close_outline = move || {
        show_outline.set(false);
        modal::focus_by_id("viewer-outline-button");
    };
    // Leaving Zen focuses the toolbar's Zen button; entering Zen focuses the viewer container
    // (the toolbar button disappears with the toolbar, which would drop focus to the page body).
    let mut was_zen = use_signal(|| false);
    use_effect(move || {
        let zen_now = *zen_mode.read();
        let before = *was_zen.peek();
        if zen_now != before {
            was_zen.set(zen_now);
            modal::focus_by_id(if zen_now {
                "viewer-root"
            } else {
                "viewer-zen-button"
            });
        }
    });
    // The single dispatcher of the shortcut resolver (RFC 033 § 5.3).
    let mut dispatch = move |action: ShortcutAction| match action {
        ShortcutAction::ToggleZen => zen_mode.toggle(),
        ShortcutAction::ScaleUp => change_scale(ScaleChange::By(0.2)),
        ShortcutAction::ScaleDown => change_scale(ScaleChange::By(-0.2)),
        ShortcutAction::ResetScale => change_scale(ScaleChange::To(default_scale)),
        ShortcutAction::Dismiss(Dismissal::Dialog | Dismissal::Zoom) => zoom_page.set(None),
        ShortcutAction::Dismiss(Dismissal::Search) => close_search(),
        ShortcutAction::Dismiss(Dismissal::Outline) => close_outline(),
        ShortcutAction::Dismiss(Dismissal::Zen) => zen_mode.set(false),
        ShortcutAction::Dismiss(Dismissal::LeaveDocument) => phase.set(Phase::Dashboard),
        ShortcutAction::Dismiss(Dismissal::Nothing) => {}
        // zoom-layer actions belong to the zoom overlay, which never lets them reach this handler
        ShortcutAction::ZoomScaleUp
        | ShortcutAction::ZoomScaleDown
        | ShortcutAction::ZoomPrev
        | ShortcutAction::ZoomNext
        | ShortcutAction::ZoomFirst
        | ShortcutAction::ZoomLast => {}
    };
    let on_jump = Callback::new(move |idx: usize| scroll_tile_into_view(PageIndex(idx)));
    let on_outline_navigate = Callback::new(move |idx: PageIndex| scroll_tile_into_view(idx));
    let reveal_path = doc_path.clone();
    let on_reveal = Callback::new(move |_| {
        if let Some(ref p) = reveal_path
            && let Err(e) = reveal_in_file_manager(p)
        {
            reveal_error.set(Some(format!("{e:?}")));
        }
    });

    let search_summaries: Vec<PageSearchSummary> = match &*search_state.read() {
        SearchState::Results { results, .. } => results.pages.clone(),
        _ => Vec::new(),
    };
    let search_highlights: Vec<PageHighlightSet> = match &*search_state.read() {
        SearchState::Results { highlights, .. } => highlights.pages.clone(),
        _ => Vec::new(),
    };
    let in_zen = *zen_mode.read();

    rsx! {
        div {
            id: "viewer-root",
            class: if in_zen { "viewer zen" } else { "viewer" },
            tabindex: "0",
            // A freshly opened document starts with the viewer focused, so shortcuts work at once.
            onmounted: move |evt: Event<MountedData>| {
                spawn(async move {
                    let _ = evt.set_focus(true).await;
                });
            },
            onkeydown: move |evt: Event<KeyboardData>| {
                // Each read in its own statement: no guard may be alive when a signal is set.
                let zoom_open = zoom_page.peek().is_some();
                let search_open = *show_search.peek();
                let outline_open = *show_outline.peek();
                let zen = *zen_mode.peek();
                let top_layer = if zoom_open {
                    Layer::Zoom
                } else if search_open {
                    Layer::Search
                } else if outline_open {
                    Layer::Outline
                } else {
                    Layer::Base
                };
                let key = key_descriptor(&evt.data());
                if let Some(action) = resolve(ShortcutContext { top_layer, zen }, &key) {
                    dispatch(action);
                }
            },
            onwheel: move |evt: Event<WheelData>| {
                if evt.modifiers().ctrl() {
                    evt.prevent_default();
                    let d = evt.delta().strip_units().y;
                    change_scale(ScaleChange::By(if d < 0.0 { 0.2 } else { -0.2 }));
                }
            },

            if !in_zen {
                header { class: "viewer-header",
                    button { class: "ghost", onclick: move |_| phase.set(Phase::Dashboard),
                        {t(locale(), MessageKey::ViewerBackToDashboard)}
                    }
                    h1 { class: "viewer-title", "{display_name}" }
                    button {
                        id: "viewer-outline-button",
                        class: if *show_outline.read() { "ghost icon-btn active" } else { "ghost icon-btn" },
                        title: t(locale(), MessageKey::OutlinePanelLabel),
                        "aria-label": t(locale(), MessageKey::OutlinePanelLabel),
                        "aria-expanded": if *show_outline.read() { "true" } else { "false" },
                        onclick: move |_| show_outline.toggle(),
                        "☰"
                    }
                    button {
                        id: "viewer-search-button",
                        class: if *show_search.read() { "ghost icon-btn active" } else { "ghost icon-btn" },
                        title: t(locale(), MessageKey::SearchButton),
                        onclick: move |_| show_search.toggle(),
                        "🔍"
                    }
                    if doc_path.is_some() {
                        button {
                            class: "ghost icon-btn",
                            title: t(locale(), MessageKey::RevealInFileManager),
                            onclick: move |_| on_reveal.call(()),
                            "📂"
                        }
                    }
                    if let Some(ref e) = *reveal_error.read() {
                        span { class: "toast-error", "{e}" }
                    }
                    button {
                        id: "viewer-zen-button",
                        class: "ghost icon-btn",
                        title: t(locale(), MessageKey::ZenModeEnter),
                        onclick: move |_| zen_mode.set(true),
                        "⊞"
                    }
                }
            }

            if !in_zen && *show_search.read() {
                SearchPanel {
                    engine: rs_for_search.engine.clone(),
                    session_id: view.session.id,
                    generation: view.session.generation,
                    search_state,
                    on_close: Callback::new(move |_| close_search()),
                }
            }

            if !in_zen {
                ViewerControls { page_count, scale, mode, show_page_numbers, displayed_columns: displayed_columns(&layout.read()), on_jump }
            }

            // Tile grid — scroll position feeds the render scheduler.
            div { class: "viewer-body",
                if !in_zen && *show_outline.read() {
                    OutlinePanel {
                        state: outline_state.read().clone(),
                        on_close: Callback::new(move |_| close_outline()),
                        on_navigate: on_outline_navigate,
                    }
                }

                div {
                    id: "viewer-scroll",
                    class: "tile-grid-scroll",
                    onscroll: move |evt: Event<ScrollData>| {
                        scroll_y.set(evt.scroll_top() as f32);
                        // Re-trigger render scheduling for newly visible pages.
                        // We check viewport height from the scroll event too.
                        let h = evt.client_height() as f32;
                        if h > 0.0 { viewport_height.set(h); }
                        let next = *render_gen.peek() + 1;
                        render_gen.set(next);
                    },
                    TileGrid {
                        layout: layout.read().clone(),
                        tile_images,
                        show_page_numbers: *show_page_numbers.read(),
                        search_summaries,
                        search_highlights: search_highlights.clone(),
                        page_descriptors: session.pages.clone(),
                        on_tile_click: Some(Callback::new(move |idx: PageIndex| {
                            zoom_page.clone().set(Some(idx));
                        })),
                        // TileGrid no longer owns the scroll container — the viewer does.
                        scroll_container_id: "viewer-scroll-inner".to_string(),
                    }
                }
            }

            if zoom_page.read().is_some() {
                ZoomOverlay {
                    document_id: view.session.id,
                    generation: view.session.generation,
                    page_index: zoom_page,
                    page_count,
                    search_highlights,
                    page_descriptors,
                    on_internal_link: Callback::new(move |idx: PageIndex| {
                        zoom_page.clone().set(Some(idx));
                        scroll_tile_into_view(idx);
                    }),
                    on_close: Callback::new(move |_| zoom_page.clone().set(None)),
                }
            }

            if in_zen {
                button {
                    class: "zen-exit-btn",
                    title: t(locale(), MessageKey::ZenModeExit),
                    onclick: move |_| zen_mode.set(false),
                    "×"
                }
            }
        }
    }
}

/// Columns the grid shows right now: the tile count of its widest row.
fn displayed_columns(layout: &domain::layout::TileLayout) -> usize {
    layout
        .rows
        .iter()
        .map(|row| row.tile_indices.len())
        .max()
        .unwrap_or(1)
}

fn scroll_tile_into_view(idx: PageIndex) {
    let _ = eval(&format!(
        "document.getElementById('tile-{}')?.scrollIntoView({{behavior:'smooth',block:'start'}})",
        idx.0
    ));
}
