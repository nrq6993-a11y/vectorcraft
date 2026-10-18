//! Effect › Sketch › Halftone Pattern and Note Paper. Original filters built from public behaviour documentation:
//! <https://helpx.adobe.com/illustrator/using/summary-effects.html> (Sketch effects redraw in
//! black and white; Halftone Pattern simulates a halftone screen while keeping the range of tones)
//! and <https://helpx.adobe.com/illustrator/using/effects.html> (only Size depends on the raster
//! resolution). A visual approximation, not a reproduction of a proprietary algorithm.

use vectorcraft_geom::Point;

use super::pixelate::{Place, value_noise};
use super::texture::{luma, premultiply, straight};
use super::{PixelSpace, pick};

const SALT_NOTE: u64 = 0x4e6f_7465_7061_7065;

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

/// Note Paper: the object redrawn in black ink on fibrous white paper. `balance` (0..25) keeps more
/// of the light tones as bare paper; `graininess` (0..10) roughens the sheet; `relief` (0..50)
/// embosses the image's tones before inking. Coverage is kept.
pub(super) fn note_paper(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, balance: f64, graininess: f64, relief: f64) {
    if !(balance.is_finite() && graininess.is_finite() && relief.is_finite()) {
        return;
    }
    let Some(place) = Place::new(space, w, h) else { return };
    let balance = balance.clamp(0.0, 25.0) / 25.0;
    let grain = graininess.clamp(0.0, 10.0) / 10.0;
    let relief = relief.clamp(0.0, 50.0) / 50.0;
    let step = (2.0 / place.px).max(0.5);
    let mut tones = vec![-1.0f64; w * h];
    place.each(px, |x, y, p| {
        if p[3] == 0 {
            return;
        }
        let c = straight(*p);
        tones[y * w + x] = f64::from(1.0 - luma(c)).clamp(0.0, 1.0);
    });
    place.each(px, |x, y, p| {
        if p[3] == 0 {
            return;
        }
        let q = place.at(x, y);
        let fibre = value_noise(SALT_NOTE, q.x * 0.35, q.y * 0.35);
        let grit = value_noise(SALT_NOTE + 1, q.x * 2.7, q.y * 2.7);
        let paper = 0.55 + 0.45 * fibre + grain * (0.35 * grit - 0.175);
        let tone = tones[y * w + x];
        let emboss = |ox: f64, oy: f64| {
            let Some(i) = place.index(Point::new(q.x + ox, q.y + oy)) else { return tone };
            let t = tones[i];
            if t < 0.0 { tone } else { t }
        };
        let dx = emboss(step, 0.0) - emboss(-step, 0.0);
        let dy = emboss(0.0, step) - emboss(0.0, -step);
        let raised = (tone + relief * 0.35 * (dx + dy)).clamp(0.0, 1.0);
        // Higher image balance keeps more light tones as bare paper (a higher cutoff).
        let cutoff = (0.35 + 0.55 * balance) * paper;
        let ink = if raised > cutoff { 1.0 } else { 0.0 };
        *p = premultiply([(1.0 - ink) as f32; 3], f32::from(p[3]) / 255.0);
    });
}
