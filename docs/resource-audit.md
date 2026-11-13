# Routing and resource audit — 13 November 2026

This follow-up addresses direct mobile navigation and repeat-visit caching. It does not change the approved layouts or recompress published artwork.

## Direct navigation to Quite Off

The old router used `window.innerWidth <= 768`. Quite Off's intrinsic 2912px image had no small-screen constraint because its viewer rules were inside the desktop media query. This left a real overflow hazard before the WebAssembly router started; iOS viewport expansion could then undermine the width check.

Rust now uses `matchMedia('(max-width: 768px)')`, matching the CSS breakpoint, with `documentElement.clientWidth` as a fallback. Small-screen CSS constrains the image before WASM loads. Query parameters and fragments survive the redirect.

| Browser viewport | Direct route | Document width after redirect | Image width with router removed |
| --- | --- | ---: | ---: |
| 375px | `/quite_off_mobile` | 375px | 323px |
| 390px | `/quite_off_mobile` | 390px | 338px |
| 414px | `/quite_off_mobile` | 414px | 362px |
| 430px | `/quite_off_mobile` | 430px | 378px |

These four tests use the in-app browser's viewport override, not an iPhone user-agent override. The router-free fixture is the actual rendered desktop page with its module loader removed. Separately, direct visits in Safari on the iPhone 17 Pro Max simulator, iOS 26.4, reached the mobile page. The simulator also redirected before this fix, so this audit does not claim to have reproduced every reported timing/user-agent condition. The fallback fixture verifies that image overflow no longer depends on a successful redirect.

Desktop Quite Off still changes photographs through its existing controls. The image URL rewrite includes inline carousel JSON, `srcset` and lightbox sources.

## Safe browser caching

Before this change, public HTML, WASM, CSS and the sampled images returned `public, max-age=0, must-revalidate`. The existing image cache fingerprint protected build reuse, but was absent from public URLs: `quite_01_1600w.jpg` could change without changing its address. A width suffix alone is not safe for immutable caching.

The Rust publisher now adds:

- `/_optimized/immutable/<SHA-256 of image bytes>/projects/...` for all 463 derivatives.
- `/assets/immutable/<bundle-content SHA-256>/...` for CSS, the loader, generated bridge and WASM. Relative imports remain together.
- One-year `public, max-age=31536000, immutable` headers on those two namespaces only.

HTML and legacy stable aliases continue to revalidate. Ordinary revisits can reuse fresh resources without 304 round trips; reloads, cache eviction and browser policy can still trigger requests. This is a repeat-visit improvement, not a reduction in cold-load image bytes or a measured LCP gain.

Validation hashes every published derivative, checks its stable alias is byte-identical, checks all HTML and inline JSON resource paths, and tests that changed CSS/image bytes produce new URLs while identical inputs keep their versions. The full suite contains 24 Rust tests, plus production-page and architecture checks.

## Modern image formats: measured candidates

Pillow 12.2.0 was used for an isolated experiment on the existing web derivatives. AVIF used quality 85, speed 8, two threads; WebP used lossless mode, method 4. These are sample encodes, not production replacements or a measured whole-page saving.

| Sample | Current bytes | AVIF bytes | AVIF reduction | Lossless WebP bytes | Lossless WebP reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Quite Off, 1600w JPEG | 623,742 | 255,842 | 59.0% | 1,806,322 | −189.6% |
| About, 800w JPEG | 210,322 | 81,464 | 61.3% | 500,550 | −138.0% |
| The Threes, `threes_09`, 1024w PNG | 120,234 | 26,170 | 78.2% | 36,724 | 69.5% |

The AVIF candidates are lossy (PSNR 41.23, 44.78 and 53.51dB relative to the decoded current derivatives). Lossless WebP reproduced every decoded RGB pixel in these samples. It is promising for the book's fine text and linework; it is larger for these photographs. A production format rollout should encode from the source images, inspect grain, tonal transitions and text, and retain JPEG/PNG fallbacks. JPEG/PNG remain the live formats in this release; the reported 3.28MB/36-request cold-load result has not been independently reproduced under the same conditions.
