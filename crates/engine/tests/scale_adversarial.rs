//! Empirical Adversarial Stress Test Suite for Task J006 (Scale & Real Coordinates).
//! Tests high-precision coordinates, extreme values, round-trip residual,
//! paper-fixed stroke weights, and graticule interval math under stress.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::scale::{compute_graticule_ticks, ScaleConfig};
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;
use vectorcraft_geom::{Point, Rect};

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600}));
    s
}

/// 1. Extreme East/North values (TM EPSG:5186: X=200,000m, Y=550,000m; X=1,000,000m, Y=2,000,000m)
/// to stress-test floating-point cancellation and sub-millimeter precision.
#[test]
fn test_adversarial_extreme_coordinates_cancellation() {
    let mut s = test_session();

    // Set scale to 1:20
    s.execute("doc.setScale", &json!({"scale": "1:20", "epsg": 5186, "order": "eastnorth"})).unwrap();

    // Test with origin at (0, 0) in world, canvas origin at (400, 300)
    s.execute("doc.setOrigin", &json!({
        "canvas": [400.0, 300.0],
        "world": [0.0, 0.0]
    })).unwrap();

    let extreme_cases = [
        (200_000.0, 550_000.0),       // Standard Korean Central Belt TM
        (200_000.1234, 550_000.5678), // Sub-millimeter fractional parts
        (1_000_000.0, 2_000_000.0),   // Large continental scale
        (-200_000.0, -550_000.0),     // Negative coordinates
    ];

    for &(east, north) in &extreme_cases {
        let to_canvas = s.execute("doc.coordToCanvas", &json!({"east": east, "north": north})).unwrap();
        let cx = to_canvas["canvas_x"].as_f64().unwrap();
        let cy = to_canvas["canvas_y"].as_f64().unwrap();

        assert!(cx.is_finite(), "Canvas X must be finite");
        assert!(cy.is_finite(), "Canvas Y must be finite");

        let to_world = s.execute("doc.coordToWorld", &json!({"x": cx, "y": cy})).unwrap();
        let recovered_e = to_world["east"].as_f64().unwrap();
        let recovered_n = to_world["north"].as_f64().unwrap();

        let err_e = (recovered_e - east).abs();
        let err_n = (recovered_n - north).abs();

        // Sub-millimeter requirement: error must be strictly < 0.001 m (1 mm)
        // With IEEE 754 f64, we expect < 1e-6 m even at 1,000,000m
        assert!(
            err_e < 1e-6,
            "East residual for ({east}, {north}) exceeded 1 µm: {err_e} m"
        );
        assert!(
            err_n < 1e-6,
            "North residual for ({east}, {north}) exceeded 1 µm: {err_n} m"
        );
    }
}

/// 2. Millimeter sensitivity: verify that 1 millimeter in real-world produces
/// measurable and accurate canvas displacement without being rounded off.
#[test]
fn test_adversarial_millimeter_sensitivity() {
    let mut s = test_session();
    s.execute("doc.setScale", &json!({"scale": 20.0})).unwrap();
    s.execute("doc.setOrigin", &json!({
        "canvas": [100.0, 100.0],
        "world": [200_000.0, 550_000.0]
    })).unwrap();

    // 1 mm = 0.001 m. At 1:20 scale:
    // 0.001 m = 1 mm world -> 0.05 mm paper -> 0.05 * (72.0 / 25.4) pt ≈ 0.141732 pt
    let expected_dpt = 0.05 * (72.0 / 25.4);

    let pt0 = s.execute("doc.coordToCanvas", &json!({"east": 200_000.0, "north": 550_000.0})).unwrap();
    let pt1 = s.execute("doc.coordToCanvas", &json!({"east": 200_000.001, "north": 550_000.0})).unwrap();

    let dx = pt1["canvas_x"].as_f64().unwrap() - pt0["canvas_x"].as_f64().unwrap();
    assert!(
        (dx - expected_dpt).abs() < 1e-6,
        "1mm displacement on East must match expected canvas points: got {dx}, expected {expected_dpt}"
    );

    let pt2 = s.execute("doc.coordToCanvas", &json!({"east": 200_000.0, "north": 550_000.001})).unwrap();
    // North is UP, so canvas y decreases
    let dy = pt2["canvas_y"].as_f64().unwrap() - pt0["canvas_y"].as_f64().unwrap();
    let neg_expected_dpt = -expected_dpt;
    assert!(
        (dy - neg_expected_dpt).abs() < 1e-6,
        "1mm displacement on North must decrease canvas y by expected points: got {dy}, expected {neg_expected_dpt}"
    );
}

/// 3. Inverse round-trip precision sweep across 1,000 synthetic points and all scale presets:
/// canvas_to_world(world_to_canvas(pt)) == pt with sub-millimeter residual.
#[test]
fn test_adversarial_round_trip_sweep_all_scales() {
    let scales = [
        1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0,
    ];

    for &scale_denom in &scales {
        let mut cfg = ScaleConfig::new(scale_denom);
        cfg.set_origin(Point::new(500.0, 500.0), 200_000.0, 550_000.0);

        let mut max_world_residual = 0.0f64;
        let mut max_canvas_residual = 0.0f64;

        // Pseudorandom deterministic LCG sweep of 1,000 points
        let mut state: u64 = 0xDEADBEEF;
        let mut lcg = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 32) as f64 / 4294967296.0
        };

        for _ in 0..1000 {
            let east = 190_000.0 + lcg() * 20_000.0;
            let north = 540_000.0 + lcg() * 20_000.0;

            let canvas = cfg.world_to_canvas(east, north);
            let (rec_e, rec_n) = cfg.canvas_to_world(canvas);

            let res_e = (rec_e - east).abs();
            let res_n = (rec_n - north).abs();
            let world_res = res_e.max(res_n);
            if world_res > max_world_residual {
                max_world_residual = world_res;
            }

            // Canvas -> World -> Canvas
            let canvas_pt = Point::new(lcg() * 2000.0 - 1000.0, lcg() * 2000.0 - 1000.0);
            let (w_e, w_n) = cfg.canvas_to_world(canvas_pt);
            let rec_canvas = cfg.world_to_canvas(w_e, w_n);

            let canvas_res = (rec_canvas.x - canvas_pt.x).abs().max((rec_canvas.y - canvas_pt.y).abs());
            if canvas_res > max_canvas_residual {
                max_canvas_residual = canvas_res;
            }
        }

        assert!(
            max_world_residual < 1e-6,
            "Scale 1:{scale_denom} max world residual exceeded 1 µm: {max_world_residual} m"
        );
        assert!(
            max_canvas_residual < 1e-6,
            "Scale 1:{scale_denom} max canvas residual exceeded 1 µpt: {max_canvas_residual} pt"
        );
    }
}

/// 4. Paper-fixed stroke weights: verify a 0.8 pt line remains exactly 0.8 pt
/// regardless of drawing scale (1:20 vs 1:1 vs 1:50 vs 1:1000).
#[test]
fn test_adversarial_paper_fixed_stroke_weights() {
    let mut s = test_session();

    // Draw lines representing standard Trowel 4-tier weights (§14, §7):
    // 0.8 pt: cut border (잘린 면)
    // 0.5 pt: major elevation (50cm 해발선)
    // 0.2 pt: minor elevation (10cm 해발선)
    // 0.1 pt: hatch & boundary (빗금 및 세선)
    let weights = [0.8, 0.5, 0.2, 0.1];
    let mut node_ids = Vec::new();

    for (i, &w) in weights.iter().enumerate() {
        let res = s.execute("shape.rectangle", &json!({
            "x": 10.0 + (i as f64) * 60.0,
            "y": 10.0,
            "width": 50.0,
            "height": 50.0
        })).unwrap();
        let id = res["id"].as_u64().unwrap();
        s.execute("stroke.set", &json!({
            "ids": [id],
            "weight": w
        })).unwrap();
        node_ids.push((id, w));
    }

    // Cycle through multiple scales
    let scales_to_test = [20.0, 1.0, 50.0, 10.0, 100.0, 1000.0, 2.0];
    for &scale in &scales_to_test {
        s.execute("doc.setScale", &json!({"scale": scale})).unwrap();

        let doc = s.doc().unwrap();
        for &(id, expected_w) in &node_ids {
            let node = doc.doc.node(NodeId(id)).unwrap();
            let actual_w = node.appearance.stroke_width();
            assert_eq!(
                actual_w, expected_w,
                "Stroke weight for node {id} at scale 1:{scale} must remain exactly {expected_w} pt, got {actual_w}"
            );
        }
    }
}

/// 5. Graticule tick bounds and interval math stress testing:
/// - interval_m <= 0 returns empty Vec
/// - inverted bounds (x0 > x1 or y0 > y1)
/// - zero area bounds (x0 == x1)
/// - tick positions placed exactly on bounds edges
/// - tick values aligned to integer multiples of interval_m
#[test]
fn test_adversarial_graticule_ticks_math() {
    let mut cfg = ScaleConfig::new(20.0);
    cfg.set_origin(Point::new(0.0, 0.0), 200_000.0, 550_000.0);

    // Negative interval
    let ticks_neg = compute_graticule_ticks(Rect::new(0.0, 0.0, 800.0, 600.0), &cfg, -5.0);
    assert!(ticks_neg.is_empty(), "Negative interval must return empty");

    // Zero interval
    let ticks_zero = compute_graticule_ticks(Rect::new(0.0, 0.0, 800.0, 600.0), &cfg, 0.0);
    assert!(ticks_zero.is_empty(), "Zero interval must return empty");

    // Tiny interval <= 0.001
    let ticks_tiny = compute_graticule_ticks(Rect::new(0.0, 0.0, 800.0, 600.0), &cfg, 0.0005);
    assert!(ticks_tiny.is_empty(), "Interval <= 0.001 must return empty");

    // Standard interval = 5.0m
    let bounds = Rect::new(50.0, 50.0, 750.0, 550.0);
    let ticks = compute_graticule_ticks(bounds, &cfg, 5.0);
    assert!(!ticks.is_empty(), "Should generate ticks for valid bounds");

    for tick in &ticks {
        // Must be multiple of 5.0
        let rem = (tick.world_val % 5.0).abs();
        let rem_dist = rem.min(5.0 - rem);
        assert!(rem_dist < 1e-4, "Tick world_val {} must be multiple of 5.0m", tick.world_val);

        match tick.axis {
            "X" => {
                // Must be on top or bottom border
                let on_top = (tick.pos.y - bounds.y0).abs() < 1e-4;
                let on_bottom = (tick.pos.y - bounds.y1).abs() < 1e-4;
                assert!(on_top || on_bottom, "X tick pos.y must be on bounds border");
                assert!(
                    tick.pos.x >= bounds.x0 - 1e-4 && tick.pos.x <= bounds.x1 + 1e-4,
                    "X tick pos.x must be within bounds"
                );
            }
            "Y" => {
                // Must be on left or right border
                let on_left = (tick.pos.x - bounds.x0).abs() < 1e-4;
                let on_right = (tick.pos.x - bounds.x1).abs() < 1e-4;
                assert!(on_left || on_right, "Y tick pos.x must be on bounds border");
                assert!(
                    tick.pos.y >= bounds.y0 - 1e-4 && tick.pos.y <= bounds.y1 + 1e-4,
                    "Y tick pos.y must be within bounds"
                );
            }
            _ => panic!("Unknown tick axis: {}", tick.axis),
        }
    }

    // Inverted bounds (x0 > x1, y0 > y1)
    let inv_bounds = Rect::new(750.0, 550.0, 50.0, 50.0);
    let ticks_inv = compute_graticule_ticks(inv_bounds, &cfg, 5.0);
    // Should not panic or loop infinitely
    assert!(ticks_inv.len() <= ticks.len() + 4);

    // Zero area bounds
    let zero_bounds = Rect::new(100.0, 100.0, 100.0, 100.0);
    let ticks_zero_area = compute_graticule_ticks(zero_bounds, &cfg, 5.0);
    assert!(ticks_zero_area.len() <= 4);
}
