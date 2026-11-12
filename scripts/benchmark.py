#!/usr/bin/env python3
"""Compare the locked historical build with Rust. Node is used ONLY in the archived baseline.

Before running: build production Rust output and install the old lockfile in an
isolated archive of BASELINE. See reports/performance/README.md for exact commands.
No network timings or Core Web Vitals are inferred from these measurements.
"""
import argparse
import gzip
import hashlib
import json
import os
import platform
import re
import statistics
import subprocess
import time
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = '32b9627c096db65c37834800724005d85427aba9'
ROUTES = ['', 'about', 'contact', 'quite_off', 'the_threes', 'gallery_design', 'reframed_still', 'closing_time', 'masks']

def run(command, cwd, log):
    start = time.perf_counter()
    env = dict(os.environ, ASTRO_TELEMETRY_DISABLED='1')
    with log.open('w') as stream:
        subprocess.run(command, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT, check=True)
    return round(time.perf_counter() - start, 4)

def distribution(values):
    return {'samples': values, 'median': round(statistics.median(values), 4), 'min': min(values), 'max': max(values)}

def tree_bytes(path):
    return sum(p.stat().st_size for p in path.rglob('*') if p.is_file() and not p.name.startswith('.'))

def image_files(path):
    return {p.relative_to(path).as_posix(): p for p in path.rglob('*') if p.suffix.lower() in {'.jpg', '.jpeg', '.png'}}

class Page(HTMLParser):
    def __init__(self):
        super().__init__(); self.resources = set(); self.images = []; self.script_count = 0; self.inlined = False
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == 'img': self.images.append(a)
        if tag == 'script' and a.get('type') not in {'application/json', 'application/ld+json'}:
            self.script_count += 1
            if a.get('src', '').startswith('/'): self.resources.add(a['src'])
        if tag == 'link' and a.get('rel') in {'stylesheet', 'modulepreload'} and a.get('href', '').startswith('/'):
            self.resources.add(a['href'])

def payload(output, rust):
    pages = []; union = set()
    for route in ROUTES:
        file = output / route / 'index.html'
        data = file.read_bytes(); parser = Page(); parser.feed(data.decode())
        resources = set(parser.resources)
        if rust: resources.update({'/assets/browser.js', '/assets/browser_bg.wasm'})
        # Resolve static module imports in emitted scripts; remote fonts are excluded.
        pending = list(resources)
        while pending:
            url = pending.pop(); path = output / url.lstrip('/')
            if path.suffix != '.js' or not path.is_file(): continue
            for imported in re.findall(r'(?:from\s*|import\s*)[\'"]([^\'"]+\.js)[\'"]', path.read_text()):
                resolved = (path.parent / imported).resolve()
                if resolved.is_relative_to(output.resolve()):
                    child = '/' + resolved.relative_to(output.resolve()).as_posix()
                    if child not in resources: resources.add(child); pending.append(child)
        blobs = [data] + [(output / u.lstrip('/')).read_bytes() for u in sorted(resources) if (output / u.lstrip('/')).is_file()]
        union.update(resources)
        images = [im for im in parser.images if im.get('src')]
        pages.append({'route': '/' + route, 'html_bytes': len(data), 'control_bytes': sum(map(len, blobs)),
                      'control_gzip_bytes': sum(len(gzip.compress(b, mtime=0)) for b in blobs),
                      'scripts': parser.script_count, 'images': len(images),
                      'images_with_srcset': sum(bool(im.get('srcset')) for im in images),
                      'images_with_dimensions': sum(bool(im.get('width') and im.get('height')) for im in images),
                      'eager_images': sum(im.get('loading') != 'lazy' for im in images)})
    return {'routes': pages, 'median_control_gzip_bytes': statistics.median(p['control_gzip_bytes'] for p in pages),
            'shared_control_resources': sorted(union)}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--runs', type=int, default=3)
    parser.add_argument('--output', type=Path, default=ROOT / 'reports/performance/results.json')
    args = parser.parse_args(); baseline = args.baseline.resolve()
    if args.runs < 3: parser.error('Use at least three warm repetitions')
    expected = subprocess.check_output(['git', 'show', BASELINE + ':package-lock.json'], cwd=ROOT)
    assert (baseline / 'package-lock.json').read_bytes() == expected, 'Baseline lockfile differs from the deployed commit'
    binary = ROOT / 'target/release/portfolio-site'
    assert binary.is_file() and (ROOT / 'dist/assets/browser_bg.wasm').is_file(), 'Build production Rust output first'
    before = image_files(baseline / 'public/projects'); after = image_files(ROOT / 'public/projects')
    assert all(k in after and hashlib.sha256(p.read_bytes()).digest() == hashlib.sha256(after[k].read_bytes()).digest() for k,p in before.items()), 'Shared originals must be byte-identical'
    logs = ROOT / 'target/benchmark/logs'; logs.mkdir(parents=True, exist_ok=True)
    cold_cache = baseline / '.cache/images'
    assert not cold_cache.exists(), 'Cold image cache already exists; use a fresh baseline archive for a reproducible cold sample'
    print('Measuring identical 59-image cold pipelines…', flush=True)
    cold_old = run(['node', 'scripts/get-image-dimensions.mjs'], baseline, logs / 'images-legacy-cold.log')
    cold_new = run([str(binary), 'images', str(ROOT / 'target/benchmark/shared-images'), str(baseline)], ROOT, logs / 'images-rust-cold.log')
    warm_old = []; warm_new = []; builds_old = []; builds_new = []
    for i in range(args.runs):
        print(f'Warm repetition {i + 1}/{args.runs}…', flush=True)
        # Alternate execution order to reduce systematic thermal/order bias.
        jobs = [('old', ['node', 'scripts/get-image-dimensions.mjs'], baseline, warm_old),
                ('new', [str(binary), 'images', str(ROOT / 'target/benchmark/shared-images'), str(baseline)], ROOT, warm_new)]
        for label, cmd, cwd, samples in jobs[::(-1 if i % 2 else 1)]:
            samples.append(run(cmd, cwd, logs / f'images-{label}-warm-{i}.log'))
        jobs = [('old', ['npm', 'run', 'build'], baseline, builds_old),
                ('new', [str(binary), 'build', str(ROOT / 'target/benchmark/current'), '--production'], ROOT, builds_new)]
        for label, cmd, cwd, samples in jobs[::(-1 if i % 2 else 1)]:
            samples.append(run(cmd, cwd, logs / f'build-{label}-{i}.log'))
    result = {'measured_at': datetime.now(timezone.utc).isoformat(), 'baseline_commit': BASELINE,
              'candidate_source': 'Rust migration working tree; final commit recorded in release audit',
              'environment': {'system': platform.platform(), 'machine': platform.machine(), 'logical_cpus': os.cpu_count(),
                              'rust': subprocess.check_output(['rustc','--version'], text=True).strip(),
                              'node_for_baseline_only': subprocess.check_output(['node','--version'], text=True).strip(),
                              'python': platform.python_version()},
              'method': {'warm_runs': args.runs, 'cold_runs': 1, 'shared_originals': len(before), 'current_originals': len(after),
                         'compile_and_dependency_download_excluded': True, 'compression': 'gzip level 9, each resource independently; estimate, not HTTP trace',
                         'image_policy': 'JPEG quality 95 / PNG; widths 400,640,800,1024,1280,1600; different encoders, not byte-identical derivatives',
                         'matched_routes': ROUTES, 'remote_fonts_excluded': True},
              'images': {'legacy_cold_seconds': cold_old, 'rust_cold_seconds': cold_new,
                         'legacy_warm_seconds': distribution(warm_old), 'rust_warm_seconds': distribution(warm_new),
                         'legacy_variants_bytes': tree_bytes(baseline / 'public/_optimized'),
                         'rust_shared_variants_bytes': tree_bytes(ROOT / 'target/benchmark/shared-images/_optimized')},
              'warm_build': {'legacy_seconds': distribution(builds_old), 'rust_seconds': distribution(builds_new)},
              'output': {'legacy_bytes': tree_bytes(baseline / 'dist'), 'rust_bytes': tree_bytes(ROOT / 'target/benchmark/current'),
                         'legacy_pages': len(list((baseline / 'dist').rglob('*.html'))), 'rust_pages': len(list((ROOT / 'dist').rglob('*.html'))),
                         'rust_wasm_bytes': (ROOT / 'dist/assets/browser_bg.wasm').stat().st_size},
              'payload': {'legacy': payload(baseline / 'dist', False), 'rust': payload(ROOT / 'dist', True)}}
    sources = sorted([p for folder in ('crates', 'site') for p in (ROOT / folder).rglob('*') if p.is_file()] +
                     [ROOT / p for p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'scripts/site', 'vercel.json')])
    digest = hashlib.sha256()
    for path in sources:
        digest.update(path.relative_to(ROOT).as_posix().encode() + b'\0' + path.read_bytes() + b'\0')
    result['candidate_source_sha256'] = digest.hexdigest()
    args.output.parent.mkdir(parents=True, exist_ok=True); args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(f'Results: {args.output}', flush=True)

if __name__ == '__main__': main()
