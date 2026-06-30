//! Pure data types and product rules for PDF Tile Viewer.
//!
//! Constraints (RFC 001 §7):
//! - No dependency on Dioxus, PDFium, or platform GUI APIs.
//! - No platform I/O. All types here are computable and testable in isolation.

pub mod document;
pub mod layout;
pub mod render;
pub mod search;
pub mod settings;
