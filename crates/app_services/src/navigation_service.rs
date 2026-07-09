//! Navigation metadata cache and async extraction service (RFC 026).
//!
//! The cache is memory-only and keyed by document and generation. Extracted
//! outline data is document metadata, not executable behavior; external URI
//! opening remains governed by `uri_policy`.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use domain::document::{DocumentGeneration, DocumentId};
use domain::navigation::{
    DocumentOutline, DocumentOutlineRequest, NavigationError, NavigationResourceLimits,
};
use pdf_engine::worker::EngineHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DocumentOutlineCacheKey {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
}

impl From<DocumentOutlineRequest> for DocumentOutlineCacheKey {
    fn from(request: DocumentOutlineRequest) -> Self {
        Self {
            document_id: request.document_id,
            generation: request.generation,
        }
    }
}

pub struct NavigationCache {
    outlines: HashMap<DocumentOutlineCacheKey, Arc<DocumentOutline>>,
    suppressed_sessions: HashSet<DocumentOutlineCacheKey>,
}

impl NavigationCache {
    pub fn new() -> Self {
        Self {
            outlines: HashMap::new(),
            suppressed_sessions: HashSet::new(),
        }
    }

    pub fn get_outline(&self, key: &DocumentOutlineCacheKey) -> Option<Arc<DocumentOutline>> {
        self.outlines.get(key).cloned()
    }

    pub fn insert_outline(
        &mut self,
        key: DocumentOutlineCacheKey,
        outline: Arc<DocumentOutline>,
    ) -> bool {
        if self.outlines.contains_key(&key) || self.is_session_suppressed(key) {
            return false;
        }
        self.outlines.insert(key, outline);
        true
    }

    pub fn evict_session(&mut self, document_id: DocumentId, generation: DocumentGeneration) {
        let key = DocumentOutlineCacheKey {
            document_id,
            generation,
        };
        self.outlines.remove(&key);
        self.suppressed_sessions.insert(key);
    }

    pub fn is_session_suppressed(&self, key: DocumentOutlineCacheKey) -> bool {
        self.suppressed_sessions.contains(&key)
    }

    pub fn outline_count(&self) -> usize {
        self.outlines.len()
    }
}

impl Default for NavigationCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct NavigationService {
    pub engine: EngineHandle,
    cache: Arc<Mutex<NavigationCache>>,
}

impl NavigationService {
    pub fn new(engine: EngineHandle) -> Self {
        Self {
            engine,
            cache: Arc::new(Mutex::new(NavigationCache::new())),
        }
    }

    pub async fn get_or_extract_outline(
        &self,
        request: DocumentOutlineRequest,
    ) -> Result<Arc<DocumentOutline>, NavigationError> {
        let key = DocumentOutlineCacheKey::from(request);

        {
            let cache = self.cache.lock().unwrap();
            if let Some(outline) = cache.get_outline(&key) {
                return Ok(outline);
            }
            if cache.is_session_suppressed(key) {
                return Err(NavigationError::DocumentNotOpen);
            }
        }

        let outline = self
            .engine
            .extract_document_outline(request)
            .await
            .map_err(|_| NavigationError::EngineUnavailable)??;

        if !outline_matches_request(&outline, &request) {
            return Err(NavigationError::DocumentNotOpen);
        }

        let outline = Arc::new(outline);
        {
            let mut cache = self.cache.lock().unwrap();
            if cache.is_session_suppressed(key) {
                return Err(NavigationError::DocumentNotOpen);
            }
            cache.insert_outline(key, outline.clone());
        }

        Ok(outline)
    }

    pub fn evict_session(&self, document_id: DocumentId, generation: DocumentGeneration) {
        self.cache
            .lock()
            .unwrap()
            .evict_session(document_id, generation);
    }

    pub fn outline_count(&self) -> usize {
        self.cache.lock().unwrap().outline_count()
    }
}

pub fn default_outline_request(
    document_id: DocumentId,
    generation: DocumentGeneration,
) -> DocumentOutlineRequest {
    DocumentOutlineRequest {
        document_id,
        generation,
        limits: NavigationResourceLimits::default(),
    }
}

pub fn outline_matches_request(
    outline: &DocumentOutline,
    request: &DocumentOutlineRequest,
) -> bool {
    outline.document_id == request.document_id && outline.generation == request.generation
}

#[cfg(test)]
mod tests;
