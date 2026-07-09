# Handoff — RFC 026 PDF Link and Outline Navigation

## 1. Summary

RFC 026 has been implemented in four reviewed slices:

- PR 1: domain navigation model, URI policy, PDFium outline extraction, and
  active-page link extraction;
- PR 2: app-service outline cache and outline side-panel UI;
- PR 3: zoom-overlay internal PDF link hit areas;
- PR 4: copy-only external URI confirmation dialog.

The RFC remains in `rfcs/proposed/` until final manual QA and owner acceptance
are complete. This handoff is not release prep and is not a release point.

## 2. Files and surfaces changed

- `crates/domain/src/navigation.rs`
- `crates/pdf_engine/src/navigation.rs`
- `crates/app_services/src/navigation_service.rs`
- `crates/app_services/src/uri_policy.rs`
- `crates/app/src/components/outline_panel.rs`
- `crates/app/src/components/zoom_overlay.rs`
- `crates/app/src/components/zoom_overlay/util.rs`
- `crates/app/src/screens/viewer.rs`
- `crates/app/src/i18n.rs`
- `crates/app/src/i18n/en.rs`
- `crates/app/src/i18n/ja.rs`
- `crates/app/assets/main.css`
- `docs/src/new-users/outlines-and-links.md`
- `docs/src/new-users/features.md`
- `docs/src/new-users/tile-grid.md`
- `docs/src/intermediate/shortcuts.md`

## 3. Review record

Design reviews:

- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_design_review.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_design_rereview.md`

Implementation reviews:

- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr1_implementation_review.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr1_implementation_rereview.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr2_outline_ui_implementation_review.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr3_zoom_link_overlay_implementation_review.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr3_zoom_link_overlay_rereview.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr3_zoom_link_overlay_rereview2.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr4_external_uri_copy_confirmation_review.md`
- `.git-exclude/reviewed/pdf_tile_viewer_rfc026_pdf_link_outline_navigation_pr4_external_uri_copy_confirmation_rereview.md`

## 4. Current behavior

- The viewer toolbar exposes an outline button.
- The outline panel loads document outlines through `NavigationService` and
  scrolls the tile grid to supported page destinations.
- Nested outline entries render as expandable/collapsible tree nodes.
- The zoom overlay extracts links for the active page only.
- Internal PDF links navigate to the linked page.
- External URI links open an in-app confirmation dialog.
- The external URI dialog never launches a browser, email client, shell, or
  other external application. It lets the user copy the raw target.
- Unsupported PDF actions remain inert.

## 5. Known limitations and follow-up candidates

- Link hit areas are intentionally limited to the zoom overlay. Tile-grid link
  activation is out of scope.
- Destination view hints are preserved in the domain model but page-only
  navigation is the first implemented behavior.
- The page-link cache is bounded FIFO, not a true recency-updating LRU.
- The link hover affordance is intentionally weak because pointer handling is
  tuned to preserve text selection.
- External URI opening is not implemented; copy-only inspection is the current
  accepted behavior.

## 6. Manual QA still needed before RFC completion

- PDF with no outline opens normally.
- PDF with outline shows the outline panel and navigates to expected pages.
- Nested outline entries expand, collapse, and keep stable indentation.
- Zoom-overlay internal links navigate to expected pages.
- Zoom-overlay text selection still works over linked text.
- Search highlights still render in tile view and zoom overlay.
- External URI activation shows the confirmation dialog.
- Cancel and Escape close the external URI dialog without changing document
  state.
- Copy reports success or failure clearly.
- Focus is trapped inside the dialog while open and returns to the zoom overlay
  close button when closed.
- GUI picker/drop behavior remains unchanged from RFC 025.

## 7. Gate status

Automated Rust gates were run and reviewed during the individual PR slices.
For a final RFC 026 acceptance point, rerun the agreed current-thread gates and
record the observed output in the review or release-prep notes.

Recommended final gates:

```sh
cargo fmt --check
cargo test -p domain
cargo test -p app_services
PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine --test smoke
cargo test -p app
cargo check --workspace
cargo test --workspace --exclude app
mdbook build docs
```

## 8. Next step

Complete the manual QA checklist above, then decide whether to move
`rfcs/proposed/026-pdf-link-outline-navigation.md` to `rfcs/done/` and update
`rfcs/README.md`. Release preparation should remain a separate owner-requested
step.
