//! Shared helpers for the grep-style guard tests: find the blocks of `tag { … }` elements in the
//! app's source files.

use std::fs;
use std::path::{Path, PathBuf};

/// One element block of the source: its 1-based line and its text from the opening brace to the
/// matching closing brace.
pub struct Block<'a> {
    pub line: usize,
    pub text: &'a str,
}

/// Blocks of every `tag { … }` element in `source` (whole word, not inside a `//` comment).
pub fn element_blocks<'a>(source: &'a str, tag: &str) -> Vec<Block<'a>> {
    let bytes = source.as_bytes();
    let needle = format!("{tag} {{");
    let mut blocks = Vec::new();
    let mut from = 0;
    while let Some(found) = source[from..].find(&needle) {
        let start = from + found;
        from = start + needle.len();
        let word_start =
            start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
        if !word_start || source[line_start..start].contains("//") {
            continue;
        }
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
        blocks.push(Block {
            line: source[..start].matches('\n').count() + 1,
            text: &source[open..end],
        });
    }
    blocks
}

/// Every non-test `.rs` file under `crates/app/src` (test modules and `tests` directories are skipped).
pub fn app_sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("readable source directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name != "tests") {
                    walk(&path, out);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs")
                && path.file_name().is_some_and(|name| name != "tests.rs")
            {
                out.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("readable source file");
            (path.strip_prefix(&root).unwrap().to_path_buf(), text)
        })
        .collect()
}
