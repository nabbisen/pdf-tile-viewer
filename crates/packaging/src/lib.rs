//! Locating and verifying the bundled PDFium library and app directories.
//!
//! RFC 003 / RFC 016 boundary: this crate decides *where* PDFium and app
//! data may live; it never decides PDF document behavior.

pub mod app_dirs;
pub mod pdfium_bundle;

#[cfg(test)]
mod tests;
