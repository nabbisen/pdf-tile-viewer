//! Guard (RFC 033 § 5.3): every `input` / `textarea` element of the app is built with
//! `editable_field_keys`, so keys typed into it can never reach the viewer's shortcut handler.
//! A new editable control without the helper fails this test.

use super::source_scan::{app_sources, element_blocks};

/// Line numbers (1-based) of `input {` / `textarea {` elements whose block does not call
/// `editable_field_keys(`.
fn elements_without_helper(source: &str) -> Vec<usize> {
    let mut offenders: Vec<usize> = ["input", "textarea"]
        .iter()
        .flat_map(|tag| element_blocks(source, tag))
        .filter(|block| !block.text.contains("editable_field_keys("))
        .map(|block| block.line)
        .collect();
    offenders.sort_unstable();
    offenders
}

#[test]
fn every_input_element_uses_the_editable_field_helper() {
    let mut checked = 0;
    let mut problems = Vec::new();
    for (path, source) in app_sources() {
        checked += ["input", "textarea"]
            .iter()
            .map(|tag| element_blocks(&source, tag).len())
            .sum::<usize>();
        for line in elements_without_helper(&source) {
            problems.push(format!("{}:{line}", path.display()));
        }
    }
    assert!(
        checked >= 6,
        "the scan found only {checked} elements: is the pattern still right?"
    );
    assert!(
        problems.is_empty(),
        "input/textarea built without editable_field_keys(...): {problems:?}"
    );
}

#[test]
fn the_scanner_flags_an_input_without_the_helper() {
    let bad = "rsx! {\n    div {\n        input {\n            r#type: \"text\",\n            oninput: move |e| x.set(e.value()),\n        }\n    }\n}\n";
    assert_eq!(elements_without_helper(bad), vec![3]);
    let bad_textarea = "rsx! { textarea { rows: \"3\" } }";
    assert_eq!(elements_without_helper(bad_textarea), vec![1]);
}

#[test]
fn the_scanner_accepts_an_input_with_the_helper_and_ignores_other_words() {
    let good = "rsx! {\n    input {\n        onkeydown: editable_field_keys(esc, None),\n        value: \"{x}\",\n    }\n}\n";
    assert!(elements_without_helper(good).is_empty());
    let other = "let search_input { } // input { is a comment\nlet x = 1;";
    assert!(elements_without_helper(other).is_empty());
}
