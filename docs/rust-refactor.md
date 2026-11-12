# Rust architecture

The maintained build is a Cargo workspace. Rust owns static generation, image preprocessing, build-time sequencing and browser interaction. HTML and CSS remain presentation sources; JSON stores authored content. No Astro, TypeScript or Node application sources participate in the build.

`sh scripts/site` first builds `portfolio-browser` for `wasm32-unknown-unknown`, then runs the native `portfolio-site` compiler. The compiler produces static HTML, responsive derivatives and wasm-bindgen's JavaScript bridge. Vercel serves the resulting `dist/` directory through its CDN; it runs no application server.

The image pipeline applies EXIF orientation, creates widths 400–1600 without enlarging originals, retains JPEG quality 95 / lossless PNG, and uses a SHA-256 content-and-policy stamp for cache validation. Derivatives are written atomically. Failed builds preserve the last complete output; an output marker prevents the compiler from replacing arbitrary directories.

Desktop and mobile are separate templates and styles, connected by Rust/WASM viewport routing. Ordinary text, image markup and metadata are available in the initial HTML. Interaction still requires WASM and its generated JavaScript bridge. The post-load mobile redirect and shared WASM download have costs; see the measured report rather than assuming a framework speedup.

The local preview binds to loopback only and rejects traversal and hidden paths. Production builds omit review pages, review CSS, review data and the optional browser review feature. `scripts/check-site.py` validates output; `scripts/check-architecture.py` rejects legacy application manifests or sources. The GitHub Actions workflow checks formatting, native and WASM lints, Rust tests and a complete production build.

See [the release audit](rust-migration-audit.md), [mobile curation](mobile-design.md) and [performance methodology](../reports/performance/README.md). Historical pre-migration crawler notes are explicitly marked as historical; Git history retains the old implementation for reproducible comparison.
