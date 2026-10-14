//! Adversarial challenge test suite for J014 (Hatching Engine & Palette).
//! Stress tests topological graph construction, 2-coloring, odd-cycle conflict detection,
//! even cycles, disconnected regions, island holes, manual flip, and line emission across scales.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, PathEl, Point, Rect};
use serde_json::json;
use vectorcraft_doc::hatch::{HatchDirection, HatchStyle};
use vectorcraft_doc::legend::MM;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;
use vectorcraft_pathops::hatch::{
    bipartite_2color, build_adjacency_graph, emit_hatch_lines, solve_face_hatching,
};

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600}));
    s
}

/// Helper to extract segments from emitted hatch BezPath
fn extract_segments(path: &BezPath) -> Vec<(Point, Point)> {
    let mut segs = Vec::new();
    let mut curr = Point::ZERO;
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => curr = *p,
            PathEl::LineTo(p) => {
                segs.push((curr, *p));
                curr = *p;
            }
            PathEl::ClosePath => {}
            _ => {}
        }
    }
    segs
}

// =========================================================================
// 1. ODD CYCLES: 3 Mutually Touching Regions (K3) & C5
// =========================================================================

#[test]
fn test_stress_k3_odd_cycle_detection_and_badge() {
    // 3 non-overlapping triangular sectors meeting at a central point (50, 50).
    // Each pair shares a non-zero length radial boundary.
    let center = (50.0, 50.0);
    let mut f0 = BezPath::new();
    f0.move_to(center);
    f0.line_to((50.0, 0.0));
    f0.line_to((100.0, 50.0));
    f0.close_path();

    let mut f1 = BezPath::new();
    f1.move_to(center);
    f1.line_to((100.0, 50.0));
    f1.line_to((0.0, 100.0));
    f1.close_path();

    let mut f2 = BezPath::new();
    f2.move_to(center);
    f2.line_to((0.0, 100.0));
    f2.line_to((50.0, 0.0));
    f2.close_path();

    let faces = vec![f0.clone(), f1.clone(), f2.clone()];
    let styles = vec![HatchStyle::cut_face(), HatchStyle::cut_face(), HatchStyle::cut_face()];

    // 1. Verify build_adjacency_graph automatically detects K3 adjacencies
    let adj = build_adjacency_graph(&faces, 0.1);
    assert!(adj[0].contains(&1), "f0 must touch f1 along (50,50)-(100,50)");
    assert!(adj[1].contains(&0), "f1 must touch f0 along (50,50)-(100,50)");
    assert!(adj[1].contains(&2), "f1 must touch f2 along (50,50)-(0,100)");
    assert!(adj[2].contains(&1), "f2 must touch f1 along (50,50)-(0,100)");
    assert!(adj[2].contains(&0), "f2 must touch f0 along (50,50)-(50,0)");
    assert!(adj[0].contains(&2), "f0 must touch f2 along (50,50)-(50,0)");

    // 2. Verify bipartite_2color reports conflict
    let (_coloring, has_conflict) = bipartite_2color(&adj);
    assert!(has_conflict, "K3 odd-cycle MUST trigger has_conflict in bipartite_2color");

    // 3. Verify solve_face_hatching flags conflict on each face
    let results = solve_face_hatching(&faces, &styles, Some(&adj));
    for r in &results {
        assert!(r.conflict, "Face {} must have conflict flag set", r.face_index);
    }

    // 4. Verify through engine Session
    let mut s = test_session();
    let r0 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 50.0, "y": 0.0}, {"x": 100.0, "y": 50.0}],
        "closed": true
    })).unwrap();
    let id0 = r0["id"].as_u64().unwrap();

    let r1 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 100.0, "y": 50.0}, {"x": 0.0, "y": 100.0}],
        "closed": true
    })).unwrap();
    let id1 = r1["id"].as_u64().unwrap();

    let r2 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 0.0, "y": 100.0}, {"x": 50.0, "y": 0.0}],
        "closed": true
    })).unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    let set_res = s.execute("hatch.set", &json!({
        "direction": "alt",
        "ids": [id0, id1, id2]
    })).unwrap();

    let conflicts = set_res["conflicts"].as_array().unwrap();
    assert!(!conflicts.is_empty(), "Engine hatch.set must return conflict ids for K3");

    let get0 = s.execute("hatch.get", &json!({"id": id0})).unwrap();
    assert!(get0["has_conflict"].as_bool().unwrap(), "hatch.get must report has_conflict == true for odd cycle");
}

#[test]
fn test_stress_c5_odd_cycle_detection() {
    // 5 regions in a cycle: 0-1-2-3-4-0
    let adj = vec![
        vec![1, 4], // 0 touches 1, 4
        vec![0, 2], // 1 touches 0, 2
        vec![1, 3], // 2 touches 1, 3
        vec![2, 4], // 3 touches 2, 4
        vec![3, 0], // 4 touches 3, 0
    ];
    let (coloring, has_conflict) = bipartite_2color(&adj);
    assert_eq!(coloring.len(), 5);
    assert!(has_conflict, "C5 5-cycle is an odd cycle and MUST trigger has_conflict");
}

#[test]
fn test_stress_p3_line_no_conflict() {
    // 3 regions in a line (Path graph P3): 0 touches 1, 1 touches 2.
    // 0 and 2 do NOT touch. This is bipartite (2-colorable), NO conflict!
    let mut s = test_session();
    let r0 = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id0 = r0["id"].as_u64().unwrap();
    let r1 = s.execute("shape.rectangle", &json!({"x": 50.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id1 = r1["id"].as_u64().unwrap();
    let r2 = s.execute("shape.rectangle", &json!({"x": 100.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    let set_res = s.execute("hatch.set", &json!({
        "direction": "alt",
        "ids": [id0, id1, id2]
    })).unwrap();

    let conflicts = set_res["conflicts"].as_array().unwrap();
    assert!(conflicts.is_empty(), "P3 line has no odd cycles and must NOT have conflicts");

    let get0 = s.execute("hatch.get", &json!({"id": id0})).unwrap();
    assert!(!get0["has_conflict"].as_bool().unwrap());
}

// =========================================================================
// 2. EVEN CYCLES: 4 Adjacent Regions (C4)
// =========================================================================

#[test]
fn test_stress_even_cycle_c4_conflict_free_alternation() {
    // 4 squares in a 2x2 grid:
    // [0] [1]
    // [3] [2]
    // Adjacency: 0-1, 1-2, 2-3, 3-0.
    let mut s = test_session();
    let r0 = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id0 = r0["id"].as_u64().unwrap();

    let r1 = s.execute("shape.rectangle", &json!({"x": 50.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id1 = r1["id"].as_u64().unwrap();

    let r2 = s.execute("shape.rectangle", &json!({"x": 50.0, "y": 50.0, "width": 50.0, "height": 50.0})).unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    let r3 = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 50.0, "width": 50.0, "height": 50.0})).unwrap();
    let id3 = r3["id"].as_u64().unwrap();

    let doc = s.doc().unwrap();
    let b0 = doc.doc.node(NodeId(id0)).unwrap().path_data().unwrap().to_bezpath();
    let b1 = doc.doc.node(NodeId(id1)).unwrap().path_data().unwrap().to_bezpath();
    let b2 = doc.doc.node(NodeId(id2)).unwrap().path_data().unwrap().to_bezpath();
    let b3 = doc.doc.node(NodeId(id3)).unwrap().path_data().unwrap().to_bezpath();

    let adj = build_adjacency_graph(&[b0.clone(), b1.clone(), b2.clone(), b3.clone()], 0.1);
    assert!(adj[0].contains(&1) && adj[0].contains(&3), "0 must touch 1 and 3");
    assert!(adj[1].contains(&0) && adj[1].contains(&2), "1 must touch 0 and 2");
    assert!(adj[2].contains(&1) && adj[2].contains(&3), "2 must touch 1 and 3");
    assert!(adj[3].contains(&2) && adj[3].contains(&0), "3 must touch 2 and 0");

    let (coloring, has_conflict) = bipartite_2color(&adj);
    assert!(!has_conflict, "C4 even cycle MUST be conflict-free (bipartite)");
    assert_ne!(coloring[0], coloring[1], "0 and 1 must have opposite colors");
    assert_ne!(coloring[1], coloring[2], "1 and 2 must have opposite colors");
    assert_ne!(coloring[2], coloring[3], "2 and 3 must have opposite colors");
    assert_ne!(coloring[3], coloring[0], "3 and 0 must have opposite colors");
    assert_eq!(coloring[0], coloring[2], "0 and 2 must share same color");
    assert_eq!(coloring[1], coloring[3], "1 and 3 must share same color");

    let styles = vec![HatchStyle::cut_face(); 4];
    let results = solve_face_hatching(&[b0, b1, b2, b3], &styles, Some(&adj));
    assert_eq!(results.len(), 4);
    for r in &results {
        assert!(!r.conflict, "C4 faces must not have conflict");
    }
    assert_ne!(results[0].direction, results[1].direction);
    assert_ne!(results[1].direction, results[2].direction);
    assert_ne!(results[2].direction, results[3].direction);
    assert_ne!(results[3].direction, results[0].direction);
}

// =========================================================================
// 3. DISCONNECTED REGIONS & ISLAND HOLES
// =========================================================================

#[test]
fn test_stress_disconnected_regions() {
    let mut b0 = BezPath::new();
    b0.move_to((0.0, 0.0));
    b0.line_to((20.0, 0.0));
    b0.line_to((20.0, 20.0));
    b0.line_to((0.0, 20.0));
    b0.close_path();

    let mut b1 = BezPath::new();
    b1.move_to((200.0, 200.0));
    b1.line_to((220.0, 200.0));
    b1.line_to((220.0, 220.0));
    b1.line_to((200.0, 220.0));
    b1.close_path();

    let adj = build_adjacency_graph(&[b0.clone(), b1.clone()], 0.1);
    assert!(adj[0].is_empty(), "Disconnected faces must have no edges");
    assert!(adj[1].is_empty(), "Disconnected faces must have no edges");

    let styles = vec![HatchStyle::cut_face(), HatchStyle::cut_face()];
    let res = solve_face_hatching(&[b0, b1], &styles, Some(&adj));
    assert!(!res[0].conflict);
    assert!(!res[1].conflict);
    assert!(!res[0].lines.elements().is_empty());
    assert!(!res[1].lines.elements().is_empty());
}

#[test]
fn test_stress_island_holes_scanline_clipping() {
    // Donut shape: outer 0..100, hole 30..70
    let mut donut = BezPath::new();
    // Outer contour
    donut.move_to((0.0, 0.0));
    donut.line_to((100.0, 0.0));
    donut.line_to((100.0, 100.0));
    donut.line_to((0.0, 100.0));
    donut.close_path();
    // Inner hole contour
    donut.move_to((30.0, 30.0));
    donut.line_to((70.0, 30.0));
    donut.line_to((70.0, 70.0));
    donut.line_to((30.0, 70.0));
    donut.close_path();

    let lines = emit_hatch_lines(&donut, HatchDirection::Deg45, 2.8346);
    assert!(!lines.elements().is_empty(), "Must emit hatch lines for donut");

    let segs = extract_segments(&lines);
    assert!(!segs.is_empty());

    // Verify that NO line segment passes through the interior of the hole [31, 69] x [31, 69]
    let hole = Rect::new(31.0, 31.0, 69.0, 69.0);
    for (p1, p2) in &segs {
        // Sample points along the segment
        for step in 0..=10 {
            let t = step as f64 / 10.0;
            let pt = Point::new(p1.x + t * (p2.x - p1.x), p1.y + t * (p2.y - p1.y));
            assert!(
                !hole.contains(pt),
                "Hatch line segment ({:?} -> {:?}) penetrated interior of hole at {:?}",
                p1, p2, pt
            );
        }
    }
}

// =========================================================================
// 4. MANUAL FLIP TOGGLE
// =========================================================================

#[test]
fn test_stress_manual_flip_overrides_direction() {
    let mut s = test_session();
    let r = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 40.0, "height": 40.0})).unwrap();
    let id = r["id"].as_u64().unwrap();

    // 1. Set to 45°
    s.execute("hatch.set", &json!({"direction": "45", "ids": [id]})).unwrap();
    let get1 = s.execute("hatch.get", &json!({"id": id})).unwrap();
    assert_eq!(get1["direction"], "45");
    assert_eq!(get1["flipped"], false);

    // 2. Flip once -> 135°
    s.execute("hatch.flip", &json!({"ids": [id]})).unwrap();
    let get2 = s.execute("hatch.get", &json!({"id": id})).unwrap();
    assert_eq!(get2["flipped"], true);

    let doc = s.doc().unwrap();
    let node = doc.doc.node(NodeId(id)).unwrap();
    assert_eq!(node.hatch.as_ref().unwrap().effective_direction(), HatchDirection::Deg135);

    // 3. Flip again -> 45°
    s.execute("hatch.flip", &json!({"ids": [id]})).unwrap();
    let get3 = s.execute("hatch.get", &json!({"id": id})).unwrap();
    assert_eq!(get3["flipped"], false);
    let doc2 = s.doc().unwrap();
    let node2 = doc2.doc.node(NodeId(id)).unwrap();
    assert_eq!(node2.hatch.as_ref().unwrap().effective_direction(), HatchDirection::Deg45);
}

// =========================================================================
// 5. LINE EMISSION, SPACING ACCURACY & DOCUMENT SCALE
// =========================================================================

#[test]
fn test_stress_line_emission_spacing_accuracy() {
    // 1.0 mm paper spacing translates to 72 / 25.4 pt = 2.83464567 pt
    let style = HatchStyle::cut_face();
    let expected_spacing = 1.0 * MM;
    assert!((style.spacing_pt() - expected_spacing).abs() < 1e-9);

    let mut rect = BezPath::new();
    rect.move_to((0.0, 0.0));
    rect.line_to((100.0, 0.0));
    rect.line_to((100.0, 100.0));
    rect.line_to((0.0, 100.0));
    rect.close_path();

    let lines_45 = emit_hatch_lines(&rect, HatchDirection::Deg45, expected_spacing);
    let segs = extract_segments(&lines_45);
    assert!(!segs.is_empty());

    // Check perpendicular distance between consecutive parallel lines
    // For 45° line: unit normal is (-sin 45°, cos 45°) = (-1/sqrt(2), 1/sqrt(2))
    // Distance from origin is p.x * (-1/sqrt(2)) + p.y * (1/sqrt(2))
    let normal_x = -1.0 / std::f64::consts::SQRT_2;
    let normal_y = 1.0 / std::f64::consts::SQRT_2;

    let mut offsets: Vec<f64> = segs
        .iter()
        .map(|(p1, _)| p1.x * normal_x + p1.y * normal_y)
        .collect();
    offsets.sort_by(|a, b| a.total_cmp(b));

    // Remove duplicates (each line has 1 segment here)
    for i in 1..offsets.len() {
        let diff = offsets[i] - offsets[i - 1];
        if diff > 1e-4 {
            assert!(
                (diff - expected_spacing).abs() < 1e-3,
                "Perpendicular distance between hatch lines must be {expected_spacing:.4}, got {diff:.4}"
            );
        }
    }
}

#[test]
fn test_stress_svg_and_pdf_real_vector_line_output() {
    let mut s = test_session();
    let res = s.execute("shape.rectangle", &json!({"x": 10.0, "y": 10.0, "width": 50.0, "height": 50.0})).unwrap();
    let id = res["id"].as_u64().unwrap();

    s.execute("hatch.cutFace", &json!({"ids": [id]})).unwrap();

    let doc = s.doc().unwrap();

    // SVG export
    let svg = vectorcraft_svg::export(&doc.doc, &vectorcraft_svg::ExportOptions::default());
    assert!(svg.contains("stroke-width=\"0.1\""), "SVG must emit stroked vector lines at 0.1 pt");
    assert!(svg.contains("stroke=\"#000000\""), "SVG must emit black strokes");
    assert!(svg.contains("<path d=\""), "SVG must contain path elements for hatch lines");

    // PDF export
    let pdf = vectorcraft_pdf::export(&doc.doc, &vectorcraft_pdf::PdfOptions::default()).unwrap();
    assert!(pdf.starts_with(b"%PDF-"), "PDF must have valid PDF header");
    assert!(pdf.len() > 500, "PDF must not be empty");
}

#[test]
fn test_challenge_multi_selection_conflict_suppression() {
    // 3 faces forming K3: A, B, C
    let mut s = test_session();
    let r0 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 50.0, "y": 0.0}, {"x": 100.0, "y": 50.0}],
        "closed": true
    })).unwrap();
    let id0 = r0["id"].as_u64().unwrap();

    let r1 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 100.0, "y": 50.0}, {"x": 0.0, "y": 100.0}],
        "closed": true
    })).unwrap();
    let id1 = r1["id"].as_u64().unwrap();

    let r2 = s.execute("path.create", &json!({
        "anchors": [{"x": 50.0, "y": 50.0}, {"x": 0.0, "y": 100.0}, {"x": 50.0, "y": 0.0}],
        "closed": true
    })).unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    // 1. Set face A to Alternate
    s.execute("hatch.set", &json!({ "direction": "alt", "ids": [id0] })).unwrap();

    // 2. User selects face B and C together (length 2) and sets Alternate
    let res = s.execute("hatch.set", &json!({ "direction": "alt", "ids": [id1, id2] })).unwrap();
    let conflicts = res["conflicts"].as_array().unwrap();

    println!("Conflicts returned on setting [B, C]: {:?}", conflicts);

    // K3 cycle now exists between A, B, and C!
    // A query on face B should report conflict!
    let get_b = s.execute("hatch.get", &json!({"id": id1})).unwrap();
    assert!(
        get_b["has_conflict"].as_bool().unwrap(),
        "DEFECT: Setting hatch on [B, C] ignored already-hatched face A, failing to detect K3 conflict!"
    );
}

#[test]
fn test_challenge_document_level_alternation_in_svg() {
    // Two adjacent squares sharing an edge:
    // Box 1: (0, 0) to (50, 50)
    // Box 2: (50, 0) to (100, 50)
    let mut s = test_session();
    let r1 = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id1 = r1["id"].as_u64().unwrap();
    let r2 = s.execute("shape.rectangle", &json!({"x": 50.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    // Apply Cut Face (or Alternate hatch) to both
    s.execute("hatch.cutFace", &json!({"ids": [id1, id2]})).unwrap();

    let doc = s.doc().unwrap();
    let n1 = doc.doc.node(NodeId(id1)).unwrap();
    let n2 = doc.doc.node(NodeId(id2)).unwrap();

    // Check what effective_direction() returns on these nodes
    let dir1 = n1.hatch.as_ref().unwrap().effective_direction();
    let dir2 = n2.hatch.as_ref().unwrap().effective_direction();

    println!("Node 1 (x: 0..50) effective_direction: {:?}", dir1);
    println!("Node 2 (x: 50..100) effective_direction: {:?}", dir2);

    // In SVG, hatch lines are emitted as <path d="..." fill="none" stroke="#000000" stroke-width="0.1"/>
    // Because both nodes have dir == Deg45, every hatch line in Box 1 and Box 2 has identical 45° slope!
    assert_ne!(
        dir1, dir2,
        "CRITICAL DEFECT: Adjacent faces both have direction {:?}! Automatic 45°/135° alternation failed in document/render/export!",
        dir1
    );
}

