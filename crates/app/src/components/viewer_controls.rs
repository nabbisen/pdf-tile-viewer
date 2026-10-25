//! Viewer toolbar: primary scale control always visible; secondary controls
//! (pages-per-row, page numbers, jump-to-page) collapsed behind a ⚙ toggle.
//!
//! Design principle — less is more (RFC 008 §5):
//! A new user needs only the scale slider. The remaining controls are
//! revealed on demand so the toolbar stays uncluttered by default.

use dioxus::prelude::*;

use domain::layout::{MAX_FIXED_PAGES_PER_ROW, PagesPerRowMode, ViewerScale};

use crate::components::modal;
use crate::i18n::{Locale, MessageKey, t};
use crate::keys::editable_field_keys;

/// Starting value when the columns control switches from Auto to Fixed: the column count the
/// grid shows right now (so nothing jumps), kept inside the supported range (RFC 033 § 5.6).
pub(crate) fn fixed_start(displayed_columns: usize) -> u16 {
    displayed_columns.clamp(1, MAX_FIXED_PAGES_PER_ROW as usize) as u16
}

#[component]
pub fn ViewerControls(
    page_count: usize,
    scale: Signal<f32>,
    mode: Signal<PagesPerRowMode>,
    show_page_numbers: Signal<bool>,
    /// Columns the tile grid shows now (the first row's tile count).
    displayed_columns: usize,
    on_jump: Callback<usize>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut show_secondary = use_signal(|| false);
    let mut jump_input = use_signal(String::new);
    let mut jump_error: Signal<Option<String>> = use_signal(|| None);
    // Escape in any control here hands the keyboard back to the viewer container.
    let leave_field = Callback::new(|_: ()| modal::focus_by_id("viewer-root"));
    let jump_now = Callback::new(move |_: ()| {
        handle_jump(&jump_input.read(), page_count, &on_jump, &mut jump_error);
    });

    rsx! {
        div { class: "viewer-controls",

            // ── Primary: scale ────────────────────────────────────────────
            div { class: "control-group",
                button {
                    class: "ghost icon-btn",
                    title: t(locale(), MessageKey::ZoomOut),
                    "aria-label": t(locale(), MessageKey::ZoomOut),
                    disabled: *scale.read() <= ViewerScale::MIN,
                    onclick: move |_| {
                        let v = (*scale.read() - 0.2).max(ViewerScale::MIN);
                        scale.set((v * 10.0).round() / 10.0);
                    },
                    "−"
                }
                input {
                    r#type: "range",
                    class: "scale-slider",
                    min: "{(ViewerScale::MIN * 10.0) as i32}",
                    max: "{(ViewerScale::MAX * 10.0) as i32}",
                    step: "1",
                    value: "{(*scale.read() * 10.0).round() as i32}",
                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<f32>() {
                            scale.set((v / 10.0).clamp(ViewerScale::MIN, ViewerScale::MAX));
                        }
                    },
                    onkeydown: editable_field_keys(leave_field, None),
                }
                button {
                    class: "ghost icon-btn",
                    title: t(locale(), MessageKey::ZoomIn),
                    "aria-label": t(locale(), MessageKey::ZoomIn),
                    disabled: *scale.read() >= ViewerScale::MAX,
                    onclick: move |_| {
                        let v = (*scale.read() + 0.2).min(ViewerScale::MAX);
                        scale.set((v * 10.0).round() / 10.0);
                    },
                    "+"
                }
            }

            // ── Secondary toggle ──────────────────────────────────────────
            button {
                class: if *show_secondary.read() { "ghost icon-btn active" } else { "ghost icon-btn" },
                title: t(locale(), MessageKey::MoreControls),
                "aria-label": t(locale(), MessageKey::MoreControls),
                "aria-expanded": if *show_secondary.read() { "true" } else { "false" },
                onclick: move |_| show_secondary.toggle(),
                "⚙"
            }

            // ── Secondary: columns, page numbers, jump ────────────────────
            if *show_secondary.read() {
                // Pages per row: [Auto | Fixed] and a number field that is enabled only in Fixed (RFC 033 § 5.6)
                div { class: "control-group", role: "group", "aria-label": t(locale(), MessageKey::ColumnsLabel),
                    label { class: "control-label", {t(locale(), MessageKey::ColumnsLabel)} }
                    button {
                        class: if matches!(*mode.read(), PagesPerRowMode::Auto) { "ghost active" } else { "ghost" },
                        "aria-pressed": if matches!(*mode.read(), PagesPerRowMode::Auto) { "true" } else { "false" },
                        onclick: move |_| mode.set(PagesPerRowMode::Auto),
                        {t(locale(), MessageKey::ColumnsAuto)}
                    }
                    button {
                        class: if matches!(*mode.read(), PagesPerRowMode::Fixed(_)) { "ghost active" } else { "ghost" },
                        "aria-pressed": if matches!(*mode.read(), PagesPerRowMode::Fixed(_)) { "true" } else { "false" },
                        onclick: move |_| {
                            // Read in its own statement; switching to Fixed starts at the visible count.
                            let already_fixed = matches!(*mode.peek(), PagesPerRowMode::Fixed(_));
                            if !already_fixed {
                                mode.set(PagesPerRowMode::Fixed(fixed_start(displayed_columns)));
                            }
                        },
                        {t(locale(), MessageKey::ColumnsFixed)}
                    }
                    input {
                        r#type: "number",
                        class: "pages-per-row-input",
                        "aria-label": t(locale(), MessageKey::ColumnsLabel),
                        min: "1",
                        max: "{MAX_FIXED_PAGES_PER_ROW}",
                        disabled: !matches!(*mode.read(), PagesPerRowMode::Fixed(_)),
                        value: match *mode.read() {
                            PagesPerRowMode::Fixed(n) => format!("{n}"),
                            PagesPerRowMode::Auto => format!("{}", fixed_start(displayed_columns)),
                        },
                        oninput: move |evt| {
                            if let Ok(n) = evt.value().parse::<u16>() {
                                mode.set(PagesPerRowMode::Fixed(n.clamp(1, MAX_FIXED_PAGES_PER_ROW)));
                            }
                        },
                        onkeydown: editable_field_keys(leave_field, None),
                    }
                }

                // Page numbers
                div { class: "control-group",
                    label { class: "control-label",
                        input {
                            r#type: "checkbox",
                            checked: *show_page_numbers.read(),
                            onchange: move |evt| show_page_numbers.set(evt.checked()),
                            onkeydown: editable_field_keys(leave_field, None),
                        }
                        " " {t(locale(), MessageKey::PageNumbersLabel)}
                    }
                }

                // Jump to page
                div { class: "control-group",
                    label { class: "control-label", {t(locale(), MessageKey::JumpToPageLabel)} }
                    input {
                        r#type: "number",
                        class: "jump-input",
                        min: "1",
                        max: "{page_count}",
                        placeholder: "…",
                        value: "{jump_input.read()}",
                        oninput: move |evt| jump_input.set(evt.value()),
                        onkeydown: editable_field_keys(leave_field, Some(jump_now)),
                    }
                    button {
                        class: "ghost",
                        onclick: move |_| jump_now.call(()),
                        {t(locale(), MessageKey::JumpGoButton)}
                    }
                    if let Some(err) = &*jump_error.read() {
                        span { class: "muted jump-error", "{err}" }
                    }
                }
            }
        }
    }
}

fn handle_jump(
    raw: &str,
    page_count: usize,
    on_jump: &Callback<usize>,
    jump_error: &mut Signal<Option<String>>,
) {
    match raw.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= page_count => {
            jump_error.set(None);
            on_jump.call(n - 1);
        }
        Ok(_) => jump_error.set(Some(format!("1–{page_count}"))),
        Err(_) => jump_error.set(Some("?".to_string())),
    }
}

#[cfg(test)]
mod tests;
