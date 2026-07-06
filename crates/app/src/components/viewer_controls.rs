//! Viewer toolbar: primary scale control always visible; secondary controls
//! (pages-per-row, page numbers, jump-to-page) collapsed behind a ⚙ toggle.
//!
//! Design principle — less is more (RFC 008 §5):
//! A new user needs only the scale slider. The remaining controls are
//! revealed on demand so the toolbar stays uncluttered by default.

use dioxus::prelude::*;

use domain::layout::{MAX_FIXED_PAGES_PER_ROW, PagesPerRowMode, ViewerScale};

use crate::i18n::{Locale, MessageKey, t};

#[component]
pub fn ViewerControls(
    page_count: usize,
    scale: Signal<f32>,
    mode: Signal<PagesPerRowMode>,
    show_page_numbers: Signal<bool>,
    on_jump: Callback<usize>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut show_secondary = use_signal(|| false);
    let mut jump_input = use_signal(String::new);
    let jump_error: Signal<Option<String>> = use_signal(|| None);

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
                // Pages per row
                div { class: "control-group",
                    label { class: "control-label", {t(locale(), MessageKey::ColumnsLabel)} }
                    button {
                        class: if matches!(*mode.read(), PagesPerRowMode::Auto) {
                            "ghost active"
                        } else {
                            "ghost"
                        },
                        onclick: move |_| mode.set(PagesPerRowMode::Auto),
                        {t(locale(), MessageKey::ColumnsAuto)}
                    }
                    if matches!(*mode.read(), PagesPerRowMode::Fixed(_)) {
                        input {
                            r#type: "number",
                            class: "pages-per-row-input",
                            min: "1",
                            max: "{MAX_FIXED_PAGES_PER_ROW}",
                            value: {
                                match *mode.read() {
                                    PagesPerRowMode::Fixed(n) => format!("{n}"),
                                    PagesPerRowMode::Auto => "5".to_string(),
                                }
                            },
                            oninput: move |evt| {
                                if let Ok(n) = evt.value().parse::<u16>() {
                                    mode.set(PagesPerRowMode::Fixed(
                                        n.clamp(1, MAX_FIXED_PAGES_PER_ROW),
                                    ));
                                }
                            },
                        }
                    } else {
                        // Show a disabled input hint while in Auto mode
                        input {
                            r#type: "number",
                            class: "pages-per-row-input",
                            disabled: true,
                            placeholder: t(locale(), MessageKey::ColumnsAuto),
                            onclick: move |_| mode.set(PagesPerRowMode::Fixed(5)),
                        }
                    }
                }

                // Page numbers
                div { class: "control-group",
                    label { class: "control-label",
                        input {
                            r#type: "checkbox",
                            checked: *show_page_numbers.read(),
                            onchange: move |evt| show_page_numbers.set(evt.checked()),
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
                        onkeydown: {
                            let mut jump_error = jump_error;
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter {
                                    handle_jump(
                                        &jump_input.read(),
                                        page_count,
                                        &on_jump,
                                        &mut jump_error,
                                    );
                                }
                            }
                        },
                    }
                    button {
                        class: "ghost",
                        onclick: {
                            let mut jump_error = jump_error;
                            move |_| {
                                handle_jump(
                                    &jump_input.read(),
                                    page_count,
                                    &on_jump,
                                    &mut jump_error,
                                );
                            }
                        },
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
