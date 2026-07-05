//! Text-layer cache and async extraction service (RFC 023).
//!
//! The cache is memory-only, bounded, and keyed by document, generation, and
//! page. It stores domain-owned text-layer data only; extracted text must not
//! be persisted in settings, history, diagnostics, or logs.

use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::{Arc, Mutex};

use domain::document::{DocumentGeneration, DocumentId};
use domain::text::{
    PageTextLayer, TextLayerCacheKey, TextLayerError, TextLayerRequest, TextLayerSegment,
};
use pdf_engine::worker::EngineHandle;

pub const DEFAULT_TEXT_LAYER_CACHE_BUDGET_BYTES: usize = 8 * 1024 * 1024; // 8 MB
pub const MINIMUM_TEXT_LAYER_CACHE_BUDGET_BYTES: usize = 512 * 1024; // 512 KB

struct CacheEntry {
    layer: Arc<PageTextLayer>,
    session_key: TextLayerSessionKey,
    estimated_bytes: usize,
    /// Monotonically increasing insert order; used for FIFO eviction.
    insert_seq: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct TextLayerSessionKey {
    document_id: DocumentId,
    generation: DocumentGeneration,
}

/// Small in-memory text-layer cache for the active document session.
pub struct TextLayerCache {
    entries: HashMap<TextLayerCacheKey, CacheEntry>,
    suppressed_sessions: HashSet<TextLayerSessionKey>,
    insert_seq: u64,
    pub used_bytes: usize,
    pub budget_bytes: usize,
}

impl TextLayerCache {
    pub fn new(budget_bytes: usize) -> Self {
        TextLayerCache {
            entries: HashMap::new(),
            suppressed_sessions: HashSet::new(),
            insert_seq: 0,
            used_bytes: 0,
            budget_bytes,
        }
    }

    pub fn get(&self, key: &TextLayerCacheKey) -> Option<Arc<PageTextLayer>> {
        self.entries.get(key).map(|entry| entry.layer.clone())
    }

    pub fn insert(&mut self, key: TextLayerCacheKey, layer: Arc<PageTextLayer>) -> bool {
        if self.entries.contains_key(&key) || self.is_session_suppressed(session_key(key)) {
            return false;
        }

        let estimated_bytes = estimate_layer_bytes(&layer);
        if estimated_bytes > self.budget_bytes {
            return false;
        }

        self.evict_to_fit(estimated_bytes);
        self.insert_seq += 1;
        self.entries.insert(
            key,
            CacheEntry {
                layer,
                session_key: session_key(key),
                estimated_bytes,
                insert_seq: self.insert_seq,
            },
        );
        self.used_bytes += estimated_bytes;
        true
    }

    pub fn evict_session(&mut self, document_id: DocumentId, generation: DocumentGeneration) {
        let session_key = TextLayerSessionKey {
            document_id,
            generation,
        };
        self.entries.retain(|_, entry| {
            if entry.session_key == session_key {
                self.used_bytes = self.used_bytes.saturating_sub(entry.estimated_bytes);
                false
            } else {
                true
            }
        });
        self.suppressed_sessions.insert(session_key);
    }

    fn is_session_suppressed(&self, session_key: TextLayerSessionKey) -> bool {
        self.suppressed_sessions.contains(&session_key)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn evict_to_fit(&mut self, need_bytes: usize) {
        while self.used_bytes + need_bytes > self.budget_bytes && !self.entries.is_empty() {
            let victim = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.insert_seq)
                .map(|(key, _)| *key);

            if let Some(key) = victim {
                if let Some(entry) = self.entries.remove(&key) {
                    self.used_bytes = self.used_bytes.saturating_sub(entry.estimated_bytes);
                }
            } else {
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct TextLayerService {
    pub engine: EngineHandle,
    cache: Arc<Mutex<TextLayerCache>>,
}

impl TextLayerService {
    pub fn new(engine: EngineHandle, budget_bytes: usize) -> Self {
        TextLayerService {
            engine,
            cache: Arc::new(Mutex::new(TextLayerCache::new(
                budget_bytes.max(MINIMUM_TEXT_LAYER_CACHE_BUDGET_BYTES),
            ))),
        }
    }

    pub async fn get_or_extract(
        &self,
        request: TextLayerRequest,
    ) -> Result<Arc<PageTextLayer>, TextLayerError> {
        let key = request.cache_key();

        {
            let cache = self.cache.lock().unwrap();
            if let Some(layer) = cache.get(&key) {
                return Ok(layer);
            }
            if cache.is_session_suppressed(session_key(key)) {
                return Err(TextLayerError::StaleResult);
            }
        }

        let layer = self
            .engine
            .extract_page_text_layer(request)
            .await
            .map_err(|_| TextLayerError::EngineUnavailable)??;

        if !layer_matches_request(&layer, &request) {
            return Err(TextLayerError::StaleResult);
        }

        let layer = Arc::new(layer);
        {
            let mut cache = self.cache.lock().unwrap();
            if cache.is_session_suppressed(session_key(key)) {
                return Err(TextLayerError::StaleResult);
            }
            cache.insert(key, layer.clone());
        }

        Ok(layer)
    }

    pub fn evict_session(&self, document_id: DocumentId, generation: DocumentGeneration) {
        let mut cache = self.cache.lock().unwrap();
        cache.evict_session(document_id, generation);
    }

    pub fn cache_used_bytes(&self) -> usize {
        self.cache.lock().unwrap().used_bytes
    }

    pub fn cache_entry_count(&self) -> usize {
        self.cache.lock().unwrap().entry_count()
    }
}

pub fn layer_matches_request(layer: &PageTextLayer, request: &TextLayerRequest) -> bool {
    layer.document_id == request.document_id
        && layer.generation == request.generation
        && layer.page_index == request.page_index
}

fn estimate_layer_bytes(layer: &PageTextLayer) -> usize {
    mem::size_of::<PageTextLayer>()
        + layer.segments.len() * mem::size_of::<TextLayerSegment>()
        + layer
            .segments
            .iter()
            .map(|segment| segment.text.len())
            .sum::<usize>()
}

fn session_key(key: TextLayerCacheKey) -> TextLayerSessionKey {
    TextLayerSessionKey {
        document_id: key.document_id,
        generation: key.generation,
    }
}

#[cfg(test)]
mod tests;
