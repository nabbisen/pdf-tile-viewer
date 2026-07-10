use std::time::SystemTime;

use domain::document::{
    DocumentGeneration, DocumentId, DocumentMetadata, DocumentSession, DocumentSource,
    DocumentState, FileFingerprint,
};

use crate::history_service::SessionHistory;

fn session(id: u64, path: &str, name: &str) -> DocumentSession {
    DocumentSession {
        id: DocumentId(id),
        source: DocumentSource::LocalFile {
            path: path.into(),
            fingerprint: FileFingerprint {
                size_bytes: 123,
                modified_at: None,
            },
        },
        display_name: name.to_string(),
        metadata: DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            creator: None,
            producer: None,
            page_count: 3,
            encrypted: false,
        },
        pages: Vec::new(),
        opened_at: SystemTime::now(),
        generation: DocumentGeneration(id),
        state: DocumentState::Ready,
    }
}

#[test]
fn reopening_same_file_dedupes_to_top() {
    let mut history = SessionHistory::default();
    history.record_opened(&session(1, "/tmp/a.pdf", "a.pdf"));
    history.record_opened(&session(2, "/tmp/b.pdf", "b.pdf"));
    history.record_opened(&session(3, "/tmp/a.pdf", "a.pdf"));

    let names: Vec<_> = history
        .entries()
        .iter()
        .map(|e| e.display_name.as_str())
        .collect();
    assert_eq!(names, vec!["a.pdf", "b.pdf"]);
    // The surviving a.pdf entry is the most recent open.
    assert_eq!(history.entries()[0].document_id, DocumentId(3));
}

#[test]
fn history_is_capped_at_max_entries() {
    let mut history = SessionHistory::new(2);
    history.record_opened(&session(1, "/tmp/a.pdf", "a.pdf"));
    history.record_opened(&session(2, "/tmp/b.pdf", "b.pdf"));
    history.record_opened(&session(3, "/tmp/c.pdf", "c.pdf"));
    let names: Vec<_> = history
        .entries()
        .iter()
        .map(|e| e.display_name.as_str())
        .collect();
    assert_eq!(names, vec!["c.pdf", "b.pdf"]);
}

#[test]
fn clear_empties_history() {
    let mut history = SessionHistory::default();
    history.record_opened(&session(1, "/tmp/a.pdf", "a.pdf"));
    history.clear();
    assert!(history.entries().is_empty());
}
