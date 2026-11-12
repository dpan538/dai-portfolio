# Independent mobile views

Updated 12 November 2026. This document records the approved mobile design; release verification is recorded in rust-migration-audit.md.

Mobile and desktop have separate route files, body templates, layout shells and presentation styles. They share work facts, image originals, metadata and the Rust/WASM implementation of common controls. A mobile view does not render the desktop sidebar or project-info disclosure and does not load desktop presentation.css.

- Desktop shell: `site/templates/layout.html`.
- Mobile shell: `site/templates/mobile-layout.html`.
- Mobile templates: `site/templates/pages/*_mobile.html`.
- Mobile styles: `site/assets/mobile/common.css` and one stylesheet per page.
- Routing: `crates/browser/src/routes.rs`. It preserves query strings and fragments and switches at the existing 768px breakpoint, using replacement navigation. Root maps to `/home_mobile`; all other canonical views append `_mobile`.
- Navigation links in mobile views point directly to the mobile routes. Both views retain the desktop canonical URL and author/work identity. Only the ten canonical pages enter the sitemap. The build produces 21 page files including ten mobile views and 404, plus two local review pages.

## Reading patterns

| Work | Mobile treatment |
| --- | --- |
| In Praise of Time | Fourteen photographs in authored order, II between chapters, a short introduction and closing statement, six short image notes and a five-line A Flowing Song excerpt with attribution. Individual scales and compact 24px sequence gaps; no full desktop poem. |
| The Threes | Original cover and ending mockups, eight selected spreads (03, 05, 08, 10, 12, 16, 20, 22) in a native horizontal snap strip. Tap to enlarge. The desktop retains all 22 spreads. |
| Quite Off | Original mobile pattern: eight photographs interleaved with four short passages. No desktop keyword panel. |
| Monolith for Light | Paired opening images and drawing groups, six short explanatory passages, four daylight moments together in one horizontal row. Other studies can be enlarged. No new renderings. |
| Reframed Still | Preserve full paper width; restore horizontal swipe browsing, snap positions, 1/8 progress, enlargement and previous/next browsing. Description below the strip. |
| Closing Time | Print followed by title, medium and visible description. No +. |
| Masks | Original shorter mobile descriptions within the print sequence; enlargement and previous/next browsing. No +. |
| About | Independent short biography and portrait, education, complete exhibition and award lists in named expandable sections, a contiguous disclosure group for exhibitions, awards, Contact and Selected research, with Writing last. The biography includes a fuller account of the image/text practice. No visible About heading or +. |
| Home and Contact | A short introduction and a separate style statement in a lighter sans-serif treatment; Selected works opens the existing navigation menu. Independent Contact template retains the facts and links, including GitHub. |

The + disclosure remains on desktop Reframed Still, Closing Time and Masks only. Native mobile carousel scrolling stays inside its own horizontal container; vertical page reading remains available. Menu and image viewer preserve reading position, and the header hides during image viewing.

## In Praise of Time scales

`site/content/mobile-time.json` controls only the mobile edit, including optional `copy_key` slots for its independently written short passages. Rust checks its sequence against the fourteen authored photographs and requires the wall study to be the only full-width portrait. Percentage widths refer to the image column; photos remain uncropped. The sheep photograph uses `align_left_with` to share the sundae's left edge. The returning lips photograph uses `align_right_with_caption` to follow the first sentence's right edge, ending at “scale.” Other photographs remain centred. Responsive `sizes` follows each actual display width.

| Photograph | Width | Maximum |
| --- | --- | --- |
| Opening lips | 100% | Column |
| Branches | 60% | 280px |
| Plant | 58% | 270px |
| Sundae | 90% | Column |
| Sheep | 54% | 250px |
| Bed / elephant | 100% | Column |
| Chapter-two meal | 100% | Column |
| Hand sanitiser | 50.544% | Column |
| Water reflection | 87.95% | 352px |
| Wall shadows | 100% | Column |
| Figure under leaves | 54% | 250px |
| Table | 100% | Column |
| Second lips close-up | 38% | 168px |
| Closing portrait | 58.8% | 231px |

## Validation

- Twenty-one Rust tests pass, including bidirectional routing and gesture separation.
- Static checks cover 23 local / 21 production pages, 62 copy blocks, canonical relationships, internal links, image variants and mobile shell separation.
- At 320×740, all ten mobile routes were checked for horizontal overflow, English visible copy, absent desktop sidebar and absent + controls.
- At 393×852, the individual photograph widths, four selected book spreads, horizontal strip progress, image viewing, closing descriptions, About lists and navigation were inspected. The Threes advanced 1→2 by keyboard and 2→3 by horizontal scrolling without page displacement. Reframed Still advanced 1→2 and its enlarged viewer advanced to the next photograph.
- At 430×932, In Praise of Time, Monolith, Reframed Still and About were checked for overflow and disclosure absence. Returning to the default viewport restored the desktop routes and styles. Desktop disclosures remain on the three intended works; The Threes retains 22 spreads.
- Screenshots are under `target/review/mobile-*.png`. This is browser viewport verification, not a real-device test or a measured performance audit.


### Compact-reading follow-up

At 393×852, the time sequence is 4,407px tall (previously 4,558px); restored short passages do not make the page longer. The second lips close-up measures 150px wide. The four daylight panels share a single top edge and occupy 184px in height (previously 683px), reducing the Monolith page to 2,137px (previously 2,715px). Checks at 320×740 and 430×932 also confirm a single comparison row, no horizontal overflow and English visible copy on all four affected routes.

Selected works opens the existing menu, retains the URL and restores focus and scroll when closed. Contact in About expands and collapses natively, while the complete exhibition and award lists remain available. Image viewing was rechecked after adding mobile passage wrappers. Desktop routing, the fourteen-photo sequence, full poem and statement remain intact after resetting the viewport.


### Expanded mobile context and final scale adjustments

The Threes now includes eight original selected spreads (03, 05, 08, 10, 12, 16, 20, 22) in the same horizontal browser, retaining the two original book mockups. Monolith includes six explanatory passages about the concept, circulation, material and structure, interior experience, changing daylight, and site. The four moments remain in a single row.

In Praise of Time retains 24px sequence gaps, fourteen photographs and a short attributed poem excerpt. Its opening and two-paragraph closing statement are fuller. Independent notes accompany the plant, sheep, wall/body correspondence and returning lips. The light comparison is approximately twice its former length. The second lips close-up is 38% (168px maximum), with a note below. Glass is 62.4%, exactly 20% narrower than its former 78%; water reflection is 66% (264px maximum), 10% larger. Sheep and bed/elephant share a left edge, calculated by Rust from the referenced photo scale.

Reframed Still centres every image vertically within the existing horizontal strip without resizing the print. Closing Time centres its print in a dedicated first-screen area. Mobile paragraphs use `text-wrap:pretty` to reduce isolated final words while preserving authored poetry line breaks. No desktop stylesheet, artwork original or book mockup was changed.

About groups its three disclosures together, then shows MGD Archive, AusFigures and Words Over Time under Selected research, with Writing last. Research links were taken from the existing poetry-site About page. The homepage keeps two brief paragraphs, with a style statement instead of a repeated media list.


The final 393px scale check measures glass at 212.78px, water reflection at 225.05px and the second lips close-up at 129.58px. Sheep and bed left edges coincide within 0.01px. All ten mobile pages were inspected at 320px and 430px with no horizontal overflow; paragraph pretty wrapping is active, eight selected spreads remain available, and the four daylight moments remain a single row. The final homepage wording and photo alignment were rechecked at both widths. Screenshots with the late follow-up are listed in `docs/rust-migration-audit.md`.


### Mobile flow corrections and centred home

The book showcase uses a 49:37 canvas, matching the seven landscape spreads. Every selected image fits entirely inside it with `object-fit:contain`; the single portrait ending no longer sets the height of the whole strip. At 393px the showcase height is 257px instead of 531px. Closing Time removes its viewport-height reserve, places the print above the screen centre and lets the title, medium and description follow naturally. These corrections supersede the earlier dedicated full-screen print treatment.

Quite Off now has four mobile passages totalling 161 words. The time sequence makes the chapter transition clearer with a 20px II marker and short passages on both sides. The bed/elephant image is full column width, matching the text, and sheep keeps its shared left edge. Hand sanitiser is reduced a further 10% (62.4% → 56.16%); water reflection grows 15% (66% → 75.9%, cap 304px). The poem excerpt expands from three to five authored lines and the lips passage from 25 to 52 words. The full poem is still reserved for desktop.

Selected research is initially collapsed, with Modern Graphic Design Archive written in full. The homepage centres both paragraphs, navigation and copyright. The mobile hamburger and close symbols are SVG line icons with identical 1.5px strokes and the existing 52px touch target. Desktop presentation is unchanged.


### Final mobile sequencing and prose checks

The sheep now references the sundae for its left edge. Water reflection is enlarged leftward to align with the full-width wall below (87.95%, maximum 352px); the second lips close-up aligns with the closing portrait. Sanitiser is reduced a further 10% to 50.544%. The chapter-two order is meal photograph, sanitiser close-up, the gesture passage, then water reflection. The II marker remains normal weight at 16px, with 40px top and bottom margins.

Mobile prose binding is performed by Rust while rendering copy: the first two words after sentence punctuation and the last pair in a text segment stay together. This complements pretty wrapping. It leaves HTML attributes, destination URLs, artwork metadata and authored blockquote line breaks unchanged; desktop copy rendering is unchanged. Three typography tests cover sentence openings, endings, markup and poetry preservation. Total Rust tests: 21. Thirty page/width checks at 320px, 393px and 430px found no horizontal overflow, confirmed all three alignment relationships and retained fourteen photographs and eight spreads. Showcase heights are 202px, 257px and 285px respectively.


The closing portrait is the explicit spacing exception: 58.8% width, capped at 231px (5% larger), with 56px clearances before and after at the checked mobile widths. The preceding lips photograph follows its left edge through the existing reference-based alignment. Other sequence gaps remain 24px.

The homepage uses a centred text block (90% of the column, capped at 320px) with left-aligned paragraphs. Navigation remains centred and copyright is independently centred. This supersedes the temporary centred-paragraph treatment.
