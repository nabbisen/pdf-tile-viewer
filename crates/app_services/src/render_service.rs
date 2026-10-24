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
//! Memory policy (RFC 007 §10, RFC 035 § 5.3):
//! - Default budget: 256 MB (minimum 64 MB); never exceeded by eviction policy choices.
//! - Eviction order: entries of a generation other than the active one first, then the
//!   least recently used entry. A cache hit refreshes recency. Entries of a closed
//!   session are removed on close and later inserts for it are refused.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use domain::document::{DocumentGeneration, DocumentId};
use domain::render::{RenderCacheKey, RenderError, RenderPageRequest, RenderedImagePayload};
use pdf_engine::worker::EngineHandle;

use crate::closed_sessions::ClosedSessions;

pub const DEFAULT_CACHE_BUDGET_BYTES: usize = 256 * 1024 * 1024; // 256 MB
pub const MINIMUM_CACHE_BUDGET_BYTES: usize = 64 * 1024 * 1024; // 64 MB

// ---------------------------------------------------------------------------
// RenderCache
// ---------------------------------------------------------------------------

struct CacheEntry {
    bytes: Arc<[u8]>,
    /// Recency stamp from the cache's counter; refreshed on insert and on every hit.
    last_used_seq: u64,
}

/// In-memory LRU render cache (RFC 007 §6, RFC 035 § 5.3).
///
/// Eviction priority when the byte budget would be exceeded:
/// 1. Entries whose generation differs from the active generation (when one is set).
/// 2. Among those candidates (or among all entries if there is none), the least
///    recently used one; `get` and `insert` both count as use.
pub struct RenderCache {
    entries: HashMap<RenderCacheKey, CacheEntry>,
    seq: u64,
    active_generation: Option<DocumentGeneration>,
    closed_sessions: ClosedSessions,
    pub used_bytes: usize,
    pub budget_bytes: usize,
}

impl RenderCache {
    pub fn new(budget_bytes: usize) -> Self {
        RenderCache {
            entries: HashMap::new(),
            seq: 0,
            active_generation: None,
            closed_sessions: ClosedSessions::new(),
            used_bytes: 0,
            budget_bytes,
        }
    }

    /// Sets the generation of the document that is open now; entries of any other
    /// generation become the first eviction victims.
    pub fn set_active_generation(&mut self, generation: Option<DocumentGeneration>) {
        self.active_generation = generation;
    }

    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Look up a cached render; a hit refreshes the entry's recency. `None` on miss.
    pub fn get(&mut self, key: &RenderCacheKey) -> Option<Arc<[u8]>> {
        let seq = self.next_seq();
        self.entries.get_mut(key).map(|e| {
            e.last_used_seq = seq;
            e.bytes.clone()
        })
    }

    /// Insert a rendered page; evicts entries if over budget. Refuses keys of a closed
    /// session and keys that are already cached.
    pub fn insert(&mut self, key: RenderCacheKey, bytes: Arc<[u8]>) {
        if self.entries.contains_key(&key)
            || self.is_session_closed(key.document_id, key.document_generation)
        {
            return;
        }
        let byte_len = bytes.len();
        self.evict_to_fit(byte_len);
        let last_used_seq = self.next_seq();
        self.entries.insert(
            key,
            CacheEntry {
                bytes,
                last_used_seq,
            },
        );
        self.used_bytes += byte_len;
    }

    /// Evict entries until `budget_bytes - used_bytes >= need_bytes`, or the
    /// cache is empty.
    fn evict_to_fit(&mut self, need_bytes: usize) {
        while self.used_bytes + need_bytes > self.budget_bytes && !self.entries.is_empty() {
            let active = self.active_generation;
            // `false` sorts before `true`: other-generation entries first, then LRU.
            let victim = self
                .entries
                .iter()
                .min_by_key(|(key, e)| {
                    let current = active.is_none_or(|g| key.document_generation == g);
                    (current, e.last_used_seq)
                })
                .map(|(k, _)| *k);
            let Some(k) = victim else { break };
            if let Some(e) = self.entries.remove(&k) {
                self.used_bytes = self.used_bytes.saturating_sub(e.bytes.len());
            }
        }
    }

    /// Evicts every entry of a closed session and refuses later inserts for it (RFC 035
    /// § 5.2): a render that completes after the close must not repopulate the cache.
    pub fn evict_session(&mut self, document_id: DocumentId, generation: DocumentGeneration) {
        self.entries.retain(|key, e| {
            if key.document_id == document_id && key.document_generation == generation {
                self.used_bytes = self.used_bytes.saturating_sub(e.bytes.len());
                false
            } else {
                true
            }
        });
        self.closed_sessions.mark_closed(document_id, generation);
    }

    pub fn is_session_closed(
        &self,
        document_id: DocumentId,
        generation: DocumentGeneration,
    ) -> bool {
        self.closed_sessions.is_closed(document_id, generation)
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
            let mut cache = self.cache.lock().unwrap();
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

    /// Sets the generation of the open document (see [`RenderCache::set_active_generation`]).
    pub fn set_active_generation(&self, generation: Option<DocumentGeneration>) {
        self.cache.lock().unwrap().set_active_generation(generation);
    }

    pub fn evict_session(&self, document_id: DocumentId, generation: DocumentGeneration) {
        let mut cache = self.cache.lock().unwrap();
        cache.evict_session(document_id, generation);
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
