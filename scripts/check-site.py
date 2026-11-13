#!/usr/bin/env python3
"""Check the actual static output without third-party Python or Node packages."""
import json
import hashlib
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parent.parent
output = (root / (sys.argv[1] if len(sys.argv) > 1 else 'target/preview')).resolve()
failures = []
pages = list(output.rglob('*.html'))

class PageParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.assets = []
        self.canonicals = []
        self.json_blocks = []
        self.json_type = None
        self.json_text = ''
        self.copies = set()
        self.images = []
        self.descriptions = []
        self.robots = []
        self.themes = []
        self.links = []
        self.disclosures = 0
        self.content_order = []
        self.semantic_keys = set()
        self.public_text = []
        self.in_script = False
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == 'script': self.in_script = True
        if tag in ('img', 'script', 'link'):
            value = a.get('src') if tag != 'link' else a.get('href')
            if value and value.startswith('/') and not value.startswith('//'):
                self.assets.append(value)
        if tag == 'img':
            self.images.append(a)
            self.content_order.append(('image', a.get('src', '')))
            for item in a.get('srcset', '').split(','):
                if item.strip(): self.assets.append(item.strip().split()[0])
        if tag == 'a': self.links.append(a.get('href', ''))
        if tag == 'meta' and a.get('name') == 'description':
            self.descriptions.append(a.get('content', ''))
        if tag == 'meta' and a.get('name') == 'robots':
            self.robots.append(a.get('content', ''))
        if tag == 'meta' and a.get('name') == 'theme-color':
            self.themes.append(a.get('content', ''))
        if tag == 'link' and a.get('rel') == 'canonical':
            self.canonicals.append(a.get('href'))
            self.assets = [u for u in self.assets if not u.startswith('https:')]
        if 'data-copy' in a:
            self.copies.add(a['data-copy'])
            self.content_order.append(('copy', a['data-copy']))
        if a.get('id') == 'project-info': self.disclosures += 1
        if 'semantic' in a.get('class', '').split() and a.get('data-key'):
            self.semantic_keys.add(a['data-key'])
        if tag == 'script' and a.get('type') in ('application/json', 'application/ld+json'):
            self.json_type = a['type']
            self.json_text = ''
    def handle_data(self, data):
        if self.json_type: self.json_text += data
        if not self.in_script: self.public_text.append(data)
    def handle_endtag(self, tag):
        if tag == 'script': self.in_script = False
        if tag == 'script' and self.json_type:
            self.json_blocks.append(self.json_text)
            self.json_type = None

def check(condition, message):
    if not condition: failures.append(message)

expected = json.loads((root / 'site/content/pages.json').read_text())
galleries = json.loads((root / 'site/content/galleries.json').read_text())
person = json.loads((root / 'site/content/person.json').read_text())
local = (output / '.portfolio-build').read_text().strip() == 'local'
cache_manifest = json.loads((output / 'assets/cache-manifest.json').read_text())
asset_prefix = cache_manifest['asset_prefix']
check(bool(re.fullmatch(r'/assets/immutable/[0-9a-f]{64}', asset_prefix)), 'Assets need a content-derived version')
for old_url, immutable_url in cache_manifest['images'].items():
    data = (output / immutable_url.lstrip('/')).read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    check(immutable_url.startswith(f'/_optimized/immutable/{digest}/'), f'Image URL does not match its bytes: {immutable_url}')
    check(data == (output / old_url.lstrip('/')).read_bytes(), f'Legacy image alias changed pixels: {old_url}')
for dependency in ['start.js', 'browser.js', 'browser_bg.wasm']:
    check((output / asset_prefix.lstrip('/') / dependency).is_file(), f'Versioned loader dependency missing: {dependency}')
check(len(pages) in (len(expected), len(expected) + 1 + len(galleries)), f'Unexpected page count: {len(pages)}')
for page in expected:
    path = output / ('index.html' if page['key'] == 'home' else '404.html' if page['key'] == '404' else page['key'] + '/index.html')
    check(path.is_file(), f'Missing route {path}')

all_copies = set()
for path in pages:
    html = path.read_text()
    parser = PageParser()
    parser.feed(html)
    is_review = 'review' in path.relative_to(output).parts
    if not is_review:
        check(parser.themes == ['#ffffff'], f'{path}: expected one shell-controlled browser theme color')
        check(not re.search(r'[\u3400-\u9fff]', ''.join(parser.public_text)), f'{path}: Chinese public copy found')
        key = '404' if path.name == '404.html' else 'home' if path.parent == output else path.parent.name
        check(parser.disclosures == int(key in {'reframed_still', 'closing_time', 'masks'}), f'{path}: incorrect project disclosure mode')
        if key.endswith('_mobile'):
            check('data-view="mobile"' in html, f'{path}: independent mobile shell missing')
            check('width=device-width, initial-scale=1, viewport-fit=cover' in html, f'{path}: mobile Safari safe-area viewport missing')
            check(f'{asset_prefix}/mobile/common.css' in parser.assets, f'{path}: mobile design system missing')
            check(f'{asset_prefix}/presentation.css' not in parser.assets, f'{path}: desktop presentation loaded in mobile view')
            check('<aside' not in html, f'{path}: desktop sidebar present in mobile shell')
        if key == 'the_threes_mobile':
            check(sum(bool(re.search(r'/threes_\d+(?:_\d+w)?\.(?:jpg|png)$', im.get('src',''))) for im in parser.images) == 10, f'{path}: curated mobile book must contain two mockups and eight selected spreads')
        if key == 'in_praise_of_time_mobile':
            check(sum(bool(im.get('src')) for im in parser.images) == 14, f'{path}: every photograph plus the empty preview must be retained')
            check('Riding wings' in html and 'A flowing song' in html and 'https://daipan.ink/still-life/a-flowing-song' in parser.links, f'{path}: short attributed poem excerpt or direct poem link missing')
            check('Names are written on the map' not in html and 'time-vocabulary' not in html, f'{path}: full desktop poem or vocabulary leaked into mobile source')
        if key == 'quite_off':
            data = next((json.loads(block) for block in parser.json_blocks if isinstance(json.loads(block), dict) and 'semanticMap' in json.loads(block)), {})
            mapping = data.get('semanticMap', {})
            check(set(mapping) == {str(i) for i in range(len(data.get('images', [])))}, f'{path}: every frame needs a keyword mapping')
            used = set()
            for frame, keys in mapping.items():
                check(bool(keys) and set(keys) <= parser.semantic_keys, f'{path}: frame {frame} references missing keyword text')
                used.update(keys)
            check(used == parser.semantic_keys, f'{path}: unused narrative keyword')
        if key == 'the_threes':
            ending = next((i for i, (kind, src) in enumerate(parser.content_order) if kind == 'image' and re.search(r'/threes_24(?:_\d+w)?\.jpg$', src)), None)
            last_spread = next((i for i, (kind, src) in enumerate(parser.content_order) if kind == 'image' and re.search(r'/threes_22(?:_\d+w)?\.png$', src)), None)
            for copy_key in ['the_threes.01', 'the_threes.02', 'the_threes.03', 'the_threes.04']:
                position = next((i for i, item in enumerate(parser.content_order) if item == ('copy', copy_key)), None)
                check(None not in (last_spread, position, ending) and last_spread < position < ending, f'{path}: book statement must follow spreads and precede final photo')
        check(len(parser.canonicals) == 1, f'{path}: one canonical required')
        check(all(u.startswith('https://daipan.art') for u in parser.canonicals), f'{path}: incorrect canonical')
        check(f'{asset_prefix}/start.js' in parser.assets, f'{path}: versioned WASM loader missing')
        check(not re.search(r'/_optimized/projects/|/assets/(?!immutable/)', html), f'{path}: mutable runtime resource reference')
        check(len(parser.descriptions) == 1 and bool(parser.descriptions[0].strip()), f'{path}: one nonempty description required')
        check(('noindex' in ','.join(parser.robots)) == (local or path.name == '404.html'), f'{path}: incorrect indexing mode')
    check('{{' not in html, f'{path}: unfilled template')
    check('/_astro/' not in html and 'data-astro-' not in html, f'{path}: legacy runtime reference')
    for asset in parser.assets:
        resource = output / urlsplit(asset).path.lstrip('/')
        check(resource.is_file(), f'{path}: missing asset {asset}')
    for image in parser.images:
        check('alt' in image, f'{path}: image missing alt')
        if image.get('src') and not is_review:
            check(bool(image.get('alt', '').strip()), f'{path}: content image has empty alt')
            check('srcset' in image and 'sizes' in image, f'{path}: content image lacks responsive variants')
            check('width' in image and 'height' in image, f'{path}: content image lacks intrinsic dimensions')
    for block in parser.json_blocks:
        try: data = json.loads(block)
        except json.JSONDecodeError as e: failures.append(f'{path}: invalid JSON: {e}')
        else:
            def image_urls(value):
                if isinstance(value, str) and value.startswith('/_optimized/'):
                    yield from re.findall(r'/_optimized/[^\s,]+', value)
                elif isinstance(value, dict):
                    for item in value.values(): yield from image_urls(item)
                elif isinstance(value, list):
                    for item in value: yield from image_urls(item)
            for image_url in image_urls(data):
                check(image_url.startswith('/_optimized/immutable/') and (output / image_url.lstrip('/')).is_file(), f'{path}: unversioned or missing carousel image: {image_url}')
            if isinstance(data, dict) and data.get('@type') == 'Person':
                check(data == person, f'{path}: author entity has drifted')
            if isinstance(data, dict) and 'artMedium' in data:
                check(data.get('creator', {}).get('@id') == person['@id'], f'{path}: work is not linked to the shared author')
                for src in data.get('image', []):
                    check((output / urlsplit(src).path.lstrip('/')).is_file(), f'{path}: missing schema image {src}')
            if not local and isinstance(data, dict):
                check(not data.get('drafts') and not data.get('notes'), f'{path}: production contains editorial data')
    for href in parser.links:
        if href.startswith('/') and not href.startswith('//'):
            route = output / urlsplit(href).path.lstrip('/')
            check(route.is_file() or (route / 'index.html').is_file(), f'{path}: broken internal link {href}')
    all_copies.update(parser.copies)

copy = json.loads((root / 'site/content/copy.json').read_text())
drafts = json.loads((root / 'site/content/professional.json').read_text())
for page, blocks in drafts.items():
    for key in blocks:
        check(key in copy.get(page, {}), f'Draft has unknown copy key {key}')
        check(key in all_copies, f'Draft is not connected to a page: {key}')
notes = json.loads((root / 'site/content/review.json').read_text())
check(len(set(n['id'] for n in notes)) == len(notes), 'Duplicate review IDs')
check((output / 'assets/browser_bg.wasm').is_file(), 'WASM binary missing')
loader = (output / 'assets/start.js').read_text()
check('browser.js' in loader and 'browser_bg.wasm' in loader, 'Loader does not initialise WASM')
sitemap = ET.parse(output / 'sitemap.xml')
urls = [e.text for e in sitemap.iter('{http://www.sitemaps.org/schemas/sitemap/0.9}loc')]
canonical_pages = [p for p in expected if p['key'] != '404' and not p['key'].endswith('_mobile')]
check(len(urls) == len(canonical_pages), f'Unexpected canonical URL count: {len(urls)}')
check(not any('_mobile' in u for u in urls), 'Mobile duplicate in sitemap')
llms_path = output / 'llms.txt'
check(llms_path.is_file(), 'Missing llms.txt')
if llms_path.is_file():
    llms = llms_path.read_text()
    for url in re.findall(r'\]\((https://daipan\.art[^)]*)\)', llms):
        check(url in urls, f'llms.txt: noncanonical or unknown page {url}')
    for key in ['gallery_design', 'in_praise_of_time', 'the_threes', 'about', 'contact']:
        page = next(p for p in expected if p['key'] == key)
        check(page['canonical'] in llms, f'llms.txt: missing {key}')
for gallery in galleries:
    page = output / gallery['page'] / 'index.html'
    parser = PageParser(); parser.feed(page.read_text())
    work = next((json.loads(b) for b in parser.json_blocks if isinstance(json.loads(b), dict) and json.loads(b).get('@type') == 'VisualArtwork'), {})
    check(len(work.get('image', [])) == gallery['expected'], f'{page}: schema image count has drifted from gallery')
check('github.com/dpan538' in (output / 'contact/index.html').read_text(), 'Contact GitHub link missing')
check(not (output / '_astro').exists(), 'Legacy Astro assets in output')

if failures:
    print('\n'.join(failures), file=sys.stderr)
    sys.exit(1)
print(f'Checked {len(pages)} pages, responsive images, internal links, {len(all_copies)} copy blocks, {len(notes)} review notes, author/work schemas, indexing mode, sitemap and llms.txt.')
