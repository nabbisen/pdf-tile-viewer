//! Tests validate RFC behaviour, not just code paths:
//! - RFC 002 §7 intake validation ordering and rejections,
//! - RFC 008 §11 settings fallback/backup policy,
//! - RFC 009 §6 history dedupe-to-top,
//! - RFC 003/016 production loader policy via `engine_boot`.

mod document;
mod engine_boot;
mod history;
mod render_service;
mod settings;
