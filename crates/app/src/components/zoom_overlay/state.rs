//! Load states of the zoom overlay's asynchronous page resources.

use domain::document::PageIndex;

use domain::navigation::PageLinkSet;
use domain::render::ScaleBucket;
use domain::text::PageTextLayer;

use crate::i18n::MessageKey;

/// A rendered page image of the overlay, with the page and scale bucket it belongs to.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ReadyImage {
    pub uri: String,
    pub width_px: u32,
    pub height_px: u32,
    pub page_index: PageIndex,
    pub bucket: ScaleBucket,
}

/// State of the zoom overlay's current page render (RFC 035 § 5.5).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ZoomImageState {
    /// A render is pending. `previous` is the image of the same page at another scale
    /// that stays visible (without text, highlight or link layers) until the new one
    /// arrives; `None` shows the placeholder (first load, or the page changed).
    Loading {
        previous: Option<ReadyImage>,
    },
    Ready(ReadyImage),
    Failed(MessageKey),
}

/// State of the zoom overlay's current selectable text layer.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ZoomTextLayerState {
    Loading,
    Ready(PageTextLayer),
    /// Why there is no text layer, as a catalog key (nothing to select, or an error).
    Unavailable(MessageKey),
}

/// State of the zoom overlay's current page links.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ZoomPageLinksState {
    Loading,
    Ready(PageLinkSet),
    Unavailable,
}
