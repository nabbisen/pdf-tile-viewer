//! Bounded record of closed document sessions (RFC 035 § 5.2).
//!
//! Every cache keeps one of these so that a late async result for a session that has
//! been closed is refused. Only one document is open at a time and document ids are
//! never reused, so a small FIFO window is sufficient; the record stays bounded.

use std::collections::VecDeque;

use domain::document::{DocumentGeneration, DocumentId};

/// Number of closed sessions remembered; the oldest is forgotten first.
pub const CLOSED_SESSIONS_CAPACITY: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct ClosedSessions {
    sessions: VecDeque<(DocumentId, DocumentGeneration)>,
}

impl ClosedSessions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a session as closed. Re-marking a known session keeps its place (no
    /// duplicates); a new one beyond the capacity drops the oldest entry.
    pub fn mark_closed(&mut self, document_id: DocumentId, generation: DocumentGeneration) {
        if self.is_closed(document_id, generation) {
            return;
        }
        if self.sessions.len() >= CLOSED_SESSIONS_CAPACITY {
            self.sessions.pop_front();
        }
        self.sessions.push_back((document_id, generation));
    }

    /// `true` only for the exact `(document, generation)` pair that was marked closed.
    pub fn is_closed(&self, document_id: DocumentId, generation: DocumentGeneration) -> bool {
        self.sessions.contains(&(document_id, generation))
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests;
