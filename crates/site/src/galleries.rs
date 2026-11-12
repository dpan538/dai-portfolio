use crate::{
    escape,
    images::{ImageMeta, Manifest},
    read_json,
};
use anyhow::{Result, ensure};
use gallery_core::rhythmic;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Gallery {
    page: String,
    title: String,
    source: String,
    expected: usize,
    anchor: String,
    #[serde(default)]
    pairs: Vec<[String; 2]>,
    images: Vec<Frame>,
    #[serde(default)]
    hero: Option<Hero>,
    #[serde(default)]
    annotations: Vec<Annotation>,
    #[serde(default)]
    ending: Option<String>,
    #[serde(default)]
    chapter_breaks: Vec<[String; 2]>,
    #[serde(default)]
    chapter_quote: Option<Quotation>,
    #[serde(default)]
    frame_quotes: Vec<FrameQuotation>,
    #[serde(default)]
    sequence: Vec<String>,
}
#[derive(Deserialize)]
struct Quotation {
    text: String,
    title: String,
    source: String,
    reference: Reference,
    #[serde(default)]
    translation_note: String,
    #[serde(default)]
    locator: String,
}
#[derive(Deserialize)]
struct Reference {
    author: String,
    year: String,
    publisher: String,
    #[serde(default)]
    translated_title: String,
    #[serde(default)]
    original_year: Option<u16>,
}
#[derive(Deserialize)]
struct FrameQuotation {
    file: String,
    quote: Quotation,
    #[serde(default)]
    beside: bool,
    #[serde(default)]
    illustration: Option<VerseIllustration>,
}
#[derive(Deserialize)]
struct VerseIllustration {
    file: String,
    align_to: String,
}
#[derive(Deserialize)]
struct Hero {
    files: [String; 2],
    upper: Vec<String>,
}
#[derive(Deserialize)]
struct Annotation {
    file: String,
    words: Vec<String>,
    caption: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    reverse: bool,
    #[serde(default)]
    spacious: bool,
    #[serde(default)]
    columns: Vec<VocabularyColumn>,
}
#[derive(Deserialize)]
struct VocabularyColumn {
    label: String,
    words: Vec<String>,
}
#[derive(Deserialize)]
struct Frame {
    file: String,
    alt: String,
    family: String,
    #[serde(default)]
    placement: Option<[usize; 2]>,
    #[serde(default)]
    inset_before: Option<String>,
}
#[derive(Deserialize)]
struct MobileFrame {
    file: String,
    width_percent: f64,
    #[serde(default)]
    max_width: Option<u16>,
    #[serde(default)]
    copy_key: Option<String>,
    #[serde(default)]
    align_left_with: Option<String>,
    #[serde(default)]
    align_right_with_caption: bool,
}
fn galleries(root: &Path) -> Result<Vec<Gallery>> {
    read_json(&root.join("site/content/galleries.json"))
}
fn image<'a>(manifest: &'a Manifest, gallery: &Gallery, frame: &Frame) -> Result<&'a ImageMeta> {
    manifest
        .get(&format!("/projects/{}/{}", gallery.page, frame.file))
        .ok_or_else(|| anyhow::anyhow!("Missing gallery source {}", frame.file))
}
fn figure(
    meta: &ImageMeta,
    frame: &Frame,
    i: usize,
    row: usize,
    placement: Option<(usize, usize)>,
) -> String {
    let largest = &meta.variants.last().expect("responsive variants").url;
    let portrait = meta.width < meta.height;
    let (column, span) = if portrait {
        match i % 4 {
            0 => (2, 5),
            1 => (7, 5),
            2 => (1, 5),
            _ => (5, 6),
        }
    } else {
        match i % 4 {
            0 => (1, 10),
            1 => (4, 9),
            2 => (2, 10),
            _ => (1, 9),
        }
    };
    let (column, span) = placement
        .or(frame.placement.map(|[column, span]| (column, span)))
        .unwrap_or((column, span));
    let orientation = if portrait { "portrait" } else { "landscape" };
    format!(
        r#"<figure class="time-frame" data-index="{i}" data-orientation="{orientation}" style="--column:{column};--span:{span};--row:{}"><img class="time-photo" src="{}" alt="{}" width="{}" height="{}"></figure>"#,
        row,
        escape(largest),
        escape(&frame.alt),
        meta.width,
        meta.height
    )
}
fn quotation(q: &Quotation) -> String {
    quotation_layout(q, None)
}
fn quotation_layout(q: &Quotation, illustration: Option<(&str, &str)>) -> String {
    let passage = q.text.lines().map(escape).collect::<Vec<_>>().join("<br>");
    let passage = if q.translation_note.is_empty() && q.text.split_whitespace().count() < 40 {
        format!("“{passage}”")
    } else {
        passage
    };
    let (passage, picture) = if let Some((anchor, picture)) = illustration {
        let lines = q.text.lines().collect::<Vec<_>>();
        let at = lines
            .iter()
            .position(|line| *line == anchor)
            .expect("Validated verse anchor");
        let leading = lines[..at]
            .iter()
            .map(|line| escape(line))
            .collect::<Vec<_>>()
            .join("<br>");
        let anchored = lines[at..]
            .iter()
            .map(|line| escape(line))
            .collect::<Vec<_>>()
            .join("<br>");
        (
            format!(
                r#"<p class="time-verse-leading">{leading}</p><p class="time-verse-anchor">{anchored}</p>"#
            ),
            picture,
        )
    } else {
        (format!("<p>{passage}</p>"), "")
    };
    let r = &q.reference;
    let translated_title = if r.translated_title.is_empty() {
        String::new()
    } else {
        format!(" [{}]", escape(&r.translated_title))
    };
    let original = r
        .original_year
        .map(|year| format!(r#" data-original-year="{year}""#))
        .unwrap_or_default();
    format!(
        r#"<blockquote cite="{}"{original} data-locator="{}">{passage}</blockquote>{picture}<figcaption><p class="time-reference">{} ({}). <a href="{}" target="_blank" rel="noopener noreferrer"><cite>{}</cite></a>{translated_title}. {}.</p></figcaption>"#,
        escape(&q.source),
        escape(&q.locator),
        escape(&r.author),
        escape(&r.year),
        escape(&q.source),
        escape(&q.title),
        escape(&r.publisher),
    )
}
fn ordered(g: &Gallery, manifest: &Manifest, out: &Path) -> Result<Vec<usize>> {
    let features = g
        .images
        .iter()
        .map(|f| {
            let meta = image(manifest, g, f)?;
            crate::visual::extract(
                &out.join(meta.variants[0].url.trim_start_matches('/')),
                meta.width as f64 / meta.height as f64,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let anchor = g
        .images
        .iter()
        .position(|f| f.file == g.anchor)
        .unwrap_or(0);
    let mut order = rhythmic(
        &features,
        &g.images
            .iter()
            .map(|f| f.family.as_str())
            .collect::<Vec<_>>(),
        anchor,
    );
    // Author-defined juxtapositions take precedence over general visual rhythm.
    let mut paired = std::collections::BTreeSet::new();
    for pair in &g.pairs {
        ensure!(
            pair[0] != pair[1],
            "A pair must contain two different photographs"
        );
        for file in pair {
            ensure!(
                paired.insert(file),
                "Photograph belongs to multiple pairs: {file}"
            );
        }
        let first = g
            .images
            .iter()
            .position(|f| f.file == pair[0])
            .ok_or_else(|| anyhow::anyhow!("Unknown paired photo {}", pair[0]))?;
        let second = g
            .images
            .iter()
            .position(|f| f.file == pair[1])
            .ok_or_else(|| anyhow::anyhow!("Unknown paired photo {}", pair[1]))?;
        order.retain(|i| *i != second);
        let at = order
            .iter()
            .position(|i| *i == first)
            .expect("All photographs in order");
        order.insert(at + 1, second);
    }
    for boundary in &g.chapter_breaks {
        let before = g
            .images
            .iter()
            .position(|f| f.file == boundary[0])
            .ok_or_else(|| anyhow::anyhow!("Missing section-boundary photo {}", boundary[0]))?;
        let after = g
            .images
            .iter()
            .position(|f| f.file == boundary[1])
            .ok_or_else(|| anyhow::anyhow!("Missing section-boundary photo {}", boundary[1]))?;
        ensure!(
            before != after,
            "A section break needs two different photographs"
        );
        order.retain(|i| *i != after);
        let at = order
            .iter()
            .position(|i| *i == before)
            .expect("Boundary photograph in order");
        order.insert(at + 1, after);
    }
    if let Some(hero) = &g.hero {
        let mut opening = Vec::new();
        for file in &hero.files {
            let index = g
                .images
                .iter()
                .position(|f| f.file == *file)
                .ok_or_else(|| anyhow::anyhow!("Missing hero photograph {file}"))?;
            ensure!(
                !opening.contains(&index),
                "Hero photographs must be different"
            );
            ensure!(
                !g.pairs.iter().flatten().any(|f| f == file),
                "A hero photograph cannot also belong to a later pair"
            );
            opening.push(index);
        }
        order.retain(|i| !opening.contains(i));
        opening.extend(order);
        order = opening;
    }
    if let Some(ending) = &g.ending {
        let index = g
            .images
            .iter()
            .position(|f| f.file == *ending)
            .ok_or_else(|| anyhow::anyhow!("Missing ending photograph {ending}"))?;
        ensure!(
            !g.pairs.iter().flatten().any(|f| f == ending),
            "Ending photograph cannot be part of a pair"
        );
        ensure!(
            g.hero
                .as_ref()
                .map(|h| !h.files.contains(ending))
                .unwrap_or(true),
            "Ending photograph cannot be in the hero"
        );
        order.retain(|i| *i != index);
        order.push(index);
    }
    if !g.sequence.is_empty() {
        ensure!(
            g.sequence.len() == g.images.len(),
            "The authored sequence must contain every photograph"
        );
        order = g
            .sequence
            .iter()
            .map(|file| {
                g.images
                    .iter()
                    .position(|f| f.file == *file)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Unknown photograph in authored sequence: {file}")
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            order
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == order.len(),
            "Duplicate photograph in authored sequence"
        );
        if let Some(hero) = &g.hero {
            ensure!(
                order
                    .iter()
                    .take(2)
                    .map(|i| &g.images[*i].file)
                    .eq(hero.files.iter()),
                "The authored sequence must retain the opening photographs"
            );
        }
        if let Some(ending) = &g.ending {
            ensure!(
                &g.images[*order.last().expect("Nonempty gallery")].file == ending,
                "The authored sequence must retain the ending"
            );
        }
        for pair in g.pairs.iter().chain(g.chapter_breaks.iter()) {
            ensure!(
                order
                    .windows(2)
                    .any(|p| g.images[p[0]].file == pair[0] && g.images[p[1]].file == pair[1]),
                "The authored sequence must retain paired and chapter-boundary photographs"
            );
        }
    }
    Ok(order)
}
pub fn fill(root: &Path, out: &Path, key: &str, mut template: String) -> Result<String> {
    if !template.contains("{{gallery:") {
        return Ok(template);
    }
    let manifest: Manifest = read_json(&out.join("assets/image-manifest.json"))?;
    let desktop_grid_style = regex::Regex::new(r#" style="--column:[^"]*""#).unwrap();
    for g in galleries(root)?
        .into_iter()
        .filter(|g| g.page == key.trim_end_matches("_mobile"))
    {
        ensure!(
            g.images.len() == g.expected,
            "{} needs {} photos; sequence currently has {}",
            g.title,
            g.expected,
            g.images.len()
        );
        let listed: std::collections::BTreeSet<_> =
            g.images.iter().map(|f| f.file.clone()).collect();
        let available: std::collections::BTreeSet<_> = fs::read_dir(root.join(&g.source))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| matches!(s.to_lowercase().as_str(), "jpg" | "jpeg" | "png"))
                    .unwrap_or(false)
            })
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        ensure!(
            listed.len() == g.images.len(),
            "Duplicate image in {} sequence",
            g.title
        );
        ensure!(
            listed == available,
            "Source files and {} sequence differ; update galleries.json",
            g.title
        );
        for f in &g.images {
            ensure!(
                root.join(&g.source).join(&f.file).is_file(),
                "Missing source {}",
                f.file
            );
        }
        let order = ordered(&g, &manifest, out)?;
        if key.ends_with("_mobile") {
            let edit: Vec<MobileFrame> = read_json(&root.join("site/content/mobile-time.json"))?;
            ensure!(
                edit.len() == order.len(),
                "Mobile edit must retain every photograph"
            );
            let mut blocks = Vec::new();
            for (position, index) in order.iter().enumerate() {
                let frame = &g.images[*index];
                let scale = &edit[position];
                ensure!(
                    scale.file == frame.file && (1.0..=100.0).contains(&scale.width_percent),
                    "Invalid mobile photographic edit"
                );
                let meta = image(&manifest, &g, frame)?;
                if meta.width < meta.height {
                    ensure!(
                        (scale.width_percent == 100.0) == (frame.file == "km-74.JPG"),
                        "Only the wall study is full-width in the mobile portrait edit"
                    );
                }
                if g.chapter_breaks.iter().any(|b| b[1] == frame.file) {
                    blocks.push("<div class=\"m-time-chapter\" role=\"separator\" aria-label=\"Chapter two\">II</div>".to_string());
                }
                let sizes = format!(
                    "calc({}vw - {}px)",
                    scale.width_percent,
                    scale.width_percent * 0.52
                );
                let sizes = scale
                    .max_width
                    .map(|max| format!("min({max}px, {sizes})"))
                    .unwrap_or(sizes);
                let max = scale
                    .max_width
                    .map(|w| format!("{w}px"))
                    .unwrap_or("100%".into());
                let alignment = if scale.align_right_with_caption {
                    ensure!(
                        scale.align_left_with.is_none(),
                        "Mobile alignment must use one anchor"
                    );
                    // Container units preserve the photographic scale while the grid's
                    // first column follows the intrinsic width of the opening sentence.
                    format!(";width:min({}cqw, {max})", scale.width_percent)
                } else if let Some(file) = &scale.align_left_with {
                    let anchor = edit.iter().find(|f| &f.file == file).ok_or_else(|| {
                        anyhow::anyhow!("Unknown mobile alignment photograph: {file}")
                    })?;
                    ensure!(
                        anchor.align_left_with.is_none(),
                        "Mobile alignment must reference a centred photograph"
                    );
                    let anchor_max = anchor
                        .max_width
                        .map(|w| format!("{w}px"))
                        .unwrap_or("100%".into());
                    format!(
                        ";align-self:flex-start;margin-left:calc((100% - min({}%, {anchor_max})) / 2)",
                        anchor.width_percent
                    )
                } else {
                    String::new()
                };
                let photo = figure(meta, frame, position, position + 1, None)
                    .replace(
                        "class=\"time-frame\"",
                        &format!(
                            "class=\"m-time-frame\" style=\"width:{}%;max-width:{max}{alignment}\"",
                            scale.width_percent
                        ),
                    )
                    .replace(
                        "class=\"time-photo\"",
                        &format!("class=\"time-photo\" sizes=\"{sizes}\""),
                    );
                // Mobile uses its own scale and short passages, without desktop grid coordinates.
                let photo = desktop_grid_style.replace_all(&photo, "").into_owned();
                let passage = scale
                    .copy_key
                    .as_ref()
                    .map(|copy_key| {
                        format!(
                            r#"<div class="m-time-passage" data-copy="{}">{{{{copy:{copy_key}}}}}</div>"#,
                            escape(copy_key)
                        )
                    })
                    .unwrap_or_default();
                let entry_class = if scale.align_right_with_caption {
                    "m-time-entry m-time-entry--return"
                } else {
                    "m-time-entry"
                };
                blocks.push(format!(
                    r#"<div class="{entry_class}">{photo}{passage}</div>"#
                ));
            }
            template = template.replace("{{gallery:mobile-frames}}", &blocks.join("\n"));
            continue;
        }
        let mut hero_html = String::new();
        let opening = if let Some(hero) = &g.hero {
            let mouth = &g.images[order[0]];
            let branches = &g.images[order[1]];
            let first = figure(image(&manifest, &g, mouth)?, mouth, 0, 1, None).replace(
                "class=\"time-frame\"",
                "class=\"time-frame time-hero-mouth\"",
            );
            let second = figure(image(&manifest, &g, branches)?, branches, 1, 1, None).replace(
                "class=\"time-frame\"",
                "class=\"time-frame time-hero-branches\"",
            );
            let upper = hero
                .upper
                .iter()
                .map(|w| format!("<span>{}</span>", escape(w)))
                .collect::<String>();
            hero_html = format!(
                r#"<section class="time-hero" aria-label="Opening photographs: lips, then branches">{first}<div class="time-hero-statement"><p class="time-hero-words" lang="en">{upper}</p></div>{second}</section>"#
            );
            2
        } else {
            0
        };
        let mut blocks = Vec::new();
        let mut ending_html = String::new();
        let mut position = opening;
        let mut row = 1;
        while position < order.len() {
            let f = &g.images[order[position]];
            if g.chapter_breaks.iter().any(|b| b[1] == f.file) {
                if let Some(q) = &g.chapter_quote {
                    blocks.push(format!(r#"<figure class="time-chapter-break" role="group" aria-label="Literary passage between chapters" style="--row:{row}">{}</figure>"#, quotation(q)));
                } else {
                    blocks.push(format!(r#"<div class="time-chapter-break" role="separator" aria-label="Chapter two" style="--row:{row}"><span class="time-chapter-number">II</span></div>"#));
                }
                row += 1;
            }
            if g.ending.as_deref() == Some(f.file.as_str()) {
                ending_html = figure(image(&manifest, &g, f)?, f, position, 1, Some((1, 3)))
                    .replace(
                        "class=\"time-frame\"",
                        "class=\"time-frame time-frame--ending\"",
                    );
                position += 1;
                continue;
            } else if let Some(before) = &f.inset_before {
                ensure!(
                    order.get(position + 1).map(|i| &g.images[*i].file) == Some(before),
                    "An inset must precede its paired composition"
                );
                ensure!(
                    g.pairs.iter().any(|pair| pair[0] == *before),
                    "An inset must lead into a defined pair"
                );
                blocks.push(
                    figure(image(&manifest, &g, f)?, f, position, row, None).replace(
                        "class=\"time-frame\"",
                        "class=\"time-frame time-frame--inset\"",
                    ),
                );
                position += 1;
                continue;
            } else if let Some(pair) = g.pairs.iter().find(|p| p[0] == f.file) {
                let other = &g.images[order[position + 1]];
                ensure!(other.file == pair[1], "Paired photographs must be adjacent");
                let left = figure(image(&manifest, &g, f)?, f, position, 1, Some((1, 7))).replace(
                    "class=\"time-frame\"",
                    "class=\"time-frame time-pair-primary\"",
                );
                let right = figure(
                    image(&manifest, &g, other)?,
                    other,
                    position + 1,
                    1,
                    Some((10, 3)),
                )
                .replace(
                    "class=\"time-frame\"",
                    "class=\"time-frame time-pair-secondary\"",
                );
                blocks.push(format!(r#"<div class="time-pair" role="group" aria-label="Paired photographs" style="--row:{row}">{left}<span class="time-pair-mark" aria-hidden="true"><span></span></span>{right}</div>"#));
                position += 2;
            } else if let Some(q) = g.frame_quotes.iter().find(|q| q.file == f.file) {
                let photo = figure(image(&manifest, &g, f)?, f, position, 1, None);
                let (group, caption) = if q.beside {
                    ("time-quoted-frame", "time-frame-quote")
                } else {
                    ("time-chaos", "time-chaos-quote")
                };
                let picture = if let Some(illustration) = &q.illustration {
                    ensure!(
                        q.quote
                            .text
                            .lines()
                            .any(|line| line == illustration.align_to),
                        "Unknown illustration anchor in verse"
                    );
                    let frame = order
                        .get(position + 1)
                        .map(|i| &g.images[*i])
                        .ok_or_else(|| anyhow::anyhow!("Missing verse illustration"))?;
                    ensure!(
                        frame.file == illustration.file,
                        "The verse illustration must follow its photograph"
                    );
                    figure(image(&manifest, &g, frame)?, frame, position + 1, 1, None).replace(
                        "class=\"time-frame\"",
                        "class=\"time-frame time-poem-image\"",
                    )
                } else {
                    String::new()
                };
                let quote = quotation_layout(
                    &q.quote,
                    q.illustration
                        .as_ref()
                        .map(|i| (i.align_to.as_str(), picture.as_str())),
                );
                let modifier = if q.illustration.is_some() {
                    " time-chaos-quote--illustrated"
                } else {
                    ""
                };
                blocks.push(format!(r#"<section class="{group}" role="group" aria-label="Photograph and literary passage" style="--row:{row}">{photo}<figure class="{caption}{modifier}">{quote}</figure></section>"#));
                position += if q.illustration.is_some() { 2 } else { 1 };
            } else if let Some(annotation) = g.annotations.iter().find(|a| a.file == f.file) {
                let photo = figure(image(&manifest, &g, f)?, f, position, 1, None);
                let spacing = if annotation.spacious {
                    " time-annotated--spacious"
                } else {
                    ""
                };
                if annotation.kind == "vocabulary" {
                    ensure!(
                        annotation.columns.len() == 2,
                        "Word fragments need two columns"
                    );
                    let columns = annotation.columns.iter().map(|column| {
                        let words = column.words.iter().map(|word| format!("<li>{}</li>", escape(word))).collect::<String>();
                        format!("<div class=\"time-vocabulary-column\" aria-label=\"{}\"><ul>{words}</ul></div>", escape(&column.label))
                    }).collect::<String>();
                    blocks.push(format!(r#"<section class="time-annotated time-annotated--vocabulary time-annotated--reverse{spacing}" style="--row:{row}" aria-label="Photograph and word fragments">{photo}<div class="time-vocabulary" lang="en">{columns}</div></section>"#));
                    position += 1;
                    row += 1;
                    continue;
                }
                let prose = matches!(annotation.kind.as_str(), "prose" | "poem");
                let content = if prose {
                    annotation
                        .text
                        .lines()
                        .map(escape)
                        .collect::<Vec<_>>()
                        .join("<br>")
                } else {
                    annotation
                        .words
                        .iter()
                        .map(|w| format!("<span>{}</span>", escape(w)))
                        .collect::<String>()
                };
                let column = if prose {
                    "time-prose-column"
                } else {
                    "time-word-column"
                };
                let modifier = format!(
                    "{}{}{}{}",
                    if prose { " time-annotated--prose" } else { "" },
                    if annotation.reverse {
                        " time-annotated--reverse"
                    } else {
                        ""
                    },
                    spacing,
                    if annotation.kind == "poem" {
                        " time-annotated--poem"
                    } else {
                        ""
                    }
                );
                let caption = if annotation.caption.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p class=\"time-word-caption\" lang=\"en\">{}</p>",
                        escape(&annotation.caption)
                    )
                };
                blocks.push(format!(r#"<section class="time-annotated{modifier}" style="--row:{row}" aria-label="Photograph and text">{photo}<div class="{column}"><p lang="en">{content}</p>{caption}</div></section>"#));
                position += 1;
            } else {
                let mut photo = figure(image(&manifest, &g, f)?, f, position, row, None);
                if g.chapter_breaks.iter().any(|b| b[1] == f.file) {
                    photo = photo.replace(
                        "class=\"time-frame\"",
                        "class=\"time-frame time-frame--chapter-hero\"",
                    );
                }
                blocks.push(photo);
                position += 1;
            }
            row += 1;
        }
        let frames = blocks.join("\n");
        template = template
            .replace("{{gallery:frames}}", &frames)
            .replace("{{gallery:hero}}", &hero_html)
            .replace("{{gallery:ending}}", &ending_html)
            .replace("{{gallery:closing-row}}", &row.to_string());
    }
    Ok(template)
}
pub fn review(root: &Path, out: &Path) -> Result<()> {
    let manifest: Manifest = read_json(&out.join("assets/image-manifest.json"))?;
    for g in galleries(root)? {
        let cards=ordered(&g,&manifest,out)?.into_iter().enumerate().map(|(position,index)|{
            let f=&g.images[index];let meta=image(&manifest,&g,f)?;let thumb=meta.variants.iter().find(|v|v.width==640).unwrap_or(&meta.variants[0]);
            Ok(format!(r#"<figure><img src="{}" alt="{}" loading="lazy"><figcaption><b>{:02}</b> {}</figcaption></figure>"#,thumb.url,escape(&f.alt),position+1,escape(&f.file)))
        }).collect::<Result<Vec<_>>>()?.join("\n");
        let html = format!(
            r#"<!doctype html><html lang="zh-Hans"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="robots" content="noindex,nofollow"><title>{} · 内部编排</title><link rel="stylesheet" href="/assets/review.css"><style>.sequence-grid{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:24px 18px}}.sequence-grid figure{{margin:0}}.sequence-grid img{{width:100%;height:190px;object-fit:contain;background:#eaece7}}.sequence-grid figcaption{{font-size:12px;padding-top:8px}}@media(max-width:700px){{.sequence-grid{{grid-template-columns:repeat(2,minmax(0,1fr))}}}}</style><body class="review-dashboard"><main><a href="/review/">← 内容审阅</a><p class="review-eyebrow">IN PRAISE OF TIME · INTERNAL EDIT</p><h1>{} · {} 张</h1><p>这是本地编辑用的编号总览，文件名不会出现在作品页。Rust 在构建时生成固定视觉顺序，人物与环境彼此独立。</p><p><a href="/{}">查看纵向 grid 排版 ↗</a></p><div class="sequence-grid">{cards}</div></main></body></html>"#,
            escape(&g.title),
            escape(&g.title),
            g.images.len(),
            g.page
        );
        let dest = out.join("review").join(&g.page);
        fs::create_dir_all(&dest)?;
        fs::write(dest.join("index.html"), html)?;
    }
    Ok(())
}
