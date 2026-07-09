//! PDFium binding, document sessions, page rendering, and text search.
//!
//! Architecture rules (ADR §3.4, RFC 004 §8, Appendix A §8):
//! - PDFium is a serialized native boundary: all engine calls run on one
//!   dedicated worker thread owned by [`worker::EngineHandle`].
//! - Native PDFium document/page handles never leave this crate; the public
//!   surface speaks only `domain` types.
//! - Search never mutates PDF bytes (RFC 010 §11).

pub mod engine;
pub mod loader;
pub mod navigation;
pub mod render;
pub mod search;
pub mod text_layer;
pub mod worker;
