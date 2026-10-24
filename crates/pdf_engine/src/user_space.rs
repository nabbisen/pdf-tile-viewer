//! PDF user space → display space conversion (RFC 034 § 5.3).
//!
//! **The only place with rotation and crop logic.** PDFium reports page size with
//! `/Rotate` already applied, but text, search, link and destination geometry in
//! unrotated *user space* (origin bottom-left of the page box, y up). Everything the
//! engine returns is converted here into *display space*: the page as displayed,
//! origin top-left, y down, unit PDF points. Pure arithmetic; no PDFium handles.

use domain::geometry::PageRect;

/// Maximum difference (points) between the size PDFium reports for a page and the
/// size computed from the page box and rotation before the page's geometry is
/// considered inconsistent.
pub const SIZE_TOLERANCE_POINTS: f32 = 0.5;

/// The page box in user space (PDFium: crop box ∩ media box): `(x0, y0)` is the
/// bottom-left corner, `(x1, y1)` the top-right corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

/// Page rotation (`/Rotate`), clockwise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    /// `degrees` modulo 360; anything that is not a multiple of 90 counts as `R0`
    /// (PDF only allows multiples of 90).
    pub fn from_degrees(degrees: i32) -> Self {
        match degrees.rem_euclid(360) {
            90 => Rotation::R90,
            180 => Rotation::R180,
            270 => Rotation::R270,
            _ => Rotation::R0,
        }
    }
}

/// A rectangle in user space as PDFium reports it (bottom-left origin, y up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserRect {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

/// Size of the page as displayed: width and height swap for 90° and 270°.
pub fn display_size(page_box: PageBox, rotation: Rotation) -> (f32, f32) {
    let w = page_box.x1 - page_box.x0;
    let h = page_box.y1 - page_box.y0;
    match rotation {
        Rotation::R0 | Rotation::R180 => (w, h),
        Rotation::R90 | Rotation::R270 => (h, w),
    }
}

/// Maps a user-space point to display space (origin top-left, y down).
pub fn point_to_display(page_box: PageBox, rotation: Rotation, ux: f32, uy: f32) -> (f32, f32) {
    let PageBox { x0, y0, x1, y1 } = page_box;
    match rotation {
        Rotation::R0 => (ux - x0, y1 - uy),
        Rotation::R90 => (uy - y0, ux - x0),
        Rotation::R180 => (x1 - ux, uy - y0),
        Rotation::R270 => (y1 - uy, x1 - ux),
    }
}

/// Axis of display space (a single user-space coordinate lands on one of them).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayAxis {
    X,
    Y,
}

/// Maps a single user-space `x` to the display axis it lands on and its value there. For
/// 90° and 270° pages user x runs along the displayed vertical axis.
pub fn x_to_display(page_box: PageBox, rotation: Rotation, ux: f32) -> (DisplayAxis, f32) {
    let PageBox { x0, x1, .. } = page_box;
    match rotation {
        Rotation::R0 => (DisplayAxis::X, ux - x0),
        Rotation::R90 => (DisplayAxis::Y, ux - x0),
        Rotation::R180 => (DisplayAxis::X, x1 - ux),
        Rotation::R270 => (DisplayAxis::Y, x1 - ux),
    }
}

/// Maps a single user-space `y` to the display axis it lands on and its value there.
pub fn y_to_display(page_box: PageBox, rotation: Rotation, uy: f32) -> (DisplayAxis, f32) {
    let PageBox { y0, y1, .. } = page_box;
    match rotation {
        Rotation::R0 => (DisplayAxis::Y, y1 - uy),
        Rotation::R90 => (DisplayAxis::X, uy - y0),
        Rotation::R180 => (DisplayAxis::Y, uy - y0),
        Rotation::R270 => (DisplayAxis::X, y1 - uy),
    }
}

/// Converts a user-space rectangle to a display-space [`PageRect`]: two opposite
/// corners are transformed and normalized to `(min, absolute size)`. Returns `None`
/// for non-finite values or a rectangle without positive width and height.
pub fn rect_to_display(page_box: PageBox, rotation: Rotation, rect: UserRect) -> Option<PageRect> {
    let values = [
        rect.left,
        rect.bottom,
        rect.right,
        rect.top,
        page_box.x0,
        page_box.y0,
        page_box.x1,
        page_box.y1,
    ];
    if values.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let (ax, ay) = point_to_display(page_box, rotation, rect.left, rect.bottom);
    let (bx, by) = point_to_display(page_box, rotation, rect.right, rect.top);
    let width = (bx - ax).abs();
    let height = (by - ay).abs();
    if !(width.is_finite() && height.is_finite()) || width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(PageRect {
        x: ax.min(bx),
        y: ay.min(by),
        width,
        height,
    })
}

/// Geometry of one page, computed once when the document is opened.
///
/// If the size computed from the page box and rotation disagrees with the size
/// PDFium reports (more than [`SIZE_TOLERANCE_POINTS`]), the page is flagged
/// **inconsistent**: it is still opened and rendered, but no overlay rectangle is
/// produced for it, so a misplaced overlay can never appear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageGeometry {
    pub page_box: PageBox,
    pub rotation: Rotation,
    consistent: bool,
}

impl PageGeometry {
    /// `reported_display_size` is the page size PDFium reports (rotation applied).
    pub fn new(
        page_box: PageBox,
        rotation: Rotation,
        reported_display_size: (f32, f32),
    ) -> PageGeometry {
        let (w, h) = display_size(page_box, rotation);
        let consistent = w.is_finite()
            && h.is_finite()
            && w > 0.0
            && h > 0.0
            && (w - reported_display_size.0).abs() <= SIZE_TOLERANCE_POINTS
            && (h - reported_display_size.1).abs() <= SIZE_TOLERANCE_POINTS;
        PageGeometry {
            page_box,
            rotation,
            consistent,
        }
    }

    /// Geometry for a page whose box or rotation could not be read: flagged
    /// inconsistent, so it never yields overlay geometry.
    pub fn unavailable() -> PageGeometry {
        PageGeometry {
            page_box: PageBox {
                x0: 0.0,
                y0: 0.0,
                x1: 0.0,
                y1: 0.0,
            },
            rotation: Rotation::R0,
            consistent: false,
        }
    }

    /// Whether the computed size agrees with PDFium's reported size.
    pub fn is_consistent(&self) -> bool {
        self.consistent
    }

    /// Display-space rectangle, or `None` for an inconsistent page, non-finite
    /// values or a degenerate rectangle.
    pub fn rect_to_display(&self, rect: UserRect) -> Option<PageRect> {
        if !self.consistent {
            return None;
        }
        rect_to_display(self.page_box, self.rotation, rect)
    }

    /// A single user-space `x`: display axis and value, or `None` (inconsistent page, non-finite).
    pub fn x_to_display(&self, ux: f32) -> Option<(DisplayAxis, f32)> {
        (self.consistent && ux.is_finite()).then(|| x_to_display(self.page_box, self.rotation, ux))
    }

    /// A single user-space `y`: display axis and value, or `None` (inconsistent page, non-finite).
    pub fn y_to_display(&self, uy: f32) -> Option<(DisplayAxis, f32)> {
        (self.consistent && uy.is_finite()).then(|| y_to_display(self.page_box, self.rotation, uy))
    }

    /// Display-space point, or `None` for an inconsistent page or non-finite input.
    pub fn point_to_display(&self, ux: f32, uy: f32) -> Option<(f32, f32)> {
        if !self.consistent || !ux.is_finite() || !uy.is_finite() {
            return None;
        }
        Some(point_to_display(self.page_box, self.rotation, ux, uy))
    }
}

#[cfg(test)]
mod tests;
