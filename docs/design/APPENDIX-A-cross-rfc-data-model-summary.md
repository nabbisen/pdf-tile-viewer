---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-06-07
status: Draft for implementation planning
baseline: PDF Tile Viewer v1.1.2 reverse-engineered design + approved Dioxus goal-state design
---

# Appendix A — Cross-RFC Data Model Summary

## 1. Purpose

This appendix summarizes important data models shared across the RFC set. It is not a replacement for the individual RFCs; it is a quick reference for implementers.

## 2. Identity Types

```rust
pub struct DocumentId(/* opaque */);
pub struct PageIndex(pub usize);       // zero-based internally
pub struct DocumentGeneration(pub u64);
pub struct LayoutGeneration(pub u64);
pub struct RenderRequestId(/* opaque */);
pub struct RecentEntryId(/* opaque */);
```

Rules:

- User-facing page numbers are one-based.
- Internal page indices are zero-based.
- All async render/search results must include document id and generation.

## 3. Document Model

```rust
pub struct DocumentSession {
    pub id: DocumentId,
    pub source: DocumentSource,
    pub display_name: String,
    pub metadata: DocumentMetadata,
    pub pages: Vec<PageDescriptor>,
    pub generation: DocumentGeneration,
    pub state: DocumentState,
}
```

## 4. Layout Model

```rust
pub struct TileLayout {
    pub rows: Vec<TileRow>,
    pub tiles: Vec<PageTileLayout>,
    pub content_width_px: f32,
    pub content_height_px: f32,
}
```

## 5. Render Model

```rust
pub struct RenderCacheKey {
    pub document_id: DocumentId,
    pub document_generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub scale_bucket: ScaleBucket,
    pub render_flags: RenderFlags,
}
```

## 6. Search Model

```rust
pub struct SearchResultSet {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub query: SearchQuery,
    pub pages: Vec<PageSearchSummary>,
    pub total_matches: usize,
}
```

## 7. Settings Model

```rust
pub struct AppSettingsV1 {
    pub schema_version: u32,
    pub viewer: ViewerSettings,
    pub window: WindowSettings,
    pub privacy: PrivacySettings,
    pub advanced: AdvancedSettings,
}
```

## 8. Cross-Cutting Invariants

1. PDFium handles never enter Dioxus UI state.
2. Layout logic never calls PDFium directly.
3. Render/search results are ignored if document id or generation no longer matches.
4. Production PDFium loading uses app-controlled paths only.
5. Search and highlights do not mutate PDF bytes.
6. Large documents are served by lazy rendering, not eager full-document rendering.
