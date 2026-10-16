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

/// The marks the tools put in are not ground lines, and points off the canvas are refused with
/// nothing made (review J024 4 and 7).
#[test]
fn level_lines_refuse_marks_and_points_off_the_canvas() {
    let mut s = session();
    let ground = pit(&mut s);
    let r = run(&mut s, "trowel.levelLines", json!({"profileId": ground["id"], "at": [150, 250], "elevationM": 32.0}));
    let lines = node(&s, &r["id"]);
    let line = named(&lines, "주 해발선").first().map(|n| n.id.0).unwrap();
    let tick = node(&s, &r["datumId"]);
    let bar = named(&tick, "해발 기준 눈금").first().map(|n| n.id.0).unwrap();
    let before = node_count(&s);
    for id in [line, bar] {
        let e = s.execute("trowel.levelLines", &json!({"profileId": id, "at": [150, 250], "elevationM": 32.0})).unwrap_err();
        assert!(e.to_string().contains("not the ground line"), "{e}");
    }
    let huge = [1e308, 0.0];
    for (cmd, p) in [
        ("trowel.levelLines", json!({"profileId": ground["id"], "at": huge, "elevationM": 32.0})),
        ("trowel.insertSectionMarker", json!({"kind": "cut", "p0": huge, "p1": [-1e308, 0.0]})),
        ("trowel.insertSectionMarker", json!({"kind": "cut", "p0": [0, 0], "p1": [5e6, 0]})),
        ("stratigraphy.boundary", json!({"points": [[0, 0], [4.1e6, 10]]})),
    ] {
        assert!(s.execute(cmd, &p).is_err(), "{cmd} {p}");
    }
    assert_eq!(node_count(&s), before);
}

/// A soil layer boundary is drawn by hand and can be the ground line (review J024 N3); a grid
/// description table's lines are a mark (N7).
#[test]
fn level_lines_take_a_strata_line_but_not_a_table() {
    let mut s = session();
    let b = run(&mut s, "stratigraphy.boundary", json!({"points": [[0, 200], [100, 200], [200, 300], [300, 200], [400, 200]]}));
    assert_eq!(node(&s, &b["id"]).name.as_deref(), Some("토층선 Ⅰ"));
    let r = run(&mut s, "trowel.levelLines", json!({"profileId": b["id"], "at": [150, 250], "elevationM": 32.0}));
    assert!(!named(&node(&s, &r["id"]), "주 해발선").is_empty(), "{r}");
    let grid = run(&mut s, "stratigraphy.generateTable", json!({"format": "grid", "at": [50, 300]}));
    let table = node(&s, &grid["nodeId"]);
    let lines = named(&table, "표 격자").first().map(|n| n.id.0).unwrap();
    let e = s.execute("trowel.levelLines", &json!({"profileId": lines, "at": [50, 300], "elevationM": 32.0})).unwrap_err();
    assert!(e.to_string().contains("not the ground line"), "{e}");
}

/// Level lines in a legend row's layer keep their own weights (review J024 H9, the default until
/// it is decided): the row doesn't count them as set apart, and neither Revert to the Legend nor
/// Change Weights at Once changes them.
#[test]
fn level_lines_keep_their_weights_in_a_legend_row() {
    let mut s = session();
    let row = NodeId(run(&mut s, "layer.new", json!({"name": "단면선", "top": true}))["id"].as_u64().unwrap());
    let ground = pit(&mut s);
    run(&mut s, "stroke.set", json!({"weight": 0.8, "ids": [ground["id"]]}));
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8}));
    let r = run(&mut s, "trowel.levelLines", json!({"profileId": ground["id"], "at": [150, 250], "elevationM": 32.0}));
    let in_row = |s: &Session, id: &Value| s.doc().unwrap().doc.ancestry(NodeId(id.as_u64().unwrap())).unwrap_or_default().contains(&row);
    assert!(in_row(&s, &r["id"]), "the level lines are in the row's layer");
    let layer = s.doc().unwrap().doc.node(row).unwrap().clone();
    let counted: Vec<u64> = vectorcraft_doc::legend::lines(&layer).into_iter().map(|n| n.id.0).collect();
    assert_eq!(counted, vec![ground["id"].as_u64().unwrap()], "only the ground line is the row's");
    let widths = |s: &Session| {
        let g = node(s, &r["id"]);
        [named(&g, "주 해발선"), named(&g, "해발선")].concat().iter().map(|n| n.appearance.stroke().map(|st| st.width)).collect::<Vec<_>>()
    };
    let before = widths(&s);
    assert!(before.contains(&Some(0.5)) && before.contains(&Some(0.3)), "{before:?}");
    run(&mut s, "legend.revert", json!({"all": true}));
    run(&mut s, "legend.weights", json!({"how": "value", "from": 0.5, "to": 0.4, "overrides": true}));
    run(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8, "overrides": true}));
    assert_eq!(widths(&s), before);
}

/// A datum tick from the drawing set's element keeps its one decimal; level lines ask for the
/// height as typed (`format: 'auto'`, review J024 6).
#[test]
fn datum_ticks_keep_one_decimal_unless_asked() {
    let mut s = session();
    let one = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "datum", "at": [100, 100], "elevationM": 32.45}));
    let auto = run(&mut s, "trowel.insertSectionMarker", json!({"kind": "datum", "at": [100, 200], "elevationM": 32.45, "format": "auto"}));
    assert_eq!(node(&s, &one["id"]).name.as_deref(), Some("단면 해발 눈금 (32.5m)"));
    assert_eq!(node(&s, &auto["id"]).name.as_deref(), Some("단면 해발 눈금 (32.45m)"));
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
    // The label shows (review J024 UI 1): its white halo paints under the characters, so the
    // black of the figures is there at 4×.
    let label = named(&g, "표고 3").first().map(|n| bounds(n)).unwrap();
    let xf = vectorcraft_geom::Affine::scale(4.0) * vectorcraft_geom::Affine::translate((-60.0, -60.0));
    let img = vectorcraft_render::Renderer::new().render(&s.doc().unwrap().doc, 400, 400, xf, &Default::default());
    let on = xf.transform_rect_bbox(label);
    let dark = (on.y0.max(0.0) as u32..on.y1.min(400.0) as u32)
        .flat_map(|y| (on.x0.max(0.0) as u32..on.x1.min(400.0) as u32).map(move |x| (x, y)))
        .filter(|&(x, y)| img.pixel(x, y)[3] > 200 && img.pixel(x, y)[0] < 90)
        .count();
    assert!(dark > 40, "dark pixels in the label {on:?}: {dark}");
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

/// Past Ⅹ the numbers go on in letters; an inline table's block takes the next line in its own
/// type size under its last line; a grid table keeps its grid and the lines go in a 7 pt block
/// beside it (review J024 2 and 3).
#[test]
fn strata_boundaries_past_ten_and_with_a_table_already_there() {
    let mut s = session();
    let mut last = json!(null);
    for i in 0..11 {
        let y = 20 + i * 30;
        last = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, y], [250, y + 5]]}));
    }
    assert_eq!(last["badge"], "XI");
    assert_eq!(text_of(&node(&s, &last["noteId"])), "XI층 : ");

    // An inline table in 9 pt, made from the layers so far.
    let mut s = session();
    for y in [20, 50] {
        run(&mut s, "stratigraphy.boundary", json!({"points": [[50, y], [250, y + 5]]}));
    }
    run(&mut s, "stratigraphy.generateTable", json!({"format": "inline", "fontSizePt": 9, "at": [50, 300]}));
    let c = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 80], [250, 85]]}));
    let block = node(&s, &c["blockId"]);
    let ys: Vec<f64> = block
        .children()
        .unwrap()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Text(t) => Some(t.xf.translation().y),
            _ => None,
        })
        .collect();
    let step = (9.0 * 1.65f64).max(11.0);
    assert!(ys.len() == 3 && ys.windows(2).all(|w| (w[1] - w[0] - step).abs() < 1e-6), "9 pt lines {step} apart: {ys:?}");
    let note = node(&s, &c["noteId"]);
    let NodeKind::Text(t) = &note.kind else { panic!("{:?}", note.name) };
    assert_eq!(t.runs.first().map(|r| r.style.size), Some(9.0));
    // In 6 pt the table's lines are 11 pt apart (its least spacing): the line added keeps to it
    // (review J024 N6). In 100 pt, the size the table has.
    for (pt, step) in [(6.0, 11.0), (100.0, 165.0)] {
        let mut s = session();
        for y in [20, 50] {
            run(&mut s, "stratigraphy.boundary", json!({"points": [[50, y], [250, y + 5]]}));
        }
        run(&mut s, "stratigraphy.generateTable", json!({"format": "inline", "fontSizePt": pt, "at": [50, 300]}));
        let c = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 80], [250, 85]]}));
        let ys: Vec<f64> = node(&s, &c["blockId"])
            .children()
            .unwrap()
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Text(t) => Some(t.xf.translation().y),
                _ => None,
            })
            .collect();
        assert!(ys.len() == 3 && ys.windows(2).all(|w| (w[1] - w[0] - step).abs() < 1e-6), "{pt} pt lines {step} apart: {ys:?}");
        let NodeKind::Text(t) = &node(&s, &c["noteId"]).kind else { panic!("not text") };
        assert_eq!(t.runs.first().map(|r| r.style.size), Some(pt));
    }

    // A grid table: it keeps its placement, the lines go in a block of their own, found again.
    let mut s = session();
    run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 20], [250, 25]]}));
    let grid = run(&mut s, "stratigraphy.generateTable", json!({"format": "grid", "at": [50, 300]}));
    let placement = |s: &Session| s.doc().unwrap().doc.stratigraphy_tables.first().and_then(|t| t.placement.clone()).unwrap();
    let grid_node = placement(&s).node_id;
    let d = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 80], [250, 85]]}));
    let e = run(&mut s, "stratigraphy.boundary", json!({"points": [[50, 110], [250, 115]]}));
    assert_eq!(placement(&s).format, vectorcraft_doc::stratigraphy::TableRenderFormat::Grid, "{grid}");
    assert_eq!(placement(&s).node_id, grid_node);
    assert_ne!(Some(NodeId(d["blockId"].as_u64().unwrap())), grid_node);
    assert_eq!(d["blockId"], e["blockId"]);
    assert_eq!(node(&s, &d["blockId"]).name.as_deref(), Some("토층설명 7p"));
}
