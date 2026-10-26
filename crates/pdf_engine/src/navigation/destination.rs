//! Destination conversion for outline entries and links (RFC 034): coordinates and fit views are
//! converted to display space with the geometry of the **destination** page.

use domain::document::PageIndex;
use domain::navigation::{
    DestinationView, DisabledNavigationReason, NavigationDestination, NavigationTarget,
};
use pdfium_render::prelude::*;

use crate::user_space::{DisplayAxis, PageGeometry, UserRect};

/// Destination coordinates are converted to display space with the geometry of the
/// **destination** page (not the page holding the link); without usable geometry the
/// view degrades to `PageOnly`, the navigation itself is kept.
pub(super) fn destination_target(
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
