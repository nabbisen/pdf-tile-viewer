//! Keyboard plumbing shared by the viewer and its layers (RFC 033 § 5.3).
//!
//! * [`key_descriptor`] reduces a Dioxus key event to the domain's `KeyDescriptor`, the only input
//!   of the shortcut resolver.
//! * [`editable_field_keys`] is the one way to build the `onkeydown` of an `input`/`textarea`:
//!   keys typed into a field never reach the viewer's shortcut handler. A test
//!   (`tests/editable_fields.rs`) fails when an input is built without it.

use dioxus::prelude::*;

use domain::shortcuts::KeyDescriptor;

/// Pure core of [`key_descriptor`]. `command` is true when Ctrl, Alt or Meta is held: those
/// combinations belong to the system or the web view, never to our shortcuts.
pub fn descriptor_from(key: &Key, command: bool) -> KeyDescriptor {
    if command {
        return KeyDescriptor::Other;
    }
    match key {
        Key::Escape => KeyDescriptor::Escape,
        Key::ArrowLeft => KeyDescriptor::ArrowLeft,
        Key::ArrowRight => KeyDescriptor::ArrowRight,
        Key::PageUp => KeyDescriptor::PageUp,
        Key::PageDown => KeyDescriptor::PageDown,
        Key::Home => KeyDescriptor::Home,
        Key::End => KeyDescriptor::End,
        Key::Character(text) => {
            let mut chars = text.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => KeyDescriptor::Character(c),
                _ => KeyDescriptor::Other,
            }
        }
        _ => KeyDescriptor::Other,
    }
}

pub fn key_descriptor(event: &KeyboardData) -> KeyDescriptor {
    let modifiers = event.modifiers();
    descriptor_from(
        &event.key(),
        modifiers.ctrl() || modifiers.alt() || modifiers.meta(),
    )
}

/// What an editable field does with a key (pure; see [`editable_field_keys`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditableKey {
    /// Forward to the dismissal callback; do not propagate.
    Escape,
    /// Run the field's action, if it has one; do not propagate.
    Enter,
    /// Typed into the field; do not propagate.
    Swallow,
    /// Left to bubble: `Tab` is navigation, never an action, and the enclosing modal layer needs
    /// it for its focus trap. No resolver action exists for it.
    PassThrough,
}

pub fn editable_key(key: &Key) -> EditableKey {
    match key {
        Key::Escape => EditableKey::Escape,
        Key::Enter => EditableKey::Enter,
        Key::Tab => EditableKey::PassThrough,
        _ => EditableKey::Swallow,
    }
}

/// The `onkeydown` of every editable control (text, number, search, password inputs, the scale
/// slider, checkboxes): stops propagation of every key except `Tab`, forwards `Escape` to
/// `on_escape` and `Enter` to `on_enter`. Default actions (typing, arrow keys on a slider) are
/// not prevented.
pub fn editable_field_keys(
    on_escape: Callback<()>,
    on_enter: Option<Callback<()>>,
) -> impl FnMut(Event<KeyboardData>) + 'static {
    move |event: Event<KeyboardData>| match editable_key(&event.key()) {
        EditableKey::PassThrough => {}
        EditableKey::Escape => {
            event.stop_propagation();
            on_escape.call(());
        }
        EditableKey::Enter => {
            event.stop_propagation();
            if let Some(on_enter) = on_enter {
                on_enter.call(());
            }
        }
        EditableKey::Swallow => event.stop_propagation(),
    }
}

#[cfg(test)]
mod tests;
