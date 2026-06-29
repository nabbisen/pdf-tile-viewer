//! Session history for the dashboard (RFC 009 §6).
//!
//! Privacy-respecting by default (RFC 016 §7): history lives in memory for
//! the current session only. Persistence is opt-in via
//! `PrivacySettings::persist_recent_files` and is intentionally NOT
//! implemented in the vertical-slice milestones.

use std::path::PathBuf;
use std::time::SystemTime;

use domain::document::{DocumentId, DocumentSession, DocumentSource};

pub const DEFAULT_MAX_ENTRIES: usize = 20;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub document_id: DocumentId,
    pub display_name: String,
    /// Full path; shown only where privacy settings allow (RFC 016 §7).
    pub path: Option<PathBuf>,
    pub page_count: usize,
    pub last_opened_at: SystemTime,
}

#[derive(Clone, Debug)]
pub struct SessionHistory {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl Default for SessionHistory {
    fn default() -> Self {
        SessionHistory::new(DEFAULT_MAX_ENTRIES)
    }
}

impl SessionHistory {
    pub fn new(max_entries: usize) -> Self {
        SessionHistory {
            entries: Vec::new(),
            max_entries: max_entries.max(1),
        }
    }

    /// Most recent first.
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Record an opened document. An entry for the same file (by path,
    /// falling back to display name) is deduplicated and moved to the top
    /// (RFC 009 §6 "re-open moves the entry up").
    pub fn record_opened(&mut self, session: &DocumentSession) {
        let path = match &session.source {
            DocumentSource::LocalFile { path, .. } => Some(path.clone()),
        };
        let entry = HistoryEntry {
            document_id: session.id,
            display_name: session.display_name.clone(),
            path,
            page_count: session.pages.len(),
            last_opened_at: SystemTime::now(),
        };
        self.entries.retain(|existing| !same_file(existing, &entry));
        self.entries.insert(0, entry);
        self.entries.truncate(self.max_entries);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

fn same_file(a: &HistoryEntry, b: &HistoryEntry) -> bool {
    match (&a.path, &b.path) {
        (Some(pa), Some(pb)) => pa == pb,
        _ => a.display_name == b.display_name,
    }
}
