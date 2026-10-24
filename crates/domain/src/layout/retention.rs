//! Tile retention plan (RFC 040 § 4.1).
//!
//! Decides which already rendered tiles may stay in memory: the ones near the viewport.
//! Pure and deterministic; the viewer applies the result by returning victims to `Pending`.
//! The policy lives here so a later transport/virtualization redesign can keep it.

use std::cmp::Ordering;

use super::{TileLayout, ViewportRect};
use crate::document::PageIndex;

/// Kept distance above and below the viewport, in viewport heights.
pub const RETENTION_WINDOW_VIEWPORTS: f32 = 2.0;
/// Lower bound of the kept distance (px); larger than the 600 px prefetch margin.
pub const RETENTION_MIN_WINDOW_PX: f32 = 1500.0;
/// Hard cap on tiles held in the `Ready` state.
pub const RETENTION_MAX_READY_TILES: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetentionLimits {
    /// Kept distance above and below the viewport (px).
    pub window_px: f32,
    /// Hard cap on tiles held in the `Ready` state.
    pub max_ready_tiles: usize,
}

impl RetentionLimits {
    /// Default limits for a viewport: `window_px = max(2 × height, 1500)`, 80 tiles.
    pub fn for_viewport(viewport_height_px: f32) -> Self {
        RetentionLimits {
            window_px: (RETENTION_WINDOW_VIEWPORTS * viewport_height_px)
                .max(RETENTION_MIN_WINDOW_PX),
            max_ready_tiles: RETENTION_MAX_READY_TILES,
        }
    }

    /// Raises the cap to at least `count`. The scheduler calls this with the number of
    /// tiles it wants rendered (visible + prefetch): at small scales that can exceed the
    /// default cap, and evicting tiles that are about to be requested again would only
    /// make them churn.
    pub fn with_min_cap(self, count: usize) -> Self {
        RetentionLimits {
            max_ready_tiles: self.max_ready_tiles.max(count),
            ..self
        }
    }
}

/// Which of the `ready` tiles to release.
///
/// A `Ready` tile is kept if its rectangle intersects
/// `[viewport.top - window_px, viewport.bottom + window_px]` (edges inclusive); others are
/// victims, as are pages unknown to the layout. If more than `max_ready_tiles` remain, the
/// kept tiles farthest from the viewport centre are added until the cap holds. Tiles that
/// intersect the viewport are never victims, not even for the cap. The result is sorted by
/// page index.
pub fn retention_victims(
    layout: &TileLayout,
    viewport: ViewportRect,
    ready: &[PageIndex],
    limits: RetentionLimits,
) -> Vec<PageIndex> {
    let view_top = viewport.y_px;
    let view_bottom = viewport.y_px + viewport.height_px;
    let view_centre = viewport.y_px + viewport.height_px / 2.0;
    let keep_top = view_top - limits.window_px;
    let keep_bottom = view_bottom + limits.window_px;

    let mut victims: Vec<PageIndex> = Vec::new();
    // (page, distance of the tile centre from the viewport centre) of kept, non-visible tiles
    let mut removable: Vec<(PageIndex, f32)> = Vec::new();
    let mut kept = 0usize;

    for &page in ready {
        let Some(tile) = tile_of(layout, page) else {
            victims.push(page);
            continue;
        };
        let (top, bottom) = (tile.y_px, tile.y_px + tile.height_px);
        if bottom < keep_top || top > keep_bottom {
            victims.push(page);
            continue;
        }
        kept += 1;
        if bottom < view_top || top > view_bottom {
            removable.push((page, ((top + bottom) / 2.0 - view_centre).abs()));
        }
    }

    if kept > limits.max_ready_tiles {
        // Farthest first; equal distances: the higher page index goes first (deterministic).
        removable.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(Ordering::Equal)
                .then(b.0.cmp(&a.0))
        });
        let excess = kept - limits.max_ready_tiles;
        victims.extend(removable.iter().take(excess).map(|(page, _)| *page));
    }

    victims.sort();
    victims.dedup();
    victims
}

fn tile_of(layout: &TileLayout, page: PageIndex) -> Option<&super::PageTileLayout> {
    // Tiles are laid out in page order, so the index is normally the page number.
    layout
        .tiles
        .get(page.0)
        .filter(|tile| tile.page_index == page)
        .or_else(|| layout.tiles.iter().find(|tile| tile.page_index == page))
}

#[cfg(test)]
mod tests;
