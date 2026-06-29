//! Search panel (RFC 010 §7): input, submit/clear, summary, matched pages.
//!
//! The panel is toggled by the viewer toolbar. Search runs on the PDF engine
//! worker (non-mutating), returning `SearchResultSet` + `SearchHighlightSet`.
//! Page-level markers are shown immediately; per-tile highlight rects are
//! driven by the `highlights` signal passed back to the caller.

use dioxus::prelude::*;

use domain::search::{SearchQuery, SearchRequest};
use pdf_engine::worker::EngineHandle;

use crate::i18n::{Locale, MessageKey, t};
use crate::state::SearchState;

#[component]
pub fn SearchPanel(
    engine: EngineHandle,
    session_id: domain::document::DocumentId,
    generation: domain::document::DocumentGeneration,
    search_state: Signal<SearchState>,
    on_close: Callback<()>,
) -> Element {
    let locale: Memo<Locale> = use_context();
    let mut input_text = use_signal(|| {
        // Pre-fill if there's an existing confirmed query.
        match &*search_state.read() {
            SearchState::Results { results, .. } => results.query.text.clone(),
            SearchState::NoResults { query, .. } => query.text.clone(),
            _ => String::new(),
        }
    });
    let is_running = use_memo(move || matches!(*search_state.read(), SearchState::Searching));

    let do_search = Callback::new(move |_: ()| {
        let text = input_text.read().trim().to_string();
        let query = SearchQuery::plain(text);
        if !query.is_runnable() {
            return;
        }
        let request = SearchRequest {
            document_id: session_id,
            generation,
            query,
        };
        let engine = engine.clone();
        let mut ss = search_state.clone();
        spawn(async move {
            ss.set(SearchState::Searching);
            match engine.search_document_with_highlights(request).await {
                Ok(Ok((results, highlights))) => {
                    if results.pages.is_empty() {
                        ss.set(SearchState::NoResults {
                            query: results.query,
                        });
                    } else {
                        ss.set(SearchState::Results {
                            results,
                            highlights,
                        });
                    }
                }
                Ok(Err(e)) => ss.set(SearchState::Failed(format!("{e:?}"))),
                Err(_) => ss.set(SearchState::Failed("Engine unavailable".into())),
            }
        });
    });

    let mut do_clear = move || {
        search_state.set(SearchState::Idle);
        input_text.set(String::new());
    };

    let summary = match &*search_state.read() {
        SearchState::Idle => None,
        SearchState::Searching => Some(t(locale(), MessageKey::Searching).to_string()),
        SearchState::Results { results, .. } => {
            let n = results.total_matches;
            let p = results.pages.len();
            Some(format!(
                "{n} {} / {}p",
                t(locale(), MessageKey::SearchSummaryMatches),
                p
            ))
        }
        SearchState::NoResults { .. } => Some(t(locale(), MessageKey::SearchNoMatches).to_string()),
        SearchState::Failed(msg) => Some(format!("Error: {msg}")),
    };

    let matched_pages_str = match &*search_state.read() {
        SearchState::Results { results, .. } => {
            let indices: Vec<_> = results.pages.iter().map(|p| p.page_index).collect();
            Some(domain::search::format_matched_pages(&indices))
        }
        _ => None,
    };

    rsx! {
        div { class: "search-panel", role: "search",
            div { class: "search-input-row",
                input {
                    r#type: "search",
                    class: "search-input",
                    placeholder: t(locale(), MessageKey::SearchPlaceholder),
                    "aria-label": t(locale(), MessageKey::SearchPlaceholder),
                    value: "{input_text.read()}",
                    disabled: *is_running.read(),
                    oninput: move |e| input_text.set(e.value()),
                    onkeydown: {
                        let ds = do_search.clone();
                        move |e: Event<KeyboardData>| {
                            if e.key() == Key::Enter { ds.call(()); }
                        }
                    },
                }
                button {
                    class: "primary search-btn",
                    disabled: *is_running.read(),
                    onclick: {
                        let ds = do_search.clone();
                        move |_| ds.call(())
                    },
                    {t(locale(), MessageKey::SearchButton)}
                }
                button {
                    class: "ghost",
                    onclick: move |_| do_clear(),
                    {t(locale(), MessageKey::SearchClear)}
                }
                button {
                    class: "ghost icon-btn",
                    "aria-label": "Close search",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            if let Some(ref msg) = summary {
                p { class: if matches!(*search_state.read(), SearchState::NoResults { .. }) { "search-summary no-results" } else { "search-summary" },
                    "{msg}"
                }
            }
            if let Some(ref pages) = matched_pages_str {
                p { class: "search-pages muted", "p. {pages}" }
            }
        }
    }
}
