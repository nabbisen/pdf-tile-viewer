---
project: PDF Tile Viewer
document_family: 2.0 corrective RFCs
language: English
date: 2026-10-21
status: Accepted (planned for 2.0.2)
accepted: 2026-10-21
baseline: PDF Tile Viewer 2.0.0
priority: Corrective
depends_on: RFC 012, RFC 013, RFC 026
amends: RFC 013 (§ shortcuts, § Escape), RFC 012 (§ keyboard)
amended: 2026-10-24 — § 5.4 two-step rollout (owner decision B; versioning review)
origin: architect onboarding review 2026-10-21 (internal), owner decisions D1
---

# RFC 033 — Keyboard Routing, Modal Layers, and Dismissal Model

## 1. Summary

Replace the current ad-hoc keyboard handling with one explicit model:

1. a **single shortcut resolver** (pure, unit-tested) that decides what a key
   means in the current context;
2. a **layer model** in which modal layers (zoom overlay, dialogs, password
   prompt) own the keyboard and editable fields opt out of viewer shortcuts
   through one shared helper;
3. a **dismissal rule**: Escape dismisses the top-most visible transient layer
   and **never navigates away from the document**;
4. a clearer **Auto / Fixed columns control** that does not rely on clicking a
   disabled input.

## 2. Motivation (evidence)

Dioxus 0.7 bubbles keyboard events from the target through every ancestor
handler until propagation is stopped (`dioxus-core` `handle_bubbling_event`).
Today:

- `.viewer` handles `z`, `+`, `=`, `-`, `0` for any descendant. Typing `z` in
  the search box toggles Zen mode and hides the search panel; typing `0`, `-`,
  `+` in the search, jump, or columns inputs changes the tile scale. This
  contradicts RFC 013 ("Shortcuts must be disabled or adjusted when focus is
  inside text input") and `shortcuts.md`.
- `ZoomOverlay` is a descendant of `.viewer` and does not stop propagation, so
  overlay keys also reach viewer handlers. Predicted effects: `+`/`-` also
  rescale the tile grid; Escape closes the overlay and then falls through the
  viewer's chain to the dashboard. (Manual QA reported Escape as passing, so
  this is verified in the evidence run before implementation — see § 9.)
- Escape currently leaves the document when nothing else is open. This is
  surprising and discards scroll and search state.
- In Auto mode the only route to Fixed columns is `onclick` on a *disabled*
  input, which browsers generally do not deliver.

## 3. Goals

- Make keyboard behavior predictable and documented in one table.
- Make it impossible for a new input or overlay to accidentally trigger viewer
  shortcuts (secure-by-construction, not by remembering `stop_propagation`).
- Give every modal layer the same keyboard and focus contract.
- Keep behavior testable without a GUI.

## 4. Non-goals

- New shortcuts, key rebinding, or a shortcut help overlay.
- A keyboard shortcut for "back to dashboard" (the Back button remains
  focusable and reachable with Tab).
- Changing what `0` resets to (owned by RFC 031).
- Changing PDF text-selection or link activation behavior (RFC 023/026).

## 5. Design

### 5.1 Layers

```text
Dialog   (external-URI dialog, password prompt)      modal
Zoom     (zoom overlay)                              modal
Search   (search panel)                              transient panel
Outline  (outline panel)                             transient panel
Zen      (distraction-free mode)                     mode
Base     (tile grid)
```

"Visible" matters: in Zen mode the Search and Outline panels are not rendered,
so they are not dismissal candidates even if their open flags are set.

### 5.2 Pure resolver (`domain::shortcuts`)

```rust
pub enum ShortcutAction { ToggleZen, ScaleUp, ScaleDown, ResetScale,
                          ZoomScaleUp, ZoomScaleDown, ZoomPrev, ZoomNext,
                          ZoomFirst, ZoomLast, Dismiss }

pub struct ShortcutContext {
    pub top_layer: Layer,      // top-most visible layer
    pub zen: bool,
}

pub fn resolve(ctx: ShortcutContext, key: &KeyDescriptor) -> Option<ShortcutAction>;
```

`KeyDescriptor` is a small domain enum (`Character(char)`, `Escape`,
`ArrowLeft`, …) so the resolver has no Dioxus dependency and is covered by
`cargo test --workspace --exclude app`. The resolver is the **only** place
that maps keys to actions; the documented table in `shortcuts.md` is generated
from the same test table (a test asserts every documented row resolves).

Rules encoded by the resolver:

- Base-layer shortcuts (`ToggleZen`, `ScaleUp/Down`, `ResetScale`) resolve only
  when `top_layer == Base` (or Zen over Base).
- Zoom-layer shortcuts resolve only when `top_layer == Zoom`.
- No action resolves while `top_layer == Dialog` except `Dismiss`.

### 5.3 Event ownership

- **Viewer root** has exactly one `onkeydown`, which converts the event to a
  `KeyDescriptor`, calls `resolve`, and dispatches.
- **Modal layers** (zoom overlay, dialogs, password prompt) attach their own
  `onkeydown` at their root, handle their keys via the resolver, and **always
  call `stop_propagation()`** at that root. Nothing a modal receives can reach
  the viewer root.
- **Editable controls** (search input, jump input, columns number input,
  password input) are built through one helper, `editable_field_keys(on_escape)`,
  which stops propagation for every key and forwards only `Escape` to the
  dismissal callback. New editable controls must use the helper; a test-time
  check (grep-based unit test in `app`) fails if an `input`/`textarea` is added
  without it.

### 5.4 Dismissal

`Escape` → `Dismiss` → close the top-most **visible** layer in this order:
`Dialog` → `Zoom` → `Search` → `Outline` → `Zen` (exit). Exactly one layer is
closed per press and the event never reaches a second handler.

**Two-step rollout (amendment 2026-10-24, owner decision B).** With nothing to
dismiss:

| Release | Escape behavior when no layer is open and the focus is inside the viewer |
|---|---|
| **2.0.x (this RFC's first release, planned 2.0.2)** | **unchanged and documented:** returns to the dashboard (`shortcuts.md`: "close zoom overlay → close search → exit zen → back to dashboard"). Only the *bug* is fixed: an Escape that closes a layer must not also leave the document |
| **2.1.0** | Escape does nothing; leaving the document is done only with the Back button. `Phase` is never changed by Escape |

Reason: removing a documented shortcut behavior is not a bug fix and does not
belong in a patch release (semantic-versioning review, 2026-10-24); a minor
release is where users expect behavior to change. Implementation: the resolver
returns `Dismiss` with a `NothingToDismiss` outcome; a single constant
`ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE: bool` (in `domain::shortcuts`) selects the
2.0.x or 2.1.0 behavior, with unit tests for both values. The 2.1.0 step is
tracked in `ROADMAP.md`; the CHANGELOG of that release lists it under "Changed".

### 5.5 Modal focus contract

Every modal layer: (1) moves focus to its first meaningful control on open;
(2) traps Tab within the layer; (3) restores focus to the opener on close
(zoom overlay → originating tile; dialogs → the control that opened them).
The external-URI dialog already does this with scripted focus; the password
prompt and zoom overlay are brought to the same contract using one shared
helper rather than per-component scripts.

### 5.6 Columns control

Replace the Auto button plus disabled input with a **segmented control**
`[Auto | Fixed]` and a number stepper that is enabled only in Fixed. Switching
to Fixed starts from the **currently displayed column count** (not a hard-coded
5), so the grid does not visibly jump. Range remains 1–24.

## 6. Alternatives considered

- **`stop_propagation` on each input only.** Rejected: every future input must
  remember it.
- **JavaScript `document.activeElement` check in the viewer handler.**
  Rejected: asynchronous (cannot `prevent_default` in time), adds JS logic to a
  Rust-first app.
- **Keep Escape-to-dashboard.** Rejected as the end state (owner decision D1): it is a
  navigation action hidden behind a dismissal key. Retained temporarily in 2.0.x
  only to respect semantic versioning (§ 5.4).

## 7. Security and privacy

No new data flows. The change reduces accidental activation of actions while a
password is being typed (the password input uses the editable helper).

## 8. Tests and verification

- `domain::shortcuts` unit tests: full context × key matrix, including
  "no base shortcut resolves in Dialog/Zoom" and the dismissal order table.
- Resolver ↔ documentation test (every documented shortcut resolves).
- Manual QA matrix (Linux WebKitGTK, Windows WebView2, macOS WKWebView):
  type `z`, `0`, `-`, `+`, `=` in each input; `Escape` in every layer
  combination including Zen with the search flag set; Tab trap and focus
  restore for zoom, external-URI dialog, password prompt; Auto→Fixed keeps the
  visible column count.

## 9. Preconditions

The evidence run (WP0) must confirm or withdraw the zoom-overlay propagation
and disabled-input findings before the handoff is issued.

## 10. Acceptance criteria

- No viewer shortcut fires while any editable control has focus.
- No overlay or dialog key reaches the viewer root.
- Escape closes at most one layer per press and no closing press also leaves the
  document; dismissal order matches § 5.4. (The idle Escape → dashboard fallback
  remains in 2.0.x and is removed in 2.1.0.)
- Auto→Fixed starts at the visible column count; Fixed→Auto restores auto.
- `shortcuts.md` documents the same table the resolver implements.

## 11. Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| WebView differences in focus handling | Medium | Manual matrix on all three engines before acceptance |
| Helper bypassed by a future input | Medium | Grep-based test plus review checklist |
| Behavior change surprises existing users (Escape) | Low | Idle fallback kept until 2.1.0; CHANGELOG "Changed" entry then; Back button remains visible |
