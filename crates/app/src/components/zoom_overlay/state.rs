//! Load states of the zoom overlay's asynchronous page resources.

use domain::navigation::PageLinkSet;
use domain::text::PageTextLayer;

/// State of the zoom overlay's current page render.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ZoomImageState {
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
pub(super) enum ZoomTextLayerState {
    Loading,
    Ready(PageTextLayer),
    Unavailable,
}

/// State of the zoom overlay's current page links.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ZoomPageLinksState {
    Loading,
    Ready(PageLinkSet),
    Unavailable,
}
