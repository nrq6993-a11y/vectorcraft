use vectorcraft_geom::{Point, shapes};

use crate::{Document, Node, NodeKind};

#[test]
fn promote_axis_guide_paths_turns_straight_lines_into_ruler_guides() {
    let mut d = Document::new(400.0, 300.0);
    let layer = d.layers[0].id;
    let h = d.alloc_id();
    let mut n = Node::path(h, shapes::line(Point::new(10.0, 80.0), Point::new(200.0, 80.0)), Default::default());
    if let NodeKind::Path { guide, .. } = &mut n.kind {
        *guide = true;
    }
    d.insert(Some(layer), 0, n).unwrap();
    let v = d.alloc_id();
    let mut n = Node::path(v, shapes::line(Point::new(150.0, 20.0), Point::new(150.0, 250.0)), Default::default());
    if let NodeKind::Path { guide, .. } = &mut n.kind {
        *guide = true;
    }
    d.insert(Some(layer), 0, n).unwrap();
    assert_eq!(d.promote_axis_guide_paths(), 2);
    assert_eq!(d.guides.len(), 2);
    let h = d.guides.iter().find(|g| !g.vertical).expect("horizontal");
    let v = d.guides.iter().find(|g| g.vertical).expect("vertical");
    assert!((h.pos - 80.0).abs() < 1e-9 && h.layer == Some(layer));
    assert!((v.pos - 150.0).abs() < 1e-9 && v.layer == Some(layer));
    d.walk(|n| assert!(!matches!(n.kind, NodeKind::Path { guide: true, .. })));
}

#[test]
fn promote_axis_guide_paths_keeps_angled_guides_as_paths() {
    let mut d = Document::new(200.0, 200.0);
    let layer = d.layers[0].id;
    let id = d.alloc_id();
    let mut n = Node::path(id, shapes::line(Point::new(0.0, 0.0), Point::new(100.0, 50.0)), Default::default());
    if let NodeKind::Path { guide, .. } = &mut n.kind {
        *guide = true;
    }
    d.insert(Some(layer), 0, n).unwrap();
    assert_eq!(d.promote_axis_guide_paths(), 0);
    assert!(d.guides.is_empty());
    let mut guides = 0;
    d.walk(|n| guides += usize::from(matches!(n.kind, NodeKind::Path { guide: true, .. })));
    assert_eq!(guides, 1);
}
