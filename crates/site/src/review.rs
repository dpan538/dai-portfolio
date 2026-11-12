use crate::escape;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub page: String,
    pub target: String,
    pub priority: String,
    pub category: String,
    pub title: String,
    pub observation: String,
    pub proposal: String,
    pub needed: String,
}
pub fn write(out: &Path, notes: &[Note]) -> Result<()> {
    let cards:String=notes.iter().map(|n|{
        let route=if n.page=="all"||n.page=="home" {"/".into()}else{format!("/{}",n.page)};
        format!("<article class=\"review-card\"><header><span>{} · {} · {}</span><a href=\"{}?review=1&amp;mark={}\">在页面查看 ↗</a></header><h2>{}</h2><p><strong>观察</strong> {}</p><p><strong>建议</strong> {}</p><p><strong>待补材料</strong> {}</p></article>",escape(&n.id),escape(&n.priority),escape(&n.category),route,escape(&n.id),escape(&n.title),escape(&n.observation),escape(&n.proposal),escape(&n.needed))
    }).collect();
    let html = format!(
        r#"<!doctype html><html lang="zh-Hans"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="robots" content="noindex,nofollow"><title>作品集重构 · 内容与体验审阅</title><link rel="stylesheet" href="/assets/review.css"><body class="review-dashboard"><main><a href="/">← 作品集预览</a><p class="review-eyebrow">DAI PAN · LOCAL REVIEW</p><h1>内容与体验审阅</h1><p class="review-intro">Rust 静态生成 + Rust/WASM 交互。已完成 Rust 重构、图片预览与 Info 浮层。下面的标记记录编辑建议与已处理的体验问题。P1 = 优先补齐，P2 = 体验改进，P3 = 后续增强。</p><nav class="review-shortcuts"><a href="/?review=1">进入逐页标记</a><a href="/?review=1&amp;draft=1">专业文案草稿</a><a href="/about?review=1&amp;draft=1">About 草稿</a><a href="/review/in_praise_of_time/">新项目编排</a></nav><p>草稿只使用仓库已有信息；分工、版本、制作参数和成果需要本人确认。点击「在页面查看」可定位原页面，审阅面板只在带 review=1 的本地页面显示。</p>{cards}</main></body></html>"#
    );
    fs::create_dir_all(out.join("review"))?;
    fs::write(out.join("review/index.html"), html)?;
    Ok(())
}
