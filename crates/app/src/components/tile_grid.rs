//! Tile grid: renders all page tiles using absolute-positioned divs inside
//! a scroll container sized to the layout's content dimensions (RFC 006).
//!
//! Each tile shows one of: a rendered PNG image, a loading placeholder, or
//! an error indicator. The tile image state comes from the caller's signal
//! (RFC 007 §11 TileImageState).

use dioxus::prelude::*;

use domain::document::PageIndex;
use domain::layout::TileLayout;

use crate::i18n::{Locale, MessageKey, t};
use crate::state::{TileImageState, TileImages};

const TILE_LABEL_HEIGHT: f32 = 20.0;

#[component]
pub fn TileGrid(
    layout: TileLayout,
    tile_images: Signal<TileImages>,
    show_page_numbers: bool,
    /// Called when a tile is clicked; carries the zero-based PageIndex.
    on_tile_click: Option<Callback<PageIndex>>,
    /// Scroll container id for JS scroll-into-view (RFC 008 jump-to-page).
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

                        rsx! {
                            div {
                                key: "{idx.0}",
                                id: "{tile_id}",
                                class: "page-tile",
                                style: "position: absolute; \
                                        left: {x}px; top: {y}px; \
                                        width: {w}px; height: {h}px;",
                                onclick: move |_| {
                                    if let Some(cb) = &click_cb {
                                        cb.call(idx);
                                    }
                                },

                                // Image area
                                div {
                                    class: "tile-image-area",
                                    style: "width: {w}px; height: {h}px; overflow: hidden;",

                                    match &image_state {
                                        TileImageState::Ready(uri) => rsx! {
                                            img {
                                                class: "tile-img",
                                                src: "{uri}",
                                                alt: "Page {display_num}",
                                                width: "{w}",
                                                height: "{h}",
                                                style: "display: block; \
                                                        width: {w}px; height: {h}px; \
                                                        object-fit: contain;",
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
                                                span { class: "muted", {t(locale(), MessageKey::ErrRenderFailed)} }
                                            }
                                        },
                                    }
                                }

                                // Optional page number label
                                if show_page_numbers {
                                    div {
                                        class: "tile-label",
                                        style: "height: {TILE_LABEL_HEIGHT}px; \
                                                line-height: {TILE_LABEL_HEIGHT}px;",
                                        "{display_num}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
