//! Loading effects of the zoom overlay: page image, page links, text layer.
//!
//! Each hook starts one effect that re-runs when the displayed page (or, for the
//! image, the scale) changes. Results are applied only while their request is
//! still current (see `util` for the guards).

use base64::Engine as _;
use dioxus::prelude::*;

use app_services::navigation_service::{NavigationService, default_page_links_request};
use app_services::render_service::RenderService;
use app_services::text_service::TextLayerService;
use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};
use domain::text::TextLayerRequest;

use crate::i18n::MessageKey;
use crate::i18n::failure;

use super::state::{ReadyImage, ZoomImageState, ZoomPageLinksState, ZoomTextLayerState};
use super::util;

/// Render the current page when the page or scale changes.
pub(super) fn use_page_image_loader(
    render_service: RenderService,
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: Signal<Option<PageIndex>>,
    zoom_scale: Signal<f32>,
    mut image_request_seq: Signal<u64>,
    mut zoom_image: Signal<ZoomImageState>,
) {
    let rs = render_service;
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
        // The image currently shown (or kept while a render is pending), if any.
        let shown: Option<ReadyImage> = match &*zoom_image.peek() {
            ZoomImageState::Ready(image) => Some(image.clone()),
            ZoomImageState::Loading { previous } => previous.clone(),
            ZoomImageState::Failed(_) => None,
        };
        // Already showing this page at this bucket: nothing to render.
        if matches!(&*zoom_image.peek(), ZoomImageState::Ready(image)
            if image.page_index == idx && image.bucket == scale_bucket)
        {
            return;
        }
        let request_id = *image_request_seq.peek() + 1;
        image_request_seq.set(request_id);
        // Scale change on the same page: keep the old image until the new one arrives
        // (RFC 035 § 5.5). Page change: placeholder.
        let previous = shown
            .filter(|image| util::keep_previous(image.page_index, image.bucket, idx, scale_bucket));
        zoom_image.set(ZoomImageState::Loading { previous });

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
                        zi.set(ZoomImageState::Ready(ReadyImage {
                            uri: format!("data:image/png;base64,{b64}"),
                            width_px: w,
                            height_px: h,
                            page_index: idx,
                            bucket: scale_bucket,
                        }));
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
                        zi.set(ZoomImageState::Failed(failure::render_failed(&e)));
                    }
                }
            }
        });
    });
}

/// Page link overlay when the page changes (RFC 026 PR3).
pub(super) fn use_page_links_loader(
    navigation_service: NavigationService,
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: Signal<Option<PageIndex>>,
    mut links_request_seq: Signal<u64>,
    mut zoom_page_links: Signal<ZoomPageLinksState>,
) {
    let ns = navigation_service;
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
                Err(e) if is_current() => {
                    // links have no message of their own: the page simply has no link areas
                    let _ = failure::navigation_failed(&e);
                    link_state.set(ZoomPageLinksState::Unavailable);
                }
                Err(_) => {}
            }
        });
    });
}

/// Selectable text layer when the page changes (RFC 023).
pub(super) fn use_text_layer_loader(
    text_service: TextLayerService,
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: Signal<Option<PageIndex>>,
    mut text_request_seq: Signal<u64>,
    mut zoom_text_layer: Signal<ZoomTextLayerState>,
) {
    let ts = text_service;
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
                        text_state.set(ZoomTextLayerState::Unavailable(
                            MessageKey::ZoomTextSelectionUnavailable,
                        ));
                    } else {
                        text_state.set(ZoomTextLayerState::Ready((*layer).clone()));
                    }
                }
                Ok(_) => {}
                Err(e) if is_current() => {
                    text_state.set(ZoomTextLayerState::Unavailable(failure::text_layer_failed(
                        &e,
                    )));
                }
                Err(_) => {}
            }
        });
    });
}
