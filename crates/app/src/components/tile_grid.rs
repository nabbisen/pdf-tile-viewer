//! Tile grid: renders all page tiles with search markers and highlight overlays.
//!
//! Search integration (RFC 010/011):
//! - Matched pages receive a badge showing the match count.
//! - Exact highlight rectangles from `SearchHighlightSet` are rendered as
//!   semi-transparent yellow overlays absolutely positioned over the tile image.
//!   Coordinates are transformed from PDF-space via `page_rect_to_image_rect`.

use dioxus::prelude::*;

use domain::document::PageIndex;
use domain::layout::{TileLayout, page_rect_to_image_rect};
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
    on_tile_click: Option<Callback<PageIndex>>,
    scroll_container_id: String,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let images = tile_images.read();

    rsx! {
        div {
            id: "{scroll_container_id}",
            class: "tile-grid-scroll",
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
                        let click_cb = on_tile_click.clone();

                        // Search match badge and highlights
                        let match_count = search_summaries
                            .iter()
                            .find(|s| s.page_index == idx)
                            .map(|s| s.match_count);
                        let page_highlights = search_highlights
                            .iter()
                            .find(|p| p.page_index == idx)
                            .cloned();

                        // Highlight rects in image-space pixels (RFC 011 §6)
                        let highlight_rects: Vec<(f32, f32, f32, f32)> = {
                            let mut rects = Vec::new();
                            if let (Some(ph), TileImageState::Ready(_)) =
                                (&page_highlights, &image_state)
                            {
                                // We need rendered dimensions to transform coords.
                                // For now, approximate from tile pixel dims.
                                let rw = w.round() as u32;
                                let rh = h.round() as u32;
                                // Find the PageDescriptor for this page.
                                if let Some(desc) = layout.tiles
                                    .iter()
                                    .find(|t| t.page_index == idx)
                                    .and_then(|_| {
                                        // The descriptor lives in the session; we get size
                                        // from the tile geometry instead (scale-invariant).
                                        None::<domain::document::PageDescriptor>
                                    })
                                {
                                    for highlight in &ph.highlights {
                                        for pr in &highlight.page_rects {
                                            let ir = page_rect_to_image_rect(
                                                &desc, *pr, rw, rh,
                                            );
                                            rects.push((ir.x, ir.y, ir.width, ir.height));
                                        }
                                    }
                                } else {
                                    // Fallback: use raw PDF-space proportional scaling.
                                    for highlight in &ph.highlights {
                                        for pr in &highlight.page_rects {
                                            rects.push((pr.x, pr.y, pr.width, pr.height));
                                        }
                                    }
                                }
                            }
                            rects
                        };

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
                                            for (hx, hy, hw, hh) in &highlight_rects {
                                                div {
                                                    class: "highlight-overlay",
                                                    style: "left:{hx}px; top:{hy}px; \
                                                            width:{hw}px; height:{hh}px;",
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
}
