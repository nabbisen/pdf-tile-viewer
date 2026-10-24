use std::sync::Arc;

use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::render::{RenderBackground, RenderCacheKey, RenderFlags, ScaleBucket};

use crate::render_service::{MINIMUM_CACHE_BUDGET_BYTES, RenderCache};

fn key(page: usize, document_gen: u64, scale_pct: u16) -> RenderCacheKey {
    RenderCacheKey {
        document_id: DocumentId(1),
        document_generation: DocumentGeneration(document_gen),
        page_index: PageIndex(page),
        scale_bucket: ScaleBucket(scale_pct),
        render_flags: RenderFlags {
            background: RenderBackground::White,
        },
    }
}

fn bytes(n: usize) -> Arc<[u8]> {
    vec![0u8; n].into()
}

#[test]
fn cache_miss_returns_none() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    assert!(cache.get(&key(0, 1, 100)).is_none());
}

#[test]
fn insert_then_get_returns_bytes() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    let b = bytes(1024);
    cache.insert(key(0, 1, 100), b.clone());
    let got = cache.get(&key(0, 1, 100)).unwrap();
    assert_eq!(got.len(), 1024);
}

#[test]
fn duplicate_insert_is_ignored() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(0, 1, 100), bytes(200)); // should not overwrite
    assert_eq!(cache.used_bytes, 100);
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn eviction_occurs_when_over_budget() {
    // Budget of 300 bytes; inserting 3×100-byte pages causes eviction of the
    // first when the fourth is inserted.
    let mut cache = RenderCache::new(300);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    cache.insert(key(2, 1, 100), bytes(100));
    assert_eq!(cache.used_bytes, 300);
    assert_eq!(cache.entry_count(), 3);

    cache.insert(key(3, 1, 100), bytes(100)); // triggers eviction of page 0
    assert!(cache.used_bytes <= 300);
    assert_eq!(cache.entry_count(), 3);
    // Oldest (page 0) should be evicted.
    assert!(cache.get(&key(0, 1, 100)).is_none());
    assert!(cache.get(&key(3, 1, 100)).is_some());
}

#[test]
fn evict_session_removes_matching_entries() {
    // RFC 035: a closed session's entries are evicted.
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    cache.insert(key(0, 2, 100), bytes(100)); // different generation

    cache.evict_session(DocumentId(1), DocumentGeneration(1));
    assert_eq!(cache.entry_count(), 1, "only gen-2 entry should remain");
    assert!(cache.get(&key(0, 1, 100)).is_none());
    assert!(cache.get(&key(0, 2, 100)).is_some());
}

#[test]
fn different_scale_buckets_are_separate_cache_entries() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(0, 1, 150), bytes(150));
    assert_eq!(cache.entry_count(), 2);
}

#[test]
fn evict_session_refuses_late_inserts_for_that_session_only() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.evict_session(DocumentId(1), DocumentGeneration(1));
    assert!(cache.is_session_closed(DocumentId(1), DocumentGeneration(1)));

    cache.insert(key(1, 1, 100), bytes(100)); // late render of the closed session
    assert_eq!(cache.entry_count(), 0);
    assert_eq!(cache.used_bytes, 0);

    cache.insert(key(0, 2, 100), bytes(100)); // another generation is unaffected
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn evict_session_does_not_affect_the_same_generation_of_another_document() {
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    let mut other = key(0, 1, 100);
    other.document_id = DocumentId(2);
    cache.insert(other, bytes(100));
    cache.evict_session(DocumentId(1), DocumentGeneration(1));

    assert!(cache.get(&other).is_some());
    cache.insert(other_key_page(1), bytes(100));
    assert_eq!(cache.entry_count(), 2);
}

fn other_key_page(page: usize) -> RenderCacheKey {
    let mut k = key(page, 1, 100);
    k.document_id = DocumentId(2);
    k
}

#[test]
fn a_get_protects_an_entry_from_eviction_lru() {
    // Budget for three 100-byte pages. Page 0 is the oldest insert, but it is read
    // before page 3 arrives, so page 1 (the least recently used) is the victim.
    let mut cache = RenderCache::new(300);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    cache.insert(key(2, 1, 100), bytes(100));
    assert!(cache.get(&key(0, 1, 100)).is_some());

    cache.insert(key(3, 1, 100), bytes(100));

    assert!(cache.get(&key(1, 1, 100)).is_none(), "LRU victim");
    assert!(cache.get(&key(0, 1, 100)).is_some(), "read entry survives");
    assert!(cache.get(&key(2, 1, 100)).is_some());
    assert!(cache.get(&key(3, 1, 100)).is_some());
}

#[test]
fn other_generation_entries_are_evicted_before_recently_used_current_ones() {
    let mut cache = RenderCache::new(300);
    cache.set_active_generation(Some(DocumentGeneration(2)));
    cache.insert(key(0, 1, 100), bytes(100)); // old generation, inserted first
    cache.insert(key(1, 2, 100), bytes(100));
    cache.insert(key(2, 2, 100), bytes(100));
    // Touch the old-generation entry: recency must not save it.
    assert!(cache.get(&key(0, 1, 100)).is_some());

    cache.insert(key(3, 2, 100), bytes(100));

    assert!(
        cache.get(&key(0, 1, 100)).is_none(),
        "wrong generation goes first"
    );
    assert_eq!(cache.entry_count(), 3);
    assert!(cache.get(&key(1, 2, 100)).is_some());
}

#[test]
fn without_an_active_generation_eviction_is_plain_lru() {
    let mut cache = RenderCache::new(200);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(0, 2, 100), bytes(100));
    cache.insert(key(1, 2, 100), bytes(100));
    assert!(
        cache.get(&key(0, 1, 100)).is_none(),
        "oldest insert evicted"
    );
    assert!(cache.get(&key(0, 2, 100)).is_some());
}

#[test]
fn eviction_removes_as_many_entries_as_the_new_page_needs() {
    let mut cache = RenderCache::new(300);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    cache.insert(key(2, 1, 100), bytes(100));

    cache.insert(key(3, 1, 100), bytes(250));

    assert!(
        cache.used_bytes <= 300,
        "budget respected: {}",
        cache.used_bytes
    );
    assert!(cache.get(&key(3, 1, 100)).is_some());
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn used_bytes_tracks_inserts_and_evictions() {
    let mut cache = RenderCache::new(250);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    assert_eq!(cache.used_bytes, 200);
    cache.insert(key(2, 1, 100), bytes(100));
    assert_eq!(cache.used_bytes, 200);
    cache.evict_session(DocumentId(1), DocumentGeneration(1));
    assert_eq!(cache.used_bytes, 0);
}
