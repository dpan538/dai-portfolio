# China Crawlability Checklist

Use this after each deployment that changes routing, metadata, DNS, or crawl-related files.

## Deployment Verification

```bash
npm run build
npm run preview -- --host 127.0.0.1
curl -I http://127.0.0.1:4321/
curl -L http://127.0.0.1:4321/robots.txt
curl -L http://127.0.0.1:4321/sitemap.xml
```

After deployment:

```bash
curl -I -L https://daipan.art
curl -I -L https://www.daipan.art
curl -I -L http://daipan.art
curl -I -L http://www.daipan.art
curl -L https://daipan.art/robots.txt
curl -I -L https://daipan.art/sitemap.xml
curl -L https://daipan.art/sitemap.xml
curl -L https://daipan.art/sitemap-index.xml
```

Expected:

- `https://daipan.art` returns `200`.
- `www` redirects to `https://daipan.art/`.
- `http` redirects to `https`.
- `/robots.txt` returns `200`.
- `/sitemap.xml` returns `200`.
- `/sitemap-index.xml` returns `200`.

## DNS Checks

```bash
dig daipan.art
dig www.daipan.art
dig +short daipan.art A
dig +short daipan.art AAAA
dig +short www.daipan.art CNAME
dig +short www.daipan.art A
dig +short @1.1.1.1 daipan.art A
dig +short @8.8.8.8 daipan.art A
dig +short @223.5.5.5 daipan.art A
dig +short @114.114.114.114 daipan.art A
dig +short @223.5.5.5 www.daipan.art A
dig +short @114.114.114.114 www.daipan.art A
```

Fallback if `dig` is unavailable:

```bash
nslookup daipan.art
nslookup www.daipan.art
nslookup daipan.art 223.5.5.5
nslookup daipan.art 114.114.114.114
nslookup www.daipan.art 223.5.5.5
nslookup www.daipan.art 114.114.114.114
```

## No-JavaScript Crawler Checks

```bash
curl -L https://daipan.art -o /tmp/daipan-home.html
curl -L https://daipan.art/about -o /tmp/daipan-about.html
curl -L https://daipan.art/contact -o /tmp/daipan-contact.html
rg "Dai Pan|fine artist|designer|Quite Off|The Threes|Monolith|Reframed|Closing Time|Masks|dpan53853|WeChat" /tmp/daipan-*.html
rg "<div id=\"root\"|__NEXT_DATA__|data-reactroot" /tmp/daipan-*.html
```

Expected:

- The first `rg` finds readable identity text, project names, and contact details.
- The second `rg` finds nothing relevant for a client-only app shell.

## Metadata Checks

```bash
curl -L https://daipan.art -o /tmp/daipan-home.html
rg "title>|meta name=\"description\"|dc.description|canonical|og:title|og:description|og:image|og:url|twitter:card|twitter:image|application/ld\\+json|apple-touch-icon|theme-color" /tmp/daipan-home.html
```

Expected:

- Title: `Dai Pan | Artist & Designer Portfolio`
- Canonical: `https://daipan.art`
- OG URL: `https://daipan.art`
- OG image: stable `https://daipan.art/...` image URL
- JSON-LD includes `Person` and `WebSite`
- Chinese machine-readable description exists

## Sitemap URL Checks

Check each canonical URL from `sitemap.xml`:

```bash
curl -I -L https://daipan.art/
curl -I -L https://daipan.art/about
curl -I -L https://daipan.art/contact
curl -I -L https://daipan.art/quite_off
curl -I -L https://daipan.art/the_threes
curl -I -L https://daipan.art/gallery_design
curl -I -L https://daipan.art/reframed_still
curl -I -L https://daipan.art/closing_time
curl -I -L https://daipan.art/masks
```

Expected: each returns final `200`.

## Bot User-Agent Checks

```bash
curl -A Googlebot -I -L https://daipan.art
curl -A Bingbot -I -L https://daipan.art
curl -A Baiduspider -I -L https://daipan.art
curl -A "Sogou web spider" -I -L https://daipan.art
curl -A 360Spider -I -L https://daipan.art
curl -A Bytespider -I -L https://daipan.art
```

Expected: no `403`, no bot challenge, final `200`.

## Mainland China Manual Checks

- Baidu Search Resource Platform: verify ownership and submit `https://daipan.art/sitemap.xml`.
- Bing Webmaster Tools: submit `https://daipan.art/sitemap.xml`.
- Google Search Console: submit `https://daipan.art/sitemap.xml`.
- 360/Sogou webmaster or indexing tools: test root URL and sitemap where available.
- Aliyun/Tencent/站长工具: test DNS and HTTP from mainland nodes.
- Mainland mobile network: open `/`, `/about`, `/contact`, and two project pages.
- Check whether Google Fonts slow down visual loading; consider self-hosting if they do.

## Red Flags

- `https://daipan.art/sitemap.xml` returns `404`.
- Any canonical points to `localhost`, `www.daipan.art`, an old domain, or an empty URL.
- Sitemap includes `/quite_off_mobile`.
- HTML contains only a JS app shell and no body text.
- Bot user agents receive `403`, `503`, captcha, or WAF pages.
- Chinese DNS resolvers return no A records while global resolvers do.
