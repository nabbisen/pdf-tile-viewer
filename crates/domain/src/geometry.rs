//! Page geometry shared by search, text layer, links and layout (RFC 034).
//!
//! **Display space** is the only coordinate system above the PDF engine: the page
//! after its `/Rotate` and crop box have been applied, origin at the **top-left**,
//! y pointing **down**, unit PDF points (1/72 inch). The engine converts every
//! PDF user-space value into display space before it crosses its boundary; no
//! other crate contains rotation or crop logic.

/// A rectangle in **display space**: origin top-left of the displayed page, y down,
/// unit PDF points. `x`/`y` is the top-left corner; `width`/`height` are positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PageRect {
    /// Right edge (`x + width`).
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    /// Bottom edge in display space (`y + height`).
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[cfg(test)]
mod tests;
