//! Document outline side panel (RFC 026 PR2).

use std::collections::HashSet;

use dioxus::prelude::*;

use domain::document::PageIndex;
use domain::navigation::{DocumentOutline, NavigationTarget, OutlineNode, OutlineTitle};

use crate::i18n::{Locale, MessageKey, t};

#[derive(Clone, Debug, PartialEq)]
pub enum OutlinePanelState {
    Idle,
    Loading,
    Ready(DocumentOutline),
    /// Why the outline could not be read, as a catalog key.
    Unavailable(MessageKey),
}

#[component]
pub fn OutlinePanel(
    state: OutlinePanelState,
    on_close: Callback<()>,
    on_navigate: Callback<PageIndex>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let expanded: Signal<HashSet<String>> = use_signal(HashSet::new);

    rsx! {
        aside {
            class: "outline-panel",
            "aria-label": t(locale(), MessageKey::OutlinePanelLabel),
            div { class: "outline-header",
                h2 { {t(locale(), MessageKey::OutlinePanelLabel)} }
                button {
                    id: "outline-close",
                    class: "ghost icon-btn",
                    "aria-label": t(locale(), MessageKey::OutlineClose),
                    title: t(locale(), MessageKey::OutlineClose),
                    // Opening the panel moves focus into it (RFC 033 § 5.5); once the entries
                    // have loaded, the first entry takes over.
                    onmounted: move |evt: Event<MountedData>| {
                        spawn(async move {
                            let _ = evt.set_focus(true).await;
                        });
                    },
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            match state {
                OutlinePanelState::Idle | OutlinePanelState::Loading => rsx! {
                    p { class: "muted outline-status", {t(locale(), MessageKey::OutlineLoading)} }
                },
                OutlinePanelState::Unavailable(key) => rsx! {
                    p { class: "muted outline-status", role: "alert", {t(locale(), key)} }
                },
                OutlinePanelState::Ready(outline) if outline.roots.is_empty() => rsx! {
                    p { class: "muted outline-status", {t(locale(), MessageKey::OutlineEmpty)} }
                },
                OutlinePanelState::Ready(outline) => rsx! {
                    nav { class: "outline-tree",
                        for (position, node) in outline.roots.into_iter().enumerate() {
                            OutlineNodeRow {
                                key: "{node.id.0}",
                                node,
                                depth: 0,
                                first_entry: position == 0,
                                expanded,
                                on_navigate,
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn OutlineNodeRow(
    node: OutlineNode,
    depth: usize,
    /// The first entry of the panel: it receives the focus when the entries have loaded.
    #[props(default = false)]
    first_entry: bool,
    expanded: Signal<HashSet<String>>,
    on_navigate: Callback<PageIndex>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let node_id = node.id.0.clone();
    // The first focusable control of the first entry: its expander, or its title when it has none.
    let focus_first = |evt: Event<MountedData>| {
        spawn(async move {
            let _ = evt.set_focus(true).await;
        });
    };
    let has_children = !node.children.is_empty();
    let is_expanded = expanded.read().contains(&node_id);
    let title = outline_title_label(&node.title, locale());
    let internal_page = match &node.target {
        NavigationTarget::InternalDestination(destination) => Some(destination.page_index),
        NavigationTarget::ExternalUri(_) | NavigationTarget::Disabled(_) => None,
    };
    let indent = depth as f32 * 0.75;

    rsx! {
        div { class: "outline-node",
            div {
                class: "outline-row",
                style: "padding-left: {indent}rem;",
                if has_children {
                    button {
                        class: "outline-expander",
                        onmounted: move |evt| { if first_entry { focus_first(evt); } },
                        "aria-label": if is_expanded {
                            t(locale(), MessageKey::OutlineCollapse)
                        } else {
                            t(locale(), MessageKey::OutlineExpand)
                        },
                        "aria-expanded": if is_expanded { "true" } else { "false" },
                        onclick: {
                            let node_id = node_id.clone();
                            move |_| {
                                let mut current = expanded.write();
                                if !current.remove(&node_id) {
                                    current.insert(node_id.clone());
                                }
                            }
                        },
                        if is_expanded { "▾" } else { "▸" }
                    }
                } else {
                    span { class: "outline-expander-spacer" }
                }

                if let Some(page_index) = internal_page {
                    button {
                        class: "outline-title",
                        title: "{title}",
                        onmounted: move |evt| { if first_entry && !has_children { focus_first(evt); } },
                        onclick: move |_| on_navigate.call(page_index),
                        "{title}"
                    }
                } else {
                    span {
                        class: "outline-title disabled",
                        title: t(locale(), MessageKey::OutlineUnsupported),
                        "{title}"
                    }
                }
            }

            if has_children && is_expanded {
                div { class: "outline-children",
                    for child in node.children {
                        OutlineNodeRow {
                            key: "{child.id.0}",
                            node: child,
                            depth: depth + 1,
                            expanded,
                            on_navigate,
                        }
                    }
                }
            }
        }
    }
}

fn outline_title_label(title: &OutlineTitle, locale: Locale) -> String {
    match title {
        OutlineTitle::Present(value) | OutlineTitle::Truncated(value) if !value.is_empty() => {
            value.clone()
        }
        OutlineTitle::Present(_)
        | OutlineTitle::Truncated(_)
        | OutlineTitle::Missing
        | OutlineTitle::Empty => t(locale, MessageKey::OutlineUntitled).to_string(),
    }
}

#[cfg(test)]
mod tests;
