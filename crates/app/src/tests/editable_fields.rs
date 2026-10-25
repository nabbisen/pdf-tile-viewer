//! Guard (RFC 033 § 5.3): every `input` / `textarea` element of the app is built with
//! `editable_field_keys`, so keys typed into it can never reach the viewer's shortcut handler.
//! A new editable control without the helper fails this test.

use std::fs;
use std::path::{Path, PathBuf};

/// Line numbers (1-based) of `input {` / `textarea {` elements whose block does not call
/// `editable_field_keys(`.
fn elements_without_helper(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut offenders = Vec::new();
    for tag in ["input", "textarea"] {
        let needle = format!("{tag} {{");
        let mut from = 0;
        while let Some(found) = source[from..].find(&needle) {
            let start = from + found;
            from = start + needle.len();
            // a whole word, not the tail of an identifier such as `search_input {`
            let word_start = start == 0
                || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
            // not inside a `//` comment
            let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
            let in_comment = source[line_start..start].contains("//");
            if !word_start || in_comment {
                continue;
            }
            // the element block: from the opening brace to its matching closing brace
            let open = start + tag.len() + 1;
            let mut depth = 0usize;
            let mut end = source.len();
            for (offset, byte) in bytes[open..].iter().enumerate() {
                match byte {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = open + offset;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if !source[open..end].contains("editable_field_keys(") {
                offenders.push(source[..start].matches('\n').count() + 1);
            }
        }
    }
    offenders.sort_unstable();
    offenders
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable source directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "tests") {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_input_element_uses_the_editable_field_helper() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut checked = 0;
    let mut problems = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).expect("readable source file");
        checked += source.matches("input {").count() + source.matches("textarea {").count();
        for line in elements_without_helper(&source) {
            problems.push(format!(
                "{}:{line}",
                file.strip_prefix(&src).unwrap().display()
            ));
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
