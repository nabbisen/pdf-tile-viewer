use std::sync::Arc;

use domain::document::{DocumentGeneration, DocumentId};
use domain::navigation::{
    DocumentOutline, DocumentOutlineRequest, NavigationLimitStatus, NavigationResourceLimits,
};

use crate::navigation_service::{
    DocumentOutlineCacheKey, NavigationCache, default_outline_request, outline_matches_request,
};

fn key(document_id: u64, generation: u64) -> DocumentOutlineCacheKey {
    DocumentOutlineCacheKey {
        document_id: DocumentId(document_id),
        generation: DocumentGeneration(generation),
    }
}

fn outline(document_id: u64, generation: u64) -> Arc<DocumentOutline> {
    Arc::new(DocumentOutline {
        document_id: DocumentId(document_id),
        generation: DocumentGeneration(generation),
        roots: Vec::new(),
        limit_status: NavigationLimitStatus::default(),
    })
}

#[test]
fn cache_miss_returns_none() {
    let cache = NavigationCache::new();
    assert!(cache.get_outline(&key(1, 1)).is_none());
}

#[test]
fn insert_then_get_returns_outline() {
    let mut cache = NavigationCache::new();
    let outline = outline(1, 2);

    assert!(cache.insert_outline(key(1, 2), outline.clone()));

    let got = cache.get_outline(&key(1, 2)).unwrap();
    assert_eq!(got.document_id, DocumentId(1));
    assert_eq!(got.generation, DocumentGeneration(2));
    assert_eq!(cache.outline_count(), 1);
}

#[test]
fn duplicate_insert_is_ignored() {
    let mut cache = NavigationCache::new();
    let first = outline(1, 2);
    let second = outline(1, 2);

    assert!(cache.insert_outline(key(1, 2), first.clone()));
    assert!(!cache.insert_outline(key(1, 2), second));

    assert!(Arc::ptr_eq(&cache.get_outline(&key(1, 2)).unwrap(), &first));
}

#[test]
fn evict_session_removes_outline_and_suppresses_late_insert() {
    let mut cache = NavigationCache::new();
    let outline = outline(1, 2);

    assert!(cache.insert_outline(key(1, 2), outline.clone()));
    cache.evict_session(DocumentId(1), DocumentGeneration(2));

    assert!(cache.get_outline(&key(1, 2)).is_none());
    assert!(cache.is_session_suppressed(key(1, 2)));
    assert!(!cache.insert_outline(key(1, 2), outline));
}

#[test]
fn evict_session_does_not_suppress_same_generation_for_other_document() {
    let mut cache = NavigationCache::new();
    let first = outline(1, 2);
    let second = outline(9, 2);

    assert!(cache.insert_outline(key(1, 2), first.clone()));
    cache.evict_session(DocumentId(1), DocumentGeneration(2));

    assert!(!cache.insert_outline(key(1, 2), first));
    assert!(cache.insert_outline(key(9, 2), second));
}

#[test]
fn default_outline_request_uses_rfc_026_limits() {
    let request = default_outline_request(DocumentId(1), DocumentGeneration(2));

    assert_eq!(request.document_id, DocumentId(1));
    assert_eq!(request.generation, DocumentGeneration(2));
    assert_eq!(request.limits, NavigationResourceLimits::default());
}

#[test]
fn outline_match_checks_document_generation() {
    let request = DocumentOutlineRequest {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        limits: NavigationResourceLimits::default(),
    };
    let matching = DocumentOutline {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        roots: Vec::new(),
        limit_status: NavigationLimitStatus::default(),
    };
    let stale = DocumentOutline {
        generation: DocumentGeneration(3),
        ..matching.clone()
    };

    assert!(outline_matches_request(&matching, &request));
    assert!(!outline_matches_request(&stale, &request));
}
