//! Static guards of RFC 037 §§ 5.4–5.6: no `Debug` text in the UI, no icon-only button without an
//! accessible name, no input without a name or a label association.

use super::source_scan::{app_sources, element_blocks};

/// Line numbers of `button` elements that are icon-only (their last child is a quoted literal with
/// no ASCII letters, such as `"🔍"` or `"×"`) and carry no `aria-label`.
fn glyph_buttons_without_name(source: &str) -> Vec<usize> {
    element_blocks(source, "button")
        .into_iter()
        .filter(|block| {
            let last_literal = block
                .text
                .trim_end()
                .rsplit_once('\n')
                .map_or(block.text.trim(), |(_, last)| last.trim());
            let is_glyph = last_literal.starts_with('"')
                && last_literal.ends_with('"')
                && last_literal.len() > 2
                && !last_literal.chars().any(|c| c.is_ascii_alphabetic());
            is_glyph && !block.text.contains("aria-label")
        })
        .map(|block| block.line)
        .collect()
}

/// Line numbers of `input` elements with neither an `aria-label` nor an `id` (which a
/// `label { r#for: … }` must then point to).
fn inputs_without_name(source: &str) -> Vec<usize> {
    element_blocks(source, "input")
        .into_iter()
        .filter(|block| !block.text.contains("aria-label") && !block.text.contains("id:"))
        .map(|block| block.line)
        .collect()
}

#[test]
fn every_icon_only_button_has_an_aria_label() {
    let mut problems = Vec::new();
    let mut glyph_buttons = 0;
    for (path, source) in app_sources() {
        glyph_buttons += element_blocks(&source, "button")
            .iter()
            .filter(|b| {
                b.text.contains("\"×\"") || b.text.contains("\"🔍\"") || b.text.contains("\"⚙\"")
            })
            .count();
        for line in glyph_buttons_without_name(&source) {
            problems.push(format!("{}:{line}", path.display()));
        }
    }
    assert!(
        glyph_buttons >= 3,
        "the scan found only {glyph_buttons} glyph buttons"
    );
    assert!(
        problems.is_empty(),
        "glyph-only buttons without aria-label: {problems:?}"
    );
}

#[test]
fn every_input_has_a_name_or_a_label_association() {
    let mut problems = Vec::new();
    for (path, source) in app_sources() {
        for line in inputs_without_name(&source) {
            problems.push(format!("{}:{line}", path.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "inputs without aria-label or id: {problems:?}"
    );
}

#[test]
fn an_input_with_an_id_has_a_label_that_points_to_it() {
    for (path, source) in app_sources() {
        for block in element_blocks(&source, "input") {
            let Some(rest) = block.text.split("id: \"").nth(1) else {
                continue;
            };
            let id = rest.split('"').next().unwrap();
            assert!(
                source.contains(&format!("r#for: \"{id}\"")) || block.text.contains("aria-label"),
                "{}:{}: input id {id:?} has no label for it",
                path.display(),
                block.line
            );
        }
    }
}

#[test]
fn no_debug_formatting_reaches_ui_code() {
    let mut problems = Vec::new();
    for (path, source) in app_sources() {
        // the one sanctioned place: the stderr detail log of the failure module
        if path.ends_with("i18n/failure.rs") {
            continue;
        }
        for (index, line) in source.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains(":?}") || code.contains("{:?") || code.contains("{:#?") {
                problems.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "`Debug` formatting in UI code: {problems:?}"
    );
}

#[test]
fn the_scanners_flag_what_they_should() {
    let bad_button =
        "rsx! {\n    button {\n        onclick: move |_| (),\n        \"🔍\"\n    }\n}\n";
    assert_eq!(glyph_buttons_without_name(bad_button), vec![2]);
    let good_button = "rsx! { button { \"aria-label\": \"x\",\n \"🔍\"\n } }";
    assert!(glyph_buttons_without_name(good_button).is_empty());
    let text_button = "rsx! {\n button {\n  onclick: move |_| (),\n  {t(locale(), Key::Go)}\n }\n}";
    assert!(glyph_buttons_without_name(text_button).is_empty());
    assert_eq!(
        inputs_without_name("rsx! { input { r#type: \"text\" } }"),
        vec![1]
    );
    assert!(inputs_without_name("rsx! { input { \"aria-label\": \"x\" } }").is_empty());
}
