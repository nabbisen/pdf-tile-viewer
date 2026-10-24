# Owner-run protocol — screen-reader listening session (release-candidate step)

Governing work: RFC 037 § 5.6 and RFC 033 § 5.5. Run **after** RFC 033 and RFC 037 are on `main`, on the release candidate build (staged release layout). The dev team's automated evidence only proves what the accessibility tree contains; this session proves what a person hears. Allow about one hour. Record results in `.git-exclude/review-request/rc-listening-session/README.md` (copy the table below).

## Setup
- **Linux:** use a session where Orca can grab keys (for example GNOME). In the niri/Wayland session used for development Orca cannot inject its navigation keys, so only focus-driven speech can be judged. Start the app with a scratch config: `XDG_CONFIG_HOME=$(mktemp -d)`.
- **Windows (when available):** Narrator or NVDA with the MSIX build. **macOS:** VoiceOver. Mark each platform you did not run as "not run".
- Documents: `crates/pdf_engine/tests/fixtures/navigation-links-outline.pdf`, `…/multi-page-search.pdf`, `…/password-protected.pdf` (password `pdf-tile-viewer-test`), `…/not-a-pdf.pdf`.

## Scenarios — write what is *said*, and mark Good / Unclear / Silent / Wrong
| # | Scenario | What should be heard |
|---|---|---|
| 1 | Launch | window title, the app heading, "Open PDF" button |
| 2 | Choose `not-a-pdf.pdf` | the error is announced without moving focus |
| 3 | Open `navigation-links-outline.pdf` | document name; toolbar buttons by **name** (Search, Show in file manager, Zen mode…), not emoji |
| 4 | Tile size slider | name "Tile size" and a value |
| 5 | Search "tile": results | result summary announced; no results announces "No matches found." |
| 6 | Search with one character | the "enter at least 2 characters" hint is announced |
| 7 | Outline panel | panel name; entries readable; focus moves in and back out on close |
| 8 | Zoom view (pointer needed to open) | dialog name; page position ("Page 2 of 3"); close button; focus returns after close |
| 9 | Zoom view text | reading the page text line by line (arrow keys) — does the text read sensibly? |
| 10 | Password prompt | dialog name, field name, the "password not accepted" message; background not reachable |
| 11 | External-link dialog | dialog name, address readable, copy status announced |
| 12 | Zen mode in and out | state change audible; focus lands on a sensible control |
| 13 | Japanese UI (`ui.locale` `ja`) | speech uses the Japanese voice for Japanese text (document language is set) |
| 14 | Columns control (Auto / Fixed) | segmented control reads as a choice with its state; the number field reads its label |

## Also note
Anything repeated, announced twice, announced too late, or that traps focus. Result: a go/no-go line for the accessibility claims in the release notes ("improved accessibility" only if scenarios 1–7 and 10–12 are Good).
