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
    let cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
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
fn evict_generation_removes_matching_entries() {
    // RFC 007 §10: wrong-generation entries are evicted on document close.
    let mut cache = RenderCache::new(MINIMUM_CACHE_BUDGET_BYTES);
    cache.insert(key(0, 1, 100), bytes(100));
    cache.insert(key(1, 1, 100), bytes(100));
    cache.insert(key(0, 2, 100), bytes(100)); // different generation

    cache.evict_generation(DocumentGeneration(1));
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
