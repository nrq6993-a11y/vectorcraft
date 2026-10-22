//! `currentColor` paints export as the SVG keyword.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorcraft_color::Paint;
use vectorcraft_doc::{Appearance, AppearanceItem, Document, FillLayer, Node, StrokeLayer};
use vectorcraft_geom::{PathData, Point, SubPath};
use vectorcraft_svg::{ExportOptions, export};

fn path_doc(items: Vec<AppearanceItem>) -> Document {
    let mut d = Document::new(40.0, 40.0);
    let path =
        PathData::single(SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(30.0, 0.0), Point::new(30.0, 20.0), Point::new(0.0, 20.0)], true));
    let n = Node::path(d.alloc_id(), path, Appearance { items, ..Default::default() });
    d.insert(Some(d.layers[0].id), usize::MAX, n).unwrap();
    d
}

#[test]
fn export_writes_current_color_for_fill_and_stroke() {
    let fill_svg = export(&path_doc(vec![AppearanceItem::Fill(FillLayer::new(Paint::current_color()))]), &ExportOptions::default());
    assert!(fill_svg.contains("fill=\"currentColor\""), "{fill_svg}");
    let stroke_svg = export(&path_doc(vec![AppearanceItem::Stroke(StrokeLayer::new(Paint::current_color(), 1.0))]), &ExportOptions::default());
    assert!(stroke_svg.contains("stroke=\"currentColor\""), "{stroke_svg}");
}
