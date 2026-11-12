# Rust migration audit

Release review: 12 November 2026. Baseline: `32b9627c096db65c37834800724005d85427aba9`, the commit behind the previous Vercel production deployment.

## Replacement boundary

| Responsibility | Maintained implementation |
| --- | --- |
| Static HTML, metadata and discovery | `crates/site` |
| Responsive images and content-based cache | Rust `image` + `sha2` |
| Build-time visual sequencing | `crates/gallery-core` |
| Menus, galleries, gestures and routing | `crates/browser`, compiled to WASM |
| Presentation and authored content | Independent desktop/mobile HTML, CSS and JSON |
| Production delivery | Existing Vercel project, static CDN output |

No active `.astro` source, Astro configuration, Node package manifest, TypeScript controller or legacy JavaScript controller remains. Editor tasks now invoke Rust. Production includes wasm-bindgen's generated JavaScript bridge; this is required to load WASM and is not a retained Node/Astro application. HTML/CSS are real maintained sources, not hidden or relabelled to manipulate GitHub's language statistics.

The toolchain is pinned to Rust 1.97.1, including rustfmt, Clippy and the WASM target. Cargo dependencies are locked. The build performs no npm installation. `vercel.json` selects framework Other, `sh scripts/build-vercel`, empty install command and `dist` output in the existing project.

## Automated verification

- 23 Rust tests pass, including routing, gesture intent, metadata escaping, path traversal rejection, typography and cache reuse/invalidation/recovery.
- Formatting and strict native/WASM Clippy checks pass.
- Production generates 21 page files, 79 original-image records and 463 responsive variants. All content images have alt text, intrinsic dimensions, srcset and sizes.
- Output checks cover internal links/assets, 62 connected copy blocks, shared author/work schemas, canonical URLs, indexing mode and ten canonical discovery URLs.
- All visible production copy is English. Machine-readable identity data may retain the author's Chinese alternate name.
- Production has no review pages, review stylesheet or review WASM feature. Preview retains its local review workflow.
- Architecture checks reject Astro/Node manifests, `.astro` files and handwritten JavaScript/TypeScript application sources.
- GitHub CI repeats formatting, native/WASM lints, Rust tests, production generation and both output guards.

## Browser review

The actual production output was served separately from the local review build. All ten mobile routes were inspected at 320, 393 and 430 CSS pixels with no document overflow, missing visible images or desktop information disclosures. Desktop routes were checked at laptop and larger widths. An inherited Closing Time negative-margin overflow was fixed by matching the actual content padding and constraining the grid item.

Observed interaction checks include: automatic mobile routing, Selected works opening the menu, Escape closing it and restoring focus, original-image enlargement with the header hidden, closing the viewer, eight curated mobile spreads and a compact viewer canvas, four Monolith daylight studies in one row, native About disclosures, Quite Off photo switching and its semantic highlights, and the final Time image edge relationships and regular-weight II separator. Browser console inspection found no application warnings or errors in the tested flows. These are browser viewport tests, not physical-device certification.

Original The Threes mockups remain in use. Monolith retains the approved building geometry. In Praise of Time retains all fourteen authored photos, separate desktop/mobile edits and the closing portrait's exceptional spacing.

## Performance evidence

The [report](../reports/performance/README.md) retains raw samples, environment, commands, scope and caveats. Shared originals are byte-identical. It distinguishes 59-image comparisons from the current 79-image portfolio and compares nine canonical route payloads. Cold image processing and first-visit WASM costs are reported as regressions, not omitted. No Lighthouse score, Core Web Vitals improvement, real-device result or search-ranking gain is asserted.

## Hosting handoff

The existing `dai-portfolio` project and custom domains are retained. Its framework setting is now Other; existing SSO protection is unchanged. The authorized release is delivered through GitHub `main`, followed by verification of the exact Git commit, build logs, production deployment state and public routes. Deployment IDs and live verification are recorded in the final delivery response.
