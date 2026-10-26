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

// The scripts are plain multi-line JavaScript with `__NAME__` placeholders (no `\` line
// continuations: those would join the lines and let a `//` comment swallow the rest).

const FOCUS_SCRIPT: &str = r#"
const m = window.__ptvModal;
m.later(() => {
  const e = document.getElementById(__ID__);
  if (!e) return false;
  e.focus();
  return document.activeElement === e;
});
"#;

const TRAP_SCRIPT: &str = r#"
const m = window.__ptvModal;
const root = document.getElementById(__ROOT__);
if (root) {
  const f = m.focusable(root);
  if (f.length) {
    const backward = __BACKWARD__;
    const i = f.indexOf(document.activeElement);
    const next = i < 0 ? (backward ? f.length - 1 : 0) : (i + (backward ? -1 : 1) + f.length) % f.length;
    f[next].focus();
  }
}
"#;

const OPEN_SCRIPT: &str = r#"
const m = window.__ptvModal;
m.later(() => {
  const root = document.getElementById(__ROOT__);
  if (!root) return false;
  const active = document.activeElement;
  const opener = (active && root.contains(active)) ? m.prev : active;
  const firstControl = () => document.getElementById(__FIRST__) || m.focusable(root)[0];
  // A focused control that becomes disabled (Next on the last page, a zoom limit) drops the focus
  // to the page body, where the layer would no longer receive keys: put it back.
  const watch = new MutationObserver(() => requestAnimationFrame(() => setTimeout(() => {
    const a = document.activeElement;
    if (root.isConnected && (!a || a === document.body)) {
      const f = firstControl();
      if (f) f.focus();
    }
  }, 0)));
  watch.observe(root, { subtree: true, childList: true, attributes: true, attributeFilter: ['disabled'] });
  m.stacks[__ROOT__] = { opener, marks: m.isolate(root), watch };
  const first = firstControl();
  if (first) first.focus();
  return true;
});
"#;

const CLOSE_SCRIPT: &str = r#"
const m = window.__ptvModal;
const st = m.stacks[__ROOT__];
delete m.stacks[__ROOT__];
if (st) { m.release(st.marks); st.watch.disconnect(); }
m.later(() => {
  let t = st && st.opener;
  // The page body is not a place to return to: focusing it does nothing (an opener that was never focused).
  const usable = (e) => e && e !== document.body && e !== document.documentElement && e.isConnected && !e.disabled && !e.closest('[inert]');
  // The opener may be gone (or never was a control): the first fallback id that exists wins.
  if (!usable(t)) {
    t = null;
    for (const id of __FALLBACKS__) {
      const e = document.getElementById(id);
      if (usable(e)) { t = e; break; }
    }
  }
  if (!t) return false;
  t.focus();
  return document.activeElement === t;
});
"#;

fn focus_script(id: &str) -> String {
    FOCUS_SCRIPT.replace("__ID__", &js_str(id))
}

fn trap_script(root_id: &str, backward: bool) -> String {
    TRAP_SCRIPT
        .replace("__ROOT__", &js_str(root_id))
        .replace("__BACKWARD__", if backward { "true" } else { "false" })
}

fn open_script(root_id: &str, first_focus_id: &str) -> String {
    OPEN_SCRIPT
        .replace("__ROOT__", &js_str(root_id))
        .replace("__FIRST__", &js_str(first_focus_id))
}

fn close_script(root_id: &str, fallback_focus_ids: &[&str]) -> String {
    let ids = fallback_focus_ids
        .iter()
        .map(|id| js_str(id))
        .collect::<Vec<_>>()
        .join(", ");
    CLOSE_SCRIPT
        .replace("__ROOT__", &js_str(root_id))
        .replace("__FALLBACKS__", &format!("[{ids}]"))
}

fn run(call: &str) {
    let _ = eval(&format!("{HELPER}\n{call}"));
}

/// Focus the element with `id`, retrying for a few frames until it exists.
pub fn focus_by_id(id: &str) {
    run(&focus_script(id));
}

/// Move focus to the next (or previous) control inside the modal `root_id`, wrapping around.
pub fn trap_tab(root_id: &str, backward: bool) {
    run(&trap_script(root_id, backward));
}

/// Registers a modal layer for the lifetime of the calling component: on mount it records the
/// opener, makes the rest of the page inert and focuses `first_focus_id`; on drop it lifts the
/// isolation and focuses the opener, or, when the opener is gone, the first of `fallback_focus_ids`
/// that exists (in order).
pub fn use_modal(
    root_id: &'static str,
    first_focus_id: &'static str,
    fallback_focus_ids: &'static [&'static str],
) {
    use_effect(move || {
        run(&open_script(root_id, first_focus_id));
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
    use_drop(move || run(&close_script(root_id, fallback_focus_ids)));
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
