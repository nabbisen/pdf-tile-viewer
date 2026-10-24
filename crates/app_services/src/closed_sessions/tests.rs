use domain::document::{DocumentGeneration, DocumentId};

use crate::closed_sessions::{CLOSED_SESSIONS_CAPACITY, ClosedSessions};

fn id(n: u64) -> DocumentId {
    DocumentId(n)
}

fn generation(n: u64) -> DocumentGeneration {
    DocumentGeneration(n)
}

#[test]
fn nothing_is_closed_initially() {
    let closed = ClosedSessions::new();
    assert!(closed.is_empty());
    assert!(!closed.is_closed(id(1), generation(1)));
}

#[test]
fn is_closed_matches_the_exact_document_and_generation() {
    let mut closed = ClosedSessions::new();
    closed.mark_closed(id(1), generation(2));
    assert!(closed.is_closed(id(1), generation(2)));
    assert!(!closed.is_closed(id(1), generation(3)), "other generation");
    assert!(!closed.is_closed(id(2), generation(2)), "other document");
}

#[test]
fn marking_twice_does_not_duplicate() {
    let mut closed = ClosedSessions::new();
    closed.mark_closed(id(1), generation(1));
    closed.mark_closed(id(1), generation(1));
    assert_eq!(closed.len(), 1);
}

#[test]
fn capacity_is_sixteen() {
    assert_eq!(CLOSED_SESSIONS_CAPACITY, 16);
}

#[test]
fn the_seventeenth_insert_drops_the_oldest() {
    let mut closed = ClosedSessions::new();
    for n in 1..=CLOSED_SESSIONS_CAPACITY as u64 {
        closed.mark_closed(id(n), generation(n));
    }
    assert_eq!(closed.len(), CLOSED_SESSIONS_CAPACITY);
    assert!(closed.is_closed(id(1), generation(1)));

    closed.mark_closed(id(17), generation(17));
    assert_eq!(closed.len(), CLOSED_SESSIONS_CAPACITY, "stays bounded");
    assert!(!closed.is_closed(id(1), generation(1)), "oldest dropped");
    assert!(closed.is_closed(id(2), generation(2)), "second oldest kept");
    assert!(closed.is_closed(id(17), generation(17)));
}

#[test]
fn remarking_a_known_session_does_not_refresh_its_position() {
    // FIFO, not LRU: the window is about how long ago a session was closed.
    let mut closed = ClosedSessions::new();
    for n in 1..=CLOSED_SESSIONS_CAPACITY as u64 {
        closed.mark_closed(id(n), generation(n));
    }
    closed.mark_closed(id(1), generation(1));
    closed.mark_closed(id(17), generation(17));
    assert!(!closed.is_closed(id(1), generation(1)));
}
