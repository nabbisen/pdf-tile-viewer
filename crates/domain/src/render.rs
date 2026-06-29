//! Render request/result model (RFC 005, RFC 007).

use crate::document::{DocumentGeneration, DocumentId, PageIndex};

/// Scale normalized into integer buckets (percent) so that tiny floating
/// point differences cannot fragment the render cache (RFC 007 §5).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ScaleBucket(pub u16);

impl ScaleBucket {
    pub fn from_scale(scale: f32) -> Self {
        // Bucket to 10%-steps: 0.2 → 20, 1.0 → 100, 2.75 → 280.
        let pct = (scale * 100.0).round() as i32;
        let bucketed = ((pct + 5) / 10) * 10;
        ScaleBucket(bucketed.clamp(10, 1000) as u16)
    }

    pub fn as_scale(self) -> f32 {
        f32::from(self.0) / 100.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub enum RenderBackground {
    #[default]
    White,
    Transparent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct RenderFlags {
    pub background: RenderBackground,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RenderCacheKey {
    pub document_id: DocumentId,
    pub document_generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub scale_bucket: ScaleBucket,
    pub render_flags: RenderFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RenderOutputFormat {
    Png,
    RawRgba,
}

#[derive(Clone, Debug)]
pub struct RenderPageRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub scale_bucket: ScaleBucket,
    pub flags: RenderFlags,
    pub format: RenderOutputFormat,
}

#[derive(Clone, Debug)]
pub enum RenderedImagePayload {
    /// Provisional transport for the RFC 005 vertical slice only.
    /// Must be replaced by a registry/cache URI before large-PDF support
    /// (RFC 005 §7, RFC 007).
    DataUri(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct RenderedPageImage {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub scale_bucket: ScaleBucket,
    pub payload: RenderedImagePayload,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderError {
    DocumentNotOpen,
    PageIndexOutOfRange,
    RenderFailed(String),
    EncodingFailed(String),
    EngineUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RenderPriority {
    Visible,
    NearVisible,
    SearchRelevant,
    Predictive,
    Background,
}
