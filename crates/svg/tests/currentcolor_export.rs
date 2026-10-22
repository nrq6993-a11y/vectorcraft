//! SVG export writes the CSS `currentColor` keyword for paints that use it (#3543).
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node};
use vectorcraft_geom::{Point, shapes};
use vectorcraft_svg::{ExportOptions, export};

#[test]
fn stroke_currentcolor_exports_as_keyword() {
    let mut d = Document::new(24.0, 24.0);
    let l = d.layers[0].id;
    let appearance = Appearance::basic(Paint::None, Paint::solid(Color::CURRENT_COLOR), 2.0);
    let n = Node::path(d.alloc_id(), shapes::line(Point::new(4.0, 12.0), Point::new(20.0, 12.0)), appearance);
    d.insert(Some(l), usize::MAX, n).unwrap();
    let svg = export(&d, &ExportOptions::default());
    assert!(svg.contains("stroke=\"currentColor\""), "{svg}");
    assert!(!svg.contains("stroke=\"#000000\""), "{svg}");
}
