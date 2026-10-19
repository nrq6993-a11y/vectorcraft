//! Effect › Sketch › Halftone Pattern. Original filter built from public behaviour documentation:
//! <https://helpx.adobe.com/illustrator/using/summary-effects.html> (Sketch effects redraw in
//! black and white; Halftone Pattern simulates a halftone screen while keeping the range of tones)
//! and <https://helpx.adobe.com/illustrator/using/effects.html> (only Size depends on the raster
//! resolution). A visual approximation, not a reproduction of a proprietary algorithm.

use super::pixelate::{Place, value_noise};
use super::texture::{luma, premultiply, straight};
use super::{PixelSpace, pick};
use vectorcraft_geom::Point;

const SALT_CONTE: u64 = 0xC0A7E_C0A7E;

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

/// Conté Crayon: the object redrawn as dense black conté in its dark areas and white crayon in its
/// lights on grainy paper. `foreground` and `background` (1..25) set how strongly the black and
/// white crayons bite into shadows and highlights. Coverage is kept; hues become grey like the
/// public Sketch summary describes.
pub(super) fn conte_crayon(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, foreground: f64, background: f64) {
    if !(foreground.is_finite() && background.is_finite()) {
        return;
    }
    let Some(place) = Place::new(space, w, h) else { return };
    let fg = foreground.clamp(1.0, 25.0) / 25.0;
    let bg = background.clamp(1.0, 25.0) / 25.0;
    let grain_at = |salt: u64, q: Point, scale: f64| 2.0 * value_noise(salt, q.x / scale, q.y / scale) - 1.0;
    place.each(px, |x, y, p| {
        if p[3] == 0 {
            return;
        }
        let q = place.at(x, y);
        let dark = f64::from(1.0 - luma(straight(*p))).clamp(0.0, 1.0);
        let light = 1.0 - dark;
        let px_u = place.px.max(1e-9);
        let grit = 0.5
            + 0.28 * grain_at(SALT_CONTE, q, 2.8 * px_u)
            + 0.17 * grain_at(SALT_CONTE + 1, q, 1.1 * px_u)
            + 0.12 * grain_at(SALT_CONTE + 2, q, 0.45 * px_u);
        let chalk = grit.clamp(0.0, 1.0);
        let black = fg * dark * (0.45 + 0.55 * chalk);
        let white = bg * light * (0.45 + 0.55 * (1.0 - chalk));
        let ink = (black - 0.65 * white * (1.0 - black)).clamp(0.0, 1.0);
        let paper = (1.0 - ink).clamp(0.0, 1.0);
        *p = premultiply([paper as f32; 3], f32::from(p[3]) / 255.0);
    });
}
