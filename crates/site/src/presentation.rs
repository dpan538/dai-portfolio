use crate::{CopyBook, Page, escape, images::Manifest};
use regex::Regex;
use std::{collections::BTreeMap, sync::LazyLock};

static COPY_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
static SENTENCE_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"([.!?][”’"']?)(\s+)([\p{Lu}][\p{L}’'-]*)([ \t]+)([\p{L}’'-]+)"#).unwrap()
});
static PARAGRAPH_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"([\p{L}\p{N}][\p{L}\p{N}’'-]*)([ \t]+)([\p{L}\p{N}][\p{L}\p{N}’'-]*[.!?,;:’”"')]*)(\s*)$"#).unwrap()
});

fn bind_copy_words(text: &str) -> String {
    let starts = SENTENCE_START.replace_all(text, |c: &regex::Captures| {
        format!("{}{}{}\u{a0}{}", &c[1], &c[2], &c[3], &c[5])
    });
    PARAGRAPH_END
        .replace(&starts, |c: &regex::Captures| {
            format!("{}\u{a0}{}{}", &c[1], &c[3], &c[4])
        })
        .into_owned()
}

/// Keep short sentence openings and final word pairs together in mobile prose.
/// HTML attributes and authored verse are left intact.
pub fn mobile_copy(html: &str) -> String {
    if html.contains("<blockquote") {
        return html.to_owned();
    }
    let mut output = String::with_capacity(html.len());
    let mut cursor = 0;
    for tag in COPY_TAG.find_iter(html) {
        output.push_str(&bind_copy_words(&html[cursor..tag.start()]));
        output.push_str(tag.as_str());
        cursor = tag.end();
    }
    output.push_str(&bind_copy_words(&html[cursor..]));
    output
}

pub fn images(html: &str, manifest: &Manifest, key: &str) -> String {
    let tags = Regex::new(r"<img\b[^>]*>").unwrap();
    let source = Regex::new(r#"\ssrc="([^"]*)""#).unwrap();
    let attrs=Regex::new(r#"\s(?:src|srcset|sizes|width|height|loading|fetchpriority|decoding|data-lightbox-src)="[^"]*""#).unwrap();
    let aliases: BTreeMap<_, _> = manifest
        .iter()
        .flat_map(|(original, m)| {
            m.variants
                .iter()
                .map(move |v| (v.url.as_str(), original.as_str()))
        })
        .collect();
    let mut visible = 0;
    tags.replace_all(html,|cap:&regex::Captures|{
        let tag=&cap[0];let Some(src)=source.captures(tag).map(|c|c[1].to_owned())else{return tag.to_owned();};
        let original=aliases.get(src.as_str()).copied().unwrap_or(&src);
        let Some(meta)=manifest.get(original)else{return tag.to_owned();};
        let Some(largest)=meta.variants.last()else{return tag.to_owned();};
        let is_preview=tag.contains("lb_view")||tag.contains("lightbox-image");
        if !is_preview{visible+=1;}
        let eager=visible==1&&!is_preview;
        let authored_sizes=Regex::new(r#"\ssizes="([^"]*)""#).unwrap().captures(tag).map(|c|c[1].to_owned());
        let sizes=if key.ends_with("_mobile") { authored_sizes.as_deref().unwrap_or("calc(100vw - 52px)") } else { match key {
            "gallery_design" if tag.contains("monolith-daylight-image")=>"(max-width:768px) 360vw, (min-width:1400px) 1000px, 75vw",
            "gallery_design" if meta.width<meta.height=>"(max-width:768px) 90vw, 36vw",
            "in_praise_of_time" if meta.width<meta.height=>"(max-width:552px) calc(72vw - 37.44px), (max-width:768px) 360px, 40vw",
            "in_praise_of_time"=>"(max-width:768px) 90vw, 60vw",
            "masks"=>"(max-width:768px) 90vw, 55vw",
            "the_threes"=>"(max-width:768px) 90vw, 65vw",
            "about"=>"(max-width:768px) 190px, (max-width:1257px) 21vw, 264px",
            _=>"(max-width:768px) 90vw, (min-width:1400px) 1000px, 75vw",
        }};
        let srcset=meta.variants.iter().map(|v|format!("{} {}w",v.url,v.width)).collect::<Vec<_>>().join(", ");
        let clean=attrs.replace_all(tag,"");let clean=clean.trim_end_matches('>').trim_end_matches('/').trim_end();
        format!(r#"{clean} src="{}" srcset="{}" sizes="{sizes}" width="{}" height="{}" loading="{}" fetchpriority="{}" decoding="async" data-lightbox-src="{}">"#,escape(&largest.url),escape(&srcset),meta.width,meta.height,if eager{"eager"}else{"lazy"},if eager{"high"}else{"auto"},escape(&largest.url))
    }).into_owned()
}
pub fn info(page: &Page, copy: &CopyBook) -> String {
    let title = match page.key.as_str() {
        "reframed_still" => "Reframed Still",
        "closing_time" => "Closing Time",
        "masks" => "Masks",
        _ => return String::new(),
    };
    let project = page.schemas.iter().find(|s| s.get("artMedium").is_some());
    let year = if page.key == "in_praise_of_time" {
        "2026"
    } else {
        project
            .and_then(|p| p.get("temporalCoverage"))
            .and_then(|v| v.as_str())
            .unwrap_or("2025")
    };
    let medium = project
        .and_then(|p| p.get("artMedium"))
        .and_then(|v| v.as_str())
        .unwrap_or("Black-and-white photography · Pentax K-3 Mark III Monochrome");
    let paragraphs = copy
        .get(&page.key)
        .into_iter()
        .flatten()
        .take(match page.key.as_str() {
            "masks" => 4,
            "reframed_still" | "in_praise_of_time" => 1,
            _ => usize::MAX,
        })
        .map(|(key, value)| format!("<p data-copy=\"{}\">{value}</p>", escape(key)))
        .collect::<String>();
    format!(
        r#"<details class="project-info" id="project-info"><summary aria-label="Project information"><span aria-hidden="true">+</span></summary><section class="project-info-panel" aria-label="Project information"><header><h2>{title}</h2><button type="button" class="project-info-close" aria-label="Close project information">×</button></header><p class="project-info-meta">{} · {}</p>{paragraphs}</section></details>"#,
        escape(year),
        escape(medium)
    )
}

#[cfg(test)]
mod typography_tests {
    use super::mobile_copy;

    #[test]
    fn mobile_prose_keeps_sentence_openings_and_final_words_together() {
        let source = "Gesture returns. These encounters gather around a hand.";
        let rendered = mobile_copy(source);
        assert!(rendered.contains("These\u{a0}encounters"));
        assert!(rendered.ends_with("a\u{a0}hand."));
        assert_eq!(rendered.replace('\u{a0}', " "), source);
    }

    #[test]
    fn mobile_word_binding_preserves_markup_and_links() {
        let source = r#"<p>A face returns. These encounters recall <cite>And Then</cite> and remain open.</p><a href="https://daipan.ink/" target="_blank">Related writing</a>"#;
        let rendered = mobile_copy(source);
        assert!(rendered.contains(r#"href="https://daipan.ink/" target="_blank""#));
        assert!(rendered.contains("<cite>And\u{a0}Then</cite>"));
        assert_eq!(rendered.replace('\u{a0}', " "), source);
    }

    #[test]
    fn authored_poetry_and_its_reference_are_preserved_exactly() {
        let source = r#"<blockquote cite="https://daipan.ink/"><p>Riding wings that beat with the light of night,<br>alternating lamps flatten my sight,</p></blockquote><p>Pan, D. (n.d.). A flowing song.</p>"#;
        assert_eq!(mobile_copy(source), source);
    }
}
