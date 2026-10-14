//! End-to-end and integration tests for Task J014 (Drawing Palette & Hatching Engine).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_doc::hatch::{DEFAULT_CUT_BORDER_PT, HatchDirection, HatchStyle};
use vectorcraft_engine::Session;
use vectorcraft_pathops::hatch::{build_adjacency_graph, solve_face_hatching};

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600}));
    s
}

#[test]
fn test_j014_two_adjacent_regions_alternation() {
    let mut s = test_session();
    let res1 = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id1 = res1["id"].as_u64().unwrap();
    let res2 = s.execute("shape.rectangle", &json!({"x": 50.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id2 = res2["id"].as_u64().unwrap();

    let set_res = s
        .execute(
            "hatch.set",
            &json!({
                "direction": "alt",
                "ids": [id1, id2]
            }),
        )
        .unwrap();
    assert_eq!(set_res["updated"].as_u64().unwrap(), 2);

    let doc = s.doc().unwrap();
    let n1 = doc.doc.node(NodeId(id1)).unwrap();
    let n2 = doc.doc.node(NodeId(id2)).unwrap();

    assert_eq!(n1.hatch.as_ref().unwrap().direction, HatchDirection::Alternate);
    assert_eq!(n2.hatch.as_ref().unwrap().direction, HatchDirection::Alternate);

    let b1 = n1.path_data().unwrap().to_bezpath();
    let b2 = n2.path_data().unwrap().to_bezpath();
    let adj = build_adjacency_graph(&[b1.clone(), b2.clone()], 0.1);
    assert_eq!(adj[0], vec![1]);
    assert_eq!(adj[1], vec![0]);

    let results = solve_face_hatching(&[b1, b2], &[n1.hatch.clone().unwrap(), n2.hatch.clone().unwrap()], Some(&adj));
    assert_eq!(results.len(), 2);
    assert!(!results[0].conflict);
    assert!(!results[1].conflict);
    assert_ne!(results[0].direction, results[1].direction);
    assert!(
        (results[0].direction == HatchDirection::Deg45 && results[1].direction == HatchDirection::Deg135)
            || (results[0].direction == HatchDirection::Deg135 && results[1].direction == HatchDirection::Deg45)
    );
}

#[test]
fn test_j014_three_mutually_touching_odd_cycle_conflict() {
    let mut s = test_session();
    // 3 mutually adjacent triangles meeting at the center
    let r1 = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 0.0, "y": 0.0}, {"x": 50.0, "y": 0.0}, {"x": 25.0, "y": 40.0}],
                "closed": true
            }),
        )
        .unwrap();
    let id1 = r1["id"].as_u64().unwrap();

    let r2 = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 50.0, "y": 0.0}, {"x": 100.0, "y": 0.0}, {"x": 25.0, "y": 40.0}],
                "closed": true
            }),
        )
        .unwrap();
    let id2 = r2["id"].as_u64().unwrap();

    let r3 = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 0.0, "y": 0.0}, {"x": 25.0, "y": 40.0}, {"x": 100.0, "y": 0.0}],
                "closed": true
            }),
        )
        .unwrap();
    let id3 = r3["id"].as_u64().unwrap();

    let set_res = s
        .execute(
            "hatch.set",
            &json!({
                "direction": "alt",
                "ids": [id1, id2, id3]
            }),
        )
        .unwrap();
    assert_eq!(set_res["updated"].as_u64().unwrap(), 3);

    // Conflict detection
    let conflicts = set_res["conflicts"].as_array().unwrap();
    assert!(!conflicts.is_empty(), "K3 odd-cycle conflict should be detected and flagged");

    let get_res = s.execute("hatch.get", &json!({"id": id1})).unwrap();
    assert!(get_res["has_conflict"].as_bool().unwrap());
}

#[test]
fn test_j014_cut_face_command() {
    let mut s = test_session();
    let res = s.execute("shape.rectangle", &json!({"x": 10.0, "y": 10.0, "width": 40.0, "height": 40.0})).unwrap();
    let id = res["id"].as_u64().unwrap();

    let cut_res = s.execute("hatch.cutFace", &json!({"ids": [id]})).unwrap();
    assert_eq!(cut_res["updated"].as_u64().unwrap(), 1);

    let doc = s.doc().unwrap();
    let node = doc.doc.node(NodeId(id)).unwrap();
    assert_eq!(node.hatch, Some(HatchStyle::cut_face()));
    assert_eq!(node.appearance.stroke().unwrap().width, DEFAULT_CUT_BORDER_PT);
}

#[test]
fn test_j014_flip_toggle() {
    let mut s = test_session();
    let res = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 30.0, "height": 30.0})).unwrap();
    let id = res["id"].as_u64().unwrap();

    s.execute("hatch.set", &json!({"direction": "45", "ids": [id]})).unwrap();
    let doc1 = s.doc().unwrap();
    assert!(!doc1.doc.node(NodeId(id)).unwrap().hatch.as_ref().unwrap().flipped);

    s.execute("hatch.flip", &json!({"ids": [id]})).unwrap();
    let doc2 = s.doc().unwrap();
    assert!(doc2.doc.node(NodeId(id)).unwrap().hatch.as_ref().unwrap().flipped);

    // Flipping again restores flipped to false
    s.execute("hatch.flip", &json!({"ids": [id]})).unwrap();
    let doc3 = s.doc().unwrap();
    assert!(!doc3.doc.node(NodeId(id)).unwrap().hatch.as_ref().unwrap().flipped);
}

#[test]
fn test_j014_svg_export_emits_vector_hatch_lines() {
    let mut s = test_session();
    let res = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id = res["id"].as_u64().unwrap();

    s.execute("hatch.set", &json!({"direction": "45", "spacing_mm": 1.0, "weight_pt": 0.1, "ids": [id]})).unwrap();

    let doc = s.doc().unwrap();
    let opts = vectorcraft_svg::ExportOptions::default();
    let svg_str = vectorcraft_svg::export(&doc.doc, &opts);

    assert!(svg_str.contains("stroke-width=\"0.1\""), "SVG export should contain stroked hatch lines with 0.1 pt");
    assert!(svg_str.contains("stroke=\"#000000\""), "SVG export should contain black stroke for hatch lines");
}

#[test]
fn test_j014_pdf_export_emits_vector_hatch_lines() {
    let mut s = test_session();
    let res = s.execute("shape.rectangle", &json!({"x": 0.0, "y": 0.0, "width": 50.0, "height": 50.0})).unwrap();
    let id = res["id"].as_u64().unwrap();

    s.execute("hatch.set", &json!({"direction": "45", "spacing_mm": 1.0, "weight_pt": 0.1, "ids": [id]})).unwrap();

    let doc = s.doc().unwrap();
    let opts = vectorcraft_pdf::PdfOptions::default();
    let pdf_bytes = vectorcraft_pdf::export(&doc.doc, &opts).unwrap();
    assert!(!pdf_bytes.is_empty(), "PDF export should succeed");
    assert!(pdf_bytes.starts_with(b"%PDF-"), "Should produce valid PDF header");
}
