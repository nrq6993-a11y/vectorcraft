//! `currentColor` in command paint params (#3543).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_engine::{Session, cmd::color_value};

#[test]
fn color_value_accepts_currentcolor() {
    let c = color_value(&json!("currentColor")).expect("currentColor");
    assert!(c.is_current_color());
}

#[test]
fn paint_set_stroke_accepts_currentcolor() {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap();
    s.execute("paint.setStroke", &json!({"color": "currentColor", "weight": 2})).unwrap();
    let id = s.doc().unwrap().selection.objects[0];
    let paint = s.doc().unwrap().doc.node(id).unwrap().appearance.stroke_paint();
    assert!(paint.color().unwrap().is_current_color());
}
