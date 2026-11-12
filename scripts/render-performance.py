#!/usr/bin/env python3
"""Plot measured migration results as PNG charts; never synthesize samples."""
import csv
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
os.environ.setdefault('MPLCONFIGDIR', str(ROOT / '.cache/matplotlib'))
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

OUT = ROOT / 'reports/performance'
data = json.loads((OUT / 'results.json').read_text())
legacy = data['payload']['legacy']; rust = data['payload']['rust']
OLD = '#a46b50'; NEW = '#315e70'; INK = '#26343a'; MUTED = '#68767b'
plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 11, 'text.color': INK,
                     'axes.labelcolor': MUTED, 'xtick.color': MUTED, 'ytick.color': MUTED,
                     'axes.edgecolor': '#ced7d9', 'axes.spines.top': False, 'axes.spines.right': False,
                     'figure.facecolor': '#fafbf9', 'axes.facecolor': '#fafbf9',
                     'savefig.facecolor': '#fafbf9', 'axes.titleweight': 'semibold'})

metrics = [
    ('Warm image pipeline', 'seconds', data['images']['legacy_warm_seconds']['median'], data['images']['rust_warm_seconds']['median']),
    ('Warm static build', 'seconds', data['warm_build']['legacy_seconds']['median'], data['warm_build']['rust_seconds']['median']),
    ('Cold image pipeline', 'seconds', data['images']['legacy_cold_seconds'], data['images']['rust_cold_seconds']),
    ('Shared image derivatives', 'MiB', data['images']['legacy_variants_bytes']/2**20, data['images']['rust_shared_variants_bytes']/2**20),
    ('Median control payload', 'KiB gzip', legacy['median_control_gzip_bytes']/1024, rust['median_control_gzip_bytes']/1024),
]

def delta(a, b): return (b/a-1)*100

def clean(ax):
    ax.set_axisbelow(True); ax.grid(axis='y', color='#e0e6e5', linewidth=.7)
    ax.tick_params(length=0, pad=8)

def repetitions(ax, old, new, title, note):
    x = range(1, len(old['samples'])+1)
    ax.plot(x, old['samples'], '-o', color=OLD, linewidth=2.3, markersize=6, label='Astro')
    ax.plot(x, new['samples'], '-o', color=NEW, linewidth=2.3, markersize=6, label='Rust')
    ax.set_xticks(list(x), [f'Run {i}' for i in x]); ax.set_ylim(0, max(old['max'],new['max'])*1.20)
    ax.set_ylabel('Seconds'); ax.set_title(title, loc='left', fontsize=15, pad=25)
    ax.text(0, 1.035, note, transform=ax.transAxes, fontsize=9, color=MUTED)
    ax.legend(frameon=False, loc='center right', ncol=2)
    ax.text(1, -.23, f'Median: {old["median"]:.2f}s → {new["median"]:.2f}s  ({delta(old["median"],new["median"]):+.1f}%)',
            ha='right', transform=ax.transAxes, fontsize=11, color=NEW)
    clean(ax)

def bars(ax, metric, title, note):
    _, unit, a, b = metric
    ax.bar(['Astro','Rust'], [a,b], width=.48, color=[OLD,NEW], zorder=3)
    ax.set_ylim(0,max(a,b)*1.35); ax.set_ylabel(unit)
    ax.set_title(title, loc='left', fontsize=13, pad=29)
    ax.text(0,1.04,note,transform=ax.transAxes,fontsize=9,color=MUTED)
    for i, v in enumerate([a,b]): ax.text(i, v+max(a,b)*.035, f'{v:.2f}', ha='center',fontsize=11)
    ax.text(.5,-.22,f'Rust change: {delta(a,b):+.1f}%',ha='center',transform=ax.transAxes,fontsize=11)
    clean(ax)

fig = plt.figure(figsize=(16,10.7))
grid = fig.add_gridspec(2,6,left=.075,right=.96,top=.78,bottom=.13,hspace=.68,wspace=1.2)
fig.text(.075,.94,'Dai Pan / Rust migration',fontsize=27,weight='semibold')
fig.text(.075,.895,'Faster cached builds. Higher cold-processing and first-visit costs.',fontsize=16,color=NEW)
fig.text(.075,.855,'59 identical originals · 3 warm repetitions · macOS / ARM64 · Lower is better',fontsize=11,color=MUTED)
repetitions(fig.add_subplot(grid[0,:3]),data['warm_build']['legacy_seconds'],data['warm_build']['rust_seconds'],
            'Warm static build','Compiled tools and dependencies already available; portfolio scope differs: 11 vs 21 pages.')
repetitions(fig.add_subplot(grid[0,3:]),data['images']['legacy_warm_seconds'],data['images']['rust_warm_seconds'],
            'Warm image preparation','The same 59 originals. Old pipeline regenerates; Rust validates and reuses cached variants.')
bars(fig.add_subplot(grid[1,:2]),metrics[2],'Cold image preparation','59 originals · one observation')
bars(fig.add_subplot(grid[1,2:4]),metrics[3],'Shared image derivatives','Same width policy · different encoders')
bars(fig.add_subplot(grid[1,4:]),metrics[4],'First-visit control payload','Median of 9 routes · gzip estimate')
fig.text(.075,.045,'Build timings exclude installs and compilation. Payload excludes images and remote fonts. These are not Core Web Vitals.',fontsize=10,color=MUTED)
fig.text(.075,.022,f'Baseline {data["baseline_commit"][:12]} · Measured {data["measured_at"][:10]} · Raw samples: results.json',fontsize=9,color=MUTED)
fig.savefig(OUT/'summary.png',dpi=200); plt.close(fig)

fig, ax = plt.subplots(figsize=(14,6.8)); fig.subplots_adjust(left=.08,right=.95,top=.80,bottom=.24)
labels = ['Home','About','Contact','Quite Off','The Threes','Monolith','Reframed Still','Closing Time','Masks']
ax.plot(labels,[p['control_gzip_bytes']/1024 for p in legacy['routes']],'-o',color=OLD,label='Astro',linewidth=2)
ax.plot(labels,[p['control_gzip_bytes']/1024 for p in rust['routes']],'-o',color=NEW,label='Rust',linewidth=2)
ax.set_ylim(0,105); ax.set_ylabel('KiB gzip per first visit'); ax.tick_params(axis='x',labelrotation=25)
ax.legend(frameon=False,ncol=2,loc='center left'); clean(ax)
fig.text(.08,.91,'Matched route control payloads',fontsize=24,weight='semibold')
fig.text(.08,.85,'HTML + local CSS + JavaScript bindings + WASM; images and external fonts excluded.',fontsize=11,color=MUTED)
fig.text(.08,.06,'Route labels are categories, not time. Lines show the payload profile. Each file is independently gzip-compressed; no HTTP trace is implied.',fontsize=9,color=MUTED)
fig.savefig(OUT/'route-payloads.png',dpi=200); plt.close(fig)

with (OUT/'metrics.csv').open('w',newline='') as stream:
    writer=csv.writer(stream, lineterminator='\n'); writer.writerow(['metric','unit','astro','rust','change_percent'])
    for title,unit,a,b in metrics: writer.writerow([title,unit,a,b,round(delta(a,b),3)])
for obsolete in ['index.html','summary.svg']:
    (OUT/obsolete).unlink(missing_ok=True)
print(f'Created summary.png and route-payloads.png with Matplotlib {matplotlib.__version__}')
