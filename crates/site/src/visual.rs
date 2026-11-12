use anyhow::Result;
use gallery_core::Features;
use image::imageops::FilterType;
use std::path::Path;

pub fn extract(path: &Path, ratio: f64) -> Result<Features> {
    // Web derivative already has camera orientation applied. Analysis is independent of EXIF.
    let image = image::open(path)?
        .resize_exact(96, 96, FilterType::Triangle)
        .to_luma8();
    let mut sum = 0.;
    let mut squares = 0.;
    let mut edges = 0.;
    let mut histogram = [0.; 16];
    let mut grid = [0.; 9];
    for y in 0..96 {
        for x in 0..96 {
            let l = image.get_pixel(x, y)[0] as f64 / 255.;
            sum += l;
            squares += l * l;
            histogram[((l * 16.) as usize).min(15)] += 1.;
            grid[((y / 32) * 3 + x / 32) as usize] += l;
            if x > 0 {
                edges += (l - image.get_pixel(x - 1, y)[0] as f64 / 255.).abs();
            }
            if y > 0 {
                edges += (l - image.get_pixel(x, y - 1)[0] as f64 / 255.).abs();
            }
        }
    }
    let n = 96. * 96.;
    let brightness = sum / n;
    for h in &mut histogram {
        *h /= n;
    }
    for cell in &mut grid {
        *cell /= 32. * 32.;
    }
    Ok(Features {
        brightness,
        contrast: (squares / n - brightness * brightness).max(0.).sqrt(),
        edges: edges / (2. * 96. * 95.),
        histogram,
        grid,
        ratio,
    })
}
