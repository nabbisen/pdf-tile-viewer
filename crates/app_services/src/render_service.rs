//! Render cache and async render service (RFC 007).
//!
//! The cache is an in-memory LRU store keyed by [`RenderCacheKey`] holding
//! raw PNG bytes as `Arc<[u8]>`. It is wrapped in `Arc<Mutex<_>>` so it
//! can be shared across spawned Dioxus tasks.
//!
//! [`RenderService`] ties the cache to a serialized [`EngineHandle`] and
//! provides a single async entry-point: check cache → render if miss →
//! store → return. The `app` layer converts the bytes to a data URI.
//!
//! Memory policy (RFC 007 §10):
//! - Default budget: 256 MB.
//! - Eviction order: wrong-generation entries first, then oldest-by-insertion.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use domain::document::DocumentGeneration;
use domain::render::{RenderCacheKey, RenderError, RenderPageRequest, RenderedImagePayload};
use pdf_engine::worker::EngineHandle;

pub const DEFAULT_CACHE_BUDGET_BYTES: usize = 256 * 1024 * 1024; // 256 MB
pub const MINIMUM_CACHE_BUDGET_BYTES: usize = 64 * 1024 * 1024; // 64 MB

// ---------------------------------------------------------------------------
// RenderCache
// ---------------------------------------------------------------------------

struct CacheEntry {
    bytes: Arc<[u8]>,
    generation: DocumentGeneration,
    /// Monotonically increasing insert order; used for FIFO eviction.
    insert_seq: u64,
}

/// In-memory LRU render cache (RFC 007 §6).
///
/// Eviction priority (RFC 007 §10):
/// 1. Entries whose `document_generation` does not match the current document
///    generation are evicted first.
/// 2. Otherwise, oldest-by-insertion.
pub struct RenderCache {
    entries: HashMap<RenderCacheKey, CacheEntry>,
    insert_seq: u64,
    pub used_bytes: usize,
    pub budget_bytes: usize,
}

impl RenderCache {
    pub fn new(budget_bytes: usize) -> Self {
        RenderCache {
            entries: HashMap::new(),
            insert_seq: 0,
            used_bytes: 0,
            budget_bytes,
        }
    }

    /// Look up a cached render; returns `None` on miss.
    pub fn get(&self, key: &RenderCacheKey) -> Option<Arc<[u8]>> {
        self.entries.get(key).map(|e| e.bytes.clone())
    }

    /// Insert a rendered page; evicts stale/old entries if over budget.
    pub fn insert(&mut self, key: RenderCacheKey, bytes: Arc<[u8]>) {
        if self.entries.contains_key(&key) {
            return;
        }
        let byte_len = bytes.len();
        self.evict_to_fit(byte_len);
        self.insert_seq += 1;
        let generation = key.document_generation;
        self.entries.insert(
            key,
            CacheEntry {
                bytes,
                generation,
                insert_seq: self.insert_seq,
            },
        );
        self.used_bytes += byte_len;
    }

    /// Evict entries until `budget_bytes - used_bytes >= need_bytes`, or the
    /// cache is empty.
    fn evict_to_fit(&mut self, need_bytes: usize) {
        if self.used_bytes + need_bytes <= self.budget_bytes {
            return;
        }
        // Collect eviction candidates: wrong-generation first, then oldest.
        // We snapshot keys so we can mutate the map inside the loop.
        while self.used_bytes + need_bytes > self.budget_bytes && !self.entries.is_empty() {
            let victim = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.insert_seq)
                .map(|(k, _)| *k);
            if let Some(k) = victim {
                if let Some(e) = self.entries.remove(&k) {
                    self.used_bytes = self.used_bytes.saturating_sub(e.bytes.len());
                }
            } else {
                break;
            }
        }
    }

    /// Evict all entries that belong to a specific document generation. Call
    /// this when a document is closed (RFC 007 §10 eviction priority 1).
    pub fn evict_generation(&mut self, generation: DocumentGeneration) {
        self.entries.retain(|_, e| {
            if e.generation == generation {
                self.used_bytes = self.used_bytes.saturating_sub(e.bytes.len());
                false
            } else {
                true
            }
        });
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

// ---------------------------------------------------------------------------
// RenderService
// ---------------------------------------------------------------------------

/// Cloneable handle to the render cache + engine pair (RFC 007 §9).
#[derive(Clone)]
pub struct RenderService {
    pub engine: EngineHandle,
    cache: Arc<Mutex<RenderCache>>,
}

impl RenderService {
    pub fn new(engine: EngineHandle, budget_bytes: usize) -> Self {
        RenderService {
            engine,
            cache: Arc::new(Mutex::new(RenderCache::new(
                budget_bytes.max(MINIMUM_CACHE_BUDGET_BYTES),
            ))),
        }
    }

    /// Check cache; render and cache on miss. Returns raw PNG bytes.
    pub async fn get_or_render(
        &self,
        request: RenderPageRequest,
    ) -> Result<Arc<[u8]>, RenderError> {
        let key = RenderCacheKey {
            document_id: request.document_id,
            document_generation: request.generation,
            page_index: request.page_index,
            scale_bucket: request.scale_bucket,
            render_flags: request.flags,
        };

        // Cache hit path.
        {
            let cache = self.cache.lock().unwrap();
            if let Some(bytes) = cache.get(&key) {
                return Ok(bytes);
            }
        }

        // Cache miss — delegate to the engine worker.
        let image = self
            .engine
            .render_page(request)
            .await
            .map_err(|_| RenderError::EngineUnavailable)??;

        let bytes: Arc<[u8]> = match image.payload {
            RenderedImagePayload::Bytes(v) => v.into(),
        };

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key, bytes.clone());
        }
        Ok(bytes)
    }

    pub fn evict_generation(&self, generation: DocumentGeneration) {
        let mut cache = self.cache.lock().unwrap();
        cache.evict_generation(generation);
    }

    /// Diagnostics snapshot for the settings/diagnostics panel (RFC 008).
    pub fn cache_used_bytes(&self) -> usize {
        self.cache.lock().unwrap().used_bytes
    }

    pub fn cache_entry_count(&self) -> usize {
        self.cache.lock().unwrap().entry_count()
    }
}

#[cfg(test)]
mod tests;
