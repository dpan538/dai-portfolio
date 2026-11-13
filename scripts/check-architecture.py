#!/usr/bin/env python3
"""Guard the maintained source and production output against a legacy runtime."""
import json
import os
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
output = root / (sys.argv[1] if len(sys.argv) > 1 else 'dist')
excluded = {'.git', '.cache', 'target', 'dist', '.vercel'}
files = []
for directory, children, names in os.walk(root):
    children[:] = [name for name in children if name not in excluded]
    files.extend(Path(directory) / name for name in names)
errors = []

def check(condition, message):
    if not condition:
        errors.append(message)

legacy_names = {'package.json', 'package-lock.json', 'yarn.lock', 'pnpm-lock.yaml', 'astro.config.mjs', 'tsconfig.json'}
for path in files:
    check(path.name not in legacy_names, f'Legacy build manifest: {path.relative_to(root)}')
    check(path.suffix not in {'.astro', '.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs'}, f'Non-Rust application source: {path.relative_to(root)}')
for crate in ['site', 'browser', 'gallery-core']:
    check((root / f'crates/{crate}/Cargo.toml').is_file(), f'Missing Rust crate: {crate}')
config = json.loads((root / 'vercel.json').read_text())
check(config.get('framework') is None, 'Vercel framework must be Other (null)')
check(config.get('installCommand') == '', 'Vercel must not run a Node install')
check(config.get('buildCommand') == 'sh scripts/build-vercel', 'Vercel must use the Rust build entry point')
check(config.get('outputDirectory') == 'dist', 'Vercel output must be dist')
immutable_rules = {rule['source'] for rule in config['headers'] if any('immutable' in header['value'] for header in rule['headers'])}
check(immutable_rules == {'/assets/immutable/(.*)', '/_optimized/immutable/(.*)'}, 'Only content-versioned resources may be immutable')
marker = output / '.portfolio-build'
check(marker.is_file() and marker.read_text().strip() == 'static', 'Expected a production Rust build')
check(not (output / 'review').exists(), 'Local review pages published')
check(not (output / 'assets/review.css').exists(), 'Local review stylesheet published')
wasm = output / 'assets/browser_bg.wasm'
check(wasm.is_file() and wasm.read_bytes().startswith(b'\0asm'), 'Invalid or missing WebAssembly module')
if wasm.is_file():
    check('本地审阅'.encode() not in wasm.read_bytes(), 'Local review code included in production WASM')
check(not (output / '_astro').exists(), 'Legacy Astro output published')
for path in output.rglob('*.html'):
    html = path.read_text()
    check('data-astro-' not in html and '/_astro/' not in html, f'Legacy generated HTML: {path}')
    check('"drafts":' not in html and '"notes":' not in html, f'Local editorial data: {path}')
if errors:
    sys.exit('\n'.join(errors))
print(f'Architecture verified: 3 Rust crates, {sum(p.suffix == ".rs" for p in files)} Rust modules, no Node/Astro application sources, production-only WASM and static Vercel configuration.')
