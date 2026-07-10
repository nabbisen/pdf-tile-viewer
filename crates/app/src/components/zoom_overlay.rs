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
use dioxus::document::eval;
use dioxus::prelude::*;

use app_services::navigation_service::{NavigationService, default_page_links_request};
use app_services::render_service::RenderService;
use app_services::text_service::TextLayerService;
use app_services::uri_policy::{ExternalUriOpenDecision, validate_external_uri_for_open};
use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::layout::RectPx;
use domain::navigation::{NavigationResourceLimits, NavigationTarget, PageLinkSet};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};
use domain::search::PageHighlightSet;
use domain::settings::AppSettingsV1;
use domain::text::{PageTextLayer, TextLayerRequest};

use crate::i18n::{Locale, MessageKey, t};

const ZOOM_SCALE_MIN: f32 = 0.5;
const ZOOM_SCALE_MAX: f32 = 8.0;
const ZOOM_SCALE_STEP: f32 = 0.3;
const LINK_CLICK_DRAG_THRESHOLD_PX: f64 = 4.0;

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

/// State of the zoom overlay's current selectable text layer.
#[derive(Clone, Debug, PartialEq)]
pub enum ZoomTextLayerState {
    Loading,
    Ready(PageTextLayer),
    Unavailable,
}

/// State of the zoom overlay's current page links.
#[derive(Clone, Debug, PartialEq)]
pub enum ZoomPageLinksState {
    Loading,
    Ready(PageLinkSet),
    Unavailable,
}

#[derive(Clone, Debug, PartialEq)]
struct ExternalUriDialogState {
    raw_uri: String,
    copy_status: ExternalUriCopyStatus,
    dialog_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExternalUriCopyStatus {
    Idle,
    Copied,
    Failed,
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
    let mut zoom_image: Signal<ZoomImageState> = use_signal(|| ZoomImageState::Loading);
    let mut zoom_text_layer: Signal<ZoomTextLayerState> =
        use_signal(|| ZoomTextLayerState::Loading);
    let mut zoom_page_links: Signal<ZoomPageLinksState> =
        use_signal(|| ZoomPageLinksState::Loading);
    let mut image_request_seq: Signal<u64> = use_signal(|| 0);
    let mut text_request_seq: Signal<u64> = use_signal(|| 0);
    let mut links_request_seq: Signal<u64> = use_signal(|| 0);
    let mut external_uri_dialog_seq: Signal<u64> = use_signal(|| 0);
    let mut external_uri_dialog: Signal<Option<ExternalUriDialogState>> = use_signal(|| None);
    // Client-viewport coordinates avoid target-relative offsetX/offsetY when
    // mouse events originate from selectable text spans inside the wrapper.
    let mut link_pointer_down: Signal<Option<(f64, f64)>> = use_signal(|| None);

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
        let request_id = *image_request_seq.peek() + 1;
        image_request_seq.set(request_id);
        zoom_image.set(ZoomImageState::Loading);

        let rs2 = rs.clone();
        let mut zi = zoom_image;
        let active_page = page_index;
        let active_scale = zoom_scale;
        spawn(async move {
            match rs2.get_or_render(request).await {
                Ok(png) => {
                    let (w, h) = util::png_dimensions(&png).unwrap_or((0, 0));
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&*png);
                    if util::zoom_image_result_is_current(
                        *image_request_seq.peek(),
                        request_id,
                        *active_page.peek(),
                        idx,
                        *active_scale.peek(),
                        scale_bucket,
                    ) {
                        zi.set(ZoomImageState::Ready {
                            uri: format!("data:image/png;base64,{b64}"),
                            width_px: w,
                            height_px: h,
                        });
                    }
                }
                Err(e) => {
                    if util::zoom_image_result_is_current(
                        *image_request_seq.peek(),
                        request_id,
                        *active_page.peek(),
                        idx,
                        *active_scale.peek(),
                        scale_bucket,
                    ) {
                        zi.set(ZoomImageState::Failed(format!("{e:?}")));
                    }
                }
            }
        });
    });

    // ── Page link overlay when page changes (RFC 026 PR3) ────────────────
    let ns = navigation_service.clone();
    use_effect(move || {
        let Some(idx) = *page_index.read() else {
            return;
        };
        let request = default_page_links_request(document_id, generation, idx);
        let request_id = *links_request_seq.peek() + 1;
        links_request_seq.set(request_id);
        zoom_page_links.set(ZoomPageLinksState::Loading);

        let ns2 = ns.clone();
        let mut link_state = zoom_page_links;
        let active_page = page_index;
        spawn(async move {
            let is_current =
                || *links_request_seq.peek() == request_id && *active_page.peek() == Some(idx);
            match ns2.get_or_extract_page_links(request).await {
                Ok(links)
                    if is_current()
                        && util::zoom_page_links_match_overlay(
                            &links,
                            document_id,
                            generation,
                            idx,
                        ) =>
                {
                    link_state.set(ZoomPageLinksState::Ready((*links).clone()));
                }
                Ok(_) => {}
                Err(_) if is_current() => {
                    link_state.set(ZoomPageLinksState::Unavailable);
                }
                Err(_) => {}
            }
        });
    });

    // ── Text layer when page changes (RFC 023) ───────────────────────────
    let ts = text_service.clone();
    use_effect(move || {
        let Some(idx) = *page_index.read() else {
            return;
        };
        let request = TextLayerRequest {
            document_id,
            generation,
            page_index: idx,
        };
        let request_id = *text_request_seq.peek() + 1;
        text_request_seq.set(request_id);
        zoom_text_layer.set(ZoomTextLayerState::Loading);

        let ts2 = ts.clone();
        let mut text_state = zoom_text_layer;
        let active_page = page_index;
        spawn(async move {
            let is_current =
                || *text_request_seq.peek() == request_id && *active_page.peek() == Some(idx);
            match ts2.get_or_extract(request).await {
                Ok(layer)
                    if is_current()
                        && util::zoom_text_layer_matches_overlay(
                            &layer,
                            document_id,
                            generation,
                            idx,
                        ) =>
                {
                    if layer.segments.is_empty() {
                        text_state.set(ZoomTextLayerState::Unavailable);
                    } else {
                        text_state.set(ZoomTextLayerState::Ready((*layer).clone()));
                    }
                }
                Ok(_) => {}
                Err(_) if is_current() => {
                    text_state.set(ZoomTextLayerState::Unavailable);
                }
                Err(_) => {}
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
    let text_segment_rects = match (&*zoom_image.read(), &*zoom_text_layer.read()) {
        (
            ZoomImageState::Ready {
                width_px,
                height_px,
                ..
            },
            ZoomTextLayerState::Ready(layer),
        ) => util::zoom_text_segment_rects(
            current_idx,
            &page_descriptors,
            layer,
            *width_px,
            *height_px,
        ),
        _ => Vec::new(),
    };
    let page_link_rects = match (&*zoom_image.read(), &*zoom_page_links.read()) {
        (
            ZoomImageState::Ready {
                width_px,
                height_px,
                ..
            },
            ZoomPageLinksState::Ready(links),
        ) => {
            util::zoom_page_link_rects(current_idx, &page_descriptors, links, *width_px, *height_px)
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
                class: "zoom-backdrop",
                role: "dialog",
                "aria-modal": "true",
                "aria-label": t(locale(), MessageKey::PageZoomView),
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
                                    onmouseup: {
                                        let page_link_rects = page_link_rects.clone();
                                        move |evt: Event<MouseData>| {
                                            let point = evt.client_coordinates();
                                            let end = (point.x, point.y);
                                            let Some(start) = *link_pointer_down.peek() else {
                                                return;
                                            };
                                            link_pointer_down.set(None);
                                            if util::movement_exceeds_click_threshold(
                                                start,
                                                end,
                                                LINK_CLICK_DRAG_THRESHOLD_PX,
                                            ) {
                                                return;
                                            }
                                            evt.stop_propagation();
                                            let on_internal_link = on_internal_link;
                                            let link_rects = page_link_rects.clone();
                                            spawn(async move {
                                                let script =
                                                    "const el = document.getElementById('zoom-image-wrap');\
                                                     if (!el) return null;\
                                                     const r = el.getBoundingClientRect();\
                                                     return [r.left, r.top];";
                                                let Ok(Some(rect_origin)) = eval(&script).join::<Option<Vec<f64>>>().await else {
                                                    return;
                                                };
                                                if rect_origin.len() != 2 {
                                                    return;
                                                }
                                                let relative = util::client_point_relative_to_rect(
                                                    (end.0, end.1),
                                                    (rect_origin[0], rect_origin[1]),
                                                );
                                                if let Some(activation) = util::link_activation_at(
                                                    &link_rects,
                                                    relative.0,
                                                    relative.1,
                                                    page_count,
                                                ) {
                                                    match activation {
                                                        util::ZoomLinkActivation::Internal(target) => {
                                                            on_internal_link.call(target);
                                                        }
                                                        util::ZoomLinkActivation::ExternalUri(raw_uri) => {
                                                            let dialog_id = *external_uri_dialog_seq.peek() + 1;
                                                            external_uri_dialog_seq.set(dialog_id);
                                                            external_uri_dialog.set(Some(ExternalUriDialogState {
                                                                raw_uri,
                                                                copy_status: ExternalUriCopyStatus::Idle,
                                                                dialog_id,
                                                            }));
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    },
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
                                    if !text_segment_rects.is_empty() {
                                        div {
                                            class: "zoom-text-layer",
                                            for positioned in text_segment_rects.iter() {
                                                {
                                                    let segment = &positioned.segment;
                                                    let rect = positioned.rect;
                                                    let key = format!("{}-{}", current_idx.0, segment.segment_index);
                                                    let (tx, ty, tw, th) = (rect.x, rect.y, rect.width, rect.height);
                                                    let font_size = th.max(1.0);
                                                    rsx! {
                                                        span {
                                                            key: "{key}",
                                                            class: "zoom-text-segment",
                                                            style: "left:{tx}px; top:{ty}px; \
                                                                    width:{tw}px; height:{th}px; \
                                                                    font-size:{font_size}px;",
                                                            "{segment.text}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if show_text_unavailable {
                                        div { class: "zoom-text-status",
                                            {t(locale(), MessageKey::ZoomTextSelectionUnavailable)}
                                        }
                                    }
                                    if !page_link_rects.is_empty() {
                                        div { class: "zoom-link-layer",
                                            for positioned in page_link_rects.iter() {
                                                {
                                                    let link = &positioned.link;
                                                    let rect = positioned.rect;
                                                    let key = format!("{}-{}", current_idx.0, link.id.0);
                                                    let (lx, ly, lw, lh) = (rect.x, rect.y, rect.width, rect.height);
                                                    match &link.target {
                                                        NavigationTarget::InternalDestination(destination) => {
                                                            let target = destination.page_index;
                                                            let enabled = target.0 < page_count;
                                                            rsx! {
                                                                button {
                                                                    key: "{key}",
                                                                    class: "zoom-link-overlay internal",
                                                                    disabled: !enabled,
                                                                    title: t(locale(), MessageKey::ZoomLinkInternal),
                                                                    "aria-label": t(locale(), MessageKey::ZoomLinkInternal),
                                                                    style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                                                    onkeydown: move |evt: Event<KeyboardData>| {
                                                                        match evt.key() {
                                                                            Key::Enter => {
                                                                                evt.stop_propagation();
                                                                                on_internal_link.call(target);
                                                                            }
                                                                            Key::Character(ref value) if value == " " => {
                                                                                evt.prevent_default();
                                                                                evt.stop_propagation();
                                                                                on_internal_link.call(target);
                                                                            }
                                                                            _ => {}
                                                                        }
                                                                    },
                                                                }
                                                            }
                                                        }
                                                        NavigationTarget::ExternalUri(uri) => {
                                                            let raw_uri = uri.raw_uri.clone();
                                                            rsx! {
                                                            button {
                                                                key: "{key}",
                                                                class: "zoom-link-overlay external",
                                                                title: t(locale(), MessageKey::ZoomLinkExternalDisabled),
                                                                "aria-label": t(locale(), MessageKey::ZoomLinkExternalDisabled),
                                                                style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                                                onkeydown: move |evt: Event<KeyboardData>| {
                                                                    match evt.key() {
                                                                        Key::Enter => {
                                                                            evt.stop_propagation();
                                                                            let dialog_id = *external_uri_dialog_seq.peek() + 1;
                                                                            external_uri_dialog_seq.set(dialog_id);
                                                                            external_uri_dialog.set(Some(ExternalUriDialogState {
                                                                                raw_uri: raw_uri.clone(),
                                                                                copy_status: ExternalUriCopyStatus::Idle,
                                                                                dialog_id,
                                                                            }));
                                                                        }
                                                                        Key::Character(ref value) if value == " " => {
                                                                            evt.prevent_default();
                                                                            evt.stop_propagation();
                                                                            let dialog_id = *external_uri_dialog_seq.peek() + 1;
                                                                            external_uri_dialog_seq.set(dialog_id);
                                                                            external_uri_dialog.set(Some(ExternalUriDialogState {
                                                                                raw_uri: raw_uri.clone(),
                                                                                copy_status: ExternalUriCopyStatus::Idle,
                                                                                dialog_id,
                                                                            }));
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                },
                                                            }
                                                            }
                                                        }
                                                        NavigationTarget::Disabled(_) => rsx! {
                                                            span {
                                                                key: "{key}",
                                                                class: "zoom-link-overlay disabled",
                                                                title: t(locale(), MessageKey::ZoomLinkDisabled),
                                                                "aria-label": t(locale(), MessageKey::ZoomLinkDisabled),
                                                                style: "left:{lx}px; top:{ly}px; width:{lw}px; height:{lh}px;",
                                                            }
                                                        },
                                                    }
                                                }
                                            }
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
            if let Some(dialog) = external_uri_dialog.read().clone() {
                ExternalUriDialog {
                    raw_uri: dialog.raw_uri,
                    copy_status: dialog.copy_status,
                    dialog_id: dialog.dialog_id,
                    on_copy: Callback::new(move |(dialog_id, raw_uri): (u64, String)| {
                        external_uri_dialog.set(Some(ExternalUriDialogState {
                            raw_uri: raw_uri.clone(),
                            copy_status: ExternalUriCopyStatus::Idle,
                            dialog_id,
                        }));
                        spawn(async move {
                            let next_status = if copy_text_to_clipboard(raw_uri.clone()).await {
                                ExternalUriCopyStatus::Copied
                            } else {
                                ExternalUriCopyStatus::Failed
                            };
                            let still_current = external_uri_dialog
                                .peek()
                                .as_ref()
                                .is_some_and(|state| {
                                    state.dialog_id == dialog_id && state.raw_uri == raw_uri
                                });
                            if !still_current {
                                return;
                            }
                            external_uri_dialog.set(Some(ExternalUriDialogState {
                                raw_uri,
                                copy_status: next_status,
                                dialog_id,
                            }));
                        });
                    }),
                    on_close: Callback::new(move |_| {
                        external_uri_dialog.set(None);
                        focus_zoom_close_button();
                    }),
                }
            }
        }
}

#[component]
fn ExternalUriDialog(
    raw_uri: String,
    copy_status: ExternalUriCopyStatus,
    dialog_id: u64,
    on_copy: Callback<(u64, String)>,
    on_close: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let open_decision =
        validate_external_uri_for_open(&raw_uri, &NavigationResourceLimits::default());
    let policy_message = match open_decision {
        ExternalUriOpenDecision::Allowed { .. } => t(locale(), MessageKey::ExternalUriCopyOnlyBody),
        ExternalUriOpenDecision::Rejected(_) => t(locale(), MessageKey::ExternalUriRejectedBody),
    };

    rsx! {
        div {
            class: "modal-backdrop",
            role: "presentation",
            onkeydown: move |evt: Event<KeyboardData>| {
                evt.stop_propagation();
                match evt.key() {
                    Key::Escape => on_close.call(()),
                    Key::Tab => {
                        evt.prevent_default();
                        trap_external_uri_dialog_focus(evt.modifiers().shift());
                    }
                    _ => {}
                }
            },
            section {
                class: "external-uri-dialog",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "external-uri-dialog-title",
                onmounted: move |_| {
                    focus_external_uri_cancel_button();
                },
                h2 {
                    id: "external-uri-dialog-title",
                    {t(locale(), MessageKey::ExternalUriDialogTitle)}
                }
                p { class: "muted", "{policy_message}" }
                p { class: "muted", {t(locale(), MessageKey::ExternalUriOpenUnavailable)} }
                label {
                    class: "external-uri-label",
                    r#for: "external-uri-value",
                    {t(locale(), MessageKey::ExternalUriTargetLabel)}
                }
                code {
                    id: "external-uri-value",
                    class: "external-uri-value",
                    "{raw_uri}"
                }
                match copy_status {
                    ExternalUriCopyStatus::Idle => rsx! {},
                    ExternalUriCopyStatus::Copied => rsx! {
                        p { class: "external-uri-status", role: "status",
                            {t(locale(), MessageKey::ExternalUriCopyDone)}
                        }
                    },
                    ExternalUriCopyStatus::Failed => rsx! {
                        p { class: "external-uri-status error", role: "alert",
                            {t(locale(), MessageKey::ExternalUriCopyFailed)}
                        }
                    },
                }
                div { class: "external-uri-actions",
                    button {
                        id: "external-uri-cancel",
                        class: "ghost",
                        autofocus: true,
                        onclick: move |_| on_close.call(()),
                        {t(locale(), MessageKey::ExternalUriCancel)}
                    }
                    button {
                        id: "external-uri-copy",
                        class: "primary",
                        onclick: move |_| on_copy.call((dialog_id, raw_uri.clone())),
                        {t(locale(), MessageKey::ExternalUriCopy)}
                    }
                }
            }
        }
    }
}

fn focus_external_uri_cancel_button() {
    let _ = eval(
        "const focusCancel = () => document.getElementById('external-uri-cancel')?.focus();\
         focusCancel();\
         setTimeout(focusCancel, 0);",
    );
}

fn focus_zoom_close_button() {
    let _ = eval("setTimeout(() => document.getElementById('zoom-close-button')?.focus(), 0);");
}

fn trap_external_uri_dialog_focus(backward: bool) {
    let step = if backward { "-1" } else { "1" };
    let script = format!(
        "const ids = ['external-uri-cancel', 'external-uri-copy'];\
         const active = document.activeElement && document.activeElement.id;\
         const current = ids.indexOf(active);\
         const next = current < 0 ? 0 : (current + ({step}) + ids.length) % ids.length;\
         document.getElementById(ids[next])?.focus();"
    );
    let _ = eval(&script);
}

async fn copy_text_to_clipboard(text: String) -> bool {
    let Ok(quoted) = serde_json::to_string(&text) else {
        return false;
    };
    let script = format!(
        "const text = {quoted};\
         if (!navigator.clipboard || !navigator.clipboard.writeText) return false;\
         return navigator.clipboard.writeText(text).then(() => true).catch(() => false);"
    );
    eval(&script).join::<bool>().await.unwrap_or(false)
}
