//! Tile grid: renders all page tiles with search markers and highlight overlays.
//!
//! Search integration (RFC 010/011):
//! - Matched pages receive a badge showing the match count.
//! - Exact highlight rectangles from `SearchHighlightSet` are rendered as
//!   semi-transparent yellow overlays absolutely positioned over the tile image.
//!   Coordinates are transformed from PDF-space via `page_rect_to_image_rect`.

use dioxus::prelude::*;

use domain::document::{PageDescriptor, PageIndex};
use domain::layout::{RectPx, TileLayout, page_rect_to_image_rect};
use domain::search::{PageHighlightSet, PageSearchSummary};

use crate::i18n::{Locale, MessageKey, t};
use crate::state::{TileImageState, TileImages};

#[component]
pub fn TileGrid(
    layout: TileLayout,
    tile_images: Signal<TileImages>,
    show_page_numbers: bool,
    /// Per-page search match summaries for badge display (RFC 010).
    search_summaries: Vec<PageSearchSummary>,
    /// Per-page highlight rectangles for overlay (RFC 011); empty = no overlays.
    search_highlights: Vec<PageHighlightSet>,
    /// Page descriptors from the active session, used to transform PDF-space
    /// highlight rectangles into rendered tile-image pixels (RFC 011).
    page_descriptors: Vec<PageDescriptor>,
    on_tile_click: Option<Callback<PageIndex>>,
    scroll_container_id: String,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let images = tile_images.read();

    rsx! {
        div {
                class: "tile-grid-canvas",
                style: "position: relative; \
                        width: {layout.content_width_px}px; \
                        height: {layout.content_height_px}px;",
                for tile in &layout.tiles {
                    {
                        let idx = tile.page_index;
                        let x = tile.image_rect_px.x;
                        let y = tile.image_rect_px.y;
                        let w = tile.image_rect_px.width;
                        let h = tile.image_rect_px.height;
                        let display_num = idx.display_number();
                        let image_state = images.get(&idx).cloned()
                            .unwrap_or(TileImageState::Pending);
                        let tile_id = format!("tile-{}", idx.0);
                        let click_cb = on_tile_click;

                        // Search match badge and highlights
                        let match_count = search_summaries
                            .iter()
                            .find(|s| s.page_index == idx)
                            .map(|s| s.match_count);
                        let page_highlights = search_highlights
                            .iter()
                            .find(|p| p.page_index == idx)
                            .cloned();

                        // Highlight rects in image-space pixels (RFC 011 §6).
                        let highlight_rects = tile_highlight_rects(
                            idx,
                            &page_descriptors,
                            page_highlights.as_ref(),
                            &image_state,
                            w.round() as u32,
                            h.round() as u32,
                        );

                        rsx! {
                            div {
                                key: "{idx.0}",
                                id: "{tile_id}",
                                class: if match_count.is_some() {
                                    "page-tile search-match"
                                } else {
                                    "page-tile"
                                },
                                style: "position: absolute; \
                                        left: {x}px; top: {y}px; \
                                        width: {w}px; height: {h}px;",
                                onclick: move |_| {
                                    if let Some(cb) = &click_cb {
                                        cb.call(idx);
                                    }
                                },

                                div {
                                    class: "tile-image-area",
                                    style: "position: relative; width: {w}px; height: {h}px; overflow: hidden;",

                                    match &image_state {
                                        TileImageState::Ready(uri) => rsx! {
                                            img {
                                                class: "tile-img",
                                                src: "{uri}",
                                                alt: "Page {display_num}",
                                                style: "display: block; \
                                                        width: {w}px; height: {h}px; \
                                                        object-fit: contain;",
                                            }
                                            // RFC 011 highlight overlays
                                            for rect in highlight_rects.iter().copied() {
                                                {
                                                    let (hx, hy, hw, hh) = (rect.x, rect.y, rect.width, rect.height);
                                                    rsx! {
                                                        div {
                                                            class: "highlight-overlay",
                                                            style: "left:{hx}px; top:{hy}px; \
                                                                    width:{hw}px; height:{hh}px;",
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        TileImageState::Pending | TileImageState::Rendering(_) => rsx! {
                                            div {
                                                class: "tile-placeholder loading",
                                                style: "width: {w}px; height: {h}px;",
                                                span { class: "tile-loading-dot" }
                                            }
                                        },
                                        TileImageState::Failed(_) => rsx! {
                                            div {
                                                class: "tile-placeholder error",
                                                style: "width: {w}px; height: {h}px;",
                                                span { "⚠" }
                                                span { class: "muted",
                                                    {t(locale(), MessageKey::ErrRenderFailed)}
                                                }
                                            }
                                        },
                                    }
                                }

                                // Match-count badge (RFC 010)
                                if let Some(count) = match_count {
                                    div {
                                        class: "search-match-badge",
                                        "aria-label": "{count} matches",
                                        "{count}"
                                    }
                                }

                                if show_page_numbers {
                                    div { class: "tile-label", "{display_num}" }
                                }
                            }
                        }
                    }
            }
        }
    }
}

fn tile_highlight_rects(
    page_index: PageIndex,
    page_descriptors: &[PageDescriptor],
    page_highlights: Option<&PageHighlightSet>,
    image_state: &TileImageState,
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> Vec<RectPx> {
    if !matches!(image_state, TileImageState::Ready(_)) {
        return Vec::new();
    }
    let Some(highlights) = page_highlights else {
        return Vec::new();
    };
    let Some(descriptor) = page_descriptors
        .iter()
        .find(|page| page.page_index == page_index)
    else {
        return Vec::new();
    };

    highlights
        .highlights
        .iter()
        .flat_map(|highlight| &highlight.page_rects)
        .map(|rect| {
            page_rect_to_image_rect(descriptor, *rect, rendered_width_px, rendered_height_px)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::search::{PageCoordinateSpace, PageRect, TextHighlight};

    fn descriptor() -> PageDescriptor {
        PageDescriptor {
            page_index: PageIndex(0),
            width_points: 612.0,
            height_points: 792.0,
            rotation_degrees: 0,
        }
    }

    fn highlights() -> PageHighlightSet {
        PageHighlightSet {
            page_index: PageIndex(0),
            highlights: vec![TextHighlight {
                match_index: 0,
                page_rects: vec![PageRect {
                    x: 0.0,
                    y: 712.8,
                    width: 612.0,
                    height: 79.2,
                    space: PageCoordinateSpace::PdfPointsBottomLeft,
                }],
            }],
        }
    }

    #[test]
    fn tile_highlights_use_page_descriptor_transform() {
        let rects = tile_highlight_rects(
            PageIndex(0),
            &[descriptor()],
            Some(&highlights()),
            &TileImageState::Ready("data:image/png;base64,".to_string()),
            1224,
            1584,
        );

        assert_eq!(rects.len(), 1);
        assert!((rects[0].width - 1224.0).abs() < 1.0);
        assert!((rects[0].height - 158.4).abs() < 1.0);
        assert!(rects[0].y < 2.0);
    }

    #[test]
    fn tile_highlights_do_not_fallback_to_raw_pdf_rects() {
        let rects = tile_highlight_rects(
            PageIndex(0),
            &[],
            Some(&highlights()),
            &TileImageState::Ready("data:image/png;base64,".to_string()),
            1224,
            1584,
        );

        assert!(rects.is_empty());
    }
}
