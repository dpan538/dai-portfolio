# daipan.art China Crawlability & AI Visibility Audit

Audit date: 2026-05-06  
Repository: `/Users/jarlgiovanni/Desktop/dai-portfolio`  
Canonical domain checked: `https://daipan.art`

## 1. Executive Summary

The site is an Astro static site, not a client-only SPA. The home page, About page, Contact page, project navigation, project titles, contact details, and meaningful body text are present in initial HTML, so the largest crawler risk is not JavaScript rendering.

The strongest online evidence points away from a real DNS outage: `daipan.art` resolves through local DNS, Cloudflare `1.1.1.1`, Google `8.8.8.8`, AliDNS `223.5.5.5`, and `114.114.114.114`. `www.daipan.art` also resolves and redirects to the non-www canonical domain. HTTPS is valid, HTTP redirects to HTTPS, and Vercel does not block tested `Baiduspider` or `Bytespider` user agents.

The main issues found were crawlability and disambiguation details: the deployed site had no `/sitemap.xml`, `robots.txt` only pointed to `sitemap-index.xml`, Astro's generated sitemap included `/quite_off_mobile/` even though that page canonicalizes to `/quite_off`, and Chinese/AI crawler identity signals could be clearer. These are plausible causes for weaker Chinese search/AI interpretation or tool misclassification, especially combined with the `.art` TLD and the generic phrase-like domain `daipan.art`.

Low-risk fixes were applied locally: `robots.txt` now explicitly allows common Chinese crawlers and references both `/sitemap.xml` and `/sitemap-index.xml`; a root `/sitemap.xml` now lists canonical pages with `lastmod`; Astro sitemap excludes the mobile duplicate; core metadata, Open Graph/Twitter image coverage, Chinese machine-readable description, theme color, apple touch icon, and JSON-LD identity wording were improved. These changes need deployment before online tools will see them.

## 2. Evidence Table

| Area | Test | Command / Method | Result | Severity | Recommendation |
|---|---|---|---|---|---|
| Tech stack | Identify framework and output mode | `package.json`, `astro.config.mjs`, `npm run build` | Astro `5.17.1`, `output: "static"`, pages generated to `dist/` | Info | Keep static output for crawler friendliness. |
| Initial HTML | Home no-JS text | `curl -L https://daipan.art` | Contains title, description, nav project titles, JSON-LD, and body text beginning `I am Dai Pan...` | Pass | No SSR/SPA emergency. |
| Initial HTML | About no-JS text | `curl -L https://daipan.art/about` | Contains bio, education/exhibition sections, contact email, disambiguation text | Pass | Keep About text in HTML. |
| Initial HTML | Contact no-JS text | `curl -L https://daipan.art/contact` | Contains email, phone, WeChat, Instagram, writing link | Pass | Contact is crawler-readable. |
| DNS | Apex A | `dig daipan.art` | `76.76.21.21`, `76.76.21.22`, `216.198.79.1` | Pass | Not an apex DNS outage from tested network. |
| DNS | Apex AAAA | `dig +short daipan.art AAAA` | No AAAA record | Low | Acceptable; optional IPv6 support depends on hosting needs. |
| DNS | www CNAME/A | `dig +short www.daipan.art CNAME`; `dig +short www.daipan.art A` | CNAME `cname.vercel-dns.com.`, A records returned | Pass | www is configured. |
| China DNS | AliDNS / 114 DNS | `dig +short @223.5.5.5 daipan.art A`; `dig +short @114.114.114.114 daipan.art A` | Both returned Vercel A records | Pass | Misreports by tools are likely tool-side or transient, not reproduced here. |
| Redirects | HTTPS apex | `curl -I -L https://daipan.art` | Final `HTTP/2 200`, server `Vercel` | Pass | Canonical endpoint healthy. |
| Redirects | HTTPS www | `curl -I -L https://www.daipan.art` | `308` to `https://daipan.art/`, then `200` | Pass | No www/non-www duplicate. |
| Redirects | HTTP apex | `curl -I -L http://daipan.art` | `308` to `https://daipan.art/`, then `200` | Pass | HTTPS enforced. |
| Redirects | HTTP www | `curl -I -L http://www.daipan.art` | `308` to `https://www.daipan.art/`, then `308` to apex, then `200` | Pass | Works, though two hops from HTTP www. |
| TLS | Apex certificate | `curl -vI https://daipan.art` | TLS 1.3, Let's Encrypt R12, SAN matches `daipan.art`, verify ok | Pass | No certificate-chain issue reproduced. |
| TLS | www certificate | `curl -vI https://www.daipan.art` | TLS 1.3, Let's Encrypt R12, SAN matches `www.daipan.art`, verify ok | Pass | www certificate healthy. |
| Bot access | Baiduspider UA | `curl -A Baiduspider -I -L https://daipan.art` | `HTTP/2 200` | Pass | No UA block reproduced. |
| Bot access | Bytespider UA | `curl -A Bytespider -I -L https://daipan.art` | `HTTP/2 200` | Pass | No UA block reproduced. |
| robots.txt | Deployed robots | `curl -L https://daipan.art/robots.txt` | Allowed `*`, sitemap points only to `sitemap-index.xml` | Low | Fixed locally to add Chinese crawlers and root sitemap. |
| sitemap.xml | Deployed root sitemap | `curl -I -L https://daipan.art/sitemap.xml` | `HTTP/2 404` | Medium | Fixed locally by adding `public/sitemap.xml`; deploy required. |
| sitemap-index | Deployed Astro sitemap | `curl -L https://daipan.art/sitemap-0.xml` | Listed pages, but included `/quite_off_mobile/`; no `lastmod` | Low | Fixed locally: static root sitemap has canonical URLs; Astro sitemap excludes duplicate and adds lastmod. |
| Metadata | Canonical | `rg "canonical" dist/*.html` | Canonicals use `https://daipan.art/...`; no localhost found | Pass | Continue using apex canonical. |
| Metadata | OG/Twitter | HTML inspection | Home had OG image; About/Contact previously had no image | Low | Fixed locally with default OG/Twitter image. |
| Metadata | Chinese signals | HTML inspection | Chinese keywords existed; machine-readable Chinese description was limited | Low | Fixed locally with `dc.description` `zh-Hans` and clearer JSON-LD. |
| External resources | Fonts | `rg "fonts.googleapis|fonts.gstatic" dist` | Google Fonts used site-wide | Medium for China UX | Consider self-hosting fonts or robust system fallback. |
| External resources | Embeds/analytics | `rg "analytics|iframe|youtube|cdn"` | No analytics, map, video, or iframe blockers found | Pass | Keep critical content independent of third parties. |
| Images | Large assets | `du -ah public | sort -h | tail` | Original image tree about 29 MB; optimized tree about 68 MB; largest originals about 1.6 MB | Low/Medium UX | Existing optimized variants help; consider AVIF/WebP later without overwriting originals. |
| Errors | 404 status | `curl -I -L https://daipan.art/404-check-not-real` | `HTTP/2 404`, serves `404.html` | Pass | Correct status. |
| Deep links | Project pages | `curl -I -L https://daipan.art/quite_off`; `/gallery_design` | `HTTP/2 200` | Pass | Static deep links work. |
| Local post-fix | Build | `npm run build` | Succeeded, 11 pages built | Pass | Safe to deploy. |
| Local post-fix | Preview sitemap URLs | `curl -I http://127.0.0.1:4321/{about,contact,quite_off,...}` | All sitemap URLs returned `200` locally | Pass | Deploy to make online root sitemap visible. |

## 3. Domain & DNS Findings

`daipan.art` is not currently failing DNS resolution from the tested environment. The apex domain returned three Vercel A records:

- `76.76.21.21`
- `76.76.21.22`
- `216.198.79.1`

Resolver checks:

- `dig +short @1.1.1.1 daipan.art A` returned the same Vercel A record set.
- `dig +short @8.8.8.8 daipan.art A` returned the same Vercel A record set.
- `dig +short @223.5.5.5 daipan.art A` returned the same Vercel A record set.
- `dig +short @114.114.114.114 daipan.art A` returned the same Vercel A record set.

`www.daipan.art` resolves through `cname.vercel-dns.com.` and returns Vercel A records on all tested resolvers, including `223.5.5.5` and `114.114.114.114`.

Conclusion: the tested evidence does not support “no DNS resolution / no server IP” as a current true state. Misclassification by Chinese tools is more likely caused by stale cache, temporary resolver failure, crawler limitations, confusion with similar names/domains, or sitemap/search-console discovery gaps.

Fallback commands if `dig` is unavailable:

```bash
nslookup daipan.art
nslookup daipan.art 223.5.5.5
nslookup daipan.art 114.114.114.114
nslookup www.daipan.art 223.5.5.5
nslookup www.daipan.art 114.114.114.114
```

## 4. HTTPS / Redirect Findings

The canonical endpoint `https://daipan.art` returns `HTTP/2 200`.

Redirect behavior:

- `https://www.daipan.art` -> `308` -> `https://daipan.art/`
- `http://daipan.art` -> `308` -> `https://daipan.art/`
- `http://www.daipan.art` -> `308` -> `https://www.daipan.art/` -> `308` -> `https://daipan.art/`

TLS findings:

- `daipan.art` certificate: Let's Encrypt R12, SAN matches `daipan.art`, verification OK.
- `www.daipan.art` certificate: Let's Encrypt R12, SAN matches `www.daipan.art`, verification OK.
- TLS 1.3 negotiated successfully in curl.

Conclusion: HTTPS and canonical redirect behavior are healthy. The two-hop redirect from `http://www` is normal for Vercel but can be manually checked in Chinese tools if they have strict redirect limits.

## 5. Crawler Visibility Findings

The site is built as static Astro HTML. It does not require JavaScript to expose the primary content:

- Home initial HTML contains identity text: `I am Dai Pan, a fine artist and designer...`
- Home initial HTML contains project navigation: `Quite Off`, `The Threes`, `Monolith for Light`, `Reframed Still`, `Closing Time`, `Masks`.
- About initial HTML contains the artist/designer bio and contact email.
- Contact initial HTML contains email, phone, WeChat, Instagram, writing link.

No evidence was found of a crawler seeing only `<div id="root"></div>` or a JS-only app shell. This is a major positive finding.

The site uses some JavaScript for interaction and mobile navigation, but that does not hide the core page content from crawlers.

## 6. robots.txt / sitemap.xml Findings

Before local fixes, online `robots.txt` was:

```txt
User-agent: *
Allow: /

Sitemap: https://daipan.art/sitemap-index.xml
```

This is not blocking crawlers, but it is minimal and does not expose the expected root `sitemap.xml`. Online `https://daipan.art/sitemap.xml` returned `HTTP/2 404` during this audit.

Local fixes now provide:

```txt
User-agent: *
Allow: /

User-agent: Baiduspider
Allow: /

User-agent: Sogou web spider
Allow: /

User-agent: 360Spider
Allow: /

User-agent: Bytespider
Allow: /

Sitemap: https://daipan.art/sitemap.xml
Sitemap: https://daipan.art/sitemap-index.xml
```

Local `public/sitemap.xml` now includes canonical URLs for:

- `/`
- `/about`
- `/contact`
- `/quite_off`
- `/the_threes`
- `/gallery_design`
- `/reframed_still`
- `/closing_time`
- `/masks`

It excludes `/quite_off_mobile`, because that page redirects mobile users client-side and canonicalizes to `/quite_off`. Astro's generated sitemap was also configured to exclude `/quite_off_mobile/` and include `lastmod`.

## 7. Metadata / Open Graph Findings

Existing strengths:

- `<html lang="en">` is present.
- Page titles and descriptions exist.
- Canonical tags use `https://daipan.art/...`.
- Open Graph and Twitter Card tags exist.
- JSON-LD `Person`, `WebSite`, `BreadcrumbList`, and home `ItemList` exist.
- `favicon.svg` and `favicon.ico` exist.

Local improvements applied:

- Home title changed to `Dai Pan | Artist & Designer Portfolio`.
- Chinese machine-readable description added as `dc.description` with `lang="zh-Hans"`.
- `og:site_name`, `og:locale`, and `og:locale:alternate` added.
- Default OG/Twitter image now applies to pages that did not pass a project-specific image.
- `theme-color` added.
- `apple-touch-icon.png` generated from the existing favicon.
- JSON-LD `Person` now says `Artist and Designer` and includes a concise official portfolio description.
- JSON-LD `WebSite` now uses `Dai Pan | Artist & Designer Portfolio` and stronger official/canonical identity wording.

No localhost, old domain, `www`, or template canonical URLs were found in generated HTML.

## 8. China Internet Compatibility Findings

Potential China-access risks:

- Google Fonts are loaded from `fonts.googleapis.com` and `fonts.gstatic.com`. In mainland China, these can be slow or blocked. This does not usually block crawler access to the HTML, but it can hurt visual load and perceived availability.
- Hosting is Vercel. Vercel is globally fast in many regions but may be inconsistent from mainland China depending on ISP, region, and route.
- Instagram links are present. They are external and may not load in mainland China, but they do not block the page's primary HTML.
- No Google Analytics, map embeds, YouTube/Vimeo embeds, or third-party API dependency was found in critical rendering.
- Key images are local under `/projects` and `/_optimized`; they do not depend on a third-party image CDN.

Image size observations:

- `public/projects` is about 29 MB.
- `public/_optimized` is about 68 MB after generated responsive variants.
- Largest original assets observed are around 1.6 MB (`closing_time.jpg`, `masks01.jpg`), with many optimized variants under or around 1 MB.

Recommended China UX improvement: self-host the current fonts or add a system-font-first fallback strategy after visual approval. Do not delete the current visual font choice without checking the design.

## 9. AI Crawler / LLM Tool Compatibility Findings

AI/browser-like crawlers should be able to read the site because the content is static HTML. The most likely AI-tool misclassification causes are:

- Missing deployed `/sitemap.xml` at the common root path.
- Some tools may not follow `sitemap-index.xml` reliably.
- Generic `.art` TLD plus `daipan art` can be interpreted as keywords rather than an official domain.
- Similar-name confusion can occur without repeated official identity signals.
- Some Chinese tools may use stale DNS/cache data or limited crawler infrastructure.
- Some tools may penalize or fail on Vercel routes from certain mainland networks even when global DNS is correct.

The local fixes strengthen AI-crawler interpretation by adding canonical root sitemap, clearer title, Chinese machine-readable description, stable social preview image, and stronger JSON-LD identity text.

## 10. Recommended Fix Plan

P0: must fix

- Deploy the local changes so `https://daipan.art/sitemap.xml` returns `200`.
- Submit `https://daipan.art/sitemap.xml` in Google Search Console, Bing Webmaster Tools, and Baidu Search Resource Platform.
- Verify the site in Baidu and 360 webmaster platforms, then add their verification meta tags only after receiving real codes.

P1: should fix

- Self-host Google Fonts or define a tested system-font fallback path for China access.
- Test the deployed site from mainland networks and Chinese webmaster tools after deployment.
- Keep `/quite_off_mobile` out of public sitemaps because it is a mobile redirect variant of `/quite_off`.
- Add Chinese-visible clarification to About or footer only if you are comfortable with a small content addition, for example: `Official portfolio: daipan.art`. This was not added visually in this audit.

P2: nice to have

- Generate AVIF/WebP derivatives for large images while preserving original archival images.
- Add `width`/`height` to remaining raw `<img>` elements that do not use `OptimizedImage`.
- Consider project-level `CreativeWork` JSON-LD only where the page already contains enough clear title/medium/year information.
- Consider a future `/zh` or bilingual intro page if Chinese search visibility becomes a core goal.

## 11. Files Changed

- `public/robots.txt`  
  Added explicit allow rules for `Baiduspider`, `Sogou web spider`, `360Spider`, and `Bytespider`; added root `sitemap.xml` reference while keeping Astro `sitemap-index.xml`.

- `public/sitemap.xml`  
  Added a minimal canonical sitemap with home, About, Contact, and all project pages, with `lastmod` values.

- `astro.config.mjs`  
  Configured Astro sitemap to exclude `/quite_off_mobile/` and add `lastmod` to generated sitemap entries.

- `src/layouts/base_layout.astro`  
  Added stronger machine-readable identity metadata, Chinese `dc.description`, default OG/Twitter image coverage, OG locale fields, favicon/apple/theme metadata, and clearer JSON-LD wording.

- `src/pages/index.astro`  
  Updated home title to `Dai Pan | Artist & Designer Portfolio`.

- `public/apple-touch-icon.png`  
  Generated a 180x180 PNG touch icon from the existing SVG favicon.

- `docs/china_crawlability_audit.md`  
  This audit report.

- `docs/china_crawlability_checklist.md`  
  Follow-up operational checklist.

## 12. Manual Checks Required

These checks cannot be completed fully from this local environment:

- Deploy the changes, then re-run `curl -I https://daipan.art/sitemap.xml` and confirm online `200`.
- Test DNS in Aliyun DNS, Tencent Cloud DNS, Baidu webmaster tools, 360 webmaster tools, and common Chinese “站长工具”.
- Verify site ownership in Baidu Search Resource Platform.
- Submit `https://daipan.art/sitemap.xml` to Baidu Search Resource Platform.
- Submit `https://daipan.art/sitemap.xml` to Bing Webmaster Tools.
- Submit `https://daipan.art/sitemap.xml` to Google Search Console.
- Test access from mainland China using mobile data, a domestic VPN endpoint, or a trusted friend in mainland China.
- Check whether Vercel routes are unstable from specific Chinese ISPs; if so, consider a mainland-friendly CDN or mirror strategy.
- Decide whether to self-host fonts after visual comparison.

## 13. Verification Commands Run

Build and local preview:

```bash
npm run build
npm run preview -- --host 127.0.0.1
curl -I http://127.0.0.1:4321/
curl -L http://127.0.0.1:4321/robots.txt
curl -L http://127.0.0.1:4321/sitemap.xml
curl -I http://127.0.0.1:4321/not-real
```

Local post-fix results:

- `npm run build` succeeded.
- Local `/` returned `200`.
- Local `/robots.txt` included Chinese crawler allow rules and both sitemap references.
- Local `/sitemap.xml` returned canonical URLs with `lastmod`.
- Local unknown path returned `404`.
- Local sitemap page URLs returned `200`.
- Local key images returned `200`.

Online checks run before deployment of local fixes:

```bash
curl -I -L https://daipan.art
curl -I -L https://www.daipan.art
curl -I -L http://daipan.art
curl -I -L http://www.daipan.art
curl -L https://daipan.art
curl -L https://daipan.art/about
curl -L https://daipan.art/contact
curl -L https://daipan.art/robots.txt
curl -I -L https://daipan.art/sitemap.xml
curl -L https://daipan.art/sitemap-index.xml
curl -L https://daipan.art/sitemap-0.xml
curl -A Baiduspider -I -L https://daipan.art
curl -A Bytespider -I -L https://daipan.art
```

Pre-deployment note: online `https://daipan.art/sitemap.xml` returned `404` during the initial audit before the crawlability changes were deployed.

## Post-deployment Verification

Verification date: 2026-05-06  
Deployment target observed: Vercel (`server: Vercel` response header)  
Git state before verification: `main` was clean and up to date with `origin/main`; latest visible commit was `7ade4da 中国网站抓取适配`.

| Test | URL / Command | Expected | Actual | Status |
|---|---|---|---|---|
| Homepage status | `curl -I https://daipan.art` | `200` | `HTTP/2 200`, `content-type: text/html; charset=utf-8`, `server: Vercel` | Pass |
| www redirect | `curl -I -L https://www.daipan.art` | Redirect to `https://daipan.art/`, final `200` | `308` to `https://daipan.art/`, final `HTTP/2 200` | Pass |
| HTTP to HTTPS redirect | `curl -I -L http://daipan.art` | Redirect to HTTPS canonical | `308` to `https://daipan.art/` | Pass |
| HTTP www redirect | `curl -I -L http://www.daipan.art` | Redirect to HTTPS non-www canonical | `308` to `https://www.daipan.art/`, then `308` to `https://daipan.art/`, final `200` | Pass |
| robots.txt | `curl -L https://daipan.art/robots.txt` | `200`, allows crawlers, includes sitemap | Returned crawler allow rules and `Sitemap: https://daipan.art/sitemap.xml` | Pass |
| sitemap.xml | `curl -I -L https://daipan.art/sitemap.xml` | `200`, XML response | `HTTP/2 200`, `content-type: application/xml`, `content-length: 960` | Pass |
| sitemap URL status check | `curl -L -o /dev/null -w "%{http_code}"` for all sitemap URLs | Every listed URL returns `200` | `/`, `/about`, `/contact`, `/quite_off`, `/the_threes`, `/gallery_design`, `/reframed_still`, `/closing_time`, `/masks` all returned `200` | Pass |
| sitemap exclusions | `rg "localhost|daipan\.ink|quite_off_mobile" /tmp/daipan-online-sitemap.xml` | No matches | No matches found | Pass |
| Homepage metadata | `curl -L https://daipan.art \| head -n 120` | Title, description, canonical, OG, Twitter, Chinese machine-readable description | Found `Dai Pan | Artist & Designer Portfolio`, meta description, `dc.description`, canonical `https://daipan.art`, OG/Twitter tags | Pass |
| JSON-LD | `rg "application/ld\+json|Artist and Designer|Official portfolio" /tmp/daipan-online-home.html` | JSON-LD present and identity clear | `Person`, `WebSite`, and `ItemList` JSON-LD present with official portfolio wording | Pass |
| canonical | HTML inspection | Canonical uses `https://daipan.art` | Homepage canonical `https://daipan.art`; About/Contact canonical use `https://daipan.art/about` and `/contact` | Pass |
| no-JS content | `curl -L https://daipan.art/about`; `curl -L https://daipan.art/contact` | Readable body text in initial HTML | About contains bio/education/contact; Contact contains email, phone, WeChat, writing link | Pass |
| 404 behavior | `curl -I -L https://daipan.art/not-real-post-deploy-check` | `404` | `HTTP/2 404`, serves `404.html` | Pass |

Post-deployment conclusion: the root sitemap issue is fixed online. `https://daipan.art/sitemap.xml` now returns `200` instead of `404`, `robots.txt` references it, all sitemap URLs return `200`, and the deployed HTML exposes metadata, JSON-LD, canonical tags, and no-JS readable content.
