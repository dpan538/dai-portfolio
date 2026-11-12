# Mobile interaction checks — 13 November 2026

This follow-up checks the mobile Safari issues reported after the Rust migration. It is an interaction and layout audit, not a performance benchmark.

| Check | Coverage | Result |
| --- | --- | --- |
| Image previews | Time: 14; The Threes: 8; Monolith: 11; Reframed Still: 8; Closing Time: 1; Masks: 4 | All 46 opened and closed without a retained focus outline or text selection. Reading position was restored. |
| Previous / next controls | Both directions on every selected Threes spread and Masks print; desktop Quite Off buttons | Images changed correctly; pointer focus left no outline. |
| Navigation | All 8 mobile menu links, brand/home link, Selected works trigger and close button | Correct independent mobile routes; no retained pointer focus box. |
| About disclosures | Exhibitions, awards, Contact and Selected research | Each opened and closed without a selection or pointer focus ring. |
| Keyboard navigation | Tab after a pointer-opened menu | Keyboard focus remained visible; a subsequent pointer action removed the ring. |
| Viewport height change | Open preview, 390 × 844 → 390 × 744 | Preview stayed open, matching Safari toolbar expansion/collapse. |
| Layout | 10 mobile routes at 320, 390 and 430px | All 30 checks had no document-level horizontal overflow. |
| Home alignment | Both paragraph boxes at the three widths | Internally left aligned, with equal page margins: 26/26, 59/59 and 79/79px. |
| Returning lips alignment | Three phone widths | Photograph and the sentence ending “scale.” had the same right edge. |
| Poem reference | Direct public HTTP request | `https://daipan.ink/still-life/a-flowing-song` returned 200 and the title *A Flowing Song — Dai Pan*. |

The complete interaction matrix was exercised through the in-app browser. Separately, Safari in the iPhone 17 Pro Max simulator running iOS 26.4 was used for visual checks of the home page, menu opening/closing, long presses on the menu button and a menu link, and The Threes preview, next-page and close controls. Long presses did not produce the reported selection rectangles or link callouts. The viewer's controls remained below the status area, with no underlying artwork showing through the surrounding browser areas. This does not claim testing on physical iPhone hardware.

The fix distinguishes pointer input from keyboard input in Rust before dialog handlers move focus. Text selection/callouts are disabled on controls and images, while ordinary prose remains selectable. Mobile shells use `viewport-fit=cover`; viewers reserve safe-area padding, lock the background and restore its scroll position. There is one shell-owned `theme-color` tag per page. Browser-height changes no longer dismiss a preview.

Validation also passed all 23 Rust tests, native and production-WASM Clippy with warnings denied, formatting, the production build, all 21 generated-page checks and the Rust architecture audit. Public source contains the Rust favicon assets; legacy favicon paths also contain the new artwork.

Local screenshots are saved under `target/review/` and the raw interaction matrix under `.cache/mobile-touch-audit/`. These local verification artifacts are excluded from the production site.
