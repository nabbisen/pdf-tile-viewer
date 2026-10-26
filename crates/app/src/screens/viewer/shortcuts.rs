//! Key handling of the viewer root (RFC 033 § 5.3): which layer is on top, and what the resolver
//! says about a key there. A pure move out of `viewer.rs`; the dispatcher that acts on the result
//! stays in the component because it owns the signals.

use dioxus::prelude::*;

use domain::shortcuts::{Layer, ShortcutAction, ShortcutContext, resolve};

use crate::keys::key_descriptor;

/// Which transient layers are open (the viewer's own flags).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct LayerFlags {
    pub zoom_open: bool,
    pub search_open: bool,
    pub outline_open: bool,
}

/// The top-most open layer: zoom, then search, then outline, else the grid.
pub(super) fn top_layer(flags: LayerFlags) -> Layer {
    if flags.zoom_open {
        Layer::Zoom
    } else if flags.search_open {
        Layer::Search
    } else if flags.outline_open {
        Layer::Outline
    } else {
        Layer::Base
    }
}

/// What the resolver makes of `event` with these layers open.
pub(super) fn action_for_key(
    event: &KeyboardData,
    flags: LayerFlags,
    zen: bool,
) -> Option<ShortcutAction> {
    resolve(
        ShortcutContext {
            top_layer: top_layer(flags),
            zen,
        },
        &key_descriptor(event),
    )
}

#[cfg(test)]
mod tests;
