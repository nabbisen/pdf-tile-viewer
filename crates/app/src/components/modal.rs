//! The shared modal focus contract (RFC 033 § 5.5), implemented once.
//!
//! Every modal layer (zoom overlay, external-link dialog, password prompt):
//! 1. moves focus to its first meaningful control when it opens,
//! 2. keeps `Tab` inside the layer,
//! 3. makes everything outside it **inert** while it is open (falls back to `aria-hidden` if the
//!    web view has no `inert`), and
//! 4. restores focus to the control that opened it when it closes.
//!
//! The layer's component calls [`use_modal`] once; its root `onkeydown` calls [`modal_key`].
//! The scripting is one small helper object injected into the page (`window.__ptvModal`), keyed
//! by element ids. [`focus_by_id`] is the same retrying focus used by the panel and Zen rules.

use dioxus::document::eval;
use dioxus::prelude::*;

use domain::shortcuts::{KeyDescriptor, Layer, ShortcutAction, ShortcutContext, resolve};

use crate::keys::key_descriptor;

/// Defines `window.__ptvModal` once per page. Idempotent.
const HELPER: &str = r#"
if (!window.__ptvModal) {
  // Retry a few frames: the element may not be in the DOM yet when the script runs.
  const later = (f) => { let n = 0; const step = () => { if (f() || ++n > 40) return; requestAnimationFrame(step); }; step(); };
  const focusable = (root) => Array.from(root.querySelectorAll('button, [href], input, select, textarea, [tabindex]'))
    .filter((e) => !e.disabled && e.tabIndex >= 0 && e.getClientRects().length > 0 && !e.closest('[inert]'));
  // Everything that is not on the path from the document body to `root` becomes inert.
  const isolate = (root) => {
    const marks = [];
    for (let node = root; node && node !== document.body && node.parentElement; node = node.parentElement) {
      for (const sib of Array.from(node.parentElement.children)) {
        if (sib === node || ['SCRIPT', 'STYLE', 'LINK'].includes(sib.tagName)) continue;
        sib.__ptvIso = (sib.__ptvIso || 0) + 1;
        if ('inert' in sib) sib.inert = true; else sib.setAttribute('aria-hidden', 'true');
        marks.push(sib);
      }
    }
    return marks;
  };
  const release = (marks) => {
    for (const s of marks) {
      s.__ptvIso = Math.max(0, (s.__ptvIso || 1) - 1);
      if (s.__ptvIso === 0) { if ('inert' in s) s.inert = false; else s.removeAttribute('aria-hidden'); }
    }
  };
  const state = { cur: null, prev: null, stacks: {}, later, focusable, isolate, release,
                  mode: ('inert' in document.documentElement) ? 'inert' : 'aria-hidden' };
  document.addEventListener('focusin', (e) => { if (e.target !== state.cur) { state.prev = state.cur; state.cur = e.target; } });
  window.__ptvModal = state;
}
"#;

fn js_str(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn run(call: &str) {
    let _ = eval(&format!("{HELPER}\n{call}"));
}

/// Focus the element with `id`, retrying for a few frames until it exists.
pub fn focus_by_id(id: &str) {
    run(&format!(
        "const m = window.__ptvModal; m.later(() => {{ const e = document.getElementById({}); \
         if (!e) return false; e.focus(); return document.activeElement === e; }});",
        js_str(id)
    ));
}

/// Move focus to the next (or previous) control inside the modal `root_id`, wrapping around.
pub fn trap_tab(root_id: &str, backward: bool) {
    run(&format!(
        "const m = window.__ptvModal; const root = document.getElementById({}); if (root) {{ \
         const f = m.focusable(root); if (f.length) {{ const i = f.indexOf(document.activeElement); \
         const next = i < 0 ? (({}) ? f.length - 1 : 0) : (i + ({}) + f.length) % f.length; f[next].focus(); }} }}",
        js_str(root_id),
        if backward { "true" } else { "false" },
        if backward { "-1" } else { "1" }
    ));
}

/// Registers a modal layer for the lifetime of the calling component: on mount it records the
/// opener, makes the rest of the page inert and focuses `first_focus_id`; on drop it lifts the
/// isolation and focuses the opener, or `fallback_focus_id` when the opener is gone.
pub fn use_modal(
    root_id: &'static str,
    first_focus_id: &'static str,
    fallback_focus_id: &'static str,
) {
    use_effect(move || {
        run(&format!(
            "const m = window.__ptvModal; m.later(() => {{ const root = document.getElementById({root}); \
             if (!root) return false; \
             const active = document.activeElement; \
             const opener = (active && root.contains(active)) ? m.prev : active; \
             m.stacks[{root}] = {{ opener, marks: m.isolate(root) }}; \
             const first = document.getElementById({first}) || m.focusable(root)[0]; \
             if (first) first.focus(); return true; }});",
            root = js_str(root_id),
            first = js_str(first_focus_id),
        ));
        if cfg!(debug_assertions) {
            spawn(async move {
                let mode = eval(&format!("{HELPER}\nreturn window.__ptvModal.mode;"))
                    .join::<String>()
                    .await;
                if let Ok(mode) = mode {
                    eprintln!("[modal] {root_id}: background isolation = {mode}");
                }
            });
        }
    });
    use_drop(move || {
        run(&format!(
            "const m = window.__ptvModal; const st = m.stacks[{root}]; delete m.stacks[{root}]; \
             if (st) m.release(st.marks); \
             m.later(() => {{ let t = st && st.opener; \
             if (!t || !t.isConnected || t.disabled || t.closest('[inert]')) t = document.getElementById({fallback}); \
             if (!t) return false; t.focus(); return document.activeElement === t; }});",
            root = js_str(root_id),
            fallback = js_str(fallback_focus_id),
        ));
    });
}

/// The `onkeydown` body of a modal layer's root: the layer owns the keyboard, so the event is
/// **always** stopped here; `Tab` is kept inside the layer; any other key is resolved for
/// `layer`, and the resulting action is returned for the caller to dispatch.
pub fn modal_key(
    event: &Event<KeyboardData>,
    root_id: &str,
    layer: Layer,
) -> Option<ShortcutAction> {
    event.stop_propagation();
    if event.key() == Key::Tab {
        event.prevent_default();
        trap_tab(root_id, event.modifiers().shift());
        return None;
    }
    let key: KeyDescriptor = key_descriptor(&event.data());
    resolve(
        ShortcutContext {
            top_layer: layer,
            zen: false,
        },
        &key,
    )
}

#[cfg(test)]
mod tests;
