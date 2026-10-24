//! PDF outline and page-link extraction (RFC 026).
//!
//! PDFium handles stay inside this crate. Returned data is domain-owned,
//! generation-tagged, and bounded by [`NavigationResourceLimits`].

use domain::document::PageIndex;
use domain::navigation::{
    DestinationView, DisabledNavigationReason, DocumentOutline, DocumentOutlineRequest,
    ExternalUriTarget, NavigationDestination, NavigationError, NavigationLimitStatus,
    NavigationResourceLimits, NavigationTarget, OutlineNode, OutlineNodeId, OutlineTitle, PageLink,
    PageLinkId, PageLinkSet, PageLinksRequest, external_uri_target,
};
use pdfium_render::prelude::*;

use crate::engine::PdfEngine;
use crate::user_space::{DisplayAxis, PageGeometry, UserRect};

struct ExtractionBudget {
    limits: NavigationResourceLimits,
    status: NavigationLimitStatus,
    outline_nodes: usize,
    string_bytes: usize,
}

impl ExtractionBudget {
    fn new(limits: NavigationResourceLimits) -> Self {
        Self {
            limits,
            status: NavigationLimitStatus::default(),
            outline_nodes: 0,
            string_bytes: 0,
        }
    }

    fn reserve_outline_node(&mut self) -> bool {
        if self.outline_nodes >= self.limits.max_outline_nodes {
            self.status.outline_truncated = true;
            return false;
        }
        self.outline_nodes += 1;
        true
    }

    fn record_title(&mut self, title: OutlineTitle) -> OutlineTitle {
        if matches!(title, OutlineTitle::Truncated(_)) {
            self.status.strings_truncated = true;
        }

        let bytes = title.stored_bytes();
        if bytes == 0 {
            return title;
        }
        if self.string_bytes + bytes <= self.limits.max_outline_string_bytes {
            self.string_bytes += bytes;
            return title;
        }

        self.status.outline_truncated = true;
        self.status.strings_truncated = true;
        match title {
            OutlineTitle::Present(value) | OutlineTitle::Truncated(value) => {
                let remaining = self
                    .limits
                    .max_outline_string_bytes
                    .saturating_sub(self.string_bytes);
                let truncated = truncate_utf8_boundary(&value, remaining);
                self.string_bytes += truncated.len();
                OutlineTitle::Truncated(truncated)
            }
            OutlineTitle::Missing | OutlineTitle::Empty => title,
        }
    }

    fn record_uri_target(&mut self, raw_uri: String) -> NavigationTarget {
        let target = external_uri_target(raw_uri, &self.limits);
        let NavigationTarget::ExternalUri(uri) = target else {
            if matches!(
                target,
                NavigationTarget::Disabled(DisabledNavigationReason::ExternalUriTooLong)
            ) {
                self.status.strings_truncated = true;
            }
            return target;
        };

        let raw_uri = uri.raw_uri;
        if self.string_bytes + raw_uri.len() > self.limits.max_outline_string_bytes {
            self.status.outline_truncated = true;
            self.status.strings_truncated = true;
            return NavigationTarget::Disabled(DisabledNavigationReason::ExternalUriTooLong);
        }
        self.string_bytes += raw_uri.len();
        NavigationTarget::ExternalUri(ExternalUriTarget { raw_uri })
    }
}

pub fn extract_document_outline(
    engine: &PdfEngine,
    request: &DocumentOutlineRequest,
) -> Result<DocumentOutline, NavigationError> {
    let session = engine
        .session(request.document_id)
        .ok_or(NavigationError::DocumentNotOpen)?;
    if session.generation != request.generation {
        return Err(NavigationError::DocumentNotOpen);
    }

    let document = engine
        .native(request.document_id)
        .ok_or(NavigationError::DocumentNotOpen)?;
    let geometries = engine.geometries(request.document_id);
    let mut budget = ExtractionBudget::new(request.limits);
    let mut roots = Vec::new();

    if let Some(root) = document.bookmarks().root() {
        let mut current = Some(root);
        let mut sibling_index = 0usize;
        while let Some(bookmark) = current {
            let next = bookmark.next_sibling();
            if let Some(node) =
                extract_outline_node(&bookmark, vec![sibling_index], 1, &mut budget, geometries)
            {
                roots.push(node);
            }
            if budget.outline_nodes >= budget.limits.max_outline_nodes {
                budget.status.outline_truncated = true;
                break;
            }
            current = next;
            sibling_index += 1;
        }
    }

    Ok(DocumentOutline {
        document_id: request.document_id,
        generation: request.generation,
        roots,
        limit_status: budget.status,
    })
}

pub fn extract_page_links(
    engine: &PdfEngine,
    request: &PageLinksRequest,
) -> Result<PageLinkSet, NavigationError> {
    let session = engine
        .session(request.document_id)
        .ok_or(NavigationError::DocumentNotOpen)?;
    if session.generation != request.generation {
        return Err(NavigationError::DocumentNotOpen);
    }
    if request.page_index.0 >= session.pages.len() {
        return Err(NavigationError::PageOutOfBounds);
    }

    let document = engine
        .native(request.document_id)
        .ok_or(NavigationError::DocumentNotOpen)?;
    let page = document
        .pages()
        .get(request.page_index.0 as PdfPageIndex)
        .map_err(|_| NavigationError::PageOutOfBounds)?;

    let geometries = engine.geometries(request.document_id);
    let geometry = geometries.get(request.page_index.0).copied();
    let mut links = Vec::new();
    let mut status = NavigationLimitStatus::default();
    for (index, link) in page.links().iter().enumerate() {
        if links.len() >= request.limits.max_links_per_page {
            status.links_truncated = true;
            break;
        }
        let link_index = u32::try_from(index).map_err(|_| NavigationError::LinkIndexOverflow)?;
        let Ok(rect) = link.rect() else {
            continue;
        };
        // Degenerate rects and pages with inconsistent geometry get no clickable overlay.
        let Some(rect) = geometry.and_then(|g| {
            g.rect_to_display(UserRect {
                left: rect.left().value,
                bottom: rect.bottom().value,
                right: rect.right().value,
                top: rect.top().value,
            })
        }) else {
            continue;
        };

        links.push(PageLink {
            id: PageLinkId(link_index),
            rect,
            target: link_target(
                link.destination(),
                link.action(),
                &request.limits,
                None,
                geometries,
            ),
        });
    }

    Ok(PageLinkSet {
        document_id: request.document_id,
        generation: request.generation,
        page_index: request.page_index,
        links,
        limit_status: status,
    })
}

fn extract_outline_node(
    bookmark: &PdfBookmark<'_>,
    path: Vec<usize>,
    depth: usize,
    budget: &mut ExtractionBudget,
    geometries: &[PageGeometry],
) -> Option<OutlineNode> {
    if depth > budget.limits.max_outline_depth {
        budget.status.outline_truncated = true;
        return None;
    }
    if !budget.reserve_outline_node() {
        return None;
    }

    let title = OutlineTitle::from_pdf_title(bookmark.title(), &budget.limits);
    let title = budget.record_title(title);
    let mut children = Vec::new();

    if depth < budget.limits.max_outline_depth {
        for (child_index, child) in bookmark.iter_direct_children().enumerate() {
            let mut child_path = path.clone();
            child_path.push(child_index);
            if let Some(node) =
                extract_outline_node(&child, child_path, depth + 1, budget, geometries)
            {
                children.push(node);
            }
            if budget.outline_nodes >= budget.limits.max_outline_nodes {
                budget.status.outline_truncated = true;
                break;
            }
        }
    } else if bookmark.children_len() > 0 {
        budget.status.outline_truncated = true;
    }

    let limits = budget.limits;
    let target = link_target(
        bookmark.destination(),
        bookmark.action(),
        &limits,
        Some(budget),
        geometries,
    );

    Some(OutlineNode {
        id: OutlineNodeId(outline_path_id(&path)),
        title,
        target,
        children,
    })
}

fn link_target(
    destination: Option<PdfDestination<'_>>,
    action: Option<PdfAction<'_>>,
    limits: &NavigationResourceLimits,
    outline_budget: Option<&mut ExtractionBudget>,
    geometries: &[PageGeometry],
) -> NavigationTarget {
    if let Some(action) = action {
        return action_target(action, limits, outline_budget, geometries);
    }

    match destination {
        Some(destination) => destination_target(destination, geometries),
        None => NavigationTarget::Disabled(DisabledNavigationReason::NoDestination),
    }
}

fn action_target(
    action: PdfAction<'_>,
    limits: &NavigationResourceLimits,
    mut outline_budget: Option<&mut ExtractionBudget>,
    geometries: &[PageGeometry],
) -> NavigationTarget {
    match action {
        PdfAction::LocalDestination(action) => match action.destination() {
            Ok(destination) => destination_target(destination, geometries),
            Err(_) => NavigationTarget::Disabled(DisabledNavigationReason::InvalidDestination),
        },
        PdfAction::Uri(action) => match action.uri() {
            Ok(raw_uri) => {
                if let Some(budget) = outline_budget.as_mut() {
                    budget.record_uri_target(raw_uri)
                } else {
                    external_uri_target(raw_uri, limits)
                }
            }
            Err(_) => NavigationTarget::Disabled(DisabledNavigationReason::InvalidExternalUri),
        },
        PdfAction::RemoteDestination(_) => NavigationTarget::Disabled(
            disabled_reason_for_action_type(PdfActionType::GoToDestinationInRemoteDocument),
        ),
        PdfAction::EmbeddedDestination(_) => NavigationTarget::Disabled(
            disabled_reason_for_action_type(PdfActionType::GoToDestinationInEmbeddedDocument),
        ),
        PdfAction::Launch(_) => {
            NavigationTarget::Disabled(disabled_reason_for_action_type(PdfActionType::Launch))
        }
        PdfAction::Unsupported(_) => {
            NavigationTarget::Disabled(disabled_reason_for_action_type(PdfActionType::Unsupported))
        }
    }
}

fn disabled_reason_for_action_type(action_type: PdfActionType) -> DisabledNavigationReason {
    match action_type {
        PdfActionType::GoToDestinationInRemoteDocument => {
            DisabledNavigationReason::RemoteDestination
        }
        PdfActionType::GoToDestinationInEmbeddedDocument => {
            DisabledNavigationReason::EmbeddedDestination
        }
        PdfActionType::Launch => DisabledNavigationReason::LaunchAction,
        PdfActionType::Unsupported => DisabledNavigationReason::UnsupportedAction,
        PdfActionType::GoToDestinationInSameDocument | PdfActionType::Uri => {
            DisabledNavigationReason::UnsupportedAction
        }
    }
}

/// Destination coordinates are converted to display space with the geometry of the
/// **destination** page (not the page holding the link); without usable geometry the
/// view degrades to `PageOnly`, the navigation itself is kept.
fn destination_target(
    destination: PdfDestination<'_>,
    geometries: &[PageGeometry],
) -> NavigationTarget {
    match destination.page_index() {
        Ok(page_index) => {
            let geometry = geometries.get(page_index as usize).copied();
            NavigationTarget::InternalDestination(NavigationDestination {
                page_index: PageIndex(page_index as usize),
                view: destination_view(destination.view_settings().ok(), geometry),
            })
        }
        Err(_) => NavigationTarget::Disabled(DisabledNavigationReason::InvalidDestination),
    }
}

fn destination_view(
    settings: Option<PdfDestinationViewSettings>,
    geometry: Option<PageGeometry>,
) -> DestinationView {
    let Some(geometry) = geometry.filter(PageGeometry::is_consistent) else {
        return DestinationView::PageOnly;
    };
    match settings {
        Some(PdfDestinationViewSettings::SpecificCoordinatesAndZoom(x, y, zoom)) => {
            let mut x_points = None;
            let mut y_points = None;
            // A user-space coordinate lands on the display axis its rotation dictates.
            for (axis, value) in [
                x.and_then(|v| geometry.x_to_display(v.value)),
                y.and_then(|v| geometry.y_to_display(v.value)),
            ]
            .into_iter()
            .flatten()
            {
                match axis {
                    DisplayAxis::X => x_points = Some(value),
                    DisplayAxis::Y => y_points = Some(value),
                }
            }
            DestinationView::CoordinatesAndZoom {
                x_points,
                y_points,
                zoom,
            }
        }
        Some(PdfDestinationViewSettings::FitPageToWindow) => DestinationView::FitPage,
        // "Fit width, top at user y" becomes "fit height, left at display x" on a 90/270 page.
        Some(PdfDestinationViewSettings::FitPageHorizontallyToWindow(y)) => match y {
            Some(v) => geometry
                .y_to_display(v.value)
                .map_or(DestinationView::PageOnly, fit_at),
            None => fit_without_coordinate(geometry, true),
        },
        Some(PdfDestinationViewSettings::FitPageVerticallyToWindow(x)) => match x {
            Some(v) => geometry
                .x_to_display(v.value)
                .map_or(DestinationView::PageOnly, fit_at),
            None => fit_without_coordinate(geometry, false),
        },
        Some(PdfDestinationViewSettings::FitPageToRectangle(rect)) => geometry
            .rect_to_display(UserRect {
                left: rect.left().value,
                bottom: rect.bottom().value,
                right: rect.right().value,
                top: rect.top().value,
            })
            .map_or(DestinationView::PageOnly, DestinationView::FitRectangle),
        Some(_) | None => DestinationView::PageOnly,
    }
}

/// Fit view positioned by a converted coordinate: a value on the display y axis anchors a
/// horizontal fit, on the x axis a vertical fit.
fn fit_at((axis, value): (DisplayAxis, f32)) -> DestinationView {
    match axis {
        DisplayAxis::X => DestinationView::FitVertical {
            x_points: Some(value),
        },
        DisplayAxis::Y => DestinationView::FitHorizontal {
            y_points: Some(value),
        },
    }
}

/// A fit-width (`horizontal`) or fit-height request without a coordinate: the fit direction
/// follows the displayed orientation.
fn fit_without_coordinate(geometry: PageGeometry, horizontal: bool) -> DestinationView {
    let quarter_turn = matches!(
        geometry.rotation,
        crate::user_space::Rotation::R90 | crate::user_space::Rotation::R270
    );
    if horizontal != quarter_turn {
        DestinationView::FitHorizontal { y_points: None }
    } else {
        DestinationView::FitVertical { x_points: None }
    }
}

fn outline_path_id(path: &[usize]) -> String {
    path.iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

fn truncate_utf8_boundary(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut out = String::new();
    for ch in value.chars() {
        if out.len() + ch.len_utf8() > max_bytes {
            break;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests;
