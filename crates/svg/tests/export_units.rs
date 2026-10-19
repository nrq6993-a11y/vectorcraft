//! SVG export writes the document's ruler units on the root element (#864).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_doc::{Document, Unit};
use vectorcraft_geom::Rect;
use vectorcraft_svg::{export, import};

fn close_rect(a: Rect, b: Rect) -> bool {
    (a.x0 - b.x0).abs() < 0.02 && (a.y0 - b.y0).abs() < 0.02 && (a.x1 - b.x1).abs() < 0.02 && (a.y1 - b.y1).abs() < 0.02
}

#[test]
fn export_keeps_millimeter_ruler_units_on_the_root() {
    let mm = 72.0 / 25.4;
    let board = Rect::new(0.0, 0.0, 210.0 * mm, 297.0 * mm);
    let mut doc = Document::new(board.width(), board.height());
    doc.units = Unit::Millimeters;

    let svg = export(&doc, &Default::default());
    assert!(svg.contains(r#"width="210mm" height="297mm""#), "{svg}");
    assert!(svg.contains(r#"viewBox="0 0 210 297""#), "{svg}");

    let back = import(&svg).unwrap();
    assert_eq!(back.units, Unit::Millimeters);
    assert!(close_rect(back.artboards[0].rect, board), "{:?}", back.artboards[0].rect);
}

#[test]
fn export_keeps_inch_ruler_units_on_the_root() {
    let board = Rect::new(0.0, 0.0, 612.0, 792.0);
    let mut doc = Document::new(board.width(), board.height());
    doc.units = Unit::Inches;

    let svg = export(&doc, &Default::default());
    assert!(svg.contains(r#"width="8.5in" height="11in""#), "{svg}");

    let back = import(&svg).unwrap();
    assert_eq!(back.units, Unit::Inches);
    assert!(close_rect(back.artboards[0].rect, board));
}

#[test]
fn points_stay_unitless_user_units() {
    let mut doc = Document::new(400.0, 300.0);
    doc.units = Unit::Points;
    let svg = export(&doc, &Default::default());
    assert!(svg.contains(r#"width="400" height="300""#), "{svg}");
    assert!(!svg.contains("pt\""), "{svg}");
}
