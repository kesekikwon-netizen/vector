//! End-to-end and integration tests for Task J006 (Drawing Scale, TM Real Coordinates & Paper-Fixed pt).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;

const GWANGNYEONGRI_CP1: (f64, f64, f64) = (152840.123, 512340.456, 152.45);
const GWANGNYEONGRI_CP2: (f64, f64, f64) = (152860.789, 512340.123, 152.60);
const GWANGNYEONGRI_CP3: (f64, f64, f64) = (152860.456, 512360.890, 152.80);
const GWANGNYEONGRI_CP4: (f64, f64, f64) = (152840.012, 512360.234, 152.55);

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600}));
    s
}

#[test]
fn test_j006_set_scale_and_get_scale() {
    let mut s = test_session();

    // Default before setting
    let get_before = s.execute("doc.getScale", &json!({})).unwrap();
    assert_eq!(get_before["is_set"], false);

    // Set scale to 1:20 with EPSG:5186 and eastnorth
    let set_res = s
        .execute(
            "doc.setScale",
            &json!({
                "scale": "1:20",
                "epsg": 5186,
                "order": "eastnorth"
            }),
        )
        .unwrap();
    assert_eq!(set_res["scale"], "1:20");
    assert_eq!(set_res["denom"], 20.0);
    assert_eq!(set_res["epsg"], 5186);
    assert_eq!(set_res["order"], "eastnorth");

    let get_after = s.execute("doc.getScale", &json!({})).unwrap();
    assert_eq!(get_after["is_set"], true);
    assert_eq!(get_after["scale"], "1:20");
    assert_eq!(get_after["denom"], 20.0);
    assert_eq!(get_after["epsg"], 5186);
    assert_eq!(get_after["order"], "eastnorth");
    let ppm = get_after["points_per_meter"].as_f64().unwrap();
    let expected_ppm = (1000.0 / 20.0) * (72.0 / 25.4);
    assert!((ppm - expected_ppm).abs() < 1e-6);
}

#[test]
fn test_j006_gwangnyeongri_control_points_transformation() {
    let mut s = test_session();

    // 1. Set scale 1:20
    s.execute("doc.setScale", &json!({"scale": 20.0})).unwrap();

    // 2. Bind CP1 to canvas (100.0, 100.0)
    let origin_res = s
        .execute(
            "doc.setOrigin",
            &json!({
                "canvas": [100.0, 100.0],
                "world": [GWANGNYEONGRI_CP1.0, GWANGNYEONGRI_CP1.1]
            }),
        )
        .unwrap();
    assert_eq!(origin_res["scale"], "1:20");

    // 3. Convert CP1 back to world
    let cp1_world = s.execute("doc.coordToWorld", &json!({"x": 100.0, "y": 100.0})).unwrap();
    assert!((cp1_world["east"].as_f64().unwrap() - GWANGNYEONGRI_CP1.0).abs() < 1e-6);
    assert!((cp1_world["north"].as_f64().unwrap() - GWANGNYEONGRI_CP1.1).abs() < 1e-6);
    assert_eq!(cp1_world["x_readout"], "152840.123");
    assert_eq!(cp1_world["y_readout"], "512340.456");

    // 4. Test all 4 Gwangnyeong-ri control points forward & backward
    let control_points = [GWANGNYEONGRI_CP1, GWANGNYEONGRI_CP2, GWANGNYEONGRI_CP3, GWANGNYEONGRI_CP4];

    for cp in control_points {
        let to_canvas = s
            .execute(
                "doc.coordToCanvas",
                &json!({
                    "east": cp.0,
                    "north": cp.1
                }),
            )
            .unwrap();
        let cx = to_canvas["canvas_x"].as_f64().unwrap();
        let cy = to_canvas["canvas_y"].as_f64().unwrap();

        let to_world = s
            .execute(
                "doc.coordToWorld",
                &json!({
                    "x": cx,
                    "y": cy
                }),
            )
            .unwrap();
        let recovered_e = to_world["east"].as_f64().unwrap();
        let recovered_n = to_world["north"].as_f64().unwrap();

        // Round-trip error must be < 0.001 m (1 mm)
        assert!((recovered_e - cp.0).abs() < 1e-5, "East mismatch for cp {:?}", cp);
        assert!((recovered_n - cp.1).abs() < 1e-5, "North mismatch for cp {:?}", cp);
    }
}

#[test]
fn test_j006_coordinate_order_and_orientation() {
    let mut s = test_session();

    s.execute("doc.setScale", &json!({"scale": 20.0, "order": "eastnorth"})).unwrap();
    s.execute(
        "doc.setOrigin",
        &json!({
            "canvas": [0.0, 0.0],
            "world": [1000.0, 2000.0]
        }),
    )
    .unwrap();

    // Moving right (+x) on canvas increases East (X)
    let right = s.execute("doc.coordToWorld", &json!({"x": 100.0, "y": 0.0})).unwrap();
    assert!(right["east"].as_f64().unwrap() > 1000.0);
    assert!((right["north"].as_f64().unwrap() - 2000.0).abs() < 1e-6);

    // Moving up (-y) on canvas increases North (Y)
    let up = s.execute("doc.coordToWorld", &json!({"x": 0.0, "y": -100.0})).unwrap();
    assert!((up["east"].as_f64().unwrap() - 1000.0).abs() < 1e-6);
    assert!(up["north"].as_f64().unwrap() > 2000.0);

    // Moving down (+y) on canvas decreases North (Y)
    let down = s.execute("doc.coordToWorld", &json!({"x": 0.0, "y": 100.0})).unwrap();
    assert!(down["north"].as_f64().unwrap() < 2000.0);
}

#[test]
fn test_j006_graticule_ticks_generation() {
    let mut s = test_session();
    s.execute("doc.setScale", &json!({"scale": 20.0})).unwrap();
    s.execute(
        "doc.setOrigin",
        &json!({
            "canvas": [100.0, 100.0],
            "world": [152840.0, 512340.0]
        }),
    )
    .unwrap();

    let res = s
        .execute(
            "doc.graticuleTicks",
            &json!({
                "interval_m": 5.0,
                "bounds": [0.0, 0.0, 800.0, 600.0]
            }),
        )
        .unwrap();

    let count = res["count"].as_u64().unwrap();
    assert!(count > 0);
    let ticks = res["ticks"].as_array().unwrap();
    assert!(ticks.iter().any(|t| t["axis"] == "X"));
    assert!(ticks.iter().any(|t| t["axis"] == "Y"));
    for tick in ticks {
        assert!(tick["pos"].is_array());
        assert!(tick["vector"].is_array());
        assert!(tick["label"].is_string());
    }
}

#[test]
fn test_j006_paper_fixed_pt_stroke_width_across_scales() {
    let mut s = test_session();

    // Draw a rectangle with standard 0.1 pt stroke (e.g. boundary or hatch line)
    let rect_res = s
        .execute(
            "shape.rectangle",
            &json!({
                "x": 50.0,
                "y": 50.0,
                "width": 100.0,
                "height": 100.0
            }),
        )
        .unwrap();
    let id = rect_res["id"].as_u64().unwrap();

    // Set stroke width to 0.1 pt
    s.execute(
        "stroke.set",
        &json!({
            "ids": [id],
            "weight": 0.1
        }),
    )
    .unwrap();

    // 1. Verify at scale 1:20
    s.execute("doc.setScale", &json!({"scale": 20.0})).unwrap();
    let doc1 = s.doc().unwrap();
    let node1 = doc1.doc.node(NodeId(id)).unwrap();
    assert!((node1.appearance.stroke_width() - 0.1).abs() < 1e-6, "Stroke width at 1:20 must be exactly 0.1 pt");

    // 2. Change scale to 1:50
    s.execute("doc.setScale", &json!({"scale": 50.0})).unwrap();
    let doc2 = s.doc().unwrap();
    let node2 = doc2.doc.node(NodeId(id)).unwrap();
    assert!((node2.appearance.stroke_width() - 0.1).abs() < 1e-6, "Stroke width at 1:50 must remain fixed at 0.1 pt");

    // 3. Change scale to 1:1
    s.execute("doc.setScale", &json!({"scale": 1.0})).unwrap();
    let doc3 = s.doc().unwrap();
    let node3 = doc3.doc.node(NodeId(id)).unwrap();
    assert!((node3.appearance.stroke_width() - 0.1).abs() < 1e-6, "Stroke width at 1:1 must remain fixed at 0.1 pt");
}
