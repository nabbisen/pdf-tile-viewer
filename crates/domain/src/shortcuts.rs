//! Keyboard shortcut resolver (RFC 033 § 5.2, § 5.4).
//!
//! The one place that maps a key to an action in the current context. Pure: no Dioxus, no
//! platform types, so the whole matrix is unit-tested without a GUI. The app converts its key
//! events to [`KeyDescriptor`] and dispatches the returned [`ShortcutAction`]; modal layers call
//! the same function with their own layer and stop the event at their root.

/// Escape with nothing to dismiss: `true` returns to the dashboard (documented 2.0.x behavior,
/// RFC 033 § 5.4 two-step rollout), `false` does nothing (planned for 2.1.0).
pub const ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE: bool = true;

/// Top-most visible layer, lowest to highest. Search and outline are transient panels, zoom and
/// dialog are modal. Zen is a mode ([`ShortcutContext::zen`]), not a layer.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Layer {
    Base,
    Outline,
    Search,
    Zoom,
    /// External-link dialog or password prompt.
    Dialog,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShortcutContext {
    /// Top-most layer whose open flag is set (the caller does not need to know what is visible).
    pub top_layer: Layer,
    pub zen: bool,
}

impl ShortcutContext {
    /// The layer that actually receives the key: in Zen mode the search and outline panels are
    /// not rendered, so they are never dismissal or shortcut candidates even if their flags are set.
    pub fn effective_layer(self) -> Layer {
        match self.top_layer {
            Layer::Search | Layer::Outline if self.zen => Layer::Base,
            layer => layer,
        }
    }
}

/// A key event reduced to what the resolver needs. Keys pressed with Ctrl, Alt or Meta are
/// [`KeyDescriptor::Other`] (the app does that conversion): browser and system shortcuts are not ours.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyDescriptor {
    Character(char),
    Escape,
    ArrowLeft,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    Other,
}

/// What `Escape` closes (RFC 033 § 5.4): exactly one thing per press.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dismissal {
    Dialog,
    Zoom,
    Search,
    Outline,
    Zen,
    /// Nothing is open and [`ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE`] is `true`: back to the dashboard.
    LeaveDocument,
    /// Nothing is open and [`ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE`] is `false`: do nothing.
    Nothing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutAction {
    ToggleZen,
    ScaleUp,
    ScaleDown,
    ResetScale,
    ZoomScaleUp,
    ZoomScaleDown,
    ZoomPrev,
    ZoomNext,
    ZoomFirst,
    ZoomLast,
    Dismiss(Dismissal),
}

/// Resolve `key` in `ctx` with the product's idle-Escape behavior.
pub fn resolve(ctx: ShortcutContext, key: &KeyDescriptor) -> Option<ShortcutAction> {
    resolve_with(ctx, key, ESCAPE_LEAVES_DOCUMENT_WHEN_IDLE)
}

/// [`resolve`] with an explicit idle-Escape behavior (both values are tested).
pub fn resolve_with(
    ctx: ShortcutContext,
    key: &KeyDescriptor,
    escape_leaves_document_when_idle: bool,
) -> Option<ShortcutAction> {
    use KeyDescriptor::*;
    use ShortcutAction::*;
    let layer = ctx.effective_layer();
    match layer {
        // No action resolves while a dialog is on top except dismissing it.
        Layer::Dialog => matches!(key, Escape).then_some(Dismiss(Dismissal::Dialog)),
        Layer::Zoom => match key {
            Escape => Some(Dismiss(Dismissal::Zoom)),
            ArrowLeft | PageUp => Some(ZoomPrev),
            ArrowRight | PageDown => Some(ZoomNext),
            Home => Some(ZoomFirst),
            End => Some(ZoomLast),
            Character('+' | '=') => Some(ZoomScaleUp),
            Character('-') => Some(ZoomScaleDown),
            _ => None,
        },
        Layer::Base | Layer::Outline | Layer::Search => match key {
            Character('z' | 'Z') => Some(ToggleZen),
            Character('+' | '=') => Some(ScaleUp),
            Character('-') => Some(ScaleDown),
            Character('0') => Some(ResetScale),
            Escape => Some(Dismiss(match layer {
                Layer::Search => Dismissal::Search,
                Layer::Outline => Dismissal::Outline,
                _ if ctx.zen => Dismissal::Zen,
                _ if escape_leaves_document_when_idle => Dismissal::LeaveDocument,
                _ => Dismissal::Nothing,
            })),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests;
