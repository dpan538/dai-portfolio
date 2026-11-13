//! Publish immutable URLs only after rendering has consumed the build cache.
//! Stable aliases remain available for old links and always revalidate on Vercel.
use anyhow::{Context, Result, ensure};
use regex::{Captures, Regex};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
struct CacheManifest {
    asset_prefix: String,
    images: BTreeMap<String, String>,
}

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            found.extend(files(&entry.path())?);
        } else {
            found.push(entry.path());
        }
    }
    found.sort();
    Ok(found)
}

fn alias(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target.parent().context("Asset parent missing")?)?;
    if fs::hard_link(source, target).is_err() {
        fs::copy(source, target)?;
    }
    Ok(())
}

fn rewrite(text: &str, pattern: &Regex, urls: &BTreeMap<String, String>) -> String {
    pattern
        .replace_all(text, |matched: &Captures<'_>| urls[&matched[0]].clone())
        .into_owned()
}

pub fn publish(out: &Path) -> Result<()> {
    let mut images = BTreeMap::new();
    for path in files(&out.join("_optimized/projects"))? {
        let old = format!("/{}", path.strip_prefix(out)?.to_string_lossy());
        let hash = format!("{:x}", Sha256::digest(fs::read(&path)?));
        let new = format!(
            "/_optimized/immutable/{hash}/{}",
            path.strip_prefix(out.join("_optimized"))?.to_string_lossy()
        );
        alias(&path, &out.join(new.trim_start_matches('/')))?;
        images.insert(old, new);
    }
    ensure!(
        !images.is_empty(),
        "Cannot publish without image derivatives"
    );
    // One pass also covers inline carousel JSON, srcsets and lightbox sources.
    let pattern = Regex::new(
        &images
            .keys()
            .map(|url| regex::escape(url))
            .collect::<Vec<_>>()
            .join("|"),
    )?;
    let assets = files(&out.join("assets"))?;
    for path in &assets {
        if matches!(
            path.extension().and_then(|x| x.to_str()),
            Some("css" | "js" | "json")
        ) {
            let text = fs::read_to_string(path)?;
            fs::write(path, rewrite(&text, &pattern, &images))?;
        }
    }
    // Keep relative wasm-bindgen imports beside their loader in one namespace.
    let mut digest = Sha256::new();
    digest.update(b"portfolio-assets-v1\0");
    for path in &assets {
        let relative = path.strip_prefix(out.join("assets"))?.to_string_lossy();
        let bytes = fs::read(path)?;
        digest.update((relative.len() as u64).to_be_bytes());
        digest.update(relative.as_bytes());
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(&bytes);
    }
    let asset_prefix = format!("/assets/immutable/{:x}", digest.finalize());
    for path in assets {
        let relative = path.strip_prefix(out.join("assets"))?;
        alias(
            &path,
            &out.join(asset_prefix.trim_start_matches('/'))
                .join(relative),
        )?;
    }
    for path in files(out)?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "html"))
    {
        let text = rewrite(&fs::read_to_string(&path)?, &pattern, &images)
            .replace("/assets/", &format!("{asset_prefix}/"));
        fs::write(path, text)?;
    }
    fs::write(
        out.join("assets/cache-manifest.json"),
        serde_json::to_vec(&CacheManifest {
            asset_prefix,
            images,
        })?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path, css: &str, image: &[u8]) -> Result<()> {
        fs::create_dir_all(root.join("assets/mobile"))?;
        fs::create_dir_all(root.join("_optimized/projects/example"))?;
        fs::write(
            root.join("assets/start.js"),
            "import './browser.js'; new URL('./browser_bg.wasm', import.meta.url);",
        )?;
        fs::write(root.join("assets/browser.js"), "// generated bridge")?;
        fs::write(root.join("assets/browser_bg.wasm"), b"\0asm")?;
        fs::write(root.join("assets/mobile/common.css"), css)?;
        fs::write(
            root.join("_optimized/projects/example/frame_400w.jpg"),
            image,
        )?;
        fs::write(
            root.join("index.html"),
            r#"<link href="/assets/mobile/common.css"><script src="/assets/start.js"></script><img src="/_optimized/projects/example/frame_400w.jpg"><script type="application/json">{"images":["/_optimized/projects/example/frame_400w.jpg"]}</script>"#,
        )?;
        publish(root)
    }
    #[test]
    fn version_changes_with_bytes_and_preserves_loader_dependencies() -> Result<()> {
        let root = std::env::temp_dir().join(format!("portfolio-cache-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        for (name, css, image) in [
            ("first", "body{}", b"photo".as_slice()),
            ("same", "body{}", b"photo"),
            ("css", "body{color:red}", b"photo"),
            ("photo", "body{}", b"new photograph"),
        ] {
            fixture(&root.join(name), css, image)?;
        }
        let read = |name: &str| -> Result<serde_json::Value> {
            Ok(serde_json::from_slice(&fs::read(
                root.join(name).join("assets/cache-manifest.json"),
            )?)?)
        };
        let first = read("first")?;
        assert_eq!(first, read("same")?);
        assert_ne!(first["asset_prefix"], read("css")?["asset_prefix"]);
        assert_ne!(first["images"], read("photo")?["images"]);
        assert_eq!(first["images"], read("css")?["images"]);
        let prefix = first["asset_prefix"].as_str().unwrap();
        for dependency in [
            "start.js",
            "browser.js",
            "browser_bg.wasm",
            "mobile/common.css",
        ] {
            assert!(
                root.join("first")
                    .join(prefix.trim_start_matches('/'))
                    .join(dependency)
                    .is_file()
            );
        }
        let html = fs::read_to_string(root.join("first/index.html"))?;
        assert!(!html.contains("/_optimized/projects/"));
        assert!(!html.contains("\"/assets/start.js\""));
        assert_eq!(html.matches("/_optimized/immutable/").count(), 2);
        let url = first["images"]["/_optimized/projects/example/frame_400w.jpg"]
            .as_str()
            .unwrap();
        let hash = format!(
            "{:x}",
            Sha256::digest(fs::read(
                root.join("first").join(url.trim_start_matches('/'))
            )?)
        );
        assert!(url.contains(&hash));
        assert_eq!(
            fs::read(root.join("first/_optimized/projects/example/frame_400w.jpg"))?,
            b"photo"
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
