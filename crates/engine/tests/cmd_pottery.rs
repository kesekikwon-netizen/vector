//! Integration tests for Task J011 (Artifact Pottery Drafting).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, Point};
use serde_json::json;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::pottery::MM;
use vectorcraft_doc::{Appearance, Node, NodeId};
use vectorcraft_engine::Session;
use vectorcraft_geom::PathData;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 2000, "height": 1000})).unwrap();
    s
}

#[test]
fn test_pottery_create_vessel_and_centerline() {
    let mut s = test_session();

    let res = s
        .execute(
            "pottery.createVessel",
            &json!({
                "name": "공열문 심발",
                "centerlineX": 300.0,
                "sectionSide": "right",
                "fillMode": "hatch",
                "targetScale": "1:3",
                "findNumber": "3",
                "feature": "1호 주거지",
            }),
        )
        .unwrap();

    let vessel_id = res["vesselId"].as_str().unwrap();
    let cl_id = NodeId(res["centerlineNodeId"].as_u64().unwrap());

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.pottery_vessels.len(), 1);
    let v = &doc.pottery_vessels[0];
    assert_eq!(v.id, vessel_id);
    assert_eq!(v.centerline_x, 300.0);
    assert!(doc.node(cl_id).is_some());
}

#[test]
fn test_pottery_rim_fit_3point() {
    let mut s = test_session();

    // 3 points on a 184 mm diameter arc (radius 92 mm = 92 * MM pt)
    let r_pt = 92.0 * MM;
    let ang1 = 20.0_f64.to_radians();
    let ang2 = 45.0_f64.to_radians();
    let ang3 = 70.0_f64.to_radians();

    let p1 = [300.0 + r_pt * ang1.cos(), 200.0 + r_pt * ang1.sin()];
    let p2 = [300.0 + r_pt * ang2.cos(), 200.0 + r_pt * ang2.sin()];
    let p3 = [300.0 + r_pt * ang3.cos(), 200.0 + r_pt * ang3.sin()];

    let res = s
        .execute(
            "pottery.rimFit",
            &json!({
                "p1": p1,
                "p2": p2,
                "p3": p3,
                "emitArc": true,
            }),
        )
        .unwrap();

    let d_mm = res["diameterMm"].as_f64().unwrap();
    assert!((d_mm - 184.0).abs() < 1e-4);
    let d_cm = res["diameterCm"].as_f64().unwrap();
    assert!((d_cm - 18.4).abs() < 1e-4);
    assert!(res["emittedArcId"].as_u64().is_some());
}

#[test]
fn test_pottery_reflect_exterior() {
    let mut s = test_session();

    // Create outer profile on right side of centerline (x = 300)
    let mut outer = BezPath::new();
    outer.move_to((380.0, 100.0));
    outer.line_to((400.0, 180.0));
    outer.line_to((340.0, 260.0));

    let pd = PathData::from_bezpath(&outer);
    let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 0.8);

    let mut outer_id = NodeId(0);
    s.edit("Add Outer Profile", |doc, _| {
        outer_id = doc.alloc_id();
        let node = Node::path(outer_id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    })
    .unwrap();

    // Execute reflection across centerline x = 300
    let res = s
        .execute(
            "pottery.reflectExterior",
            &json!({
                "pathId": outer_id.0,
                "centerlineX": 300.0,
            }),
        )
        .unwrap();

    let ref_id = NodeId(res["reflectedId"].as_u64().unwrap());
    let doc = &s.doc().unwrap().doc;
    let ref_node = doc.node(ref_id).unwrap();

    if let vectorcraft_doc::NodeKind::Path { path, .. } = &ref_node.kind {
        // Reflected points: (220, 260), (200, 180), (220, 100) on left side of 300
        let p_first = path.subpaths[0].anchors[0].p;
        assert_eq!(p_first, Point::new(260.0, 260.0));
    } else {
        panic!("reflected node is not a path");
    }
}

#[test]
fn test_pottery_wall_offset_and_hatch() {
    let mut s = test_session();

    // Create vessel
    let v_res = s
        .execute(
            "pottery.createVessel",
            &json!({
                "name": "심발",
                "centerlineX": 300.0,
            }),
        )
        .unwrap();
    let v_id = v_res["vesselId"].as_str().unwrap();

    // Create outer profile
    let mut outer = BezPath::new();
    outer.move_to((390.0, 100.0));
    outer.line_to((350.0, 250.0));
    let pd = PathData::from_bezpath(&outer);
    let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 0.8);

    let mut outer_id = NodeId(0);
    s.edit("Add Outer", |doc, _| {
        outer_id = doc.alloc_id();
        let node = Node::path(outer_id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        doc.pottery_vessels[0].outer_profile_node = Some(outer_id);
        Ok(())
    })
    .unwrap();

    // Inset wall thickness 7.0 mm
    let off_res = s
        .execute(
            "pottery.wallOffset",
            &json!({
                "pathId": outer_id.0,
                "thicknessMm": 7.0,
                "vesselId": v_id,
            }),
        )
        .unwrap();
    assert_eq!(off_res["thicknessMm"], 7.0);

    // Generate wall hatch
    let hatch_res = s
        .execute(
            "pottery.wallHatch",
            &json!({
                "vesselId": v_id,
                "fillMode": "hatch",
                "spacingMm": 1.0,
            }),
        )
        .unwrap();

    assert!(hatch_res["sectionPolygonId"].as_u64().is_some());
    assert!(hatch_res["hatchId"].as_u64().is_some());
}

#[test]
fn test_pottery_measurements_benchmark() {
    let mut s = test_session();

    let v_res = s
        .execute(
            "pottery.createVessel",
            &json!({
                "name": "공열문 심발",
                "centerlineX": 300.0,
            }),
        )
        .unwrap();
    let v_id = v_res["vesselId"].as_str().unwrap();

    // Outer profile benchmark:
    // Rim radius 92 mm (D = 184 mm = 18.4 cm)
    // Base radius 43 mm (d = 86 mm = 8.6 cm)
    // Height 212 mm (H = 21.2 cm)
    let rim_pt = 92.0 * MM;
    let base_pt = 43.0 * MM;
    let h_pt = 212.0 * MM;

    let mut outer = BezPath::new();
    outer.move_to((300.0 + rim_pt, 100.0));
    outer.line_to((300.0 + rim_pt * 1.05, 100.0 + h_pt * 0.5));
    outer.line_to((300.0 + base_pt, 100.0 + h_pt));

    let pd = PathData::from_bezpath(&outer);
    let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 0.8);

    s.edit("Add Benchmark Outer", |doc, _| {
        let id = doc.alloc_id();
        let node = Node::path(id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        doc.pottery_vessels[0].outer_profile_node = Some(id);
        Ok(())
    })
    .unwrap();

    let m_res = s
        .execute(
            "pottery.measurements",
            &json!({
                "vesselId": v_id,
                "emitAnnotations": true,
            }),
        )
        .unwrap();

    assert!((m_res["rimDiameterCm"].as_f64().unwrap() - 18.4).abs() < 0.1);
    assert!((m_res["heightCm"].as_f64().unwrap() - 21.2).abs() < 0.1);
    assert!((m_res["baseDiameterCm"].as_f64().unwrap() - 8.6).abs() < 0.1);

    let ledger = m_res["ledger"].as_array().unwrap();
    assert_eq!(ledger[0].as_str().unwrap(), "구경 18.4 cm");
    assert_eq!(ledger[1].as_str().unwrap(), "기고 21.2 cm");
    assert_eq!(ledger[2].as_str().unwrap(), "저경 8.6 cm");
}
