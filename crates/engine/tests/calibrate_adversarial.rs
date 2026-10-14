//! Adversarial Challenge & Stress-Test Suite for Task J008 (Underlay Calibration & Registration).
//!
//! Evaluates:
//! 1. 2-Point fit with coincident, near-coincident, degenerate, and invalid points.
//! 2. N-Point similarity fit across N = 2, 3, 10, 100 control points for scale, rotation, translation accuracy.
//! 3. N-Point affine fit with collinear control points (error reporting vs fit_auto fallback).
//! 4. Sheared, anisotropic, and mirrored/reflection configurations.
//! 5. Large TM real-world coordinates (Easting 200,000m, Northing 500,000m) numerical stability and residual calculations.
//! 6. Engine commands (`calibrate.apply`, `calibrate.twoPoint`, `calibrate.controlPoints`, `calibrate.residuals`)
//!    under missing layers, empty documents, multiple images, and malformed inputs.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{Affine, Point};
use serde_json::json;
use vectorcraft_doc::node::{ImageObject, LayerColor};
use vectorcraft_doc::{Node, NodeId, NodeKind};
use vectorcraft_engine::Session;
use vectorcraft_geom::registration::{
    AxisAlignment, MM, PointPair, RegistrationError, evaluate_residuals, fit_affine, fit_auto, fit_similarity, fit_two_point,
};

fn test_session_empty() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    s
}

fn test_session_with_underlay() -> (Session, NodeId) {
    let mut s = test_session_empty();
    let img_id = s
        .edit("Add Underlay", |doc, _| {
            let layer_id = doc.alloc_id();
            let mut layer = Node::layer(layer_id, "밑그림", LayerColor::Preset(0));
            if let NodeKind::Layer { template, .. } = &mut layer.kind {
                *template = true;
            }
            layer.locked = true;
            let img_id = doc.alloc_id();
            let img = Node::new(
                img_id,
                NodeKind::Image(ImageObject {
                    key: "test_underlay_img".to_string(),
                    width: 1000,
                    height: 1000,
                    xf: Affine::IDENTITY,
                    link: None,
                    placement: Default::default(),
                }),
            );
            doc.insert(None, usize::MAX, layer).unwrap();
            doc.insert(Some(layer_id), usize::MAX, img).unwrap();
            Ok(img_id)
        })
        .unwrap();
    (s, img_id)
}

// =========================================================================
// PART 1: ALGORITHMIC REGISTRATION STRESS TESTS (`vectorcraft_geom`)
// =========================================================================

/// 1.1 Stress test: 2-point fit with coincident points, near-coincident points,
/// non-finite coordinates, and degenerate target distances.
#[test]
fn test_adversarial_two_point_coincident_and_threshold() {
    // Exact coincident source points
    let p_same = Point::new(150.0, 250.0);
    let err_exact = fit_two_point(p_same, p_same, Some(100.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_exact, RegistrationError::DegeneratePoints);

    // Near-coincident source points (distance < 1e-9)
    let p_tiny = Point::new(150.0 + 1e-10, 250.0);
    let err_tiny = fit_two_point(p_same, p_tiny, Some(100.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_tiny, RegistrationError::DegeneratePoints);

    // Distance in range [1e-9, 1e-6)
    // Note: If threshold is 1e-9, distance 1e-7 succeeds numerically without division by zero.
    let p_micro = Point::new(150.0 + 1e-7, 250.0);
    let res_micro = fit_two_point(p_same, p_micro, Some(100.0), None, None, AxisAlignment::Horizontal);
    assert!(res_micro.is_ok());
    let micro_scale = res_micro.unwrap().scale;
    assert!(micro_scale > 1e8); // Extreme scale, but finite and no panic

    // Coincident target points (q1 == q2)
    let q_same = Point::new(50.0, 50.0);
    let err_tgt_same =
        fit_two_point(Point::new(0.0, 0.0), Point::new(100.0, 0.0), None, Some(q_same), Some(q_same), AxisAlignment::None).unwrap_err();
    assert_eq!(err_tgt_same, RegistrationError::DegeneratePoints);

    // Non-finite source coordinates (NaN, Infinity)
    let err_nan = fit_two_point(Point::new(f64::NAN, 0.0), Point::new(100.0, 0.0), Some(100.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_nan, RegistrationError::InvalidCoordinates);

    let err_inf =
        fit_two_point(Point::new(0.0, 0.0), Point::new(100.0, f64::INFINITY), Some(100.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_inf, RegistrationError::InvalidCoordinates);

    // Non-positive or non-finite target distance
    let err_neg_dist = fit_two_point(Point::new(0.0, 0.0), Point::new(100.0, 0.0), Some(-50.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_neg_dist, RegistrationError::InvalidCoordinates);

    let err_zero_dist = fit_two_point(Point::new(0.0, 0.0), Point::new(100.0, 0.0), Some(0.0), None, None, AxisAlignment::Horizontal).unwrap_err();
    assert_eq!(err_zero_dist, RegistrationError::InvalidCoordinates);

    // Missing target distance when target points are absent
    let err_no_target = fit_two_point(Point::new(0.0, 0.0), Point::new(100.0, 0.0), None, None, None, AxisAlignment::Horizontal).unwrap_err();
    assert!(matches!(err_no_target, RegistrationError::InsufficientPoints { .. }));

    // Axis alignment options
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(100.0, 100.0); // angle = 45 deg
    let res_h = fit_two_point(p1, p2, Some(141.421356), None, None, AxisAlignment::Horizontal).unwrap();
    assert!((res_h.rotation_deg - (-45.0)).abs() < 1e-4);

    let res_v = fit_two_point(p1, p2, Some(141.421356), None, None, AxisAlignment::Vertical).unwrap();
    assert!((res_v.rotation_deg - (-135.0)).abs() < 1e-4);

    let res_n = fit_two_point(p1, p2, Some(141.421356), None, None, AxisAlignment::None).unwrap();
    assert!(res_n.rotation_deg.abs() < 1e-9);
}

/// 1.2 Stress test: N-point similarity fit with N = 2, 3, 10, 100 control points.
/// Verifies uniform scale, rotation, translation accuracy, and isotropic preservation.
#[test]
fn test_adversarial_similarity_n_points_scaling_rotation_translation() {
    // Ground truth parameters:
    // Scale s = 3.25, Rotation = -35.0 degrees, Translation = (876.54, -432.10)
    let s_true = 3.25;
    let theta_deg_true: f64 = -35.0;
    let theta_rad_true = theta_deg_true.to_radians();
    let tx_true = 876.54;
    let ty_true = -432.10;

    let cos_t = theta_rad_true.cos();
    let sin_t = theta_rad_true.sin();
    let a = s_true * cos_t;
    let b = s_true * sin_t;
    let ground_truth = Affine::new([a, b, -b, a, tx_true, ty_true]);

    // Test with varying point counts: N = 2, 3, 10, 100
    let test_point_counts = [2, 3, 10, 100];

    for &n in &test_point_counts {
        let mut pairs = Vec::with_capacity(n);
        for i in 0..n {
            // Generate non-collinear test points
            let u = (i as f64) * 17.3 + (i % 3) as f64 * 45.0;
            let v = (i as f64) * 23.7 - (i % 5) as f64 * 31.0;
            let src = Point::new(u, v);
            let tgt = ground_truth * src;
            pairs.push(PointPair::new(src, tgt));
        }

        let res = fit_similarity(&pairs, Some(100.0)).expect(&format!("fit_similarity N={n}"));

        // Uniform scale check (both X and Y scale must be identical)
        assert!((res.scale.0 - res.scale.1).abs() < 1e-9, "Similarity must be strictly isotropic");
        assert!((res.scale.0 - s_true).abs() < 1e-9, "N={n}: Scale mismatch");

        // Rotation check
        let rot_diff = (res.rotation_deg - theta_deg_true).abs();
        assert!(rot_diff < 1e-9, "N={n}: Rotation mismatch: {rot_diff}");

        // Translation check
        assert!((res.translation.x - tx_true).abs() < 1e-9, "N={n}: tx mismatch");
        assert!((res.translation.y - ty_true).abs() < 1e-9, "N={n}: ty mismatch");

        // Residual check: every point residual and total RMSE must be near zero
        assert!(res.rmse_pt < 1e-9, "N={n}: RMSE must be < 1e-9");
        assert!(res.max_error_pt < 1e-9, "N={n}: Max error must be < 1e-9");
        for r in &res.residuals {
            assert!(r.dist_pt < 1e-9, "Point residual must be < 1e-9");
        }
    }

    // Boundary condition: N = 1 fails with InsufficientPoints
    let single_pair = vec![PointPair::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0))];
    let err_single = fit_similarity(&single_pair, None).unwrap_err();
    assert_eq!(err_single, RegistrationError::InsufficientPoints { required: 2, actual: 1 });

    // Degenerate condition: All points in pairs are identical
    let degen_pairs = vec![
        PointPair::new(Point::new(50.0, 50.0), Point::new(100.0, 100.0)),
        PointPair::new(Point::new(50.0, 50.0), Point::new(100.0, 100.0)),
        PointPair::new(Point::new(50.0, 50.0), Point::new(100.0, 100.0)),
    ];
    let err_degen = fit_similarity(&degen_pairs, None).unwrap_err();
    assert_eq!(err_degen, RegistrationError::DegeneratePoints);
}

/// 1.3 Stress test: N-point affine fit with collinear control points.
/// Verifies returns Err(CollinearPoints) without panic in fit_affine,
/// and verifies graceful similarity fallback in fit_auto.
#[test]
fn test_adversarial_affine_collinear_control_points() {
    // 1. 3 Collinear points along y = 3x - 7
    let pts_3_collinear = [Point::new(10.0, 23.0), Point::new(30.0, 83.0), Point::new(70.0, 203.0)];
    let ground_truth_sim = Affine::new([2.0, 0.0, 0.0, 2.0, 10.0, 20.0]);
    let pairs_3: Vec<PointPair> = pts_3_collinear.iter().map(|&p| PointPair::new(p, ground_truth_sim * p)).collect();

    let err_3 = fit_affine(&pairs_3, None).unwrap_err();
    assert_eq!(err_3, RegistrationError::CollinearPoints);

    // fit_auto must fall back gracefully to similarity
    let auto_res_3 = fit_auto(&pairs_3, None).expect("fit_auto on 3 collinear");
    assert!(auto_res_3.rmse_pt < 1e-9);
    assert!((auto_res_3.scale.0 - 2.0).abs() < 1e-9);

    // 2. 10 Collinear points along horizontal line y = 150.0
    let pairs_10: Vec<PointPair> = (0..10)
        .map(|i| {
            let p = Point::new(i as f64 * 50.0, 150.0);
            PointPair::new(p, ground_truth_sim * p)
        })
        .collect();

    let err_10 = fit_affine(&pairs_10, None).unwrap_err();
    assert_eq!(err_10, RegistrationError::CollinearPoints);

    let auto_res_10 = fit_auto(&pairs_10, None).expect("fit_auto on 10 collinear");
    assert!(auto_res_10.rmse_pt < 1e-9);

    // 3. Insufficient points for affine (N = 2)
    let pairs_2 = &pairs_3[..2];
    let err_insuff = fit_affine(pairs_2, None).unwrap_err();
    assert_eq!(err_insuff, RegistrationError::InsufficientPoints { required: 3, actual: 2 });
}

/// 1.4 Stress test: Sheared, anisotropic, and mirrored configurations.
/// Verifies affine parameters absorb shear, aspect ratio skew, and reflection.
#[test]
fn test_adversarial_affine_shear_anisotropy_reflection() {
    let base_pts = [Point::new(50.0, 50.0), Point::new(250.0, 70.0), Point::new(240.0, 230.0), Point::new(60.0, 210.0), Point::new(150.0, 140.0)];

    // Case A: Strong Anisotropic Scaling (sx = 2.40, sy = 0.65)
    let xf_aniso = Affine::new([2.40, 0.0, 0.0, 0.65, 100.0, -50.0]);
    let pairs_aniso: Vec<PointPair> = base_pts.iter().map(|&p| PointPair::new(p, xf_aniso * p)).collect();

    let res_aniso = fit_affine(&pairs_aniso, None).expect("affine aniso fit");
    assert!(res_aniso.rmse_pt < 1e-9);
    assert!((res_aniso.scale.0 - 2.40).abs() < 1e-9);
    assert!((res_aniso.scale.1 - 0.65).abs() < 1e-9);

    // Case B: General Shear / Skew
    // [a11, a21, a12, a22, tx, ty]
    let xf_shear = Affine::new([1.10, 0.35, -0.25, 0.90, 45.0, 80.0]);
    let pairs_shear: Vec<PointPair> = base_pts.iter().map(|&p| PointPair::new(p, xf_shear * p)).collect();

    let res_shear = fit_affine(&pairs_shear, None).expect("affine shear fit");
    assert!(res_shear.rmse_pt < 1e-9);
    for i in 0..6 {
        assert!((res_shear.affine_coeffs[i] - xf_shear.as_coeffs()[i]).abs() < 1e-9, "Coeff {i} mismatch in shear");
    }

    // Case C: Mirrored / Reflection (det < 0, reflection across X-axis)
    // [1.0, 0.0, 0.0, -1.0, 0.0, 500.0]
    let xf_mirror = Affine::new([1.2, 0.0, 0.0, -1.2, 50.0, 500.0]);
    let pairs_mirror: Vec<PointPair> = base_pts.iter().map(|&p| PointPair::new(p, xf_mirror * p)).collect();

    let res_mirror = fit_affine(&pairs_mirror, None).expect("affine mirror fit");
    assert!(res_mirror.rmse_pt < 1e-9);
    for i in 0..6 {
        assert!((res_mirror.affine_coeffs[i] - xf_mirror.as_coeffs()[i]).abs() < 1e-9, "Coeff {i} mismatch in reflection");
    }

    // Confirm that Helmert similarity CANNOT absorb reflection (orientation-preserving only)
    let res_sim_mirror = fit_similarity(&pairs_mirror, None).expect("similarity fit");
    assert!(res_sim_mirror.rmse_pt > 10.0, "Helmert similarity must fail to fit mirrored data");
}

/// 1.5 Stress test: Large TM coordinates (Easting 200,000m, Northing 500,000m).
/// Verifies numerical stability without catastrophic cancellation, and exact residual conversions.
#[test]
fn test_adversarial_large_tm_coordinates_numerical_stability() {
    // 4 Control points matching archaeological survey boundaries in Korean TM Central Belt (EPSG:5186)
    // Target coordinates in meters
    let tm_targets = [
        Point::new(200_120.500, 500_340.250),
        Point::new(200_450.750, 500_340.800),
        Point::new(200_450.300, 500_680.900),
        Point::new(200_120.100, 500_680.400),
    ];

    // Source points in scanned underlay image pixel space [0..2000, 0..2000]
    let img_sources = [Point::new(120.0, 150.0), Point::new(1780.0, 155.0), Point::new(1775.0, 1850.0), Point::new(125.0, 1845.0)];

    let pairs: Vec<PointPair> = img_sources.iter().zip(&tm_targets).map(|(&s, &t)| PointPair::new(s, t)).collect();

    // Scale: 1:20 scale has points_per_meter = 1000/20 * 72 / 25.4 = 141.73228346...
    let ppm = 50.0 * MM;

    // Test both similarity and affine
    let res_sim = fit_similarity(&pairs, Some(ppm)).expect("similarity on large TM");
    assert!(res_sim.rmse_pt.is_finite());
    assert!(res_sim.rmse_m.is_some());
    assert!(res_sim.rmse_m.unwrap().is_finite());

    let res_aff = fit_affine(&pairs, Some(ppm)).expect("affine on large TM");
    assert!(res_aff.rmse_pt.is_finite());
    assert!(res_aff.rmse_m.is_some());

    // Verify residual unit relationships:
    // dist_mm == dist_pt / MM
    // dist_m == dist_pt / ppm
    for r in &res_aff.residuals {
        assert!((r.dist_mm - (r.dist_pt / MM)).abs() < 1e-9);
        if let Some(dm) = r.dist_m {
            assert!((dm - (r.dist_pt / ppm)).abs() < 1e-9);
        }
    }

    // Verify RMSE conversions
    let expected_rmse_m = res_aff.rmse_pt / ppm;
    assert!((res_aff.rmse_m.unwrap() - expected_rmse_m).abs() < 1e-9);
    let expected_rmse_mm = res_aff.rmse_pt / MM;
    assert!((res_aff.rmse_mm - expected_rmse_mm).abs() < 1e-9);

    // Verify evaluate_residuals produces identical metrics
    let eval_res = evaluate_residuals(res_aff.transform, &pairs, Some(ppm)).expect("evaluate_residuals on large TM");
    assert!((eval_res.rmse_pt - res_aff.rmse_pt).abs() < 1e-9);
    assert!((eval_res.rmse_mm - res_aff.rmse_mm).abs() < 1e-9);
}

// =========================================================================
// PART 2: ENGINE COMMANDS ADVERSARIAL STRESS TESTS (`vectorcraft_engine`)
// =========================================================================

/// 2.1 Stress test: `calibrate.apply` with missing underlay layer, empty document,
/// multiple images, and malformed inputs.
#[test]
fn test_adversarial_calibrate_apply_missing_underlay_and_multiple_images() {
    let mut s_empty = test_session_empty();

    // 1. Empty document: calibrate.apply must fail gracefully without panic
    let res_empty = s_empty.execute(
        "calibrate.apply",
        &json!({
            "affineCoeffs": [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            "mode": "replace",
        }),
    );
    assert!(res_empty.is_err());
    let err_msg = res_empty.unwrap_err().to_string();
    assert!(err_msg.contains("no underlay image node found"), "Unexpected error: {err_msg}");

    // 2. Document with layers and shapes, but NO image node: fails gracefully
    s_empty.execute("shape.rectangle", &json!({"x": 10.0, "y": 10.0, "width": 100.0, "height": 50.0})).unwrap();
    let res_no_img = s_empty.execute(
        "calibrate.apply",
        &json!({
            "affineCoeffs": [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            "mode": "replace",
        }),
    );
    assert!(res_no_img.is_err());

    // 3. Document with multiple images across multiple layers:
    // Verify default targets "밑그림" template layer, while explicit imageId targets specific image.
    let (mut s, underlay_img_id) = test_session_with_underlay();

    // Add a second image on a normal layer
    let second_img_id = s
        .edit("Add Second Image", |doc, _| {
            let layer_id = doc.alloc_id();
            let layer = Node::layer(layer_id, "작업 레이어", LayerColor::Preset(1));
            let img_id = doc.alloc_id();
            let img = Node::new(
                img_id,
                NodeKind::Image(ImageObject {
                    key: "second_img".to_string(),
                    width: 500,
                    height: 500,
                    xf: Affine::IDENTITY,
                    link: None,
                    placement: Default::default(),
                }),
            );
            doc.insert(None, usize::MAX, layer).unwrap();
            doc.insert(Some(layer_id), usize::MAX, img).unwrap();
            Ok(img_id)
        })
        .unwrap();

    // Default calibrate.apply without imageId -> must update underlay_img_id ("밑그림")
    let target_xf_1 = [2.0, 0.0, 0.0, 2.0, 100.0, 50.0];
    let apply_res_1 = s
        .execute(
            "calibrate.apply",
            &json!({
                "affineCoeffs": target_xf_1,
                "mode": "replace",
            }),
        )
        .expect("calibrate.apply default underlay");
    assert_eq!(apply_res_1["imageId"], underlay_img_id.0);

    // Verify underlay image transform updated
    let doc = &s.doc().unwrap().doc;
    if let NodeKind::Image(im) = &doc.node(underlay_img_id).unwrap().kind {
        assert!((im.xf.as_coeffs()[0] - 2.0).abs() < 1e-9);
    } else {
        panic!("expected image node");
    }

    // Verify second image was untouched
    if let NodeKind::Image(im) = &doc.node(second_img_id).unwrap().kind {
        assert!((im.xf.as_coeffs()[0] - 1.0).abs() < 1e-9);
    }

    // Explicit imageId -> updates second_img_id
    let target_xf_2 = [1.5, 0.0, 0.0, 1.5, -30.0, 40.0];
    let apply_res_2 = s
        .execute(
            "calibrate.apply",
            &json!({
                "affineCoeffs": target_xf_2,
                "imageId": second_img_id.0,
                "mode": "replace",
            }),
        )
        .expect("calibrate.apply explicit imageId");
    assert_eq!(apply_res_2["imageId"], second_img_id.0);

    let doc = &s.doc().unwrap().doc;
    if let NodeKind::Image(im) = &doc.node(second_img_id).unwrap().kind {
        assert!((im.xf.as_coeffs()[0] - 1.5).abs() < 1e-9);
    }

    // 4. Malformed affineCoeffs inputs
    // Missing affineCoeffs
    let err_missing = s.execute("calibrate.apply", &json!({})).unwrap_err();
    assert!(err_missing.to_string().contains("missing or invalid"));

    // Array with fewer than 6 numbers
    let err_short = s.execute("calibrate.apply", &json!({"affineCoeffs": [1.0, 2.0, 3.0]})).unwrap_err();
    assert!(err_short.to_string().contains("missing or invalid"));

    // Array with non-numbers
    let err_str = s.execute("calibrate.apply", &json!({"affineCoeffs": [1.0, "two", 3.0, 4.0, 5.0, 6.0]})).unwrap_err();
    assert!(err_str.to_string().contains("missing or invalid"));
}

/// 2.2 Stress test: `calibrate.twoPoint` command edge cases.
#[test]
fn test_adversarial_calibrate_two_point_cmd_stress() {
    let (mut s, _img_id) = test_session_with_underlay();

    // Coincident points
    let err_coincident = s
        .execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [10.0, 10.0],
                "p2": [10.0, 10.0],
                "targetDist": 100.0,
            }),
        )
        .unwrap_err();
    assert!(err_coincident.to_string().contains("degenerate"));

    // Missing target distance
    let err_no_dist = s
        .execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [10.0, 10.0],
                "p2": [110.0, 10.0],
            }),
        )
        .unwrap_err();
    assert!(err_no_dist.to_string().contains("insufficient points"));

    // Real distance in meters + document scale
    s.execute("doc.setScale", &json!({"scale": "1:20", "epsg": 5186, "order": "eastnorth"})).unwrap();

    let res_real = s
        .execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [0.0, 0.0],
                "p2": [100.0, 0.0],
                "realDistanceM": 5.0, // 5 meters at 1:20 = 5 * 50 * MM pt = 708.66 pt
                "apply": false,
            }),
        )
        .expect("realDistanceM 2-point");
    let scale_val = res_real["scale"].as_f64().unwrap();
    assert!((scale_val - (5.0 * 50.0 * MM / 100.0)).abs() < 1e-4);

    // Grid count & interval
    let res_grid = s
        .execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [0.0, 0.0],
                "p2": [100.0, 0.0],
                "gridCount": 5.0,
                "gridIntervalM": 1.0,
                "apply": false,
            }),
        )
        .expect("gridCount 2-point");
    assert!((res_grid["scale"].as_f64().unwrap() - scale_val).abs() < 1e-4);

    // Apply: true on session without underlay image -> error
    let mut s_empty = test_session_empty();
    let err_empty_apply = s_empty
        .execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [0.0, 0.0],
                "p2": [100.0, 0.0],
                "targetDist": 200.0,
                "apply": true,
            }),
        )
        .unwrap_err();
    assert!(err_empty_apply.to_string().contains("no underlay image"));
}

/// 2.3 Stress test: `calibrate.controlPoints` command with auto/affine/similarity modes,
/// and `targetSpace: "world"`.
#[test]
fn test_adversarial_calibrate_control_points_cmd_stress() {
    let (mut s, _img_id) = test_session_with_underlay();

    // Collinear points in affine mode -> Error
    let collinear_pairs = json!([
        {"source": [0.0, 0.0], "target": [10.0, 10.0]},
        {"source": [10.0, 10.0], "target": [20.0, 20.0]},
        {"source": [20.0, 20.0], "target": [30.0, 30.0]},
    ]);

    let err_collinear_aff = s
        .execute(
            "calibrate.controlPoints",
            &json!({
                "pairs": collinear_pairs,
                "mode": "affine",
            }),
        )
        .unwrap_err();
    assert!(err_collinear_aff.to_string().contains("collinear"));

    // Collinear points in auto mode -> Graceful fallback to similarity
    let res_collinear_auto = s
        .execute(
            "calibrate.controlPoints",
            &json!({
                "pairs": collinear_pairs,
                "mode": "auto",
            }),
        )
        .expect("auto mode fallback");
    assert!(res_collinear_auto["rmsePt"].as_f64().unwrap() < 1e-9);

    // Insufficient points (N = 1) -> Error
    let single_pair = json!([
        {"source": [0.0, 0.0], "target": [10.0, 10.0]},
    ]);
    let err_single = s
        .execute(
            "calibrate.controlPoints",
            &json!({
                "pairs": single_pair,
            }),
        )
        .unwrap_err();
    assert!(err_single.to_string().contains("insufficient points"));

    // targetSpace: "world" with TM coordinates
    s.execute("doc.setScale", &json!({"scale": "1:20", "epsg": 5186, "order": "eastnorth"})).unwrap();
    s.execute(
        "doc.setOrigin",
        &json!({
            "canvas": [100.0, 100.0],
            "world": [200_000.0, 500_000.0],
        }),
    )
    .unwrap();

    let world_pairs = json!([
        {"source": [0.0, 0.0], "target": [200_000.0, 500_000.0]},
        {"source": [100.0, 0.0], "target": [200_002.0, 500_000.0]},
        {"source": [100.0, 100.0], "target": [200_002.0, 500_002.0]},
    ]);

    let res_world = s
        .execute(
            "calibrate.controlPoints",
            &json!({
                "pairs": world_pairs,
                "targetSpace": "world",
                "mode": "auto",
            }),
        )
        .expect("world coordinates calibrate");
    assert!(res_world["rmsePt"].as_f64().unwrap() < 1e-4);
    assert!(res_world["rmseM"].as_f64().is_some());
    assert!(res_world["residuals"].as_array().unwrap().len() == 3);
}

/// 2.4 Stress test: `calibrate.residuals` query command.
#[test]
fn test_adversarial_calibrate_residuals_query_non_mutating() {
    let (mut s, img_id) = test_session_with_underlay();

    let initial_xf = match &s.doc().unwrap().doc.node(img_id).unwrap().kind {
        NodeKind::Image(im) => im.xf,
        _ => panic!("expected image"),
    };

    let pairs = json!([
        {"source": [0.0, 0.0], "target": [10.0, 20.0]},
        {"source": [100.0, 0.0], "target": [110.0, 20.0]},
        {"source": [100.0, 100.0], "target": [110.0, 120.0]},
    ]);

    let test_xf = [1.0, 0.0, 0.0, 1.0, 10.0, 20.0];

    let query_res = s
        .execute(
            "calibrate.residuals",
            &json!({
                "affineCoeffs": test_xf,
                "pairs": pairs,
            }),
        )
        .expect("calibrate.residuals query");

    assert!(query_res["rmsePt"].as_f64().unwrap() < 1e-9);
    assert_eq!(query_res["residuals"].as_array().unwrap().len(), 3);

    // Verify document was not mutated
    let after_xf = match &s.doc().unwrap().doc.node(img_id).unwrap().kind {
        NodeKind::Image(im) => im.xf,
        _ => panic!("expected image"),
    };
    assert_eq!(initial_xf, after_xf);
}
