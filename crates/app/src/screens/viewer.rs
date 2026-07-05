//! Document viewer screen (M9: viewport-aware rendering, scroll tracking).

mod render;

use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::platform::reveal_in_file_manager;
use app_services::render_service::RenderService;
use app_services::settings_service::SettingsStore;
use domain::document::PageIndex;
use domain::layout::{
    LayoutGeneration, PagesPerRowMode, TileLayoutInput, ViewerScale, compute_layout,
};
use domain::search::{PageHighlightSet, PageSearchSummary};
use domain::settings::{AppSettingsV1, PagesPerRowPreference};

use crate::components::search_panel::SearchPanel;
use crate::components::tile_grid::TileGrid;
use crate::components::viewer_controls::ViewerControls;
use crate::components::zoom_overlay::ZoomOverlay;
use crate::i18n::{Locale, MessageKey, t};
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
    let render_service: RenderService = use_context();

    let session = view.session.clone();
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
    let zoom_page: Signal<Option<PageIndex>> = use_signal(|| None);
    let page_descriptors = session.pages.clone();
    let rs_for_search = render_service.clone();

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
            *scale.read(),
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
    let mut invalidate = move || {
        let mut images = tile_images.write();
        for state in images.values_mut() {
            if !matches!(state, TileImageState::Ready(_)) {
                *state = TileImageState::Pending;
            }
        }
        drop(images);
        let next = *render_gen.peek() + 1;
        render_gen.set(next);
    };
    let mut change_scale = move |v: f32| {
        let c = ((v * 10.0).round() / 10.0).clamp(ViewerScale::MIN, ViewerScale::MAX);
        if (c - *scale.peek()).abs() >= 0.001 {
            scale.set(c);
            invalidate();
        }
    };
    let default_scale = settings.read().viewer.default_scale;
    let on_jump = Callback::new(move |idx: usize| {
        let _ = eval(&format!(
            "document.getElementById('tile-{idx}')?.scrollIntoView({{behavior:'smooth',block:'start'}})"
        ));
    });
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
            class: if in_zen { "viewer zen" } else { "viewer" },
            tabindex: "0",
            onkeydown: move |evt: Event<KeyboardData>| {
                match evt.key() {
                    Key::Character(ref s) => match s.as_str() {
                        "z" | "Z" => zen_mode.toggle(),
                        "+" | "=" => change_scale(*scale.peek() + 0.2),
                        "-" => change_scale(*scale.peek() - 0.2),
                        "0" => change_scale(default_scale),
                        _ => {}
                    },
                    Key::Escape => {
                        if zoom_page.peek().is_some() { zoom_page.clone().set(None); }
                        else if in_zen { zen_mode.set(false); }
                        else if *show_search.peek() { show_search.set(false); }
                        else { phase.set(Phase::Dashboard); }
                    }
                    _ => {}
                }
            },
            onwheel: move |evt: Event<WheelData>| {
                if evt.modifiers().ctrl() {
                    evt.prevent_default();
                    let d = evt.delta().strip_units().y;
                    change_scale(*scale.peek() + if d < 0.0 { 0.2 } else { -0.2 });
                }
            },

            if !in_zen {
                header { class: "viewer-header",
                    button { class: "ghost", onclick: move |_| phase.set(Phase::Dashboard),
                        {t(locale(), MessageKey::ViewerBackToDashboard)}
                    }
                    h1 { class: "viewer-title", "{display_name}" }
                    button {
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
                    on_close: Callback::new(move |_| show_search.set(false)),
                }
            }

            if !in_zen {
                ViewerControls { page_count, scale, mode, show_page_numbers, on_jump }
            }

            // Tile grid — scroll position feeds the render scheduler.
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

            if zoom_page.read().is_some() {
                ZoomOverlay {
                    document_id: view.session.id,
                    generation: view.session.generation,
                    page_index: zoom_page,
                    page_count,
                    search_highlights,
                    page_descriptors,
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
