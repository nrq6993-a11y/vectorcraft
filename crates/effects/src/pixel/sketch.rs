//! Effect › Sketch › Halftone Pattern. Original filter built from public behaviour documentation:
//! <https://helpx.adobe.com/illustrator/using/summary-effects.html> (Sketch effects redraw in
//! black and white; Halftone Pattern simulates a halftone screen while keeping the range of tones)
//! and <https://helpx.adobe.com/illustrator/using/effects.html> (only Size depends on the raster
//! resolution). A visual approximation, not a reproduction of a proprietary algorithm.

use vectorcraft_geom::Point;

use super::pixelate::Place;
use super::texture::{cell_noise, luma, premultiply, smooth_noise, straight};
use super::{PixelSpace, pick};

const SALT_NOTE: u64 = 120;

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

/// Note Paper: the object redrawn as handmade paper — simplified tones (`image_balance` 0..40
/// shifts light and dark), grain as strong as `graininess` (0..20), embossed as deeply as
/// `relief` (0..24). Dark areas read as holes in the paper surrounded by white.
pub(super) fn note_paper(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, image_balance: f64, graininess: f64, relief: f64) {
    if !(image_balance.is_finite() && graininess.is_finite() && relief.is_finite()) {
        return;
    }
    let Some(place) = Place::new(space, w, h) else { return };
    let balance = image_balance.clamp(0.0, 40.0);
    let grain = (graininess / 20.0).clamp(0.0, 1.0);
    let rel = (relief / 24.0).clamp(0.0, 1.0);
    // Higher balance leaves more paper white (raises the ink threshold).
    let threshold = (0.35 + balance / 80.0).clamp(0.1, 0.9);
    let grain_pt = 1.0 + rel * 2.5;
    let emboss_pt = 0.75 + rel * 2.0;
    let step = emboss_pt * 0.35;
    place.each(px, |x, y, p| {
        if p[3] == 0 {
            return;
        }
        let q = place.at(x, y);
        let dark = f64::from(1.0 - luma(straight(*p)).clamp(0.0, 1.0));
        let n = 0.55 * cell_noise(SALT_NOTE, q, grain_pt, 0) + 0.45 * smooth_noise(SALT_NOTE + 1, q, grain_pt * 1.5);
        let toned = (dark * (1.0 + grain * 0.4 * n)).clamp(0.0, 1.0);
        let h0 = smooth_noise(SALT_NOTE + 2, q, emboss_pt);
        let hx = smooth_noise(SALT_NOTE + 2, Point::new(q.x + step, q.y), emboss_pt);
        let hy = smooth_noise(SALT_NOTE + 2, Point::new(q.x, q.y + step), emboss_pt);
        let embossed = (toned + rel * 0.35 * (hx - hy + 0.25 * (h0 - 0.5))).clamp(0.0, 1.0);
        let soft = 0.06 + grain * 0.12;
        let ink = ((embossed - threshold) / soft + 0.5).clamp(0.0, 1.0);
        *p = premultiply([(1.0 - ink) as f32; 3], f32::from(p[3]) / 255.0);
    });
}
