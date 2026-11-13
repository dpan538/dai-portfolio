mod cache;
mod galleries;
mod images;
mod presentation;
mod review;
mod visual;

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tiny_http::{Header, Method, Response, Server, StatusCode};
use wasm_bindgen_cli_support::Bindgen;

type CopyBook = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Deserialize)]
struct Page {
    key: String,
    title: String,
    canonical: String,
    meta: BTreeMap<String, String>,
    schemas: Vec<Value>,
    css: Vec<String>,
    main_class: Vec<String>,
}
fn base_key(key: &str) -> &str {
    key.strip_suffix("_mobile").unwrap_or(key)
}

pub(crate) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| path.display().to_string())?)
        .with_context(|| path.display().to_string())
}
fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let public_name = images::public_path(Path::new(&name));
        if name.to_string_lossy().starts_with('.') || name == "_optimized" {
            continue;
        }
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target.join(&public_name))?;
        } else {
            fs::copy(entry.path(), target.join(&public_name))?;
        }
    }
    Ok(())
}
fn navigation(active: &str, mobile: bool) -> String {
    [
        ("quite_off","Quite Off"),("the_threes","The Threes"),
        ("gallery_design","Monolith for Light"),("in_praise_of_time","In Praise of Time"),
        ("reframed_still","Reframed Still"),("closing_time","Closing Time"),("masks","Masks"),
        ("about","About"),("contact","Contact"),
    ].iter().filter(|(key,_)|!mobile||*key!="contact").map(|(key,label)|{
        let base=if mobile{"mobile_overlay_link"}else{"menu_link"};
        let current=base_key(active)==*key;
        let selected=if current{" is_active"}else{""};
        let gap=if *key=="about"{if mobile{" mobile_overlay_link--before_about"}else{" menu_link--after_projects"}}else{""};
        let contents=if current{format!("{label}<span class=\"nav-current-mark\" aria-hidden=\"true\"></span>")}else{label.to_string()};
        let aria=if current{" aria-current=\"page\""}else{""};
        let route=if mobile{format!("{key}_mobile")}else{key.to_string()};
        format!("<a data-scope-layout class=\"{base}{selected}{gap}\" href=\"/{route}\" data-project=\"{key}\" aria-label=\"{label}\"{aria}>{contents}</a>\n")
    }).collect()
}
fn render(root: &Path, out: &Path, local: bool) -> Result<usize> {
    let pages: Vec<Page> = read_json(&root.join("site/content/pages.json"))?;
    let copy: CopyBook = read_json(&root.join("site/content/copy.json"))?;
    let drafts: CopyBook = read_json(&root.join("site/content/professional.json"))?;
    let notes: Vec<review::Note> = read_json(&root.join("site/content/review.json"))?;
    let shell = fs::read_to_string(root.join("site/templates/layout.html"))?;
    let mobile_shell = fs::read_to_string(root.join("site/templates/mobile-layout.html"))?;
    let person: Value = read_json(&root.join("site/content/person.json"))?;
    let image_manifest: images::Manifest = read_json(&out.join("assets/image-manifest.json"))?;
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in &pages {
        let mobile = page.key.ends_with("_mobile");
        let content_key = base_key(&page.key);
        let mut body =
            fs::read_to_string(root.join(format!("site/templates/pages/{}.html", page.key)))?;
        body = galleries::fill(root, out, &page.key, body)?;
        for (key, value) in copy
            .get(content_key)
            .into_iter()
            .flatten()
            .chain(copy.get(&page.key).into_iter().flatten())
        {
            let rendered = if mobile {
                presentation::mobile_copy(value)
            } else {
                value.clone()
            };
            body = body.replace(&format!("{{{{copy:{key}}}}}"), &rendered);
        }
        ensure!(!body.contains("{{copy:"), "Missing copy in {}", page.key);
        body = presentation::images(&body, &image_manifest, &page.key);
        body += &presentation::info(page, &copy);
        // Year is data rather than a framework expression.
        if page.key == "home" {
            body = body.replace("© 2026", "© {{year}}");
        }
        body = body.replace("{{year}}", &current_year());
        let mut metadata = format!(
            "<link rel=\"canonical\" href=\"{}\">\n",
            escape(&page.canonical)
        );
        for (name, value) in &page.meta {
            if matches!(name.as_str(), "viewport" | "theme-color") {
                continue;
            }
            let value = if name == "robots" && (local || page.key == "404") {
                "noindex, nofollow"
            } else {
                value
            };
            let attr = if name.starts_with("og:") {
                "property"
            } else {
                "name"
            };
            metadata += &format!(
                "<meta {attr}=\"{}\" content=\"{}\">\n",
                escape(name),
                escape(value)
            );
        }
        for schema in std::iter::once(&person).chain(page.schemas.iter()) {
            metadata += &format!(
                "<script type=\"application/ld+json\">{}</script>\n",
                schema.to_string().replace('<', "\\u003c")
            );
        }
        let mut styles: String = page
            .css
            .iter()
            .map(|url| format!("<link rel=\"stylesheet\" href=\"{url}\">\n"))
            .collect();
        if !mobile {
            styles += "<link rel=\"stylesheet\" href=\"/assets/presentation.css\">\n";
        }
        if !mobile && page.key != "404" {
            metadata += &format!(
                "<link rel=\"alternate\" media=\"only screen and (max-width: 768px)\" href=\"https://daipan.art/{}_mobile\">\n",
                page.key
            );
        }
        let editorial = if local {
            json!({"drafts": drafts.get(&page.key), "notes": notes.iter().filter(|n| n.page == page.key || n.page == "all" || (["quite_off","quite_off_mobile"].contains(&page.key.as_str()) && ["quite_off","quite_off_mobile"].contains(&n.page.as_str()))).collect::<Vec<_>>()})
        } else {
            json!({})
        };
        let html = (if mobile { &mobile_shell } else { &shell })
            .replace("{{title}}", &escape(&page.title))
            .replace("{{metadata}}", &metadata)
            .replace("{{styles}}", &styles)
            .replace("{{desktop_nav}}", &navigation(&page.key, false))
            .replace("{{mobile_nav}}", &navigation(&page.key, true))
            .replace("{{main_class}}", &page.main_class.join(" "))
            .replace("{{key}}", &page.key)
            .replace("{{base_key}}", content_key)
            .replace("{{view}}", if mobile { "mobile" } else { "desktop" })
            .replace("{{body}}", &body)
            .replace(
                "{{editorial}}",
                &editorial.to_string().replace('<', "\\u003c"),
            );
        ensure!(
            !html.contains("{{"),
            "Unresolved template variable in {}",
            page.key
        );
        let dest = if page.key == "home" {
            out.join("index.html")
        } else if page.key == "404" {
            out.join("404.html")
        } else {
            out.join(&page.key).join("index.html")
        };
        fs::create_dir_all(dest.parent().context("Missing page parent")?)?;
        fs::write(dest, html)?;
        if page.key != "404" && !mobile {
            sitemap += &format!("<url><loc>{}</loc></url>\n", escape(&page.canonical));
        }
    }
    sitemap += "</urlset>\n";
    fs::write(out.join("sitemap.xml"), sitemap)?;
    // Derive discovery links and summaries from the same records as the HTML head.
    let mut llms = format!(
        "# Dai Pan — Visual Art & Photography Portfolio\n\n> {}\n\n## Selected works\n",
        person["description"]
            .as_str()
            .context("Person description is required")?
    );
    for key in [
        "quite_off",
        "the_threes",
        "gallery_design",
        "in_praise_of_time",
        "reframed_still",
        "closing_time",
        "masks",
    ] {
        let page = pages
            .iter()
            .find(|p| p.key == key)
            .context("Missing work in discovery index")?;
        let name = page
            .schemas
            .iter()
            .find(|s| s.get("artMedium").is_some())
            .and_then(|s| s["name"].as_str())
            .context("Work name is required")?;
        let description = page
            .meta
            .get("description")
            .context("Page description is required")?;
        llms += &format!("- [{name}]({}): {description}\n", page.canonical);
    }
    llms += "\n## Biography and contact\n";
    for key in ["about", "contact"] {
        let page = pages
            .iter()
            .find(|p| p.key == key)
            .context("Missing biography or contact page")?;
        llms += &format!(
            "- [{}]({}): {}\n",
            page.title, page.canonical, page.meta["description"]
        );
    }
    llms += "\n## Education\n- School of Visual Arts (SVA), New York, USA — Bachelor of Fine Arts.\n- The University of Queensland, Brisbane, Australia — Master of Information Technology (in progress).\n\n## Related sites\n- [Writing — daipan.ink](https://daipan.ink/)\n- [GitHub — dpan538](https://github.com/dpan538)\n";
    fs::write(out.join("llms.txt"), llms)?;
    fs::write(
        out.join("robots.txt"),
        if local {
            "User-agent: *\nDisallow: /\n"
        } else {
            "User-agent: *\nAllow: /\nSitemap: https://daipan.art/sitemap.xml\n"
        },
    )?;
    if local {
        review::write(out, &notes)?;
        galleries::review(root, out)?;
    }
    Ok(pages.len())
}
fn current_year() -> String {
    // UTC calendar year; independent of the build host's locale and tools.
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86400;
    let mut year = 1970;
    let mut remaining = days;
    loop {
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let count = if leap { 366 } else { 365 };
        if remaining < count {
            return year.to_string();
        }
        remaining -= count;
        year += 1;
    }
}
fn build(root: &Path, output: &Path, local: bool) -> Result<()> {
    fs::create_dir_all(output)?;
    // Never erase an arbitrary user directory. A successful staging build is swapped in below.
    if output.read_dir()?.next().is_some() {
        ensure!(
            output.join(".portfolio-build").is_file(),
            "Output must be empty or a previous Rust build"
        );
    }
    let stage = output.with_file_name(format!(".portfolio-stage-{}", std::process::id()));
    ensure!(
        !stage.exists(),
        "Staging path already exists: {}",
        stage.display()
    );
    fs::create_dir(&stage)?;
    let result = (|| -> Result<usize> {
        copy_tree(&root.join("public"), &stage)?;
        copy_tree(&root.join("site/assets"), &stage.join("assets"))?;
        if !local {
            fs::remove_file(stage.join("assets/review.css"))?;
        }
        images::generate(root, &stage)?;
        Bindgen::new()
            .web(true)?
            .typescript(false)
            .out_name("browser")
            .input_path(root.join("target/wasm32-unknown-unknown/release/portfolio_browser.wasm"))
            .generate(stage.join("assets"))
            .context("Build the Rust browser crate first")?;
        fs::write(
            stage.join("assets/start.js"),
            "import init from './browser.js';\nawait init({ module_or_path: new URL('./browser_bg.wasm', import.meta.url) });\n",
        )?;
        let n = render(root, &stage, local)?;
        cache::publish(&stage)?;
        fs::write(
            stage.join(".portfolio-build"),
            if local { "local" } else { "static" },
        )?;
        Ok(n)
    })();
    let n = match result {
        Ok(n) => n,
        Err(e) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(e);
        }
    };
    let backup = output.with_file_name(format!(".portfolio-backup-{}", std::process::id()));
    ensure!(!backup.exists(), "Backup path already exists");
    fs::rename(output, &backup)?;
    if let Err(e) = fs::rename(&stage, output) {
        fs::rename(&backup, output)?;
        return Err(e.into());
    }
    // The published build is already valid. A concurrent Finder metadata write
    // must not turn a successful publication into a failed build.
    if let Err(error) = fs::remove_dir_all(&backup) {
        eprintln!(
            "Build complete; could not clean old output {}: {error}",
            backup.display()
        );
    }
    println!(
        "Built {n} portfolio pages{} → {}",
        if local { " + local review" } else { "" },
        output.display()
    );
    Ok(())
}
fn file_for(output: &Path, url: &str) -> Option<PathBuf> {
    let path = url.split('?').next()?.trim_start_matches('/');
    if path
        .split('/')
        .any(|s| s.starts_with('.') || s.contains(['%', '\\']))
    {
        return None;
    }
    let file = output.join(path);
    let file = if file.is_dir() {
        file.join("index.html")
    } else {
        file
    };
    file.is_file().then_some(file)
}
fn media(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json; charset=utf-8",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
fn serve(output: &Path, port: u16) -> Result<()> {
    let server = Server::http(("127.0.0.1", port)).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Rust portfolio: http://localhost:{port}/\nReview: http://localhost:{port}/review/");
    for request in server.incoming_requests() {
        if !matches!(request.method(), Method::Get | Method::Head) {
            let _ = request.respond(Response::empty(StatusCode(405)));
            continue;
        }
        let path = file_for(output, request.url());
        let status = if path.is_some() { 200 } else { 404 };
        let path = path.unwrap_or_else(|| output.join("404.html"));
        if let Ok(file) = fs::File::open(&path) {
            let _ = request.respond(
                Response::from_file(file)
                    .with_status_code(status)
                    .with_header(Header::from_bytes("Content-Type", media(&path)).unwrap())
                    .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap())
                    .with_header(Header::from_bytes("X-Content-Type-Options", "nosniff").unwrap()),
            );
        } else {
            let _ = request.respond(Response::empty(StatusCode(404)));
        }
    }
    Ok(())
}
fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let args: Vec<_> = std::env::args().skip(1).collect();
    let output = root.join(args.get(1).map(String::as_str).unwrap_or("target/preview"));
    match args.first().map(String::as_str).unwrap_or("build") {
        "build" => build(root, &output, !args.iter().any(|s| s == "--production")),
        "serve" => {
            build(root, &output, true)?;
            serve(
                &output,
                args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(4390),
            )
        }
        "images" => {
            fs::create_dir_all(&output)?;
            let source_root = args.get(2).map(Path::new).unwrap_or(root);
            images::generate(source_root, &output)
        }
        _ => bail!(
            "Usage: scripts/site build [output] [--production] | serve [output] [port] | images [output] [source-root]"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_paths_outside_preview() {
        for path in [
            "/../Cargo.toml",
            "/%2e%2e/Cargo.toml",
            "/a/../../b",
            "/a\\b",
            "/.git/config",
        ] {
            assert!(file_for(Path::new("."), path).is_none(), "{path}");
        }
    }
    #[test]
    fn escapes_metadata_delimiters() {
        assert_eq!(escape("a\"<&"), "a&quot;&lt;&amp;");
    }
}
