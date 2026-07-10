//! PDF outline and link navigation model (RFC 026).
//!
//! Navigation metadata is untrusted document content. This module keeps the
//! data domain-owned, generation-tagged, and bounded before UI or platform
//! layers consume it.

use crate::document::{DocumentGeneration, DocumentId, PageIndex};
use crate::search::PageRect;

pub const DEFAULT_MAX_OUTLINE_NODES: usize = 4096;
pub const DEFAULT_MAX_OUTLINE_DEPTH: usize = 16;
pub const DEFAULT_MAX_BOOKMARK_TITLE_CHARS: usize = 512;
pub const DEFAULT_MAX_BOOKMARK_TITLE_BYTES: usize = 2048;
pub const DEFAULT_MAX_LINKS_PER_PAGE: usize = 512;
pub const DEFAULT_MAX_URI_BYTES: usize = 2048;
pub const DEFAULT_MAX_CACHED_LINK_PAGES: usize = 8;
pub const DEFAULT_MAX_CACHED_LINK_BYTES: usize = 2 * 1024 * 1024;
pub const DEFAULT_MAX_OUTLINE_STRING_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigationResourceLimits {
    pub max_outline_nodes: usize,
    pub max_outline_depth: usize,
    pub max_bookmark_title_chars: usize,
    pub max_bookmark_title_bytes: usize,
    pub max_links_per_page: usize,
    pub max_uri_bytes: usize,
    pub max_cached_link_pages: usize,
    pub max_cached_link_bytes: usize,
    pub max_outline_string_bytes: usize,
}

impl Default for NavigationResourceLimits {
    fn default() -> Self {
        Self {
            max_outline_nodes: DEFAULT_MAX_OUTLINE_NODES,
            max_outline_depth: DEFAULT_MAX_OUTLINE_DEPTH,
            max_bookmark_title_chars: DEFAULT_MAX_BOOKMARK_TITLE_CHARS,
            max_bookmark_title_bytes: DEFAULT_MAX_BOOKMARK_TITLE_BYTES,
            max_links_per_page: DEFAULT_MAX_LINKS_PER_PAGE,
            max_uri_bytes: DEFAULT_MAX_URI_BYTES,
            max_cached_link_pages: DEFAULT_MAX_CACHED_LINK_PAGES,
            max_cached_link_bytes: DEFAULT_MAX_CACHED_LINK_BYTES,
            max_outline_string_bytes: DEFAULT_MAX_OUTLINE_STRING_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NavigationLimitStatus {
    pub outline_truncated: bool,
    pub links_truncated: bool,
    pub strings_truncated: bool,
}

impl NavigationLimitStatus {
    pub fn any_truncated(self) -> bool {
        self.outline_truncated || self.links_truncated || self.strings_truncated
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentOutlineRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub limits: NavigationResourceLimits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageLinksRequest {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub limits: NavigationResourceLimits,
}

impl PageLinksRequest {
    pub fn cache_key(self) -> PageLinksCacheKey {
        PageLinksCacheKey {
            document_id: self.document_id,
            generation: self.generation,
            page_index: self.page_index,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PageLinksCacheKey {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentOutline {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub roots: Vec<OutlineNode>,
    pub limit_status: NavigationLimitStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OutlineNodeId(pub String);

#[derive(Clone, Debug, PartialEq)]
pub struct OutlineNode {
    pub id: OutlineNodeId,
    pub title: OutlineTitle,
    pub target: NavigationTarget,
    pub children: Vec<OutlineNode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutlineTitle {
    Present(String),
    Missing,
    Empty,
    Truncated(String),
}

impl OutlineTitle {
    pub fn from_pdf_title(raw: Option<String>, limits: &NavigationResourceLimits) -> Self {
        match raw {
            None => OutlineTitle::Missing,
            Some(title) if title.is_empty() => OutlineTitle::Empty,
            Some(title) => truncate_title(title, limits),
        }
    }

    pub fn stored_bytes(&self) -> usize {
        match self {
            OutlineTitle::Present(value) | OutlineTitle::Truncated(value) => value.len(),
            OutlineTitle::Missing | OutlineTitle::Empty => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PageLinkSet {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub links: Vec<PageLink>,
    pub limit_status: NavigationLimitStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PageLinkId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct PageLink {
    pub id: PageLinkId,
    pub rect: PageRect,
    pub target: NavigationTarget,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NavigationTarget {
    InternalDestination(NavigationDestination),
    ExternalUri(ExternalUriTarget),
    Disabled(DisabledNavigationReason),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationDestination {
    pub page_index: PageIndex,
    pub view: DestinationView,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DestinationView {
    PageOnly,
    CoordinatesAndZoom {
        x_points: Option<f32>,
        y_points: Option<f32>,
        zoom: Option<f32>,
    },
    FitPage,
    FitHorizontal {
        y_points: Option<f32>,
    },
    FitVertical {
        x_points: Option<f32>,
    },
    FitRectangle(PageRect),
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalUriTarget {
    pub raw_uri: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisabledNavigationReason {
    NoDestination,
    InvalidDestination,
    InvalidExternalUri,
    ExternalUriTooLong,
    RemoteDestination,
    EmbeddedDestination,
    LaunchAction,
    UnsupportedAction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NavigationError {
    DocumentNotOpen,
    PageOutOfBounds,
    EngineUnavailable,
    LinkIndexOverflow,
    ExtractionFailed(String),
}

pub fn external_uri_target(raw_uri: String, limits: &NavigationResourceLimits) -> NavigationTarget {
    if raw_uri.is_empty() {
        return NavigationTarget::Disabled(DisabledNavigationReason::InvalidExternalUri);
    }
    if raw_uri.len() > limits.max_uri_bytes {
        return NavigationTarget::Disabled(DisabledNavigationReason::ExternalUriTooLong);
    }
    NavigationTarget::ExternalUri(ExternalUriTarget { raw_uri })
}

fn truncate_title(title: String, limits: &NavigationResourceLimits) -> OutlineTitle {
    if title.chars().count() <= limits.max_bookmark_title_chars
        && title.len() <= limits.max_bookmark_title_bytes
    {
        return OutlineTitle::Present(title);
    }

    let mut out = String::new();
    for ch in title.chars().take(limits.max_bookmark_title_chars) {
        if out.len() + ch.len_utf8() > limits.max_bookmark_title_bytes {
            break;
        }
        out.push(ch);
    }
    OutlineTitle::Truncated(out)
}

#[cfg(test)]
mod tests;
