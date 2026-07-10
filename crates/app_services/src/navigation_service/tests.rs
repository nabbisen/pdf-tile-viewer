use std::sync::Arc;

use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::navigation::{
    DocumentOutline, DocumentOutlineRequest, NavigationLimitStatus, NavigationResourceLimits,
    PageLinkSet, PageLinksRequest,
};

use crate::navigation_service::{
    DocumentOutlineCacheKey, NavigationCache, default_outline_request, default_page_links_request,
    outline_matches_request, page_links_match_request,
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

fn page_links(document_id: u64, generation: u64, page_index: usize) -> Arc<PageLinkSet> {
    Arc::new(PageLinkSet {
        document_id: DocumentId(document_id),
        generation: DocumentGeneration(generation),
        page_index: PageIndex(page_index),
        links: Vec::new(),
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
fn insert_then_get_returns_page_links() {
    let mut cache = NavigationCache::new();
    let links = page_links(1, 2, 3);

    assert!(cache.insert_page_links(
        links.clone(),
        NavigationResourceLimits::default().max_cached_link_pages
    ));

    let got = cache
        .get_page_links(
            &PageLinksRequest {
                document_id: DocumentId(1),
                generation: DocumentGeneration(2),
                page_index: PageIndex(3),
                limits: NavigationResourceLimits::default(),
            }
            .cache_key(),
        )
        .unwrap();
    assert_eq!(got.document_id, DocumentId(1));
    assert_eq!(got.generation, DocumentGeneration(2));
    assert_eq!(got.page_index, PageIndex(3));
    assert_eq!(cache.page_link_page_count(), 1);
}

#[test]
fn duplicate_page_link_insert_is_ignored() {
    let mut cache = NavigationCache::new();
    let first = page_links(1, 2, 3);
    let second = page_links(1, 2, 3);

    assert!(cache.insert_page_links(
        first.clone(),
        NavigationResourceLimits::default().max_cached_link_pages
    ));
    assert!(!cache.insert_page_links(
        second,
        NavigationResourceLimits::default().max_cached_link_pages
    ));

    let got = cache
        .get_page_links(
            &PageLinksRequest {
                document_id: DocumentId(1),
                generation: DocumentGeneration(2),
                page_index: PageIndex(3),
                limits: NavigationResourceLimits::default(),
            }
            .cache_key(),
        )
        .unwrap();
    assert!(Arc::ptr_eq(&got, &first));
}

#[test]
fn evict_session_removes_page_links_and_suppresses_late_insert() {
    let mut cache = NavigationCache::new();
    let links = page_links(1, 2, 3);
    let request = PageLinksRequest {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(3),
        limits: NavigationResourceLimits::default(),
    };

    assert!(cache.insert_page_links(
        links.clone(),
        NavigationResourceLimits::default().max_cached_link_pages
    ));
    cache.evict_session(DocumentId(1), DocumentGeneration(2));

    assert!(cache.get_page_links(&request.cache_key()).is_none());
    assert!(cache.is_session_suppressed(key(1, 2)));
    assert!(!cache.insert_page_links(
        links,
        NavigationResourceLimits::default().max_cached_link_pages
    ));
}

#[test]
fn page_link_cache_evicts_oldest_page_when_over_page_cap() {
    let mut cache = NavigationCache::new();

    assert!(cache.insert_page_links(page_links(1, 2, 0), 2));
    assert!(cache.insert_page_links(page_links(1, 2, 1), 2));
    assert!(cache.insert_page_links(page_links(1, 2, 2), 2));

    let first_request = PageLinksRequest {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(0),
        limits: NavigationResourceLimits::default(),
    };
    let second_request = PageLinksRequest {
        page_index: PageIndex(1),
        ..first_request
    };
    let third_request = PageLinksRequest {
        page_index: PageIndex(2),
        ..first_request
    };

    assert!(cache.get_page_links(&first_request.cache_key()).is_none());
    assert!(cache.get_page_links(&second_request.cache_key()).is_some());
    assert!(cache.get_page_links(&third_request.cache_key()).is_some());
    assert_eq!(cache.page_link_page_count(), 2);
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
fn default_page_links_request_uses_rfc_026_limits() {
    let request = default_page_links_request(DocumentId(1), DocumentGeneration(2), PageIndex(3));

    assert_eq!(request.document_id, DocumentId(1));
    assert_eq!(request.generation, DocumentGeneration(2));
    assert_eq!(request.page_index, PageIndex(3));
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

#[test]
fn page_links_match_checks_document_generation_and_page() {
    let request = PageLinksRequest {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(3),
        limits: NavigationResourceLimits::default(),
    };
    let matching = PageLinkSet {
        document_id: DocumentId(1),
        generation: DocumentGeneration(2),
        page_index: PageIndex(3),
        links: Vec::new(),
        limit_status: NavigationLimitStatus::default(),
    };
    let stale_page = PageLinkSet {
        page_index: PageIndex(4),
        ..matching.clone()
    };

    assert!(page_links_match_request(&matching, &request));
    assert!(!page_links_match_request(&stale_page, &request));
}
