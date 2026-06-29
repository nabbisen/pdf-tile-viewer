//! Utility helpers for the zoom overlay.

use domain::document::PageDescriptor;
use domain::layout::{RectPx, page_rect_to_image_rect};
use domain::search::PageHighlightSet;

/// Extract PNG image dimensions from the IHDR chunk without full decode.
pub fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes(data[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(data[20..24].try_into().ok()?);
    Some((w, h))
}

/// Compute RFC 011 highlight overlay rects for the zoom view.
///
/// Returns image-space `RectPx` values ready to position as overlay `<div>`s.
pub fn zoom_highlight_rects(
    page_index: domain::document::PageIndex,
    page_descriptors: &[PageDescriptor],
    search_highlights: &[PageHighlightSet],
    rendered_width_px: u32,
    rendered_height_px: u32,
) -> Vec<RectPx> {
    let ph = match search_highlights
        .iter()
        .find(|p| p.page_index == page_index)
    {
        Some(ph) => ph,
        None => return Vec::new(),
    };
    let desc = match page_descriptors.iter().find(|d| d.page_index == page_index) {
        Some(d) => d,
        None => return Vec::new(),
    };
    ph.highlights
        .iter()
        .flat_map(|h| &h.page_rects)
        .map(|pr| page_rect_to_image_rect(desc, *pr, rendered_width_px, rendered_height_px))
        .collect()
}
