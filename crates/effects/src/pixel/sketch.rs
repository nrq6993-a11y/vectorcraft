//! Effect › Sketch › Halftone Pattern and Stamp. Original filters built from public behaviour documentation:
//! <https://helpx.adobe.com/illustrator/using/summary-effects.html> (Sketch effects redraw in
//! black and white; Halftone Pattern simulates a halftone screen while keeping the range of tones;
//! Stamp simplifies the image to look stamped in black ink on white paper)
//! and <https://helpx.adobe.com/illustrator/using/effects.html> (only Size depends on the raster
//! resolution). A visual approximation, not a reproduction of a proprietary algorithm.

use super::pixelate::Place;
use super::texture::{luma, premultiply, straight};
use super::{PixelSpace, blur_plane, pick};

/// Halftone Pattern's screens, in its Pattern Type menu's order: (label, parameter value).
pub const HALFTONE_TYPES: [(&str, &str); 3] = [("Circle", "circle"), ("Dot", "dot"), ("Line", "line")];

/// A Halftone Pattern screen ([`HALFTONE_TYPES`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HalftonePattern {
    /// Rings round the object's centre.
    Circle,
    /// Dots on a square grid.
    Dot,
    /// Horizontal lines.
    Line,
}

impl HalftonePattern {
    const ALL: [HalftonePattern; 3] = [HalftonePattern::Circle, HalftonePattern::Dot, HalftonePattern::Line];

    /// The screen of Pattern Type `value` (a [`HALFTONE_TYPES`] value, any case); Dot otherwise.
    pub fn parse(value: &str) -> Self {
        pick(&HALFTONE_TYPES, &Self::ALL, value, HalftonePattern::Dot)
    }
}

/// Halftone Pattern: the object redrawn in black ink on white paper through a `kind` screen whose
/// cells are `size` points apart, anchored at the object's centre. The ink covers as much of a
/// cell as the colour is dark; `contrast` (0..50) hardens the screen's soft edges, to pure black
/// and white at 50. Coverage is kept.
pub(super) fn halftone_pattern(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, size: f64, contrast: f64, kind: HalftonePattern) {
    if !(size.is_finite() && size > 0.0 && contrast.is_finite()) {
        return;
    }
    let Some(place) = Place::new(space, w, h) else { return };
    // The width of the screen's soft edge, in tones: 0 is a hard threshold.
    let soft = (0.5 * (1.0 - contrast / 50.0)).clamp(0.0, 0.5);
    place.each(px, |x, y, p| {
        if p[3] == 0 {
            return;
        }
        let q = place.at(x, y);
        let band = |v: f64| (2.0 * ((v / size).rem_euclid(1.0) - 0.5)).abs();
        // Where in its cell the pixel lies, 0 where the ink starts to 1 where it ends.
        let screen = match kind {
            HalftonePattern::Line => band(q.y),
            HalftonePattern::Circle => band(q.x.hypot(q.y)),
            HalftonePattern::Dot => 0.5 * (band(q.x).powi(2) + band(q.y).powi(2)),
        };
        let dark = f64::from(1.0 - luma(straight(*p))).clamp(0.0, 1.0);
        let ink = if dark >= 1.0 {
            1.0
        } else if soft > 0.0 {
            // The threshold kept inside [soft / 2, 1 - soft / 2], so white stays white and black black.
            let threshold = 0.5 * soft + screen * (1.0 - soft);
            ((dark - threshold) / soft + 0.5).clamp(0.0, 1.0)
        } else if dark > screen {
            1.0
        } else {
            0.0
        };
        *p = premultiply([(1.0 - ink) as f32; 3], f32::from(p[3]) / 255.0);
    });
}

/// Stamp: the object redrawn in black ink on white paper like a rubber stamp. `smoothness` (1..50)
/// blurs tones before the threshold so higher values soften edges; `lightDarkBalance` (0..25) shifts
/// the threshold so higher values leave more midtones as ink. Coverage is kept.
pub(super) fn stamp(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, light_dark_balance: f64, smoothness: f64) {
    if w == 0 || h == 0 || !(light_dark_balance.is_finite() && smoothness.is_finite()) {
        return;
    }
    let balance = light_dark_balance.clamp(0.0, 25.0);
    let smoothness = smoothness.clamp(1.0, 50.0);
    let n = w * h;
    let mut tone = vec![0.0f32; n];
    let mut seen = vec![false; n];
    for (i, p) in px.iter().enumerate() {
        if p[3] == 0 {
            continue;
        }
        seen[i] = true;
        tone[i] = f64::from(1.0 - luma(straight(*p))).clamp(0.0, 1.0) as f32;
    }
    let sigma = (smoothness / 6.0) / space.px.max(1e-9);
    let mut smooth = tone.clone();
    blur_plane(&mut smooth, w, h, sigma);
    // Higher balance draws more ink (lower threshold); smoothness already softened edges.
    let threshold = (0.72 - balance / 25.0 * 0.5).clamp(0.08, 0.92) as f32;
    for (i, p) in px.iter_mut().enumerate() {
        if !seen[i] {
            continue;
        }
        let ink = f32::from(smooth[i] > threshold);
        *p = premultiply([(1.0 - ink) as f32; 3], f32::from(p[3]) / 255.0);
    }
}
