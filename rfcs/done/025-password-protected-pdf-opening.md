---
project: PDF Tile Viewer
document_family: Dioxus + embedded/bundled PDFium migration RFCs
language: English
date: 2026-07-07
status: Implemented (unreleased)
baseline: PDF Tile Viewer 2.0.0-beta.10
depends_on: RFC 024
review: .git-exclude/reviewed/pdf_tile_viewer_rfc025_password_protected_pdf_opening_design_review.md
implementation_review: .git-exclude/reviewed/pdf_tile_viewer_rfc025_password_protected_pdf_opening_implementation_review.md
---

# RFC 025 — Password-Protected PDF Opening Workflow

## 1. Summary

Add a password-entry workflow for encrypted PDF files that require a user
password to open.

RFC 024 made password-required PDFs deterministic: PDFium's typed password
error now maps to `DocumentError::EncryptedUnsupported`, and the app shows a
clear unsupported message. RFC 025 turns that reliable detection into a
recoverable open flow:

1. user opens or drops a password-protected PDF;
2. the app detects that a password is required;
3. the app prompts for a password without creating a document session yet;
4. the user can retry, cancel, or successfully unlock the document;
5. a successful unlock creates a normal document session with encrypted
   metadata set.

The password is transient input. It must not be saved to settings, recent-file
history, logs, diagnostics, review packages, or release artifacts.

## 2. Motivation

Password-protected PDFs are common in invoices, reports, bank documents,
contracts, and internal business workflows. After RFC 024, PDF Tile Viewer
correctly tells users that these files are unsupported, but that is still a
workflow dead end.

Opening password-protected PDFs is the natural next step because:

- the app now has typed PDFium evidence for the password-required path;
- the existing error state gives a clear place to branch into a prompt;
- password support restores a common PDF viewer expectation before 2.0 RC;
- the implementation can stay narrow if it handles user-password opening only.

This RFC treats password support as an open-lifecycle feature, not as a
credential-management feature.

## 3. Feature Contract

When a PDF requires a password, PDF Tile Viewer must:

- prompt the user for a password after the first password-required failure;
- allow submit, retry after wrong password, and cancel;
- open the document when PDFium accepts the supplied password;
- leave the app on the dashboard when the user cancels or all attempts fail;
- avoid creating a document session until the password succeeds;
- never persist the password;
- never include the password in user-visible errors, logs, settings, recent
  files, or test artifacts;
- treat picker and drag/drop opens consistently.

When the document is successfully opened with a password, the resulting
`DocumentMetadata` should record `encrypted = true`. This is document metadata,
not password metadata.

## 4. Goals

- Add an explicit password-required open path across `domain`,
  `pdf_engine`, `app_services`, and `app`.
- Add a password prompt UI for the open flow.
- Support retry when PDFium rejects the supplied password.
- Support cancel without changing the current session.
- Keep passwords memory-only and short-lived.
- Preserve all existing unencrypted PDF behavior.
- Keep the tile view, zoom overlay, search, and text-selection behavior working
  after a protected document is unlocked.
- Add fixture-backed automated coverage for correct-password and wrong-password
  paths when PDFium is available.
- Keep docs accurate for the new support state.

## 5. Non-Goals

- Remembering passwords.
- OS keychain integration.
- Password hints or password recovery.
- Editing, decrypting, exporting, or re-saving decrypted PDFs.
- Changing file permissions or owner-password policy.
- Enforcing PDF permission bits for copy, print, or extraction in this RFC.
- Supporting certificates, public-key encryption, or DRM-like workflows.
- Multi-document password cache.
- Command-line password input.
- Web/WASM password support.

## 6. Current State

The current open path is:

```text
app
  open_action(path)
    app_services::document_service::open_document(engine, path)

app_services
  validate_candidate(path)
  engine.open_document(path)

pdf_engine
  PdfEngine::open_document(path)
    pdfium.load_pdf_from_file(path, None)
```

When PDFium reports `PdfiumInternalError::PasswordError`, the engine maps that
to:

```text
DocumentError::EncryptedUnsupported
  -> OpenError::Engine(DocumentError::EncryptedUnsupported)
  -> MessageKey::ErrEncryptedUnsupported
  -> "Password-protected PDFs are not supported yet."
```

That path should remain as the fallback when password support is not active,
but RFC 025 adds a more specific service result that lets the app prompt
instead of ending the flow immediately.

## 7. Proposed Design

### 7.1 Open Attempt Model

Add a password-aware open operation at the service boundary.

Recommended domain/service model:

```rust
pub enum OpenDocumentOutcome {
    Opened(DocumentSession),
    PasswordRequired(PasswordRequiredContext),
}

pub struct PasswordRequiredContext {
    pub path: PathBuf,
}
```

The exact type location may be adjusted during implementation. The important
contract is that "password required" is not represented only as a display
message once the app can recover from it.

Recommended service methods:

```rust
pub async fn open_document(
    engine: &EngineHandle,
    path: &Path,
) -> Result<OpenDocumentOutcome, OpenError>;

pub async fn open_document_with_password(
    engine: &EngineHandle,
    path: &Path,
    password: DocumentPassword,
) -> Result<OpenDocumentOutcome, OpenError>;
```

`open_document()` validates the candidate and calls the engine without a
password. If the engine reports password required, the service returns
`Ok(OpenDocumentOutcome::PasswordRequired(...))`.

`open_document_with_password()` reuses the same intake validation and calls the
engine with the supplied password. If PDFium reports another password error,
the service again returns `PasswordRequired`, allowing the UI to show a wrong
password state and retry.

The method names are illustrative. If implementation prefers a single method
with an `Option<DocumentPassword>`, the API must still make the retry semantics
clear and must avoid accidental password persistence.

### 7.2 Password Type

Introduce a small domain-owned password wrapper so password-bearing APIs are
visible in signatures:

```rust
pub struct DocumentPassword(String);
```

Recommended rules:

- do not implement `Display`;
- do not derive `Debug`;
- do not derive `Clone` unless implementation proves it is needed;
- expose only the minimal borrowed string access needed by `pdf_engine`;
- document that this reduces accidental exposure but does not guarantee
  zeroization.

Optional hardening:

- future work may consider a zeroization dependency, but the first
  implementation should stay dependency-free and avoid implying a stronger
  memory-erasure guarantee than Rust `String` storage can provide.

This RFC does not require perfect memory erasure. It does require that the app
does not intentionally persist, log, display, or serialize passwords.

### 7.3 Engine API

Extend the PDF engine worker to accept an optional password for opening:

```rust
pub fn open_document_with_password(
    &self,
    path: PathBuf,
    password: Option<DocumentPassword>,
) -> impl Future<Output = Result<Result<DocumentSession, DocumentError>, EngineGone>>;
```

The existing `open_document(path)` method may remain as a convenience wrapper
for `None`.

Inside `PdfEngine`, pass the password to PDFium:

```rust
pdfium.load_pdf_from_file(path, password.as_deref())
```

The concrete adapter depends on the `pdfium-render` API. The password value
must not cross the worker boundary except as part of the explicit open command.
The worker must drop the command after the open attempt completes.

### 7.4 Domain Error Semantics

RFC 024's `DocumentError::EncryptedUnsupported` name is now too final for a
recoverable password prompt. Add a specific recoverable error:

```rust
pub enum DocumentError {
    ...
    PasswordRequired,
    EncryptedUnsupported,
    ...
}
```

Recommended semantics:

| Error | Meaning |
|---|---|
| `PasswordRequired` | PDFium reports password required or wrong password during an open attempt where the UI may prompt/retry. |
| `EncryptedUnsupported` | The app cannot support this encrypted document class. |

For the first RFC 025 implementation, PDFium `PasswordError` should map to
`PasswordRequired`. The app can then decide whether to prompt. If a future
PDFium error proves that a document is encrypted but cannot be opened with a
user password, that path should map to `EncryptedUnsupported`.

Backward compatibility rule:

- user-facing copy for old unsupported encrypted paths may remain available;
- docs and UI should stop saying password-protected PDFs are unsupported once
  this feature ships.
- `OpenError::Engine(DocumentError::PasswordRequired)` should not normally
  reach the display-error path because the service should convert it into
  `OpenDocumentOutcome::PasswordRequired`. As defense-in-depth,
  `open_error_key()` must still handle it explicitly, map it to the existing
  encrypted/password fallback message, and use a debug assertion so future
  regressions are noticed during development.

### 7.5 Metadata

For every successfully opened document, compute `DocumentMetadata.encrypted`
from PDFium permissions metadata rather than from whether the user supplied a
password.

```rust
let encrypted = !matches!(
    document.permissions().security_handler_revision(),
    Ok(PdfSecurityHandlerRevision::Unprotected)
);
```

Reason: some PDFs are encrypted but open without a user password, for example
owner-password / permission-restricted PDFs with an empty user password. A
password-success heuristic would incorrectly mark those documents as
unencrypted.

If PDFium returns an error while reading the security-handler revision, the
implementation must choose a conservative, documented fallback. Recommended
fallback: treat the document as encrypted unless the revision is explicitly
`Unprotected`.

### 7.6 UI Flow

The password prompt is part of the open workflow and should appear over the
dashboard or current screen without navigating to a failed document page.

Required states:

```text
idle
opening_without_password
password_required(path)
opening_with_password(path)
wrong_password(path)
failed(non_password_error)
opened
cancelled
```

User actions:

| Action | Result |
|---|---|
| Submit non-empty password | Retry open with that password. |
| Submit empty password | Allowed; some PDFs may use an empty user password. |
| Cancel | Close prompt and leave current screen/session unchanged. |
| Escape / close button | Same as cancel. |
| Wrong password | Keep prompt open, clear the password field, show concise error. |
| Successful password | Close prompt and navigate to viewer. |

The prompt should use a native password input type where available, so the
password is masked by default.

Recommended English copy:

```text
Title: Password required
Body: This PDF is password-protected.
Field label: Password
Wrong password: The password was not accepted.
Submit: Open
Cancel: Cancel
```

Copy must go through the i18n catalog. Do not add hardcoded UI strings.

### 7.7 Phase and Overlay Integration

The password prompt must be modeled as an overlay or separate signal that does
not replace the current `Phase`.

The existing app phase model is:

```text
Dashboard | Opening | Viewer(view)
```

RFC 025 must preserve the user's current screen while a password-protected
second document is being opened. If a user is viewing one document and opens
another file that requires a password:

- the current viewer remains the current phase while the password prompt is
  visible;
- cancelling the prompt returns to that same viewer;
- submitting a wrong password keeps that same viewer behind the prompt;
- only a successful unlock replaces the current viewer with the new document;
- non-password failures restore the previous phase instead of unconditionally
  returning to the dashboard.

Implementation may either defer the eager `Phase::Opening` transition until an
open is known to proceed without a prompt, or record and restore the previous
phase on cancel/failure. The hard requirement is that password prompt state is
path-specific and stale completions cannot replace the wrong screen.

### 7.8 Picker and Drag/Drop Consistency

Both file picker and drag/drop open flows should use the same service path.

Expected behavior:

- picker encrypted PDF -> prompt;
- drag/drop encrypted PDF -> prompt;
- wrong password in either path -> same retry state;
- cancel in either path -> dashboard or current document remains unchanged;
- successful unlock in either path -> viewer opens normally.

If the user opens a second file while a password prompt is visible, the app
should cancel the old prompt and start the new open flow. It should not keep a
stale password prompt for a path the user has moved away from.

### 7.9 Search and Text Selection After Unlock

Once unlocked, a password-protected PDF should behave like an opened document
for existing read-only workflows:

- tile rendering;
- lazy render cache;
- page navigation;
- search;
- zoom overlay;
- single-page text selection.

The engine worker may keep PDFium document handles internally after successful
open, as it already does for normal documents. No password should be needed for
later page rendering/search/text extraction if PDFium keeps the document open.

### 7.10 Documentation

Update user docs from "not supported yet" to a supported workflow:

- opening guide: password-protected PDFs prompt for a password;
- FAQ: passwords are never saved and must be re-entered on future app launches;
- settings docs: no setting persists PDF passwords;
- contributor docs: document encrypted fixture expectations if tests change.

Do not recommend decrypting PDFs externally as the primary workflow once the
feature ships.

## 8. Security and Privacy Rules

- Never log passwords.
- Never write passwords to settings.
- Never write passwords to recent-file history.
- Never include passwords in `Debug` output.
- Never add passwords to panic messages or `expect()` strings.
- Do not store passwords in global state.
- Do not cache passwords after a successful open.
- Do not include password values in review request packages.
- Treat fixture passwords as public test data and label them as such.
- Keep OS keychain integration out of scope unless a later RFC designs it.

The app may hold a password in component state while the prompt is open and in
an engine command while the open attempt is in progress. That is the intended
maximum lifetime.

## 9. Edge Cases

| Case | Expected behavior |
|---|---|
| Valid unencrypted PDF | Opens without prompt. |
| Encrypted PDF requiring password | Prompt appears. |
| Correct password | Document opens; `metadata.encrypted = true`. |
| Wrong password | Prompt remains open; password field clears; concise wrong-password error appears. |
| Empty password | Submitted as a real PDFium attempt, not blocked by validation; smoke tests must pin the observed behavior of `Some("")` against the encrypted fixture. |
| User cancels prompt | No new session; existing screen/session remains unchanged. |
| User opens another file while prompt is visible | Old prompt is cancelled; new open flow starts. |
| PDF deleted before password submit | Show file-not-found/unreadable error; close or reset prompt. |
| PDF changed between prompt and submit | Retry against current file bytes; no stale assumptions. |
| Engine dies during retry | Show existing PDF engine unavailable error. |
| PDFium returns non-password parse error after password | Show parse error, not wrong-password. |
| Owner-password-only restrictions | Open if PDFium accepts it; permission enforcement is future work. |

## 10. Alternatives Considered

### 10.1 Keep Password-Protected PDFs Unsupported

This is the current RFC 024 outcome. It is reliable but incomplete for common
user workflows. It should not be the preferred 2.0 RC endpoint unless password
support proves risky.

### 10.2 Ask For Password Before Trying To Open

Rejected. Most PDFs do not need a password, so pre-prompting adds friction.
PDFium can detect password-required files, and RFC 024 already proved that path.

### 10.3 Persist Passwords For Recent Files

Rejected. This creates credential storage, keychain, expiry, and threat-model
questions that are not required for a local viewer. Users can re-enter the
password when needed.

### 10.4 Decrypt To A Temporary File

Rejected. The viewer should not create decrypted copies of user documents. It
should pass the password to PDFium and keep the loaded document in memory.

### 10.5 Convert `EncryptedUnsupported` In Place

Rejected. The existing error name is useful for genuinely unsupported encrypted
classes, but password-required is recoverable. A separate `PasswordRequired`
state makes the app behavior clearer.

## 11. Implementation Handoff

### Summary

Implement password-protected PDF opening as an open-flow enhancement. Reuse RFC
024's typed password detection, but convert the user-facing behavior from
"unsupported" to "prompt and retry".

### Scope

Expected code areas:

- `crates/domain/src/document.rs`
- possible new domain password module or type
- `crates/pdf_engine/src/engine.rs`
- `crates/pdf_engine/src/worker.rs`
- `crates/pdf_engine/tests/smoke.rs`
- `crates/app_services/src/document_service.rs`
- `crates/app/src/app.rs`
- `crates/app/src/i18n.rs`
- `crates/app/src/i18n/en.rs`
- `crates/app/src/i18n/ja.rs`
- app tests under `crates/app/src/tests/`
- user docs under `docs/src/`

Avoid unrelated packaging, PDFium bundling, or release-script changes.

### Design Decisions

- Password-required is recoverable and should be modeled separately from
  encrypted-unsupported.
- Passwords are explicit values in function signatures, not hidden strings.
- Passwords are memory-only and short-lived.
- Wrong password and missing password both come from PDFium `PasswordError`,
  but the UI can distinguish them by whether the user has already submitted a
  password in this prompt flow.
- The first implementation should document "best effort" memory handling rather
  than adding a zeroization dependency.
- `DocumentMetadata.encrypted` should be computed from PDFium's security
  handler revision for every opened document.
- Password prompt state must not replace the current app `Phase`; cancel,
  wrong-password, and non-password failure paths must restore or preserve the
  prior dashboard/viewer state.

### Implementation Notes

- Keep `open_document(path)` as a compatibility wrapper if that reduces churn.
- Add password-bearing worker commands only for document open. Render, search,
  and text extraction should continue to use the already-open document handle.
- Make prompt state path-specific so stale retries do not apply to the wrong
  file.
- Clear the password field after every failed attempt.
- Do not format password-bearing types with `{:?}`.
- Keep all prompt copy in the i18n catalog.

### Tests and Gates To Run

Required for implementation review:

- `cargo fmt --check`
- `cargo test -p pdf_engine`
- `cargo test --workspace --exclude app`
- `cargo test -p app`
- `cargo check --workspace`
- `git diff --check`

Required when local PDFium is available:

- `cargo test -p pdf_engine --test smoke`

Recommended before release prep:

- `mdbook build docs`
- manual picker and drag/drop QA with the encrypted fixture

## 12. Task Breakdown / PR Plan

### PR 1 — Domain and Engine Password Path

- Add `DocumentPassword` or equivalent.
- Add `DocumentError::PasswordRequired`.
- Extend `PdfEngine` and `EngineHandle` open calls to accept an optional
  password.
- Map PDFium `PasswordError` to `PasswordRequired`.
- Preserve convenience behavior for ordinary `open_document(path)`.
- Compute `DocumentMetadata.encrypted` from
  `document.permissions().security_handler_revision()` for all opened
  documents.
- Add unit coverage for mapper behavior.

Review focus:

- password value does not leak via traits or logs;
- ordinary PDF opens are unchanged;
- worker command lifetime is short and explicit;
- `EncryptedUnsupported` remains available for truly unsupported encrypted
  classes.
- encryption metadata is not inferred only from password submission.

### PR 2 — Service Outcome and App Prompt

- Add a service-level `OpenDocumentOutcome`.
- Convert password-required open failures into prompt state.
- Add password prompt component/state in the app.
- Implement submit, retry, cancel, and stale-path handling.
- Add i18n keys and translations.
- Add an explicit `open_error_key()` fallback for leaked
  `DocumentError::PasswordRequired`, with a debug assertion.
- Add app tests for error-key and prompt-state behavior where practical.

Review focus:

- picker and drag/drop use the same path;
- cancel leaves existing state unchanged;
- wrong-password clears the input and does not navigate;
- no hardcoded UI strings are introduced.
- the prompt is an overlay/signal and does not destroy the current viewer
  before the user succeeds or cancels.

### PR 3 — Fixture-Backed Smoke Coverage and Docs

- Rename the RFC 024 smoke test from
  `encrypted_pdf_reports_encrypted_unsupported` to
  `encrypted_pdf_reports_password_required`, and change the expected result to
  `DocumentError::PasswordRequired`.
- Extend the encrypted fixture smoke tests to cover correct password, wrong
  password, and empty-password behavior.
- Confirm unlocked encrypted documents can render at least one page.
- Assert fixture bytes are unchanged before/after required-password,
  wrong-password, empty-password, and correct-password attempts.
- Assert wrong-password attempts create no document session and do not add to
  the engine session map.
- Confirm existing search/text-layer smoke tests still pass on normal fixtures.
- Update opening guide, FAQ, and settings docs.
- Update contributor docs only if test setup needs explanation.

Review focus:

- fixture password is public test data only;
- correct-password test proves a real opened session;
- docs no longer claim password-protected PDFs are unsupported;
- no password appears in generated logs or diagnostics.

These PRs may be combined if the implementation remains small, but the review
should still evaluate the three concerns separately.

## 13. Acceptance / QA Checklist

### Automated

- [ ] Opening a normal PDF without password still succeeds.
- [ ] Opening the encrypted fixture without password returns
      `PasswordRequired`.
- [ ] Opening the encrypted fixture with a wrong password returns
      `PasswordRequired`.
- [ ] Opening the encrypted fixture with `Some("")` has explicitly asserted
      behavior, matching observed PDFium behavior for the fixture.
- [ ] Opening the encrypted fixture with the public fixture password succeeds.
- [ ] All encrypted fixture open attempts leave fixture bytes unchanged.
- [ ] Wrong-password attempts create no document session and do not add to the
      engine session map.
- [ ] Successfully unlocked fixture session has `metadata.encrypted = true`
      because PDFium reports a protected security-handler revision.
- [ ] Successfully unlocked fixture can render page 1.
- [ ] Password-bearing types do not implement `Display` or accidental `Debug`.
- [ ] `open_error_key()` explicitly handles leaked `PasswordRequired` as a
      debug-asserting fallback.
- [ ] App i18n completeness tests include all new password prompt keys.
- [ ] Existing RFC 023 text-layer tests still pass.
- [ ] Existing RFC 024 encrypted detection smoke coverage is renamed and
      repurposed, not silently removed.
- [ ] `cargo test --workspace --exclude app` passes.
- [ ] `cargo test -p app` passes.
- [ ] `cargo check --workspace` passes.
- [ ] `git diff --check` passes.

### Manual QA

- [ ] Open a normal PDF through the picker; no password prompt appears.
- [ ] Drop a normal PDF; no password prompt appears.
- [ ] Open the encrypted fixture through the picker; password prompt appears.
- [ ] Drop the encrypted fixture; same prompt appears.
- [ ] Submit a wrong password; prompt remains open and password input clears.
- [ ] Submit the correct fixture password; viewer opens.
- [ ] Cancel the prompt; no document session is created.
- [ ] Press Escape or close the prompt; behavior matches cancel.
- [ ] Start from an already-open document, open an encrypted second document,
      then cancel; the original viewer remains visible.
- [ ] Start from an already-open document, open an encrypted second document,
      then submit a wrong password; the original viewer remains behind the
      prompt.
- [ ] Open another file while prompt is visible; old prompt is dismissed.
- [ ] After unlocking, tile rendering works.
- [ ] After unlocking, search works.
- [ ] After unlocking, zoom overlay opens.
- [ ] After unlocking, text selection in zoom overlay works for selectable text.
- [ ] Close and relaunch the app; the password is not remembered.
- [ ] Confirm recent files/history do not expose password text.

### Documentation QA

- [ ] Opening guide describes the password prompt.
- [ ] FAQ says passwords are not saved.
- [ ] Settings docs do not imply password persistence.
- [ ] Contributor docs identify fixture passwords as public test data if
      mentioned.

## 14. Risks

| Risk | Mitigation |
|---|---|
| Password accidentally appears in debug output | Use a wrapper type without `Display`/derived `Debug`; avoid logging open commands. |
| App stores password in component state longer than needed | Clear prompt state after success, cancel, and failed path replacement. |
| Wrong-password and unsupported-encryption get conflated | Add `PasswordRequired` separately from `EncryptedUnsupported`; keep tests for both. |
| PDFium requires password lifetime longer than expected | Verify successful render/search after opening; keep document handle owned by worker. |
| Empty password is incorrectly blocked | Treat empty string as a valid PDFium attempt. |
| UI prompt races with another open request | Store path/generation with prompt state and ignore stale completions. |
| `metadata.encrypted` becomes misleading | Compute it from PDFium's security-handler revision for every opened document. |

## 15. Review Decisions and Remaining Questions

Settled by design review:

- `DocumentPassword` may live in `domain::document`; a separate password module
  is optional tidiness, not required.
- Do not add `zeroize` in the first implementation.
- Replace the RFC 024 encrypted fixture expectation with
  `PasswordRequired`; do not preserve a parallel
  `EncryptedUnsupported` load-path expectation for `PasswordError`.
- Use "The password was not accepted." rather than "wrong password."
- Leave owner-password permission enforcement as future work.

Remaining implementation-time checks:

- Empirically pin `Some("")` password behavior against the encrypted fixture.
- Confirm the PDFium security-handler revision API behaves as expected for the
  existing unencrypted and encrypted fixtures.
- Decide whether prompt state is implemented as a dedicated overlay signal or
  as phase restoration around a deferred open transition.

## 16. Future Work

- Permission-aware behavior for copy, print, extraction, and metadata access.
- Optional command-line file opening flow.
- Optional OS keychain integration, only with a dedicated security RFC.
- Certificate-encrypted PDFs if PDFium and product requirements justify it.
- Web/WASM password prompt support if the offline web target is pursued.
