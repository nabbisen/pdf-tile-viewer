//! Selectable text layer drawn over the zoomed page image (RFC 023).

use dioxus::prelude::*;

use domain::document::PageIndex;

use super::util;

/// Absolutely positioned, transparent text spans aligned with the page image.
pub(super) fn text_layer(
    text_segment_rects: &[util::ZoomTextSegmentRect],
    current_idx: PageIndex,
) -> Element {
    rsx! {
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
    }
}
