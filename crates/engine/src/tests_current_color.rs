//! SVG/CSS `currentColor` in paint command params.

use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::NodeId;

use super::*;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    s
}

fn id_of(v: &Value) -> NodeId {
    NodeId(v["id"].as_u64().unwrap())
}

#[test]
fn paint_accepts_current_color_keyword() {
    let mut s = session();
    let id = id_of(&s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap());
    for color in ["currentColor", "CURRENTCOLOR", "CurrentColor"] {
        s.execute("paint.setFill", &json!({"ids": [id.0], "color": color})).unwrap();
        let fill = s.doc().unwrap().doc.node(id).unwrap().appearance.fill().unwrap().paint.clone();
        assert!(fill.is_current_color(), "{color}: {fill:?}");
        assert_eq!(fill, Paint::current_color());
    }
    s.execute("paint.setStroke", &json!({"ids": [id.0], "color": "currentColor"})).unwrap();
    let stroke = s.doc().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().paint.clone();
    assert!(stroke.is_current_color());
}
