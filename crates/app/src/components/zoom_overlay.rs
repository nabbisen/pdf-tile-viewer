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

mod external_uri_dialog;
mod link_layer;
mod loaders;
mod state;
mod text_layer;
mod util;

use dioxus::prelude::*;

use app_services::navigation_service::NavigationService;
use app_services::render_service::RenderService;
use app_services::text_service::TextLayerService;
use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::layout::RectPx;
use domain::search::PageHighlightSet;
use domain::settings::AppSettingsV1;

use domain::shortcuts::{Layer, ShortcutAction};

use crate::components::modal;
use crate::i18n::{Locale, MessageKey, t};

use external_uri_dialog::ExternalUriDialogState;
use state::{ReadyImage, ZoomImageState, ZoomPageLinksState, ZoomTextLayerState};

const ZOOM_SCALE_MIN: f32 = 0.5;
const ZOOM_SCALE_MAX: f32 = 8.0;
const ZOOM_SCALE_STEP: f32 = 0.3;

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
    on_internal_link: Callback<PageIndex>,
    on_close: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let settings: Signal<AppSettingsV1> = use_context();
    let navigation_service: NavigationService = use_context();
    let render_service: RenderService = use_context();
    let text_service: TextLayerService = use_context();

    let mut zoom_scale = use_signal(|| {
        settings
            .read()
            .viewer
            .zoom_overlay_scale
            .clamp(ZOOM_SCALE_MIN, ZOOM_SCALE_MAX)
    });
    let zoom_image: Signal<ZoomImageState> =
        use_signal(|| ZoomImageState::Loading { previous: None });
    let zoom_text_layer: Signal<ZoomTextLayerState> = use_signal(|| ZoomTextLayerState::Loading);
    let zoom_page_links: Signal<ZoomPageLinksState> = use_signal(|| ZoomPageLinksState::Loading);
    let image_request_seq: Signal<u64> = use_signal(|| 0);
    let text_request_seq: Signal<u64> = use_signal(|| 0);
    let links_request_seq: Signal<u64> = use_signal(|| 0);
    let external_uri_dialog_seq: Signal<u64> = use_signal(|| 0);
    let external_uri_dialog: Signal<Option<ExternalUriDialogState>> = use_signal(|| None);
    // Client-viewport coordinates avoid target-relative offsetX/offsetY when
    // mouse events originate from selectable text spans inside the wrapper.
    let mut link_pointer_down: Signal<Option<(f64, f64)>> = use_signal(|| None);

    // Modal focus contract (RFC 033 § 5.5): focus the close button on open, keep Tab inside,
    // make the viewer inert, give the focus back to the viewer on close.
    modal::use_modal("zoom-overlay-root", "zoom-close-button", "viewer-root");

    let current_idx = page_index.read().unwrap_or(PageIndex(0));
    let display_num = current_idx.display_number();

    // ── Loading effects (page image, page links, text layer) ─────────────
    loaders::use_page_image_loader(
        render_service,
        document_id,
        generation,
        page_index,
        zoom_scale,
        image_request_seq,
        zoom_image,
    );
    loaders::use_page_links_loader(
        navigation_service,
        document_id,
        generation,
        page_index,
        links_request_seq,
        zoom_page_links,
    );
    loaders::use_text_layer_loader(
        text_service,
        document_id,
        generation,
        page_index,
        text_request_seq,
        zoom_text_layer,
    );

    // ── Navigation helpers ────────────────────────────────────────────────
    let can_prev = current_idx.0 > 0;
    let can_next = current_idx.0 + 1 < page_count;

    let goto = move |idx: usize| {
        if idx < page_count {
            page_index.clone().set(Some(PageIndex(idx)));
        }
    };

    // ── Highlight rects in image-space for current page ───────────────────
    // Text, highlights and link areas exist only for a `Ready` image: while an image of
    // another scale is kept during a re-render, their geometry would not match it.
    let highlight_rects: Vec<RectPx> = match &*zoom_image.read() {
        ZoomImageState::Ready(image) => util::zoom_highlight_rects(
            current_idx,
            &page_descriptors,
            &search_highlights,
            image.width_px,
            image.height_px,
        ),
        _ => Vec::new(),
    };
    let text_segment_rects = match (&*zoom_image.read(), &*zoom_text_layer.read()) {
        (ZoomImageState::Ready(image), ZoomTextLayerState::Ready(layer)) => {
            util::zoom_text_segment_rects(
                current_idx,
                &page_descriptors,
                layer,
                image.width_px,
                image.height_px,
            )
        }
        _ => Vec::new(),
    };
    let page_link_rects = match (&*zoom_image.read(), &*zoom_page_links.read()) {
        (ZoomImageState::Ready(image), ZoomPageLinksState::Ready(links)) => {
            util::zoom_page_link_rects(
                current_idx,
                &page_descriptors,
                links,
                image.width_px,
                image.height_px,
            )
        }
        _ => Vec::new(),
    };
    let show_text_unavailable = matches!(&*zoom_text_layer.read(), ZoomTextLayerState::Unavailable);
    let page_alt = format!(
        "{} {display_num}",
        t(locale(), MessageKey::PageImageAltPrefix)
    );

    rsx! {
            div {
                id: "zoom-overlay-root",
                class: "zoom-backdrop",
                role: "dialog",
                "aria-modal": "true",
                "aria-label": t(locale(), MessageKey::PageZoomView),
                tabindex: "-1",
                // The overlay owns the keyboard (RFC 033 § 5.3): `modal_key` always stops the event here.
                onkeydown: move |evt: Event<KeyboardData>| {
                    let Some(action) = modal::modal_key(&evt, "zoom-overlay-root", Layer::Zoom) else {
                        return;
                    };
                    match action {
                        ShortcutAction::Dismiss(_) => on_close.call(()),
                        ShortcutAction::ZoomPrev => {
                            if can_prev { goto(current_idx.0 - 1); }
                        }
                        ShortcutAction::ZoomNext => {
                            if can_next { goto(current_idx.0 + 1); }
                        }
                        ShortcutAction::ZoomFirst => goto(0),
                        ShortcutAction::ZoomLast => goto(page_count.saturating_sub(1)),
                        ShortcutAction::ZoomScaleUp => {
                            let v = (*zoom_scale.peek() + ZOOM_SCALE_STEP).min(ZOOM_SCALE_MAX);
                            zoom_scale.set(v);
                        }
                        ShortcutAction::ZoomScaleDown => {
                            let v = (*zoom_scale.peek() - ZOOM_SCALE_STEP).max(ZOOM_SCALE_MIN);
                            zoom_scale.set(v);
                        }
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
                                title: t(locale(), MessageKey::ZoomOut),
                                "aria-label": t(locale(), MessageKey::ZoomOut),
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
                                title: t(locale(), MessageKey::ZoomIn),
                                "aria-label": t(locale(), MessageKey::ZoomIn),
                                disabled: *zoom_scale.read() >= ZOOM_SCALE_MAX,
                                onclick: move |_| {
                                    let v = (*zoom_scale.peek() + ZOOM_SCALE_STEP).min(ZOOM_SCALE_MAX);
                                    zoom_scale.set(v);
                                },
                                "+"
                            }
                        }
                        button {
                            id: "zoom-close-button",
                            class: "ghost zoom-close",
                            "aria-label": t(locale(), MessageKey::ZoomClose),
                            onclick: move |_| on_close.call(()),
                            {t(locale(), MessageKey::ZoomClose)}
                        }
                    }

                    // ── Page image ─────────────────────────────────────────
                    div { class: "zoom-image-container",
                        match &*zoom_image.read() {
                            ZoomImageState::Ready(ReadyImage { uri, width_px, height_px, .. }) => rsx! {
                                {
                                    let wrap_class = if page_link_rects.is_empty() {
                                        "zoom-image-wrap"
                                    } else {
                                        "zoom-image-wrap has-links"
                                    };
                                    rsx! {
                                div {
                                    id: "zoom-image-wrap",
                                    class: "{wrap_class}",
                                    style: "position: relative; display: inline-block; \
                                            width: {width_px}px; height: {height_px}px;",
                                    onmousedown: move |evt: Event<MouseData>| {
                                        let point = evt.client_coordinates();
                                        link_pointer_down.set(Some((point.x, point.y)));
                                    },
                                    onmouseup: link_layer::image_mouseup_handler(
                                        page_link_rects.clone(),
                                        page_count,
                                        on_internal_link,
                                        link_pointer_down,
                                        external_uri_dialog_seq,
                                        external_uri_dialog,
                                    ),
                                    img {
                                        class: "zoom-img",
                                        src: "{uri}",
                                        alt: "{page_alt}",
                                        draggable: "false",
                                        style: "display: block; \
                                                width: {width_px}px; height: {height_px}px; \
                                                max-width: none;",
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
                                    // RFC 023 selectable text layer in zoom view
                                    {text_layer::text_layer(&text_segment_rects, current_idx)}
                                    if show_text_unavailable {
                                        div { class: "zoom-text-status",
                                            {t(locale(), MessageKey::ZoomTextSelectionUnavailable)}
                                        }
                                    }
                                    {link_layer::link_layer(
                                        &page_link_rects,
                                        current_idx,
                                        page_count,
                                        locale,
                                        on_internal_link,
                                        external_uri_dialog_seq,
                                        external_uri_dialog,
                                    )}
                                }
                                    }
                                }
                            },
                            // Scale change on the same page: the old image stays, plain (no text,
                            // highlight or link layers, so no selection or link activation),
                            // with a small busy dot, until the new render replaces it.
                            ZoomImageState::Loading { previous: Some(ReadyImage { uri, width_px, height_px, .. }) } => rsx! {
                                div {
                                    id: "zoom-image-wrap",
                                    class: "zoom-image-wrap is-refreshing",
                                    "aria-busy": "true",
                                    style: "position: relative; display: inline-block; \
                                            width: {width_px}px; height: {height_px}px;",
                                    img {
                                        class: "zoom-img",
                                        src: "{uri}",
                                        alt: "{page_alt}",
                                        draggable: "false",
                                        style: "display: block; \
                                                width: {width_px}px; height: {height_px}px; \
                                                max-width: none;",
                                        width: "{width_px}",
                                        height: "{height_px}",
                                    }
                                    span { class: "zoom-busy-dot tile-loading-dot" }
                                }
                            },
                            ZoomImageState::Loading { previous: None } => rsx! {
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
            {external_uri_dialog::external_uri_dialog_layer(external_uri_dialog)}
        }
}
