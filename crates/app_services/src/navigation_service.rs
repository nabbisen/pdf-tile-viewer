//! Navigation metadata cache and async extraction service (RFC 026).
//!
//! The cache is memory-only and keyed by document and generation. Extracted
//! outline data is document metadata, not executable behavior; external URI
//! opening remains governed by `uri_policy`.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use domain::document::{DocumentGeneration, DocumentId, PageIndex};
use domain::navigation::{
    DocumentOutline, DocumentOutlineRequest, NavigationError, NavigationResourceLimits,
    PageLinkSet, PageLinksCacheKey, PageLinksRequest,
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
    page_links: HashMap<PageLinksCacheKey, Arc<PageLinkSet>>,
    page_link_order: VecDeque<PageLinksCacheKey>,
    suppressed_sessions: HashSet<DocumentOutlineCacheKey>,
}

impl NavigationCache {
    pub fn new() -> Self {
        Self {
            outlines: HashMap::new(),
            page_links: HashMap::new(),
            page_link_order: VecDeque::new(),
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

    pub fn get_page_links(&self, key: &PageLinksCacheKey) -> Option<Arc<PageLinkSet>> {
        self.page_links.get(key).cloned()
    }

    pub fn insert_page_links(&mut self, links: Arc<PageLinkSet>, max_cached_pages: usize) -> bool {
        let key = PageLinksCacheKey {
            document_id: links.document_id,
            generation: links.generation,
            page_index: links.page_index,
        };
        if max_cached_pages == 0
            || self.page_links.contains_key(&key)
            || self.is_session_suppressed(DocumentOutlineCacheKey {
                document_id: key.document_id,
                generation: key.generation,
            })
        {
            return false;
        }
        while self.page_links.len() >= max_cached_pages {
            let Some(oldest) = self.page_link_order.pop_front() else {
                break;
            };
            self.page_links.remove(&oldest);
        }
        self.page_link_order.push_back(key);
        self.page_links.insert(key, links);
        true
    }

    pub fn evict_session(&mut self, document_id: DocumentId, generation: DocumentGeneration) {
        let key = DocumentOutlineCacheKey {
            document_id,
            generation,
        };
        self.outlines.remove(&key);
        self.page_links.retain(|link_key, _| {
            link_key.document_id != document_id || link_key.generation != generation
        });
        self.page_link_order.retain(|link_key| {
            link_key.document_id != document_id || link_key.generation != generation
        });
        self.suppressed_sessions.insert(key);
    }

    pub fn is_session_suppressed(&self, key: DocumentOutlineCacheKey) -> bool {
        self.suppressed_sessions.contains(&key)
    }

    pub fn outline_count(&self) -> usize {
        self.outlines.len()
    }

    pub fn page_link_page_count(&self) -> usize {
        self.page_links.len()
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

    pub async fn get_or_extract_page_links(
        &self,
        request: PageLinksRequest,
    ) -> Result<Arc<PageLinkSet>, NavigationError> {
        let key = request.cache_key();
        let session_key = DocumentOutlineCacheKey {
            document_id: request.document_id,
            generation: request.generation,
        };

        {
            let cache = self.cache.lock().unwrap();
            if let Some(links) = cache.get_page_links(&key) {
                return Ok(links);
            }
            if cache.is_session_suppressed(session_key) {
                return Err(NavigationError::DocumentNotOpen);
            }
        }

        let links = self
            .engine
            .extract_page_links(request)
            .await
            .map_err(|_| NavigationError::EngineUnavailable)??;

        if !page_links_match_request(&links, &request) {
            return Err(NavigationError::DocumentNotOpen);
        }

        let links = Arc::new(links);
        {
            let mut cache = self.cache.lock().unwrap();
            if cache.is_session_suppressed(session_key) {
                return Err(NavigationError::DocumentNotOpen);
            }
            cache.insert_page_links(links.clone(), request.limits.max_cached_link_pages);
        }

        Ok(links)
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

    pub fn page_link_page_count(&self) -> usize {
        self.cache.lock().unwrap().page_link_page_count()
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

pub fn default_page_links_request(
    document_id: DocumentId,
    generation: DocumentGeneration,
    page_index: PageIndex,
) -> PageLinksRequest {
    PageLinksRequest {
        document_id,
        generation,
        page_index,
        limits: NavigationResourceLimits::default(),
    }
}

pub fn outline_matches_request(
    outline: &DocumentOutline,
    request: &DocumentOutlineRequest,
) -> bool {
    outline.document_id == request.document_id && outline.generation == request.generation
}

pub fn page_links_match_request(links: &PageLinkSet, request: &PageLinksRequest) -> bool {
    links.document_id == request.document_id
        && links.generation == request.generation
        && links.page_index == request.page_index
}

#[cfg(test)]
mod tests;
