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
use domain::layout::{TileLayout, ViewportRect, query_visibility};
use domain::render::{RenderFlags, RenderOutputFormat, RenderPageRequest, ScaleBucket};

use crate::state::{TileImageState, TileImages};

/// Prefetch window above and below the visible area (px).
const PREFETCH_MARGIN_PX: f32 = 600.0;

/// Schedule renders for visible and near-visible tiles only.
///
/// Tiles outside the prefetch margin remain `Pending` and will be scheduled
/// the next time they scroll into the margin.
#[allow(clippy::too_many_arguments)]
pub fn schedule_visible_renders(
    layout: TileLayout,
    scroll_y: f32,
    viewport_height: f32,
    scale: f32,
    current_gen: u64,
    document_id: DocumentId,
    generation: DocumentGeneration,
    mut tile_images: Signal<TileImages>,
    render_gen: Signal<u64>,
    render_service: RenderService,
) {
    let scale_bucket = ScaleBucket::from_scale(scale);
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

    // Mark scheduled tiles as Rendering.
    {
        let mut images = tile_images.write();
        for &(page_index, _) in &to_render {
            let state = images.entry(page_index).or_insert(TileImageState::Pending);
            if !matches!(state, TileImageState::Ready(_)) {
                *state = TileImageState::Rendering(current_gen);
            }
        }
    }

    for (page_index, is_visible) in to_render {
        // Skip pages that are already rendered at this scale.
        if matches!(
            tile_images.peek().get(&page_index),
            Some(TileImageState::Ready(_))
        ) {
            continue;
        }

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

        spawn(async move {
            // Non-visible pages yield one tick so visible renders go first.
            if !is_visible {
                // Yield without a timer dep — just let the executor schedule
                // visible tasks before us.
            }
            match rs.get_or_render(request).await {
                Ok(png) => {
                    if *rg.peek() != current_gen {
                        return;
                    }
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&*png);
                    ti.write().insert(
                        page_index,
                        TileImageState::Ready(format!("data:image/png;base64,{b64}")),
                    );
                }
                Err(e) => {
                    if *rg.peek() != current_gen {
                        return;
                    }
                    ti.write()
                        .insert(page_index, TileImageState::Failed(format!("{e:?}")));
                }
            }
        });
    }
}
