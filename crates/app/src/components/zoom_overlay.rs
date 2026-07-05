//! Zoom overlay for focused single-page inspection (RFC 012).
//!
//! Opens over the tile grid with a dimmed backdrop. Renders the page at a
//! higher overlay scale (default 2.7 from settings), shows search highlights
//! from RFC 011 if a search is active, and supports keyboard navigation.
//!
//! Keyboard (RFC 012 §9 / RFC 013 §8):
//!   ArrowLeft / PageUp  — previous page
//!   ArrowRight / PageDown — next page
//!   Home                — first page
//!   End                 — last page
//!   + / =               — scale up
//!   -                   — scale down
//!   Escape              — close

mod util;

use base64::Engine as _;
use dioxus::prelude::*;

use app_services::render_service::RenderService;
use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::layout::RectPx;
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};
use domain::search::PageHighlightSet;
use domain::settings::AppSettingsV1;

use crate::i18n::{Locale, MessageKey, t};

const ZOOM_SCALE_MIN: f32 = 0.5;
const ZOOM_SCALE_MAX: f32 = 8.0;
const ZOOM_SCALE_STEP: f32 = 0.3;

/// State of the zoom overlay's current page render.
#[derive(Clone, Debug, PartialEq)]
pub enum ZoomImageState {
    Loading,
    Ready {
        uri: String,
        width_px: u32,
        height_px: u32,
    },
    Failed(String),
}

#[component]
pub fn ZoomOverlay(
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: Signal<Option<PageIndex>>,
    page_count: usize,
    /// Search highlights for overlay rendering (RFC 011); may be empty.
    search_highlights: Vec<PageHighlightSet>,
    /// PageDescriptors from the session for coordinate transforms.
    page_descriptors: Vec<domain::document::PageDescriptor>,
    on_close: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let settings: Signal<AppSettingsV1> = use_context();
    let render_service: RenderService = use_context();

    let mut zoom_scale = use_signal(|| {
        settings
            .read()
            .viewer
            .zoom_overlay_scale
            .clamp(ZOOM_SCALE_MIN, ZOOM_SCALE_MAX)
    });
    let zoom_image: Signal<ZoomImageState> = use_signal(|| ZoomImageState::Loading);

    let current_idx = page_index.read().unwrap_or(PageIndex(0));
    let display_num = current_idx.display_number();

    // ── Render when page or scale changes ────────────────────────────────
    let rs = render_service.clone();
    use_effect(move || {
        let Some(idx) = *page_index.read() else {
            return;
        };
        let scale_val = *zoom_scale.read();
        let scale_bucket = ScaleBucket::from_scale(scale_val);
        let request = RenderPageRequest {
            document_id,
            generation,
            page_index: idx,
            scale_bucket,
            flags: RenderFlags::default(),
            format: RenderOutputFormat::Png,
        };
        let rs2 = rs.clone();
        let mut zi = zoom_image;
        spawn(async move {
            zi.set(ZoomImageState::Loading);
            match rs2.get_or_render(request).await {
                Ok(png) => {
                    let (w, h) = util::png_dimensions(&png).unwrap_or((0, 0));
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&*png);
                    zi.set(ZoomImageState::Ready {
                        uri: format!("data:image/png;base64,{b64}"),
                        width_px: w,
                        height_px: h,
                    });
                }
                Err(e) => zi.set(ZoomImageState::Failed(format!("{e:?}"))),
            }
        });
    });

    // ── Navigation helpers ────────────────────────────────────────────────
    let can_prev = current_idx.0 > 0;
    let can_next = current_idx.0 + 1 < page_count;

    let goto = move |idx: usize| {
        if idx < page_count {
            page_index.clone().set(Some(PageIndex(idx)));
        }
    };

    // ── Highlight rects in image-space for current page ───────────────────
    let highlight_rects: Vec<RectPx> = match &*zoom_image.read() {
        ZoomImageState::Ready {
            width_px,
            height_px,
            ..
        } => util::zoom_highlight_rects(
            current_idx,
            &page_descriptors,
            &search_highlights,
            *width_px,
            *height_px,
        ),
        _ => Vec::new(),
    };

    rsx! {
            div {
                class: "zoom-backdrop",
                role: "dialog",
                "aria-modal": "true",
                "aria-label": "Page zoom view",
                tabindex: "-1",
                // Keyboard navigation (RFC 012 §9)
                onkeydown: move |evt: Event<KeyboardData>| {
                    match evt.key() {
                        Key::Escape => on_close.call(()),
                        Key::ArrowLeft | Key::PageUp => {
                            if can_prev { goto(current_idx.0 - 1); }
                        }
                        Key::ArrowRight | Key::PageDown => {
                            if can_next { goto(current_idx.0 + 1); }
                        }
                        Key::Home => goto(0),
                        Key::End => goto(page_count.saturating_sub(1)),
                        Key::Character(ref s) => match s.as_str() {
                            "+" | "=" => {
                                let v = (*zoom_scale.peek() + ZOOM_SCALE_STEP).min(ZOOM_SCALE_MAX);
                                zoom_scale.set(v);
                            }
                            "-" => {
                                let v = (*zoom_scale.peek() - ZOOM_SCALE_STEP).max(ZOOM_SCALE_MIN);
                                zoom_scale.set(v);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                },
                // Click backdrop to close
                onclick: move |_| on_close.call(()),

                div {
                    class: "zoom-panel",
                    // Prevent backdrop-click from bubbling through the panel
                    onclick: move |evt| evt.stop_propagation(),

                    // ── Header ─────────────────────────────────────────────
                    div { class: "zoom-header",
                        span { class: "zoom-page-indicator",
                            {t(locale(), MessageKey::ZoomPageIndicator)}
                            " {display_num} / {page_count}"
                        }
                        div { class: "zoom-scale-ctrl",
                            label { class: "muted control-label",
                                {t(locale(), MessageKey::ZoomScaleLabel)}
                            }
                            button {
                                class: "ghost icon-btn",
                                disabled: *zoom_scale.read() <= ZOOM_SCALE_MIN,
                                onclick: move |_| {
                                    let v = (*zoom_scale.peek() - ZOOM_SCALE_STEP).max(ZOOM_SCALE_MIN);
                                    zoom_scale.set(v);
                                },
                                "−"
                            }
                            span { class: "muted scale-label",
                                "{(*zoom_scale.read() * 100.0).round() as i32}%"
                            }
                            button {
                                class: "ghost icon-btn",
                                disabled: *zoom_scale.read() >= ZOOM_SCALE_MAX,
                                onclick: move |_| {
                                    let v = (*zoom_scale.peek() + ZOOM_SCALE_STEP).min(ZOOM_SCALE_MAX);
                                    zoom_scale.set(v);
                                },
                                "+"
                            }
                        }
                        button {
                            class: "ghost zoom-close",
                            autofocus: true,
                            "aria-label": t(locale(), MessageKey::ZoomClose),
                            onclick: move |_| on_close.call(()),
                            {t(locale(), MessageKey::ZoomClose)}
                        }
                    }

                    // ── Page image ─────────────────────────────────────────
                    div { class: "zoom-image-container",
                        match &*zoom_image.read() {
                            ZoomImageState::Ready { uri, width_px, height_px } => rsx! {
                                div {
                                    class: "zoom-image-wrap",
                                    style: "position: relative; display: inline-block;",
                                    img {
                                        class: "zoom-img",
                                        src: "{uri}",
                                        alt: "Page {display_num}",
                                        style: "display: block; max-width: 100%; height: auto;",
                                        width: "{width_px}",
                                        height: "{height_px}",
                                    }
                                    // RFC 011 highlights in zoom view
                                    for rect in highlight_rects.iter().copied() {
                                        {
                                            let (rx, ry, rw, rh) = (rect.x, rect.y, rect.width, rect.height);
                                            rsx! {
                                                div {
                                                    class: "highlight-overlay",
                                                    style: "left:{rx}px; top:{ry}px; width:{rw}px; height:{rh}px;",
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            ZoomImageState::Loading => rsx! {
                                div { class: "zoom-placeholder",
                                    span { class: "tile-loading-dot" }
                                }
                            },
                            ZoomImageState::Failed(msg) => rsx! {
                                div { class: "zoom-placeholder error",
                                    "⚠ {msg}"
                                }
                            },
                        }
                    }

                    // ── Navigation footer ──────────────────────────────────
                    div { class: "zoom-footer",
    button {
                            class: "ghost",
                            disabled: !can_prev,
                            "aria-label": t(locale(), MessageKey::ZoomPrevPage),
                            onclick: move |_| { if can_prev { goto(current_idx.0 - 1); } },
                            {t(locale(), MessageKey::ZoomPrevPage)}
                        }
                        button {
                            class: "ghost",
                            disabled: !can_next,
                            "aria-label": t(locale(), MessageKey::ZoomNextPage),
                            onclick: move |_| { if can_next { goto(current_idx.0 + 1); } },
                            {t(locale(), MessageKey::ZoomNextPage)}
                        }
                    }
                }
            }
        }
}
