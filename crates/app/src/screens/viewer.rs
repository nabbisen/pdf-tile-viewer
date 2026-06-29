//! Document viewer screen (M4: tile grid + lazy rendering, RFC 006/007).
//!
//! Render scheduling model (RFC 007 §8):
//! - `render_gen` is bumped on scale/mode change to suppress stale renders.
//! - `use_effect` subscribes to `render_gen`; on bump it marks tiles as
//!   Rendering and spawns one async task per tile.
//! - Each task guard-checks the generation before writing back (Appendix A §8).

use base64::Engine as _;
use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::render_service::RenderService;
use domain::layout::{
    LayoutGeneration, PagesPerRowMode, TileLayoutInput, ViewerScale, compute_layout,
};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};
use domain::settings::{AppSettingsV1, PagesPerRowPreference};

use crate::components::tile_grid::TileGrid;
use crate::components::viewer_controls::ViewerControls;
use crate::i18n::{Locale, MessageKey, t};
use crate::state::{OpenDocumentView, Phase, TileImageState, TileImages};

const DEFAULT_VIEWPORT_PX: f32 = 1200.0;
const TILE_GAP_PX: f32 = 12.0;
const TILE_PADDING_PX: f32 = 16.0;
const TILE_LABEL_HEIGHT_PX: f32 = 20.0;

#[component]
pub fn Viewer(view: OpenDocumentView, mut phase: Signal<Phase>) -> Element {
    let locale: Memo<Locale> = use_context();
    let settings: Signal<AppSettingsV1> = use_context();
    let render_service: RenderService = use_context();

    let session = view.session.clone();
    let page_count = session.pages.len();
    let display_name = session.display_name.clone();

    // ── Viewer-local signals ──────────────────────────────────────────────
    let mut scale = use_signal(|| settings.read().viewer.default_scale);
    let mode = use_signal(|| match settings.read().viewer.pages_per_row {
        PagesPerRowPreference::Auto => PagesPerRowMode::Auto,
        PagesPerRowPreference::Fixed(n) => PagesPerRowMode::Fixed(n),
    });
    let show_page_numbers = use_signal(|| settings.read().viewer.show_page_numbers);
    let mut tile_images: Signal<TileImages> = use_signal(|| {
        session
            .pages
            .iter()
            .map(|p| (p.page_index, TileImageState::Pending))
            .collect()
    });
    let mut render_gen: Signal<u64> = use_signal(|| 0u64);

    // ── Layout memo ────────────────────────────────────────────────────────
    let session_pages = session.pages.clone();
    let layout = use_memo(move || {
        compute_layout(
            &TileLayoutInput {
                pages: session_pages.clone(),
                viewport_width_px: DEFAULT_VIEWPORT_PX,
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
    let session_effect = view.session.clone();
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
                document_id: session_effect.id,
                generation: session_effect.generation,
                page_index,
                scale_bucket,
                flags: RenderFlags::default(),
                format: RenderOutputFormat::Png,
            };
            // Capture signals by value (they are Copy).
            let mut ti = tile_images;
            let rg = render_gen;

            spawn(async move {
                match rs.get_or_render(request).await {
                    Ok(png_bytes) => {
                        if *rg.peek() != current_gen {
                            return;
                        }
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&*png_bytes);
                        let uri = format!("data:image/png;base64,{b64}");
                        ti.write().insert(page_index, TileImageState::Ready(uri));
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

    // ── Helpers ────────────────────────────────────────────────────────────
    let mut invalidate = move || {
        let mut images = tile_images.write();
        for state in images.values_mut() {
            if !matches!(state, TileImageState::Ready(_)) {
                *state = TileImageState::Pending;
            }
        }
        drop(images);
        let next_gen = *render_gen.peek() + 1;
        render_gen.set(next_gen);
    };

    let on_jump = Callback::new(move |page_idx: usize| {
        let js = format!(
            "document.getElementById('tile-{page_idx}')?.scrollIntoView(\
             {{behavior:'smooth',block:'start'}})"
        );
        let _ = eval(&js);
    });

    rsx! {
        main {
            class: "viewer",
            onwheel: move |evt: Event<WheelData>| {
                if evt.modifiers().ctrl() {
                    evt.prevent_default();
                    let delta = evt.delta().strip_units().y;
                    let step = if delta < 0.0 { 0.2_f32 } else { -0.2_f32 };
                    let new_val = *scale.peek() + step;
                    let clamped = ((new_val * 10.0).round() / 10.0)
                        .clamp(ViewerScale::MIN, ViewerScale::MAX);
                    if (clamped - *scale.peek()).abs() >= 0.001 {
                        scale.set(clamped);
                        invalidate();
                    }
                }
            },

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
            }

            ViewerControls {
                page_count,
                scale,
                mode,
                show_page_numbers,
                on_jump,
            }

            TileGrid {
                layout: layout.read().clone(),
                tile_images,
                show_page_numbers: *show_page_numbers.read(),
                on_tile_click: None,
                scroll_container_id: "viewer-scroll".to_string(),
            }
        }
    }
}
