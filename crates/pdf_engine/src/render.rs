//! Page rendering (RFC 005).
//!
//! Renders one page to RGBA via PDFium, then encodes PNG with the `png`
//! crate. Output is `RenderedPageImage` carrying document id + generation
//! so stale results can be discarded by the caller.

use domain::render::{
    RenderError, RenderOutputFormat, RenderPageRequest, RenderedImagePayload, RenderedPageImage,
};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;

/// Hard cap on the longer rendered edge, bounding memory for very large
/// pages (RFC 016 §10).
const MAX_PIXEL_EDGE: i32 = 8192;

pub fn render_page(
    engine: &PdfEngine,
    request: &RenderPageRequest,
) -> Result<RenderedPageImage, RenderError> {
    let session = engine
        .session(request.document_id)
        .ok_or(RenderError::DocumentNotOpen)?;
    if session.generation != request.generation {
        return Err(RenderError::DocumentNotOpen);
    }
    let descriptor = session
        .pages
        .get(request.page_index.0)
        .copied()
        .ok_or(RenderError::PageIndexOutOfRange)?;

    let document = engine
        .native(request.document_id)
        .ok_or(RenderError::DocumentNotOpen)?;
    let page = document
        .pages()
        .get(request.page_index.0 as i32)
        .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

    let scale = request.scale_bucket.as_scale();
    let target_w = ((descriptor.width_points * scale).round() as i32).clamp(1, MAX_PIXEL_EDGE);
    let target_h = ((descriptor.height_points * scale).round() as i32).clamp(1, MAX_PIXEL_EDGE);

    let config = PdfRenderConfig::new()
        .set_target_width(target_w)
        .set_maximum_height(MAX_PIXEL_EDGE);
    let _ = target_h; // height follows aspect ratio; kept for clarity

    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| RenderError::RenderFailed(e.to_string()))?;
    let width = bitmap.width() as u32;
    let height = bitmap.height() as u32;
    let rgba = bitmap.as_rgba_bytes();

    let payload = match request.format {
        RenderOutputFormat::Png => RenderedImagePayload::Bytes(encode_png(&rgba, width, height)?),
    };

    Ok(RenderedPageImage {
        document_id: request.document_id,
        generation: request.generation,
        page_index: request.page_index,
        pixel_width: width,
        pixel_height: height,
        scale_bucket: request.scale_bucket,
        payload,
    })
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| RenderError::EncodingFailed(e.to_string()))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| RenderError::EncodingFailed(e.to_string()))?;
    }
    Ok(out)
}
