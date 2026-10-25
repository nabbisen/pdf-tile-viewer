//! Document language and title for assistive technology (RFC 037 § 5.6.2).
//!
//! The web view's `<html lang>` and `document.title` are set from one Rust function, through a
//! script that contains no logic: screen readers pick the voice from `lang`, and the document name
//! they announce is `document.title` (it used to be the framework default and an empty language).

use crate::i18n::Locale;

/// The BCP 47 tag of a catalog language.
pub fn lang_tag(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "en",
        Locale::Ja => "ja",
    }
}

/// `<file> — <app title>` while a document is open, the app title otherwise. `document_name` is
/// the file name, or the full path when the user opted in (`privacy.show_full_path_in_title`).
pub fn title_text(app_title: &str, document_name: Option<&str>) -> String {
    match document_name {
        Some(name) => format!("{name} — {app_title}"),
        None => app_title.to_string(),
    }
}

/// The script that applies language and title. Values are JSON-quoted, so no character in a file
/// name can break out of the string.
pub fn meta_script(lang: &str, title: &str) -> String {
    let quote = |value: &str| serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "document.documentElement.lang = {};\ndocument.title = {};",
        quote(lang),
        quote(title)
    )
}

#[cfg(test)]
mod tests;
