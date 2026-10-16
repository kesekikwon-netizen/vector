//! The commands behind the tool column's archaeology tools (design 3.0 D4, J024): section marks
//! between two points with the next free letter, level lines from a clicked height with their
//! datum tick, the ▽ spot height and soil layer boundaries with their numbers and description
//! lines. Each placement is one undo step.

use serde_json::{Value, json};
use vectorcraft_doc::{Node, NodeId, NodeKind};

use super::*;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 600, "height": 400})).unwrap();
    s
}

fn run(s: &mut Session, cmd: &str, p: Value) -> Value {
    s.execute(cmd, &p).unwrap_or_else(|e| panic!("{cmd} {p}: {e}"))
}

fn node(s: &Session, id: &Value) -> Node {
    let id = NodeId(id.as_u64().unwrap_or_else(|| panic!("no id in {id}")));
    s.doc().unwrap().doc.node(id).unwrap_or_else(|| panic!("no node {}", id.0)).clone()
}

/// The children of group `g` with names starting `name`.
fn named<'a>(g: &'a Node, name: &str) -> Vec<&'a Node> {
    g.children().map_or(vec![], |c| c.iter().map(|n| n.as_ref()).filter(|n| n.name.as_deref().is_some_and(|m| m.starts_with(name))).collect())
}

fn bounds(n: &Node) -> kurbo::Rect {
    n.geometric_bounds().unwrap_or_else(|| panic!("{:?} has no bounds", n.name))
}

fn text_of(n: &Node) -> String {
    match &n.kind {
        NodeKind::Text(t) => t.plain_text(),
        _ => panic!("{:?} is not text", n.name),
    }
}

fn node_count(s: &Session) -> usize {
    s.doc().unwrap().doc.node_count()
}

#[test]
fn section_marker_between_two_points_takes_the_next_letter() {
    let mut s = session();
    let a = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "cut", "p0": [100, 100], "p1": [300, 100]}));
    assert_eq!(a["label"], "A–A′");
    let g = node(&s, &a["id"]);
    assert_eq!(g.name.as_deref(), Some("단면 기호 A–A′"));
    let letters: Vec<String> = named(&g, "기호 ").into_iter().map(text_of).collect();
    assert_eq!(letters, ["A", "A′"]);
    // The red bars reach inward from the two points; the sight arrows 16 pt outward, their barbs
    // folding up (the side a line drawn left to right is seen from).
    let bars = bounds(named(&g, "단면 붉은").first().unwrap());
    assert!((bars.x0 - 100.0).abs() < 1e-6 && (bars.x1 - 300.0).abs() < 1e-6 && (bars.height() - 2.8).abs() < 1e-6, "{bars:?}");
    let arrows = bounds(named(&g, "시선").first().unwrap());
    assert!((arrows.x0 - 84.0).abs() < 1e-6 && (arrows.x1 - 316.0).abs() < 1e-6, "{arrows:?}");
    assert!((arrows.y0 - 96.2).abs() < 1e-6 && (arrows.y1 - 101.4).abs() < 1e-6, "{arrows:?}");

    // A slanted one takes B, its arrow going out along the line beyond each point.
    let before = node_count(&s);
    let b = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "cut", "p0": [100, 200], "p1": [200, 300]}));
    assert_eq!(b["label"], "B–B′");
    let arrows = bounds(named(&node(&s, &b["id"]), "시선").first().unwrap());
    let reach = 16.0 / 2f64.sqrt();
    assert!(arrows.x0 < 100.0 - reach + 2.0 && arrows.y0 < 200.0 - reach + 2.0, "{arrows:?}");
    assert!(arrows.x1 > 200.0 + reach - 2.0 && arrows.y1 > 300.0 + reach - 2.0, "{arrows:?}");
    // One undo step, and then B is free again.
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(node_count(&s), before);
    assert_eq!(vectorcraft_doc::section_marks::next_letter(&s.doc().unwrap().doc), "B");
    // The one-point marks keep their letter when given one and take the next one when not.
    let old = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "cut", "at": [50, 50], "label": "A", "orientation": "vertical"}));
    assert_eq!(old["label"], "A–A′");
    let next = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "cut", "at": [50, 50]}));
    assert_eq!(next["label"], "B–B′");
    // Both points or neither; two different ones; not for a datum tick.
    for bad in [
        json!({"kind": "cut", "p0": [10, 10]}),
        json!({"kind": "cut", "p0": [10, 10], "p1": "x"}),
        json!({"kind": "cut", "p0": [10, 10], "p1": [10.5, 10]}),
        json!({"kind": "datum", "p0": [10, 10], "p1": [100, 10]}),
    ] {
        assert!(s.execute("trowel.insertSectionMarker", &bad).is_err(), "{bad}");
    }
}

/// A V-shaped pit in a 1:20 section: the ground at y 200 either side, its bottom at y 300.
fn pit(s: &mut Session) -> Value {
    run(s, "doc.setScale", json!({"scale": 20}));
    run(
        s,
        "path.create",
        json!({"anchors": [{"x": 0, "y": 200}, {"x": 100, "y": 200}, {"x": 200, "y": 300}, {"x": 300, "y": 200}, {"x": 400, "y": 200}]}),
    )
}

#[test]
fn level_lines_put_the_clicked_point_at_the_height() {
    let mut s = session();
    let ground = pit(&mut s);
    let before = node_count(&s);
    // Clicked on the pit's side 50 pt over its bottom: 32.0 m there.
    let r = run(&mut s, "trowel.levelLines", json!({"profileId": ground["id"], "at": [150, 250], "elevationM": 32.0}));
    let ppm = (72.0 / 0.0254) / 20.0;
    assert!((r["datumM"].as_f64().unwrap() - (32.0 - 50.0 / ppm)).abs() < 1e-9, "{r}");
    let lines = node(&s, &r["id"]);
    assert!(lines.elevation.is_some(), "live level lines");
    // A heavy 50 cm line at the clicked height, across the pit's open void only (it stops at the
    // ground line), 0.5 pt; the 10 cm ones 0.3 pt.
    let majors: Vec<kurbo::Rect> = named(&lines, "주 해발선").into_iter().map(bounds).filter(|b| (b.y0 - 250.0).abs() < 1e-6).collect();
    assert_eq!(majors.len(), 1, "one 32.0 m line");
    let m = majors.first().unwrap();
    assert!((m.x0 - 150.0).abs() < 1e-3 && (m.x1 - 250.0).abs() < 1e-3, "{m:?}");
    let weight = |n: &Node| n.appearance.stroke().map(|st| st.width);
    assert_eq!(named(&lines, "주 해발선").first().map(|n| weight(n)), Some(Some(0.5)));
    assert_eq!(named(&lines, "해발선").first().map(|n| weight(n)), Some(Some(0.3)));
    assert!(named(&lines, "해발 숫자").iter().any(|n| text_of(n) == "32.0m"), "its label");
    // The datum tick at that height right of the ground line.
    let tick = node(&s, &r["datumId"]);
    assert_eq!(tick.name.as_deref(), Some("단면 해발 눈금 (32.0m)"));
    let bar = bounds(named(&tick, "해발 기준 눈금").first().unwrap());
    assert!((bar.center().x - 416.0).abs() < 1e-6 && (bar.center().y - 250.0).abs() < 1e-6, "{bar:?}");
    // One undo step for both.
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(node_count(&s), before);
    // Two decimals when the height has them.
    let r = run(&mut s, "trowel.levelLines", json!({"profileId": ground["id"], "at": [150, 250], "elevationM": 32.45}));
    assert_eq!(node(&s, &r["datumId"]).name.as_deref(), Some("단면 해발 눈금 (32.45m)"));
    // Not a path, no height, nothing there.
    let id = ground["id"].clone();
    for bad in
        [json!({"profileId": id, "at": [1, 1]}), json!({"profileId": 99999, "at": [1, 1], "elevationM": 1}), json!({"at": [1, 1], "elevationM": 1})]
    {
        assert!(s.execute("trowel.levelLines", &bad).is_err(), "{bad}");
    }
    // A drawing with map coordinates reads its heights from its northings: no level lines there.
    run(&mut s, "doc.setOrigin", json!({"canvas": [0, 0], "world": [200000.0, 450000.0]}));
    let e = s.execute("trowel.levelLines", &json!({"profileId": ground["id"], "at": [150, 250], "elevationM": 32.0})).unwrap_err();
    assert!(e.to_string().contains("section drawing"), "{e}");
}

#[test]
fn spot_elevation_down_triangle_and_short_label() {
    let mut s = session();
    let r = run(
        &mut s,
        "trowel.spotElevation",
        json!({"at": [100, 100], "elevation": 32.5, "marker": "triangleDown", "prefixPlus": false, "format": "auto"}),
    );
    assert_eq!(r["text"], "32.5m");
    let g = node(&s, &r["id"]);
    // ▽ standing on its tip at the point.
    let tri = bounds(named(&g, "표고 기호").first().unwrap());
    assert!((tri.y1 - 100.0).abs() < 1e-9 && (tri.center().x - 100.0).abs() < 1e-9 && (tri.width() - 7.0).abs() < 1e-9, "{tri:?}");
    assert!(tri.y0 < 95.0, "{tri:?}");
    let r = run(
        &mut s,
        "trowel.spotElevation",
        json!({"at": [100, 100], "elevation": 32.45, "marker": "triangleDown", "prefixPlus": false, "format": "auto"}),
    );
    assert_eq!(r["text"], "32.45m");
    // Without `format`, two decimals as before.
    let r = run(&mut s, "trowel.spotElevation", json!({"at": [100, 100], "elevation": 32.5}));
    assert_eq!(r["text"], "+32.50m");
    assert!(s.execute("trowel.spotElevation", &json!({"at": [1, 1], "elevation": 1, "format": "long"})).is_err());
}

#[test]
fn strata_boundary_numbers_and_adds_a_description_line() {
    let mut s = session();
    // The pen: the legend row's weight.
    run(&mut s, "stroke.set", json!({"weight": 0.25}));
    let a = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 100], [150, 110], [250, 100]]}));
    assert_eq!((a["number"].as_u64(), a["badge"].as_str()), (Some(1), Some("Ⅰ")));
    let line = node(&s, &a["id"]);
    assert_eq!(line.name.as_deref(), Some("토층선 Ⅰ"));
    assert_eq!(line.appearance.stroke().map(|st| st.width), Some(0.25));
    assert!(line.appearance.fill_paint().is_none(), "never filled");
    // The number over the line's middle, upright.
    let number = node(&s, &a["numberId"]);
    assert_eq!(text_of(&number), "Ⅰ");
    let nb = bounds(&number);
    assert!((nb.center().x - 150.0).abs() < 4.0 && nb.y1 < 110.0, "{nb:?}");
    // The description block under the line with a line for it.
    let block = node(&s, &a["blockId"]);
    assert_eq!(block.name.as_deref(), Some("토층설명 7p"));
    let notes: Vec<String> = named(&block, "토층설명").into_iter().map(text_of).collect();
    assert_eq!(notes, ["Ⅰ층 : "]);
    assert!(bounds(&block).y0 > 110.0, "under the line");

    // The second goes under the first in the same block.
    let b = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 160], [250, 170]]}));
    assert_eq!(b["badge"], "Ⅱ");
    assert_eq!(b["blockId"], a["blockId"]);
    // Typed descriptions stay as they are.
    let first = NodeId(a["noteId"].as_u64().unwrap());
    s.edit("type", |d, _| {
        if let Some(NodeKind::Text(t)) = d.node_mut(first).map(|n| &mut n.kind)
            && let Some(r) = t.runs.first_mut()
        {
            r.text = "Ⅰ층 : 회황갈색 사질점토".to_string();
        }
        Ok(())
    })
    .unwrap();
    let before = node_count(&s);
    let c = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 220], [250, 230]]}));
    let block = node(&s, &c["blockId"]);
    let notes: Vec<String> = named(&block, "토층설명").into_iter().map(text_of).collect();
    assert_eq!(notes, ["Ⅰ층 : 회황갈색 사질점토", "Ⅱ층 : ", "Ⅲ층 : "]);
    let ys: Vec<f64> = named(&block, "토층설명").into_iter().map(|n| bounds(n).y1).collect();
    assert!(ys.windows(2).all(|w| w[1] > w[0] + 10.0), "one under the other: {ys:?}");
    let table = s.doc().unwrap().doc.stratigraphy_tables.first().cloned().unwrap();
    assert_eq!(table.numbering_style, vectorcraft_doc::stratigraphy::NumberingStyle::Roman);
    assert_eq!(table.layers.iter().map(|l| l.index).collect::<Vec<_>>(), [1, 2, 3]);
    // One undo step: the line, the number, the description line and the row.
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(node_count(&s), before);
    assert_eq!(s.doc().unwrap().doc.stratigraphy_tables.first().map(|t| t.layers.len()), Some(2));
    // A deleted block: the next boundary makes a new one.
    run(&mut s, "select.set", json!({"ids": [c["blockId"]]}));
    run(&mut s, "edit.clear", json!({}));
    let d = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 220], [250, 230]]}));
    assert_ne!(d["blockId"], c["blockId"]);
    assert_eq!(named(&node(&s, &d["blockId"]), "토층설명").into_iter().map(text_of).collect::<Vec<_>>(), ["Ⅲ층 : "]);
    for bad in [json!({"points": [[1, 1]]}), json!({"points": "x"}), json!({"points": [[1, 1], ["a", 2]]}), json!({})] {
        assert!(s.execute("stratigraphy.boundary", &bad).is_err(), "{bad}");
    }
}
