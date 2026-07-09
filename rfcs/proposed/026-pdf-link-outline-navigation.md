---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-07
status: Proposed
baseline: PDF Tile Viewer 2.0.0-beta.11
depends_on: RFC 012, RFC 016, RFC 023, RFC 025
review: .git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_design_review.md
rereview: .git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_design_rereview.md
---

# RFC 026 — PDF Link and Outline Navigation

## 1. Summary

Add PDF outline/bookmark navigation and best-effort PDF link navigation while
preserving PDF Tile Viewer's security posture.

PDF Tile Viewer currently renders pages, supports search, supports a
single-page zoom overlay, and restores selectable text in the single-page view.
It does not expose PDF bookmarks, table-of-contents entries, or clickable PDF
links. Users must manually locate target pages even when the PDF already
contains navigation metadata.

RFC 026 adds two related workflows:

1. a document outline panel populated from PDF bookmarks;
2. clickable link hit areas in the single-page zoom overlay.

Internal destinations in the same document are navigable. External PDF actions
remain constrained by RFC 016: the app must not auto-open external links, must
not execute embedded JavaScript, must not launch files, and must require
explicit user action before any external URI is opened.

This is not release work. The latest released version at the time of this RFC
draft is `2.0.0-beta.11`; RFC 026 is proposed follow-up work after that
release point.

## 2. Motivation

PDF links and outlines are standard PDF viewer expectations. They matter for:

- long technical documents with table-of-contents bookmarks;
- reports whose internal references jump between sections, figures, and
  appendices;
- manuals where visible links point to another page;
- PDFs generated from web pages or office documents with external references.

The app already has the right architecture for this feature:

- PDFium access is isolated in `pdf_engine`;
- UI code calls application services rather than PDFium directly;
- RFC 012 provides a bounded single-page zoom surface;
- RFC 023 already solved the pattern of page-space geometry mapped into a zoom
  overlay;
- RFC 016 already records the external action security constraints.

The feature should therefore reuse the existing service boundary and overlay
geometry instead of introducing a separate PDF viewer subsystem.

## 3. Feature Contract

PDF Tile Viewer should expose document navigation metadata without executing
PDF-provided active behavior automatically.

The implemented feature must:

- show a document outline panel when a PDF contains bookmarks;
- allow users to navigate to same-document bookmark destinations;
- allow users to activate same-document page links in the zoom overlay;
- keep link hit areas scoped to the active zoom page, not the tile grid;
- preserve search state, text-selection state, and tile rendering behavior;
- keep PDFium handles and PDFium-specific lifetime types inside `pdf_engine`;
- keep extracted link and outline data memory-only;
- enforce hard resource limits for untrusted outline, link, title, and URI
  data;
- treat malformed links, unsupported actions, and invalid destinations as
  non-fatal per-item failures;
- never execute JavaScript embedded in PDFs;
- never auto-open a URI or launch an external file;
- never execute PDF launch, remote document, or embedded document actions.

Same-document navigation is part of the core feature. External URI handling is
allowed only behind an explicit confirmation flow and only for schemes accepted
by this RFC.

## 4. Goals

- Add a PDF outline/bookmark domain model.
- Add a page-link domain model.
- Extract bookmarks and page links through PDFium in `pdf_engine`.
- Resolve same-document destinations to page indices and optional view hints.
- Add an outline panel to the viewer.
- Add clickable link hit areas to the single-page zoom overlay.
- Support keyboard and pointer activation for outline entries and zoom links.
- Add explicit, conservative handling for external URI actions.
- Preserve all existing open, search, tile, zoom, and text-selection behavior.
- Add focused automated coverage and a small navigation fixture.

## 5. Non-Goals

- Link hit areas in the tile grid.
- A document-wide link layer mounted for every visible tile.
- PDF annotation editing.
- PDF form submission.
- Executing PDF JavaScript.
- Executing launch actions or opening local files referenced by a PDF.
- Opening remote or embedded PDF documents.
- Persisting outline expansion state across app launches.
- Persisting external URI decisions.
- Replacing the current search-result navigation model.
- Full PDF named-destination parity if PDFium does not expose enough data
  through the current crate API.

## 6. Current State

### 6.1 Implemented Viewer Capabilities

The 2.0 beta line currently has:

- PDFium-backed PDF opening and page rendering;
- tile-grid overview rendering;
- lazy page rendering and cache behavior;
- search result extraction and page-space highlight rectangles;
- zoom overlay rendering;
- best-effort selectable text in the zoom overlay;
- password-protected PDF opening.

### 6.2 Missing Navigation Capabilities

The app does not currently expose:

- document outlines/bookmarks;
- clickable PDF links;
- PDF internal destination navigation;
- PDF table-of-contents navigation;
- external URI link affordances.

### 6.3 Relevant Security Baseline

RFC 016 already defines the policy that controls this RFC:

- do not execute PDF embedded JavaScript;
- do not auto-open external links;
- do not auto-launch files or external actions from PDFs;
- any future link/outline navigation must require explicit user action.

RFC 026 must implement within that policy rather than broadening it silently.

## 7. PDFium API Evidence

The workspace manifest currently allows `pdfium-render = "0.9"`, and the
current lockfile resolves that dependency to `pdfium-render 0.9.1`. That
lockfile version exposes the primitives needed for a bounded implementation:

- `PdfDocument::bookmarks()` returns document bookmarks;
- `PdfBookmark::title()` returns the visible bookmark title;
- `PdfBookmark::action()` returns an optional `PdfAction`;
- `PdfBookmark::destination()` returns an optional `PdfDestination`;
- `PdfBookmark::iter_direct_children()` supports tree traversal;
- `PdfPage::links()` returns links for a page;
- `PdfLink::rect()` returns the link rectangle;
- `PdfLink::action()` returns an optional `PdfAction`;
- `PdfLink::destination()` returns an optional `PdfDestination`;
- `PdfDestination::page_index()` resolves the target page;
- `PdfDestination::view_settings()` exposes optional view hints;
- `PdfAction` distinguishes local destination, remote destination, embedded
  destination, launch, URI, and unsupported action types;
- `PdfActionUri::uri()` returns the URI string for URI actions.

The implementation must still treat this as a best-effort API. If a bookmark or
link has an action type that PDFium reports as unsupported, the item should be
kept inert and surfaced only as disabled metadata where useful.

The current `pdfium-render 0.9.1` API does not expose JavaScript actions as a
distinct `PdfAction` variant. JavaScript actions therefore fall into the
unsupported bucket unless a future dependency version exposes a stronger typed
signal. The security rule remains the same: JavaScript from PDFs is never
executed.

## 8. Design Principles

### 8.1 Navigation Is User-Initiated

The app may extract link and outline metadata after opening a document, but it
must not follow a destination or open anything until the user activates a UI
control.

### 8.2 Internal Navigation Is Safe Viewer State

A same-document destination changes the app's current page or scroll position.
It does not leave the app, launch a process, or read another file. This is the
primary supported action in RFC 026.

### 8.3 External Actions Are Inert By Default

External PDF actions are untrusted document content. The app should classify
them, display a conservative user affordance, and require explicit confirmation
before opening an allowed URI. Launch, remote destination, embedded
destination, JavaScript, and unsupported actions remain non-executable.

The cached extraction result is not an authorization decision. Any external URI
must be parsed and revalidated by `app_services` immediately before platform
open.

### 8.4 Tile View Remains An Overview

The tile grid is optimized for scanning many pages. Mounting link hit areas for
every tile would add many invisible interactive nodes, create confusing pointer
behavior, and compete with tile selection/open behavior. RFC 026 therefore
scopes page link hit areas to the active zoom overlay only.

Outline navigation is document-level navigation and is available from the
viewer shell independently of the zoom overlay.

### 8.5 PDFium Types Do Not Cross Service Boundaries

`pdf_engine` converts PDFium handles, lifetimes, rectangles, destinations, and
actions into domain-owned structs. UI and app services never hold PDFium
objects.

## 9. User Experience

### 9.1 Outline Panel

The viewer gains an outline toggle in the toolbar. When active, it opens a side
panel containing the document outline.

Panel behavior:

- hidden by default unless the user opens it;
- disabled or absent when the document has no outline entries;
- shows nested bookmark titles as a tree;
- supports expand/collapse for nodes with children;
- supports keyboard focus and activation;
- marks entries without supported same-document destinations as disabled;
- does not show raw PDF object IDs or debug action names to normal users.

Clicking or keyboard-activating a supported outline entry navigates to the
target page.

### 9.2 Link Hit Areas In Zoom Overlay

The zoom overlay gains transparent hit areas above the rendered page bitmap and
below or alongside the text-selection layer, depending on pointer behavior
tests.

Required behavior:

- internal links show pointer affordance on hover;
- internal links are focusable and activatable by keyboard;
- activating an internal link navigates within the zoom overlay if it is open;
- activating an internal link updates the underlying viewer page/scroll target;
- external URI links show an explicit confirmation dialog or popover before
  any external open;
- disabled actions show no misleading "open" affordance.

The text-selection layer from RFC 023 must continue to work. If a link
rectangle and text-selection span overlap, the implementation should prefer
normal PDF viewer behavior:

- a simple click activates the link;
- a drag gesture selects text;
- keyboard focus can reach link controls without breaking text selection.

The exact layering may require implementation testing. The RFC accepts either:

- link buttons above the text layer with drag threshold handling; or
- link buttons below the text layer with explicit pointer-event pass-through
  rules;

provided the acceptance checklist proves both click links and text selection
still work.

### 9.3 Navigation Outcomes

When the user navigates to a same-document destination:

- from normal tile view: scroll the tile grid to the target page and select or
  briefly emphasize that page;
- from outline while zoom overlay is open: keep the zoom overlay open and
  render the target page;
- from a zoom-overlay link: keep the zoom overlay open and render the target
  page;
- preserve the current search query and search results;
- preserve current zoom overlay controls where sensible;
- do not clear document history.

View hints from `PdfDestination::view_settings()` should be represented in the
domain model, but the first implementation may apply only the target page. The
UI should not pretend to support precise coordinate or zoom placement until it
does.

### 9.4 External URI Confirmation

External URI actions are not opened automatically.

On activation of a URI link, show a confirmation dialog or popover that:

- clearly displays the target URI in sanitized, wrapped text;
- provides a copy action;
- provides an open action only after `app_services` revalidates the URI;
- makes cancel the default safe path;
- does not remember the decision.

Allowed schemes for an open action:

- `https`;
- `http`;
- `mailto`.

All other schemes are copy-only or disabled. In particular, `file`,
platform-specific app schemes, shell-like strings, and empty/invalid URIs must
not be opened.

Launch, remote destination, embedded destination, JavaScript, and unsupported
actions must not provide an open action.

The first implementation may choose copy-only external URI behavior. If it
implements external opening, the open path must still revalidate the raw URI at
the point of opening.

## 10. Domain Model

Add a navigation-focused domain module. The exact file placement may follow the
current codebase layout, but the model should be owned by `domain`, not
`pdf_engine`.

Navigation metadata is untrusted. Domain types should carry enough state for
the UI to degrade gracefully when resource limits are reached, without treating
cached extraction data as permission to open external targets.

Recommended names:

```rust
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

pub struct NavigationLimitStatus {
    pub outline_truncated: bool,
    pub links_truncated: bool,
    pub strings_truncated: bool,
}

pub struct DocumentOutline {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub roots: Vec<OutlineNode>,
    pub limit_status: NavigationLimitStatus,
}

pub struct OutlineNode {
    pub id: OutlineNodeId,
    pub title: OutlineTitle,
    pub target: NavigationTarget,
    pub children: Vec<OutlineNode>,
}

pub enum OutlineTitle {
    Present(String),
    Missing,
    Empty,
    Truncated(String),
}

pub struct PageLinkSet {
    pub document_id: DocumentId,
    pub generation: DocumentGeneration,
    pub page_index: PageIndex,
    pub links: Vec<PageLink>,
    pub limit_status: NavigationLimitStatus,
}

pub struct PageLink {
    pub id: PageLinkId,
    pub rect: PageRect,
    pub target: NavigationTarget,
}

pub enum NavigationTarget {
    InternalDestination(NavigationDestination),
    ExternalUri(ExternalUriTarget),
    Disabled(DisabledNavigationReason),
}

pub struct NavigationDestination {
    pub page_index: PageIndex,
    pub view: DestinationView,
}

pub enum DestinationView {
    PageOnly,
    CoordinatesAndZoom {
        x_points: Option<f32>,
        y_points: Option<f32>,
        zoom: Option<f32>,
    },
    FitPage,
    FitHorizontal { y_points: Option<f32> },
    FitVertical { x_points: Option<f32> },
    FitRectangle(PageRect),
    Other,
}

pub struct ExternalUriTarget {
    pub raw_uri: String,
}

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
```

The final names should match existing conventions. The important constraints
are:

- data is owned and serializable/debuggable without PDFium lifetimes;
- rectangles use the same page-space convention as search and text overlays;
- stale `document_id` / `generation` data can be rejected;
- external URI data is cached only as raw untrusted input, not as an open
  authorization;
- disabled reasons remain internal or mapped through i18n before display.

`OutlineTitle::Missing`, `OutlineTitle::Empty`, and
`OutlineTitle::Truncated` are not user-facing strings. The UI maps them through
the i18n catalog.

### 10.1 Required Resource Limits

The first implementation must define these limits in one shared location used
by extraction, services, and tests:

| Limit | Required Default | Degradation |
|-------|------------------|-------------|
| Outline nodes per document | 4096 nodes | Stop traversal after the cap, mark outline truncated, keep already collected nodes. |
| Outline nesting depth | 16 levels | Do not descend further, mark outline truncated, keep the capped node if valid. |
| Bookmark title length | 512 Unicode scalar values and 2048 UTF-8 bytes | Truncate for stored/displayed metadata, mark title truncated. |
| Page links per page | 512 links | Keep the first valid links in PDF order, mark page links truncated. |
| URI length | 2048 UTF-8 bytes | Do not store a full openable/copyable URI; mark the target disabled as too long. |
| Cached link pages | 8 pages per document generation | Evict least-recently-used page link sets. |
| Cached link memory | 2 MiB of link rectangles and URI payload | Evict least-recently-used page link sets. |
| Cached outline memory | 1 MiB of title/URI string payload | Stop collecting additional string payload and mark outline truncated. |

These values are intentionally conservative. They can be changed only with a
follow-up design note or implementation review evidence.

When a cap is exceeded, the viewer remains usable. The app should not fail the
document open because navigation metadata is too large.

## 11. Engine Extraction

### 11.1 Outline Extraction

`pdf_engine` should expose an application-service-facing operation equivalent
to:

```rust
extract_document_outline(document_id, generation) -> DocumentOutline
```

Implementation notes:

- traverse `PdfDocument::bookmarks()`;
- build tree shape using `PdfBookmark::iter_direct_children()`;
- use deterministic traversal-path IDs such as `0`, `0.1`, `0.1.2`;
- read titles with `PdfBookmark::title()` into `OutlineTitle`;
- preserve missing and empty title states instead of creating English fallback
  text in `pdf_engine`;
- enforce outline node, depth, title, URI, and string-payload limits while
  traversing;
- resolve `PdfBookmark::destination()` first when present;
- otherwise inspect `PdfBookmark::action()`;
- map same-document actions to `NavigationDestination`;
- map URI actions to `ExternalUriTarget`;
- map all other actions to `DisabledNavigationReason`;
- keep per-item failures local to the item;
- do not fail opening the document because outline extraction failed.

The extraction should run after a document opens successfully. It may be lazy
when the panel is first opened if that is simpler for the current service
architecture, but the UI must handle loading and empty states cleanly.

### 11.2 Page Link Extraction

`pdf_engine` should expose an operation equivalent to:

```rust
extract_page_links(document_id, generation, page_index) -> PageLinkSet
```

Implementation notes:

- call `PdfPage::links()` for the requested page only;
- use `PdfLink::rect()` for page-space link hit areas;
- skip rectangles that cannot be read or have non-positive dimensions;
- resolve `PdfLink::destination()` first when present;
- otherwise inspect `PdfLink::action()`;
- map same-document actions to `NavigationDestination`;
- map URI actions to `ExternalUriTarget`;
- map all other actions to disabled targets;
- enforce the page-link and URI-length limits;
- return an empty link set for pages without links;
- cache successful results per document generation and page index.

Page links should be requested for the active zoom page. They should not be
requested for every tile during ordinary tile-grid rendering.

### 11.3 Destination Mapping

Destination mapping converts:

- `PdfDestination::page_index()` to `PageIndex`;
- `PdfDestination::view_settings()` to `DestinationView`;
- PDFium rectangles and points to existing page-space geometry types.

If `page_index()` fails, the destination is disabled with
`InvalidDestination`. If `view_settings()` fails, the implementation may keep
the target page and set `DestinationView::PageOnly`.

The first implementation may ignore non-page view hints during UI navigation,
but it should preserve them in the domain model so that later work can support
coordinate scroll within the zoom overlay.

## 12. Application Services

Application services should own:

- outline extraction orchestration;
- page-link extraction orchestration;
- in-memory outline/link caches;
- stale generation checks;
- navigation command application;
- authoritative external URI parsing and policy classification;
- platform open delegation, if URI opening is implemented.

Recommended service responsibilities:

- expose `get_document_outline(document_id, generation)`;
- expose `get_page_links(document_id, generation, page_index)`;
- expose `navigate_to_destination(destination)`;
- expose `classify_external_uri_for_display(raw_uri)`;
- expose `open_external_uri_after_confirmation(raw_uri)` only for URIs that
  pass immediate revalidation.

External URI opening should remain outside `domain` and outside `pdf_engine`.
It is platform integration. If the implementation chooses copy-only URI support
for the first slice, it should still keep the domain model compatible with a
future confirm-then-open path.

### 12.1 Authoritative URI Policy

`app_services` is the single authority for external URI opening. The extraction
layer may store a raw URI string for display/copy, but it must not store a
trusted "allowed to open" decision.

Immediately before calling any platform open API, `app_services` must:

- reject URI strings longer than the configured URI byte limit;
- reject empty strings;
- reject relative references and scheme-less strings;
- reject strings containing ASCII control characters;
- trim only leading and trailing ASCII whitespace before parsing;
- lowercase the scheme for comparison;
- allow only `https`, `http`, and `mailto`;
- reject `file` and all custom/platform schemes;
- reject parse failures;
- pass a normalized URI string to the platform open API;
- never invoke a shell or concatenate the URI into a shell command.

The confirmation dialog is not sufficient authorization. Revalidation must
happen after confirmation and immediately before platform open.

## 13. UI Design

### 13.1 Toolbar

Add an outline toggle near other viewer navigation controls. The control should
be disabled when no outline exists.

If the app already uses an icon source, use a familiar outline/list/bookmark
icon with a tooltip. If not, follow existing toolbar style rather than
introducing a new icon dependency only for this RFC.

### 13.2 Outline Panel Layout

The outline panel should be a side panel in the viewer, not a modal.

Expected behavior:

- width is fixed or bounded so it does not destabilize tile layout;
- long titles wrap or ellipsize without overflowing;
- nested entries use clear indentation;
- expanded/collapsed state is local UI state;
- disabled entries remain visible only when useful and are not activatable;
- missing, empty, or truncated titles are mapped to localized UI text;
- panel empty state is short and localized.

### 13.3 Zoom Link Overlay

The link overlay should reuse the same page-to-screen transform used by search
highlights and text-selection geometry.

Expected behavior:

- stable overlay dimensions track the rendered page bitmap;
- link rectangles do not shift layout;
- hit areas have accessible labels derived from target type;
- internal links activate page navigation;
- external URI links open the confirmation flow;
- disabled actions are either omitted or represented as non-interactive.

The overlay must be tested with text selection. If the first layering approach
breaks drag-selection, the implementation should adjust pointer-event handling
before review.

### 13.4 I18n

All user-facing strings must go through the existing i18n resources.

Likely new strings:

- outline panel label;
- no-outline state;
- disabled outline entry label;
- external link confirmation title;
- external link open button;
- copy link button;
- invalid/unsupported link message;
- copied-to-clipboard result, if the app has such feedback.

Do not display raw internal enum names to users.

## 14. Security and Privacy

RFC 026 handles untrusted document-provided navigation metadata. The following
rules are mandatory:

- never execute PDF JavaScript;
- never auto-open external URIs;
- never execute launch actions;
- never open local files from PDF actions;
- never open remote or embedded documents from PDF actions;
- allow external open only after explicit confirmation and immediate
  `app_services` revalidation, and only for `https`, `http`, and `mailto`;
- do not persist external URI decisions;
- do not write extracted URIs, bookmark titles, or link targets to logs unless
  an existing user-controlled diagnostic path already permits document metadata;
- do not include private document URI targets in generated review packages.

The confirmation dialog must not obscure the URI. Users need to see what they
are about to open.

The platform open implementation must use a platform/browser API directly. It
must not spawn a shell to interpret the URI.

## 15. Performance

Outline extraction is document-level and should normally be cheap. Still, large
documents can contain many bookmarks, so extraction should:

- be cancellable or safely ignorable if the document changes;
- use generation checks before applying results;
- avoid blocking the UI thread;
- enforce the resource limits from section 10.1.

Page-link extraction is scoped to the active zoom page:

- do not extract links for all pages at document open;
- do not mount link overlays in tile view;
- cache by document generation and page index;
- cap the cache to the configured number of pages;
- discard cached data when the document closes or generation changes.

The link overlay should add at most one focusable/hit-test node per link on the
active page. This is consistent with the RFC 023 text-layer scope, where the
single-page view can afford more DOM detail than the tile grid.

## 16. Accessibility

Outline entries and link hit areas must be keyboard accessible.

Requirements:

- outline toggle is focusable;
- outline entries are focusable buttons/tree items;
- internal link hit areas in zoom overlay are focusable;
- activation works with Enter/Space as appropriate for the element;
- focus is not lost when navigation changes the active zoom page;
- disabled actions are not included in tab order unless represented as
  explanatory controls;
- external confirmation dialog traps focus while open and returns focus on
  close.

The implementation should prefer native button/anchor semantics where possible.

## 17. Error Handling

Navigation metadata extraction is best effort.

Expected handling:

- outline extraction failure: panel shows a localized unavailable state; viewer
  remains usable;
- page-link extraction failure: zoom overlay renders without clickable PDF
  links; text selection and search highlights remain usable;
- invalid destination: affected item is disabled or ignored;
- unsupported action: affected item is disabled or ignored;
- invalid URI: copy/open controls are disabled or copy-only, depending on what
  can be displayed safely;
- stale extraction result: drop silently.

The app should avoid noisy global errors for individual broken PDF links.

## 18. Fixtures and Tests

Add a small public navigation fixture under `fixtures/` unless the repository
has already moved test documents to a different accepted location by the time
this RFC is implemented.

Recommended fixture properties:

- three to five pages;
- at least two outline roots;
- at least one nested outline entry;
- at least one missing or empty bookmark title, if the fixture tooling can
  represent it deterministically;
- one same-document link from page 1 to a later page;
- one same-document link back to page 1;
- one `https` URI link;
- one `file` URI or custom-scheme URI that must not open;
- one invalid URI or URI parse-failure case;
- one unsupported or launch-style action when a safe deterministic fixture can
  be created without external dependencies at test time.

The fixture creation process should be documented. If generated with `qpdf` or
another tool, check in either the generated fixture plus a short provenance note
or a deterministic generation script that does not require network access.

Automated test coverage should include:

- domain mapping tests for allowed URI schemes;
- domain mapping tests for disabled action reasons;
- resource-limit tests for outline node count, depth, title length, URI
  length, links per page, and cache eviction;
- title-state tests proving missing/empty/truncated titles are not converted to
  English in `pdf_engine`;
- PDF engine smoke test extracting outline roots and nested children;
- PDF engine smoke test extracting same-document page links;
- PDF engine smoke test extracting URI links without opening them;
- negative URI tests for `file`, custom schemes, relative references, empty
  strings, control characters, invalid parse cases, and over-limit strings;
- negative action tests for launch, remote, embedded, and unsupported actions,
  using unit-level mapping tests when deterministic PDF fixtures cannot
  represent every action type;
- service tests proving URI revalidation happens at open time and does not
  trust cached extraction policy;
- app/service stale generation rejection tests;
- i18n completeness checks for new strings;
- UI/component tests where existing test infrastructure supports them.

Manual QA should cover:

- opening a PDF with no outline;
- opening a PDF with an outline;
- navigating from outline to page;
- navigating internal links in zoom overlay;
- dragging text selection across linked text;
- activating a URI link and cancelling;
- copying a URI link;
- confirming an allowed URI open, if implemented;
- verifying launch/file/unsupported actions cannot be opened.

## 19. Implementation Handoff

### 19.1 Scope

Implement RFC 026 as a navigation feature after `2.0.0-beta.11`.

Primary deliverables:

- proposed domain types for outlines, page links, destinations, and external
  URI policy;
- PDFium extraction for document outline and active-page links;
- viewer outline panel;
- zoom-overlay internal link activation;
- conservative external URI confirmation/copy handling;
- tests and fixture coverage.

### 19.2 Files and Areas to Inspect First

Implementation should start by inspecting current equivalents of:

- domain document/session/page index types;
- search highlight geometry types;
- RFC 023 text-layer extraction and overlay placement;
- zoom overlay component and state;
- viewer toolbar component;
- app service document-open/session state;
- i18n resource files;
- PDF engine document/page extraction code.

Do not assume these file names are stable. Follow the current codebase.

### 19.3 Recommended Sequencing

1. Add domain navigation model and URI policy tests.
2. Add PDF engine extraction for destinations, actions, outline, and page
   links.
3. Add resource-limit enforcement and limit tests.
4. Add fixture and smoke tests for outline/link extraction.
5. Add app-service cache and stale generation handling.
6. Add outline panel UI and outline navigation.
7. Add zoom-overlay link hit areas for internal links.
8. Add external URI confirmation/copy flow.
9. Verify text selection still works with link overlays.
10. Run the agreed gate commands and package a review request before commit.

### 19.4 Review Expectations

The implementation review package should include:

- this RFC;
- implementation diff or codebase tarball, per project review convention;
- test output captured in the current thread;
- notes on any unsupported PDF action behavior;
- manual QA notes for text selection plus link activation;
- explicit confirmation that no release files were changed unless the owner
  separately requested release prep.

## 20. Task Breakdown / PR Plan

### PR 1 — Domain and Engine Extraction

- Add navigation domain types.
- Add URI policy classification.
- Add resource-limit constants and degradation status.
- Add PDFium action/destination mapping.
- Add outline extraction.
- Add active-page link extraction.
- Add fixture, resource-limit tests, negative URI tests, and engine smoke
  tests.

Review point: yes. This PR creates the trust boundary and should be reviewed
before UI wiring.

### PR 2 — App Services and Outline UI

- Add service cache for document outline.
- Add stale generation checks.
- Add toolbar outline toggle.
- Add outline side panel.
- Wire outline entry activation to viewer navigation.
- Add i18n strings.

Review point: yes if the UI diff is non-trivial.

### PR 3 — Zoom Link Overlay and External URI Flow

- Add page-link cache/request path for active zoom page.
- Add zoom-overlay link hit areas.
- Wire internal link navigation.
- Add URI confirmation/copy behavior with revalidation at open time.
- Verify text selection and search highlights still work.
- Add UI/manual QA evidence.

Review point: yes. This PR touches pointer behavior, accessibility, and
external action policy.

### PR 4 — Cleanup and Documentation

- Update user-facing docs if needed.
- Update contributor/test fixture notes.
- Move RFC to `rfcs/done/` only after implementation and acceptance.
- Keep release preparation separate from this RFC unless explicitly requested.

Review point: optional, depending on the size of the docs-only cleanup.

## 21. Acceptance / QA Checklist

Design acceptance:

- [ ] RFC 026 explicitly preserves RFC 016 external action policy.
- [ ] Link hit areas are scoped to zoom overlay, not tile grid.
- [ ] Outline navigation and page links share a domain destination model.
- [ ] External URI behavior requires explicit user confirmation.
- [ ] External URI opening policy has a single authoritative enforcement point
      in `app_services`; platform integration is only a narrow opener for
      already-validated URIs.
- [ ] Cached extracted URI data is not treated as open authorization.
- [ ] Resource limits are specified for outline nodes, depth, title length,
      URI length, links per page, and cache size.
- [ ] Missing/empty title fallback is assigned to UI/i18n, not `pdf_engine`.
- [ ] Launch, JavaScript, remote, embedded, and unsupported actions are not
      executable.
- [ ] Implementation handoff and PR plan are included.

Implementation acceptance:

- [ ] PDF with no outline opens normally.
- [ ] PDF with outline shows outline toggle/panel.
- [ ] Outline entry navigates to the expected page.
- [ ] Nested outline entries render with stable indentation.
- [ ] Over-limit outline trees degrade without failing document open.
- [ ] Over-limit link pages degrade without failing zoom overlay rendering.
- [ ] Missing, empty, and truncated outline titles use localized UI text.
- [ ] Zoom-overlay internal link navigates to the expected page.
- [ ] Zoom-overlay text selection still works on a page with links.
- [ ] Search highlights still render in tile view and zoom overlay.
- [ ] URI link activation shows confirmation before any open.
- [ ] Cancelling URI confirmation leaves app state unchanged.
- [ ] URI copy action works if implemented.
- [ ] `https`, `http`, and `mailto` are the only externally openable schemes.
- [ ] `file` and custom schemes are not externally opened.
- [ ] Relative, empty, control-character, invalid, and over-limit URIs are not
      externally opened.
- [ ] URI open revalidation happens immediately before platform open.
- [ ] URI open path does not invoke a shell.
- [ ] Launch actions are not executed.
- [ ] Remote and embedded document actions are not executed.
- [ ] Unsupported actions are not executed.
- [ ] Stale outline/link extraction results are ignored after document change.
- [ ] All new user-facing strings are localized.
- [ ] Automated tests covering domain policy and PDF engine extraction pass.
- [ ] Manual GUI picker/drop behavior remains unchanged from RFC 025.

Release-point acceptance:

- [ ] The work is not described as released until the owner explicitly says the
      release happened.
- [ ] Release prep is not performed as part of RFC 026 unless separately
      requested.

## 22. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Link overlay breaks text selection | Scope overlay to zoom view, test click vs drag behavior, adjust pointer-event layering before review. |
| External action handling expands attack surface | Keep all external actions inert by default; allow only explicit confirmation plus open-time revalidation for `https`, `http`, and `mailto`. |
| Large outline trees affect UI responsiveness | Enforce hard node/depth/string caps, extract off the UI path, cache by generation, render tree lazily if needed. |
| Cached URI policy drifts from open policy | Store raw URI as untrusted data and revalidate in `app_services` immediately before platform open. |
| Engine returns user-facing fallback text | Return title state from extraction and localize fallback text in UI only. |
| PDFium exposes incomplete destination data | Support page-only navigation first; preserve view hints best-effort. |
| Tile grid becomes overloaded with invisible controls | Do not add tile-grid link hit areas in RFC 026. |
| Fixture generation becomes fragile | Check in a small deterministic fixture and document provenance. |

## 23. Open Questions

- Should the outline panel remember expanded/collapsed state within a document
  session? Recommended answer: yes, session-only.
- Should URI opening be implemented in the first UI slice, or should RFC 026
  first ship copy-only external URI handling? Recommended answer: internal
  links plus copy-only URI is acceptable for a smaller first review point; open
  after confirmation can be a later slice.
- Should destination view hints control zoom-overlay scroll position in the
  first implementation? Recommended answer: preserve hints in the model, apply
  page-only navigation first.
- Should outline panel be auto-opened for documents with outlines? Recommended
  answer: no, keep it user-controlled.
- Should disabled outline entries be shown? Recommended answer: show them only
  when they have useful children or document structure value; otherwise omit.

## 24. Future Work

- Precise destination positioning inside the zoom overlay.
- Named destination support if PDFium API coverage is sufficient.
- Optional persistent outline panel preference.
- Richer external URI policy settings for advanced users.
- Link hit testing in tile view if a future UX design proves it is useful and
  does not interfere with overview behavior.
- Remote document navigation, only after a separate security and UX RFC.
