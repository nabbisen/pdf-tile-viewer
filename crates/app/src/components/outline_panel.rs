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
    Unavailable,
}

#[component]
pub fn OutlinePanel(
    state: OutlinePanelState,
    on_close: Callback<()>,
    on_navigate: Callback<PageIndex>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let expanded: Signal<HashSet<String>> = use_signal(|| HashSet::new());

    rsx! {
        aside {
            class: "outline-panel",
            "aria-label": t(locale(), MessageKey::OutlinePanelLabel),
            div { class: "outline-header",
                h2 { {t(locale(), MessageKey::OutlinePanelLabel)} }
                button {
                    class: "ghost icon-btn",
                    "aria-label": t(locale(), MessageKey::OutlineClose),
                    title: t(locale(), MessageKey::OutlineClose),
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            match state {
                OutlinePanelState::Idle | OutlinePanelState::Loading => rsx! {
                    p { class: "muted outline-status", {t(locale(), MessageKey::OutlineLoading)} }
                },
                OutlinePanelState::Unavailable => rsx! {
                    p { class: "muted outline-status", {t(locale(), MessageKey::OutlineUnavailable)} }
                },
                OutlinePanelState::Ready(outline) if outline.roots.is_empty() => rsx! {
                    p { class: "muted outline-status", {t(locale(), MessageKey::OutlineEmpty)} }
                },
                OutlinePanelState::Ready(outline) => rsx! {
                    nav { class: "outline-tree",
                        for node in outline.roots {
                            OutlineNodeRow {
                                key: "{node.id.0}",
                                node,
                                depth: 0,
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
    expanded: Signal<HashSet<String>>,
    on_navigate: Callback<PageIndex>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let node_id = node.id.0.clone();
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
mod tests {
    use super::*;

    #[test]
    fn outline_title_label_uses_i18n_fallback_for_missing_empty_and_blank_truncated_titles() {
        assert_eq!(
            outline_title_label(&OutlineTitle::Missing, Locale::En),
            "Untitled"
        );
        assert_eq!(
            outline_title_label(&OutlineTitle::Empty, Locale::En),
            "Untitled"
        );
        assert_eq!(
            outline_title_label(&OutlineTitle::Truncated(String::new()), Locale::En),
            "Untitled"
        );
    }

    #[test]
    fn outline_title_label_keeps_present_or_truncated_text() {
        assert_eq!(
            outline_title_label(&OutlineTitle::Present("Chapter".to_string()), Locale::En),
            "Chapter"
        );
        assert_eq!(
            outline_title_label(&OutlineTitle::Truncated("Chapter".to_string()), Locale::En),
            "Chapter"
        );
    }
}
