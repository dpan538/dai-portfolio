use crate::dom::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Note {
    id: String,
    target: String,
    priority: String,
    category: String,
    title: String,
    observation: String,
    proposal: String,
    needed: String,
}
#[derive(Deserialize)]
struct Editorial {
    drafts: Option<BTreeMap<String, String>>,
    #[serde(default)]
    notes: Vec<Note>,
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn init() {
    let Some(data) = one("#editorial-data") else {
        return;
    };
    let Ok(data) = serde_json::from_str::<Editorial>(&data.text_content().unwrap_or_default())
    else {
        return;
    };
    if data.notes.is_empty() && data.drafts.is_none() {
        return;
    }
    let review = query("review").as_deref() == Some("1");
    let draft = query("draft").as_deref() == Some("1");
    if draft && let Some(drafts) = data.drafts {
        for el in all("[data-copy]") {
            if let Some(copy) = el.get_attribute("data-copy").and_then(|k| drafts.get(&k)) {
                el.set_inner_html(copy);
            }
        }
    }
    let params = match (review, draft) {
        (true, true) => "?review=1&draft=1",
        (true, false) => "?review=1",
        (false, true) => "?draft=1",
        _ => "",
    };
    if !params.is_empty() {
        for a in all("a[href^='/']") {
            if let Some(href) = a.get_attribute("href")
                && !href.starts_with("//")
                && !href.contains('?')
                && !href.contains('.')
                && href != "/review/"
            {
                attr(&a, "href", &format!("{href}{params}"));
            }
        }
    }
    if !review {
        return;
    }
    if let Ok(css) = doc().create_element("link") {
        attr(&css, "rel", "stylesheet");
        attr(&css, "href", "/assets/review.css");
        if let Some(head) = doc().query_selector("head").ok().flatten() {
            let _ = head.append_child(&css);
        }
    }
    let Ok(panel) = doc().create_element("aside") else {
        return;
    };
    attr(&panel, "id", "review-panel");
    attr(&panel, "aria-label", "本地内容与体验审阅");
    let path = window().location().pathname().unwrap_or_default();
    let switch = if draft {
        format!("{path}?review=1")
    } else {
        format!("{path}?review=1&draft=1")
    };
    let cards:String=data.notes.iter().map(|n|format!("<details data-note=\"{}\"><summary>{} · {} · {}</summary><p><strong>{}</strong></p><p>观察：{}</p><p>建议：{}</p><p>待补材料：{}</p></details>",esc(&n.id),esc(&n.id),esc(&n.priority),esc(&n.category),esc(&n.title),esc(&n.observation),esc(&n.proposal),esc(&n.needed))).collect();
    panel.set_inner_html(&format!("<div class=\"review-actions\"><h2>本地审阅 · {}</h2><button type=\"button\" id=\"review-collapse\" aria-expanded=\"true\">收起</button></div><div class=\"review-body\"><p>保留原有视觉语言；这里记录内容与体验建议。</p><nav><a href=\"/review/\">全部标记</a><a href=\"{}\">{}</a><a href=\"{}\">关闭审阅</a></nav>{}</div>",if draft{"专业文案草稿"}else{"原文对照"},esc(&switch),if draft{"查看原文"}else{"专业文案预览"},esc(&path),cards));
    if let Some(body) = doc().body() {
        let _ = body.append_child(&panel);
    }
    if let Some(button) = one("#review-collapse") {
        on(&button, "click", {
            let (panel, button) = (panel.clone(), button.clone());
            move |_| {
                let collapsed = panel.get_attribute("data-collapsed").as_deref() != Some("true");
                attr(
                    &panel,
                    "data-collapsed",
                    if collapsed { "true" } else { "false" },
                );
                attr(
                    &button,
                    "aria-expanded",
                    if collapsed { "false" } else { "true" },
                );
                button.set_text_content(Some(if collapsed { "展开审阅" } else { "收起" }));
            }
        });
    }
    let chosen = query("mark");
    for note in data.notes {
        let sel = format!("details[data-note='{}']", note.id);
        if let Some(details) = one(&sel) {
            if chosen.as_deref() == Some(&note.id) {
                attr(&details, "open", "");
                if let Some(target) = visible_target(&note.target) {
                    attr(&target, "data-review-highlight", "");
                    if note.target != "head" && note.target != "main" {
                        target.scroll_into_view();
                    }
                }
            }
            on(&details, "toggle", {
                let target = note.target.clone();
                let details = details.clone();
                move |_| {
                    for el in all("[data-review-highlight]") {
                        let _ = el.remove_attribute("data-review-highlight");
                    }
                    if details.has_attribute("open")
                        && let Some(el) = visible_target(&target)
                    {
                        attr(&el, "data-review-highlight", "");
                    }
                }
            });
        }
    }
}

fn visible_target(selector: &str) -> Option<web_sys::Element> {
    all(selector).into_iter().find(|el| {
        let r = el.get_bounding_client_rect();
        r.width() > 1. && r.height() > 1.
    })
}
