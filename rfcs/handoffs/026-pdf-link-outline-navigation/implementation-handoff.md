# Handoff — RFC 026 PDF Link and Outline Navigation

## 1. Summary

RFC 026 has been implemented in four reviewed slices:

- PR 1: domain navigation model, URI policy, PDFium outline extraction, and
  active-page link extraction;
- PR 2: app-service outline cache and outline side-panel UI;
- PR 3: zoom-overlay internal PDF link hit areas;
- PR 4: copy-only external URI confirmation dialog.

After PR 4 cleanup review, an unused internal-link helper was removed from the
zoom overlay utility module and the active `link_activation_at()` coverage was
kept.

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
- During manual QA on Linux WebKitGTK, picker open worked but dashboard
  drag/drop did not work under the observed Wayland session or an
  `GDK_BACKEND=x11` comparison run. This is deferred to Future RFC-F07 because
  it is not specific to RFC 026 link/outline behavior.

## 6. Manual QA result

Detailed test steps and an evidence template are in
`rfcs/handoffs/026-pdf-link-outline-navigation/manual-qa-checklist.md`.

Manual WebView QA passed for RFC 026 link/outline behavior after the follow-up
fixture, disabled-action, link-affordance, and dialog-focus fixes:

- no-outline and outline behavior passed;
- outline navigation passed;
- zoom-overlay internal link navigation passed;
- zoom-overlay text selection over linked pages passed;
- tile-grid and zoom-overlay search highlight preservation passed;
- external URI copy-only dialog behavior passed;
- Launch, remote, embedded, and JavaScript action non-execution passed;
- picker behavior and non-`.pdf` rejection passed.

Linux dashboard drag/drop failed during QA and is deferred to Future RFC-F07.
The picker remains the reliable intake path.

## 7. Gate status

Automated Rust gates were run and reviewed during the individual PR slices. The
following gates were also observed passing after the cleanup-docs slice and
dead-code warning cleanup:

- `cargo fmt --check`
- `cargo test -p domain` — 35 passed
- `cargo test -p app_services` — 53 passed
- `PDF_TILE_VIEWER_PDFIUM_DIR="$(pwd)/ci/.pdfium" cargo test -p pdf_engine --test smoke` — 14 passed
- `cargo test -p app` — 19 passed
- `cargo check --workspace`
- `RUSTFLAGS="-D warnings" cargo check --quiet --all-targets --all-features`
- `cargo test --workspace --exclude app`
- `mdbook build docs`

Manual QA evidence is recorded in
`rfcs/handoffs/026-pdf-link-outline-navigation/manual-qa-checklist.md`.

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

Review the final RFC 026 follow-up diff. After acceptance and commit, move
`rfcs/proposed/026-pdf-link-outline-navigation.md` to `rfcs/done/` and update
`rfcs/README.md`. Release preparation should remain a separate owner-requested
step.
