//! Document viewer screen (M5: zen mode, keyboard shortcuts, viewport
//! measurement, settings persistence, reveal-in-file-manager).
//!
//! Zen mode (RFC 013): Z key toggles; Escape exits; a floating exit
//! button is always visible in zen mode.
//!
//! Keyboard shortcuts (RFC 013 §8):
//!   Z          — toggle zen mode
//!   +/=        — scale up 0.2
//!   -          — scale down 0.2
//!   0          — reset scale to default
//!   Escape     — exit zen, or back to dashboard
//!
//! Settings persistence (RFC 008 §8): scale, pages-per-row, and
//! show-page-numbers are written back to AppSettingsV1 on every change
//! and saved asynchronously (immediate save — debounce deferred to M6+).
//!
//! Viewport width (RFC 005 stub → M5): measured once on mount via
//! `eval("return window.innerWidth")` and updated on resize.

use base64::Engine as _;
use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::platform::reveal_in_file_manager;
use app_services::render_service::RenderService;
use app_services::settings_service::SettingsStore;
use domain::layout::{
    LayoutGeneration, PagesPerRowMode, TileLayoutInput, ViewerScale, compute_layout,
};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};
use domain::settings::{AppSettingsV1, PagesPerRowPreference};

use crate::components::tile_grid::TileGrid;
use crate::components::viewer_controls::ViewerControls;
use crate::i18n::{Locale, MessageKey, t};
use crate::state::{OpenDocumentView, Phase, TileImageState, TileImages};

const TILE_GAP_PX: f32 = 12.0;
const TILE_PADDING_PX: f32 = 16.0;
const TILE_LABEL_HEIGHT_PX: f32 = 20.0;
const DEFAULT_VIEWPORT_PX: f32 = 1200.0;

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
    let mut viewport_width: Signal<f32> = use_signal(|| DEFAULT_VIEWPORT_PX);
    let mut tile_images: Signal<TileImages> = use_signal(|| {
        session
            .pages
            .iter()
            .map(|p| (p.page_index, TileImageState::Pending))
            .collect()
    });
    let mut render_gen: Signal<u64> = use_signal(|| 0u64);
    let mut reveal_error: Signal<Option<String>> = use_signal(|| None);

    // ── Viewport width measurement (RFC 005 stub → M5) ───────────────────
    use_effect(move || {
        spawn(async move {
            if let Ok(w) = eval("return window.innerWidth").join::<f64>().await {
                if w > 0.0 {
                    viewport_width.set(w as f32);
                }
            }
        });
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

    // ── Render scheduling ─────────────────────────────────────────────────
    let session_eff = view.session.clone();
    use_effect(move || {
        let current_gen = *render_gen.read();
        let current_layout = layout.read().clone();
        let scale_bucket = ScaleBucket::from_scale(*scale.read());
        {
            let mut images = tile_images.write();
            for tile in &current_layout.tiles {
                let state = images
                    .entry(tile.page_index)
                    .or_insert(TileImageState::Pending);
                if !matches!(state, TileImageState::Ready(_)) {
                    *state = TileImageState::Rendering(current_gen);
                }
            }
        }
        for tile in current_layout.tiles.clone() {
            let page_index = tile.page_index;
            let rs = render_service.clone();
            let request = RenderPageRequest {
                document_id: session_eff.id,
                generation: session_eff.generation,
                page_index,
                scale_bucket,
                flags: RenderFlags::default(),
                format: RenderOutputFormat::Png,
            };
            let mut ti = tile_images;
            let rg = render_gen;
            spawn(async move {
                match rs.get_or_render(request).await {
                    Ok(png) => {
                        if *rg.peek() != current_gen {
                            return;
                        }
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&*png);
                        ti.write().insert(
                            page_index,
                            TileImageState::Ready(format!("data:image/png;base64,{b64}")),
                        );
                    }
                    Err(e) => {
                        if *rg.peek() != current_gen {
                            return;
                        }
                        ti.write()
                            .insert(page_index, TileImageState::Failed(format!("{e:?}")));
                    }
                }
            });
        }
    });

    // ── Settings persistence (RFC 008 §8) ─────────────────────────────────
    // Runs whenever scale, mode, or show_page_numbers change.
    let store_clone = settings_store.clone();
    use_effect(move || {
        let new_scale = *scale.read();
        let new_mode = *mode.read();
        let new_spn = *show_page_numbers.read();
        let store = store_clone.clone();
        spawn(async move {
            let (mut s, _) = store.load();
            s.viewer.default_scale = new_scale;
            s.viewer.pages_per_row = match new_mode {
                PagesPerRowMode::Auto => PagesPerRowPreference::Auto,
                PagesPerRowMode::Fixed(n) => PagesPerRowPreference::Fixed(n),
            };
            s.viewer.show_page_numbers = new_spn;
            let _ = store.save(&s);
        });
    });

    // ── Scale/render helpers ──────────────────────────────────────────────
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

    // ── Jump-to-page ──────────────────────────────────────────────────────
    let on_jump = Callback::new(move |idx: usize| {
        let js = format!(
            "document.getElementById('tile-{idx}')?.scrollIntoView(\
             {{behavior:'smooth',block:'start'}})"
        );
        let _ = eval(&js);
    });

    // ── Reveal in file manager ────────────────────────────────────────────
    let reveal_path = doc_path.clone();
    let on_reveal = Callback::new(move |_| {
        if let Some(ref p) = reveal_path {
            if let Err(e) = reveal_in_file_manager(p) {
                reveal_error.set(Some(format!("{e:?}")));
            }
        }
    });

    let in_zen = *zen_mode.read();

    rsx! {
        div {
            class: if in_zen { "viewer zen" } else { "viewer" },
            // Global key handler (RFC 013 §8)
            tabindex: "0",
            onkeydown: move |evt: Event<KeyboardData>| {
                // Don't fire shortcuts when typing in inputs
                match evt.key() {
                    Key::Character(ref s) => match s.as_str() {
                        "z" | "Z" => zen_mode.toggle(),
                        "+" | "=" => change_scale(*scale.peek() + 0.2),
                        "-" => change_scale(*scale.peek() - 0.2),
                        "0" => change_scale(default_scale),
                        _ => {}
                    },
                    Key::Escape => {
                        if in_zen {
                            zen_mode.set(false);
                        } else {
                            phase.set(Phase::Dashboard);
                        }
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

            // ── Normal header (hidden in zen) ──────────────────────────
            if !in_zen {
                header { class: "viewer-header",
                    button {
                        class: "ghost",
                        onclick: move |_| phase.set(Phase::Dashboard),
                        {t(locale(), MessageKey::ViewerBackToDashboard)}
                    }
                    h1 { class: "viewer-title", "{display_name}" }
                    span { class: "muted",
                        {t(locale(), MessageKey::ViewerPageCountLabel)}
                        ": {page_count}"
                    }
                    if doc_path.is_some() {
                        button {
                            class: "ghost icon-btn",
                            title: t(locale(), MessageKey::RevealInFileManager),
                            "aria-label": t(locale(), MessageKey::RevealInFileManager),
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
                        "aria-label": t(locale(), MessageKey::ZenModeEnter),
                        onclick: move |_| zen_mode.set(true),
                        "⊞"
                    }
                }
            }

            // ── Controls (hidden in zen) ───────────────────────────────
            if !in_zen {
                ViewerControls {
                    page_count,
                    scale,
                    mode,
                    show_page_numbers,
                    on_jump,
                }
            }

            // ── Tile grid ────────────────────────────────────────────
            TileGrid {
                layout: layout.read().clone(),
                tile_images,
                show_page_numbers: *show_page_numbers.read(),
                on_tile_click: None,
                scroll_container_id: "viewer-scroll".to_string(),
            }

            // ── Zen mode exit affordance (RFC 013 §5) ─────────────────
            if in_zen {
                button {
                    class: "zen-exit-btn",
                    title: t(locale(), MessageKey::ZenModeExit),
                    "aria-label": t(locale(), MessageKey::ZenModeExit),
                    onclick: move |_| zen_mode.set(false),
                    "×"
                }
            }
        }
    }
}
