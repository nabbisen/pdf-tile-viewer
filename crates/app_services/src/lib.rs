//! App-level orchestration between the UI (`app`) and the lower layers
//! (`domain`, `pdf_engine`, `packaging`).
//!
//! Responsibilities (ADR §3.2, RFC 001 §6):
//! - file intake validation and document opening (RFC 002),
//! - engine bootstrap: PDFium resolution + worker spawn (RFC 003/004),
//! - settings persistence with safe fallback (RFC 008),
//! - session history, privacy-respecting by default (RFC 009, RFC 016 §7),
//! - platform integration: dialogs, reveal-in-file-manager (RFC 002).
//!
//! This crate contains no Dioxus types and no native PDFium handles.

pub mod document_service;
pub mod engine_boot;
pub mod history_service;
pub mod platform;
pub mod settings_service;

#[cfg(test)]
mod tests;
