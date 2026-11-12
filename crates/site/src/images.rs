use anyhow::{Context, Result, ensure};
use image::{
    DynamicImage, ImageDecoder, ImageReader, codecs::jpeg::JpegEncoder, imageops::FilterType,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct ImageMeta {
    pub width: u32,
    pub height: u32,
    pub variants: Vec<Variant>,
}
#[derive(Serialize, Deserialize)]
pub struct Variant {
    pub width: u32,
    pub url: String,
}
pub type Manifest = BTreeMap<String, ImageMeta>;
pub fn public_path(path: &Path) -> String {
    path.to_string_lossy().replace(' ', "_")
}

// Include the encoder policy so a future quality/size change invalidates old derivatives.
fn fingerprint(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"portfolio-images-v1:lanczos3:jpeg95:400,640,800,1024,1280,1600:exif");
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn walk(
    source: &Path,
    relative: &Path,
    cache: &Path,
    output: &Path,
    manifest: &mut Manifest,
) -> Result<()> {
    let mut entries = fs::read_dir(source)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let rel = relative.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            walk(&entry.path(), &rel, cache, output, manifest)?;
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
            continue;
        }
        let signature = fingerprint(&fs::read(entry.path())?);
        let stamp = cache
            .join(public_path(&rel))
            .with_extension(format!("{ext}.sha256"));
        let source_unchanged = fs::read_to_string(&stamp).ok().as_deref() == Some(&signature);
        let mut decoder = ImageReader::open(entry.path())?
            .into_decoder()
            .with_context(|| entry.path().display().to_string())?;
        let orientation = decoder.orientation()?;
        let (raw_w, raw_h) = decoder.dimensions();
        let (width, height) = if orientation.to_exif() >= 5 {
            (raw_h, raw_w)
        } else {
            (raw_w, raw_h)
        };
        let widths: Vec<u32> = [400, 640, 800, 1024, 1280, 1600]
            .into_iter()
            .filter(|w| *w <= width)
            .collect();
        let widths = if widths.is_empty() {
            vec![width]
        } else {
            widths
        };
        let mut original = None;
        let mut variants = Vec::new();
        for w in widths {
            let stem = rel.file_stem().unwrap().to_string_lossy();
            let file = rel.with_file_name(format!("{stem}_{w}w.{ext}"));
            let file = public_path(&file);
            let cached = cache.join(&file);
            let target = output.join("_optimized/projects").join(&file);
            let fresh = source_unchanged && cached.is_file();
            if !fresh {
                let img = match &original {
                    Some(img) => img,
                    None => {
                        // Apply camera orientation before discarding EXIF in web derivatives.
                        let mut decoded = image::open(entry.path())?;
                        decoded.apply_orientation(orientation);
                        original.insert(decoded)
                    }
                };
                let scaled: DynamicImage = img.resize_exact(
                    w,
                    ((height as u64 * w as u64 + width as u64 / 2) / width as u64) as u32,
                    FilterType::Lanczos3,
                );
                fs::create_dir_all(cached.parent().unwrap())?;
                let temporary = cached.with_extension(format!("{ext}.tmp"));
                if ext == "png" {
                    scaled.save_with_format(&temporary, image::ImageFormat::Png)?;
                } else {
                    JpegEncoder::new_with_quality(fs::File::create(&temporary)?, 95)
                        .encode_image(&scaled)?;
                }
                fs::rename(temporary, &cached)?;
            }
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(cached, target)?;
            variants.push(Variant {
                width: w,
                url: format!("/_optimized/projects/{file}"),
            });
        }
        fs::create_dir_all(stamp.parent().unwrap())?;
        fs::write(stamp, signature)?;
        let previous = manifest.insert(
            format!("/projects/{}", public_path(&rel)),
            ImageMeta {
                width,
                height,
                variants,
            },
        );
        ensure!(
            previous.is_none(),
            "Public image paths collide after normalisation: {}",
            rel.display()
        );
    }
    Ok(())
}

pub fn generate(root: &Path, output: &Path) -> Result<()> {
    let mut manifest = Manifest::new();
    walk(
        &root.join("public/projects"),
        Path::new(""),
        &root.join(".cache/images"),
        output,
        &mut manifest,
    )?;
    fs::create_dir_all(output.join("assets"))?;
    fs::write(
        output.join("assets/image-manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "Prepared responsive variants for {} images (Rust, cached)",
        manifest.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_identity_depends_on_contents_not_checkout_timestamps() {
        assert_eq!(fingerprint(b"same photo"), fingerprint(b"same photo"));
        assert_ne!(fingerprint(b"same photo"), fingerprint(b"edited photo"));
        assert_eq!(fingerprint(b"same photo").len(), 64);
    }

    #[test]
    fn derivatives_reuse_cache_and_rebuild_changed_or_missing_images() {
        let root =
            std::env::temp_dir().join(format!("portfolio-images-test-{}", std::process::id()));
        let originals = root.join("public/projects");
        fs::create_dir_all(&originals).unwrap();
        let source = originals.join("test.png");
        image::RgbImage::from_pixel(8, 6, image::Rgb([10, 20, 30]))
            .save(&source)
            .unwrap();
        let output = root.join("output");
        generate(&root, &output).unwrap();
        let cached = root.join(".cache/images/test_8w.png");
        let timestamp = cached.metadata().unwrap().modified().unwrap();
        generate(&root, &output).unwrap();
        assert_eq!(cached.metadata().unwrap().modified().unwrap(), timestamp);
        image::RgbImage::from_pixel(8, 6, image::Rgb([200, 100, 50]))
            .save(&source)
            .unwrap();
        generate(&root, &output).unwrap();
        assert_eq!(
            image::open(&cached).unwrap().to_rgb8().get_pixel(0, 0).0,
            [200, 100, 50]
        );
        fs::remove_file(&cached).unwrap();
        generate(&root, &output).unwrap();
        assert!(cached.is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
