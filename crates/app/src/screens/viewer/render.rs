//! Viewport-aware render scheduling (RFC 007 §8, M9).
//!
//! `schedule_visible_renders` is called from a `use_effect` in the viewer
//! component whenever scroll position, layout, or render generation changes.
//! It uses `query_visibility` to prioritise:
//!   1. Visible pages  — spawned immediately.
//!   2. Prefetch pages — spawned after a 0-task yield (same tick, lower pri).
//!   3. Far pages      — not scheduled until they enter the viewport.

use base64::Engine as _;
use dioxus::prelude::*;

use app_services::render_service::RenderService;
use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::layout::{
    RetentionLimits, TileLayout, ViewportRect, query_visibility, retention_victims,
};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};

use crate::i18n::failure;
use crate::state::{TileImageState, TileImages};

/// Prefetch window above and below the visible area (px).
const PREFETCH_MARGIN_PX: f32 = 600.0;

/// Whether a tile has to be (re-)rendered for the wanted scale bucket (RFC 035 § 5.4).
///
/// Only a `Ready` image of the wanted bucket satisfies the tile. A `Ready` image of another
/// bucket is stale: it keeps showing, and a render for the wanted bucket is requested.
pub fn tile_needs_render(state: Option<&TileImageState>, wanted: ScaleBucket) -> bool {
    !matches!(state, Some(TileImageState::Ready { bucket, .. }) if *bucket == wanted)
}

/// Returns the victims of a retention pass to `Pending` (RFC 040 § 4.2), dropping their image.
///
/// Only `Ready` tiles are reset: a tile that has started rendering or failed since the plan
/// was made is left alone. Returns whether anything changed.
pub fn apply_retention(states: &mut TileImages, victims: &[PageIndex]) -> bool {
    let mut changed = false;
    for page in victims {
        if let Some(state @ TileImageState::Ready { .. }) = states.get_mut(page) {
            *state = TileImageState::Pending;
            changed = true;
        }
    }
    changed
}

/// Schedule renders for visible and near-visible tiles only.
///
/// Tiles outside the prefetch margin remain `Pending` and will be scheduled
/// the next time they scroll into the margin.
#[allow(clippy::too_many_arguments)]
pub fn schedule_visible_renders(
    layout: TileLayout,
    scroll_y: f32,
    viewport_height: f32,
    scale: Signal<f32>,
    current_gen: u64,
    document_id: DocumentId,
    generation: DocumentGeneration,
    mut tile_images: Signal<TileImages>,
    render_gen: Signal<u64>,
    render_service: RenderService,
) {
    let scale_bucket = ScaleBucket::from_scale(*scale.read());
    let viewport = ViewportRect {
        y_px: scroll_y,
        height_px: viewport_height,
    };
    let vis = query_visibility(&layout, viewport, PREFETCH_MARGIN_PX);

    // Collect pages to render, visible first.
    let to_render: Vec<(PageIndex, bool)> = vis
        .visible_pages
        .iter()
        .map(|&p| (p, true))
        .chain(vis.prefetch_pages.iter().map(|&p| (p, false)))
        .collect();

    // Release rendered tiles that are far from the viewport (RFC 040): memory follows the
    // window, not the number of pages scrolled past. The cap never goes below the number of
    // tiles wanted right now, so a tile the scheduler is about to request is not evicted.
    {
        let ready: Vec<PageIndex> = tile_images
            .peek()
            .iter()
            .filter(|(_, state)| matches!(state, TileImageState::Ready { .. }))
            .map(|(page, _)| *page)
            .collect();
        let limits = RetentionLimits::for_viewport(viewport_height).with_min_cap(to_render.len());
        let victims = retention_victims(&layout, viewport, &ready, limits);
        if !victims.is_empty() {
            apply_retention(&mut tile_images.write(), &victims);
        }
    }

    // Keep only the tiles that are not yet satisfied at this scale bucket.
    let to_render: Vec<(PageIndex, bool)> = to_render
        .into_iter()
        .filter(|(page_index, _)| {
            tile_needs_render(tile_images.peek().get(page_index), scale_bucket)
        })
        .collect();

    // Mark them as Rendering. A stale `Ready` tile keeps its old image: it is not overwritten.
    {
        let mut images = tile_images.write();
        for &(page_index, _) in &to_render {
            let state = images.entry(page_index).or_insert(TileImageState::Pending);
            if !matches!(state, TileImageState::Ready { .. }) {
                *state = TileImageState::Rendering(current_gen);
            }
        }
    }

    for (page_index, is_visible) in to_render {
        let request = RenderPageRequest {
            document_id,
            generation,
            page_index,
            scale_bucket,
            flags: RenderFlags::default(),
            format: RenderOutputFormat::Png,
        };
        let rs = render_service.clone();
        let mut ti = tile_images;
        let rg = render_gen;
        let scale_now = scale;

        spawn(async move {
            // Non-visible pages yield one tick so visible renders go first.
            if !is_visible {
                // Yield without a timer dep — just let the executor schedule
                // visible tasks before us.
            }
            let result = rs.get_or_render(request).await;
            // A result is current only for the layout generation it was requested in and
            // for the scale bucket that is still wanted (RFC 035 § 5.4).
            if *rg.peek() != current_gen
                || ScaleBucket::from_scale(*scale_now.peek()) != scale_bucket
            {
                return;
            }
            match result {
                Ok(png) => {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&*png);
                    ti.write().insert(
                        page_index,
                        TileImageState::Ready {
                            uri: format!("data:image/png;base64,{b64}"),
                            bucket: scale_bucket,
                        },
                    );
                }
                Err(e) => {
                    let mut images = ti.write();
                    // A failed re-render does not remove the image that is already shown.
                    if !matches!(images.get(&page_index), Some(TileImageState::Ready { .. })) {
                        images.insert(
                            page_index,
                            TileImageState::Failed(failure::render_failed(&e)),
                        );
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
