//! Formal visual sequencing. No identity, relationship, location or date inference.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Features {
    pub brightness: f64,
    pub contrast: f64,
    pub edges: f64,
    pub histogram: [f64; 16],
    pub grid: [f64; 9],
    pub ratio: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Affinity,
    Light,
    Texture,
    Shuffle,
}
impl Mode {
    pub fn parse(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "texture" => Self::Texture,
            "shuffle" => Self::Shuffle,
            _ => Self::Affinity,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Affinity => "affinity",
            Self::Light => "light",
            Self::Texture => "texture",
            Self::Shuffle => "shuffle",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Affinity => "Visual affinity",
            Self::Light => "Light → dark",
            Self::Texture => "Quiet → textured",
            Self::Shuffle => "Shuffle",
        }
    }
}
pub fn distance(a: &Features, b: &Features) -> f64 {
    let histogram = a
        .histogram
        .iter()
        .zip(b.histogram)
        .map(|(x, y)| (x - y).abs())
        .sum::<f64>()
        / 2.;
    // Centre the 3×3 luminance grid so composition does not simply repeat mean lightness.
    let composition = a
        .grid
        .iter()
        .zip(b.grid)
        .map(|(x, y)| ((x - a.brightness) - (y - b.brightness)).abs())
        .sum::<f64>()
        / 9.;
    0.20 * (a.brightness - b.brightness).abs()
        + 0.15 * (a.contrast - b.contrast).abs() * 2.
        + 0.20 * histogram
        + 0.25 * composition
        + 0.15 * (a.edges - b.edges).abs() * 3.
        + 0.05 * (a.ratio.ln() - b.ratio.ln()).abs().min(1.)
}
pub fn arrange(features: &[Features], mode: Mode, seed: u64) -> Vec<usize> {
    let n = features.len();
    let mut order: Vec<_> = (0..n).collect();
    if n < 2 {
        return order;
    }
    match mode {
        Mode::Light => order.sort_by(|&a, &b| {
            features[b]
                .brightness
                .total_cmp(&features[a].brightness)
                .then(a.cmp(&b))
        }),
        Mode::Texture => order.sort_by(|&a, &b| {
            features[a]
                .edges
                .total_cmp(&features[b].edges)
                .then(a.cmp(&b))
        }),
        Mode::Shuffle => {
            let mut x = seed.wrapping_add(0x9e3779b97f4a7c15);
            for i in (1..n).rev() {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                order.swap(i, (x % (i as u64 + 1)) as usize);
            }
        }
        Mode::Affinity => {
            order.clear();
            order.push(seed as usize % n);
            while order.len() < n {
                let last = *order.last().unwrap();
                let next = (0..n)
                    .filter(|i| !order.contains(i))
                    .min_by(|&a, &b| {
                        distance(&features[last], &features[a])
                            .total_cmp(&distance(&features[last], &features[b]))
                            .then(a.cmp(&b))
                    })
                    .unwrap();
                order.push(next);
            }
            // Improve the open path while preserving the visitor's chosen first image.
            for _ in 0..n {
                let mut improved = false;
                for i in 1..n - 1 {
                    for j in i + 1..n {
                        let before = distance(&features[order[i - 1]], &features[order[i]])
                            + if j + 1 < n {
                                distance(&features[order[j]], &features[order[j + 1]])
                            } else {
                                0.
                            };
                        let after = distance(&features[order[i - 1]], &features[order[j]])
                            + if j + 1 < n {
                                distance(&features[order[i]], &features[order[j + 1]])
                            } else {
                                0.
                            };
                        if after + 1e-9 < before {
                            order[i..=j].reverse();
                            improved = true;
                        }
                    }
                }
                if !improved {
                    break;
                }
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(b: f64) -> Features {
        Features {
            brightness: b,
            contrast: 0.1,
            edges: b / 4.,
            histogram: [0.; 16],
            grid: [b; 9],
            ratio: 1.5,
        }
    }
    #[test]
    fn every_mode_keeps_every_photo_once() {
        let features = vec![f(0.1), f(0.7), f(0.3), f(0.9), f(0.4)];
        for mode in [Mode::Affinity, Mode::Light, Mode::Texture, Mode::Shuffle] {
            for seed in 0..20 {
                let mut result = arrange(&features, mode, seed);
                result.sort();
                assert_eq!(result, vec![0, 1, 2, 3, 4]);
            }
        }
    }
    #[test]
    fn affinity_preserves_selected_anchor() {
        let features = vec![f(0.1), f(0.7), f(0.3)];
        assert_eq!(arrange(&features, Mode::Affinity, 2)[0], 2);
    }
    #[test]
    fn tonal_sequence_is_monotone() {
        let features = vec![f(0.1), f(0.7), f(0.3)];
        assert_eq!(arrange(&features, Mode::Light, 0), vec![1, 2, 0]);
    }
    #[test]
    fn shuffle_is_repeatable_and_varies() {
        let f = (0..15).map(|i| self::f(i as f64 / 15.)).collect::<Vec<_>>();
        assert_eq!(
            arrange(&f, Mode::Shuffle, 123),
            arrange(&f, Mode::Shuffle, 123)
        );
        assert_ne!(
            arrange(&f, Mode::Shuffle, 123),
            arrange(&f, Mode::Shuffle, 124)
        );
    }
    #[test]
    fn empty_and_single_are_valid() {
        assert!(arrange(&[], Mode::Affinity, 9).is_empty());
        assert_eq!(arrange(&[f(0.1)], Mode::Affinity, 9), vec![0]);
    }
}

/// Create an edit with breathing room, rather than a cluster of nearest neighbours.
/// Families are editorial annotations, not inferred relationships or meanings.
pub fn rhythmic(features: &[Features], families: &[&str], anchor: usize) -> Vec<usize> {
    let n = features.len();
    if n == 0 {
        return Vec::new();
    }
    let mut distances = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            distances.push(distance(&features[i], &features[j]));
        }
    }
    distances.sort_by(f64::total_cmp);
    let target = distances.get(distances.len() / 2).copied().unwrap_or(0.);
    let mut order = vec![anchor % n];
    while order.len() < n {
        let last = *order.last().unwrap();
        let score = |i: usize| {
            let d = distance(&features[last], &features[i]);
            let adjacent = if families.get(i) == families.get(last) {
                0.75
            } else {
                0.
            };
            let recent =
                if order.len() > 1 && families.get(i) == families.get(order[order.len() - 2]) {
                    0.12
                } else {
                    0.
                };
            (d - target).abs() + adjacent + recent
        };
        let next = (0..n)
            .filter(|i| !order.contains(i))
            .min_by(|&a, &b| score(a).total_cmp(&score(b)).then(a.cmp(&b)))
            .unwrap();
        order.push(next);
    }
    order
}
#[cfg(test)]
mod rhythm_tests {
    use super::*;
    #[test]
    fn rhythm_separates_repeated_families() {
        let features = (0..6)
            .map(|i| Features {
                brightness: i as f64 / 6.,
                contrast: 0.1,
                edges: 0.02,
                histogram: [0.; 16],
                grid: [0.3; 9],
                ratio: 1.5,
            })
            .collect::<Vec<_>>();
        let families = ["body", "body", "space", "space", "object", "object"];
        let order = rhythmic(&features, &families, 1);
        assert_eq!(order[0], 1);
        assert!(order.windows(2).all(|w| families[w[0]] != families[w[1]]));
        let mut unique = order.clone();
        unique.sort();
        assert_eq!(unique, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(order, rhythmic(&features, &families, 1));
    }
}
