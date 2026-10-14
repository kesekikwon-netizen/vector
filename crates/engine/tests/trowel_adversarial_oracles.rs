//! Empirical Adversarial Stress Test Suite for Trowel Mathematical & Geometric Oracles
//!
//! Evaluates the 5 critical mathematical oracles:
//! 1. Shoelace Polygon Area & PCA Orientation (random perturbations, collinear/degenerate, self-intersecting, CW vs CCW)
//! 2. Pottery 3-Point Rim Circumcircle Fit & Bilateral Reflection (near-collinear limit |A| -> 1e-9, collinear, coincident, reflection & section cut invariants)
//! 3. Elevation Line Generation & 1D Void Clipping (inverted ranges, zero interval, multi-polygon void obstacles)
//! 4. Underlay Calibration (2-point similarity & N-point affine LS with collinear, coincident, noisy synthetic residuals)
//! 5. 45°/135° Dual Graph Bipartite Coloring (chains, trees, even cycles, odd cycles C3/C5, disconnected component isolation)

#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, Point};
use vectorcraft_doc::elevation::{ElevationLabelFormat, ElevationLabelPlacement, ElevationRangeMode, ElevationSpec};
use vectorcraft_doc::pottery::SectionSide;
use vectorcraft_doc::scale::ScaleConfig;
use vectorcraft_geom::circle_fit::{CircleFitError, fit_circle_3point};
use vectorcraft_geom::registration::{AxisAlignment, PointPair, RegistrationError, fit_affine, fit_auto, fit_two_point};
use vectorcraft_geom::{Anchor, AnchorKind, PathData, SubPath};
use vectorcraft_pathops::elevation::{clip_elevation_scanline, generate_elevation};
use vectorcraft_pathops::hatch::{bipartite_2color, bipartite_2color_components};
use vectorcraft_pathops::plan::{HOUSE_1_POLYGON_METERS, compute_polygon_stats};
use vectorcraft_pathops::pottery::{offset_wall_profile, reflect_path_across_centerline};

// =========================================================================
// 1. SHOELACE POLYGON AREA & PCA ORIENTATION STRESS TESTS
// =========================================================================

#[test]
fn test_stress_shoelace_house1_random_perturbations() {
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    // Canonical baseline
    let mut base_path = BezPath::new();
    let (bx0, by0) = HOUSE_1_POLYGON_METERS[0];
    base_path.move_to(Point::new(bx0 * ppm, -by0 * ppm));
    for &(x, y) in &HOUSE_1_POLYGON_METERS[1..] {
        base_path.line_to(Point::new(x * ppm, -y * ppm));
    }
    base_path.close_path();

    let base_stats = compute_polygon_stats(&base_path, &scale);
    assert_eq!(base_stats.format_area(), "32.91 ㎡");
    let base_area = base_stats.area_m2;
    let base_azimuth = base_stats.azimuth_deg;

    // Test 100 pseudo-random deterministic perturbations with noise in [-0.01m, +0.01m]
    let mut rng_state: u64 = 0x12345678_9abcdef0;
    let mut next_f64 = || -> f64 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (rng_state >> 32) as u32;
        (val as f64) / (u32::MAX as f64)
    };

    for iter in 0..100 {
        let max_noise_m = 0.005; // 5mm noise
        let mut perturbed_path = BezPath::new();
        for (i, &(x, y)) in HOUSE_1_POLYGON_METERS.iter().enumerate() {
            let dx = (next_f64() * 2.0 - 1.0) * max_noise_m;
            let dy = (next_f64() * 2.0 - 1.0) * max_noise_m;
            let px = (x + dx) * ppm;
            let py = -(y + dy) * ppm;
            if i == 0 {
                perturbed_path.move_to(Point::new(px, py));
            } else {
                perturbed_path.line_to(Point::new(px, py));
            }
        }
        perturbed_path.close_path();

        let p_stats = compute_polygon_stats(&perturbed_path, &scale);

        // Area continuity check: 5mm vertex perturbation cannot change 32.91 ㎡ by more than 0.15 ㎡
        let area_diff = (p_stats.area_m2 - base_area).abs();
        assert!(area_diff < 0.15, "iter {}: area changed excessively: {} vs {} (diff = {})", iter, p_stats.area_m2, base_area, area_diff);

        // PCA Azimuth stability check: long axis (~7.4m vs 5.8m) must stay stable within 2.5 degrees
        let az_diff = (p_stats.azimuth_deg - base_azimuth).abs();
        let az_diff_norm = if az_diff > 90.0 { 180.0 - az_diff } else { az_diff };
        assert!(
            az_diff_norm < 2.5,
            "iter {}: azimuth changed excessively: {} vs {} (diff = {})",
            iter,
            p_stats.azimuth_deg,
            base_azimuth,
            az_diff_norm
        );
    }
}

#[test]
fn test_stress_shoelace_collinear_and_degenerate() {
    let scale = ScaleConfig::new(20.0);
    let _ppm = scale.scale.points_per_meter();

    // 1. Zero vertices
    let empty_path = BezPath::new();
    let stats_empty = compute_polygon_stats(&empty_path, &scale);
    assert_eq!(stats_empty.area_m2, 0.0);

    // 2. Single vertex
    let mut single_path = BezPath::new();
    single_path.move_to(Point::new(10.0, 10.0));
    single_path.close_path();
    let stats_single = compute_polygon_stats(&single_path, &scale);
    assert_eq!(stats_single.area_m2, 0.0);

    // 3. 2 vertices (line segment)
    let mut line_path = BezPath::new();
    line_path.move_to(Point::new(10.0, 10.0));
    line_path.line_to(Point::new(200.0, 200.0));
    line_path.close_path();
    let stats_line = compute_polygon_stats(&line_path, &scale);
    assert_eq!(stats_line.area_m2, 0.0);

    // 4. 10 collinear points on a 45-degree line
    let mut col_path = BezPath::new();
    col_path.move_to(Point::new(0.0, 0.0));
    for i in 1..=10 {
        let t = i as f64 * 50.0;
        col_path.line_to(Point::new(t, t));
    }
    col_path.close_path();
    let stats_col = compute_polygon_stats(&col_path, &scale);
    assert!(stats_col.area_m2 < 1e-12, "Collinear vertices must yield exactly 0 area, got {}", stats_col.area_m2);
    assert!(!stats_col.area_m2.is_nan());
    assert!(!stats_col.azimuth_deg.is_nan());
}

#[test]
fn test_stress_shoelace_self_intersecting_figure8() {
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    // Symmetric Bowtie Figure-8: (0,0) -> (2, 2) -> (0, 2) -> (2, 0) -> (0,0)
    // Left lobe has area 1, right lobe has area 1, but with opposite winding.
    let mut bowtie = BezPath::new();
    bowtie.move_to(Point::new(0.0 * ppm, 0.0 * ppm));
    bowtie.line_to(Point::new(2.0 * ppm, 2.0 * ppm));
    bowtie.line_to(Point::new(0.0 * ppm, 2.0 * ppm));
    bowtie.line_to(Point::new(2.0 * ppm, 0.0 * ppm));
    bowtie.close_path();

    let stats_bowtie = compute_polygon_stats(&bowtie, &scale);
    // Green's theorem cancels out symmetric lobes with opposite signs
    assert!(stats_bowtie.area_m2 < 1e-9, "Symmetric self-intersecting bowtie area should cancel out to ~0, got {}", stats_bowtie.area_m2);
    assert!(!stats_bowtie.centroid_canvas.x.is_nan());
    assert!(!stats_bowtie.centroid_canvas.y.is_nan());
}

#[test]
fn test_stress_shoelace_clockwise_vs_counterclockwise_invariance() {
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    // CCW path
    let mut ccw = BezPath::new();
    let (x0, y0) = HOUSE_1_POLYGON_METERS[0];
    ccw.move_to(Point::new(x0 * ppm, -y0 * ppm));
    for &(x, y) in &HOUSE_1_POLYGON_METERS[1..] {
        ccw.line_to(Point::new(x * ppm, -y * ppm));
    }
    ccw.close_path();

    // CW path (reversed vertex sequence)
    let mut cw = BezPath::new();
    let rev_pts: Vec<(f64, f64)> = HOUSE_1_POLYGON_METERS.iter().copied().rev().collect();
    let (rx0, ry0) = rev_pts[0];
    cw.move_to(Point::new(rx0 * ppm, -ry0 * ppm));
    for &(x, y) in &rev_pts[1..] {
        cw.line_to(Point::new(x * ppm, -y * ppm));
    }
    cw.close_path();

    let stats_ccw = compute_polygon_stats(&ccw, &scale);
    let stats_cw = compute_polygon_stats(&cw, &scale);

    // Exact area magnitude invariance
    assert_eq!(stats_ccw.format_area(), "32.91 ㎡");
    assert_eq!(stats_cw.format_area(), "32.91 ㎡");
    assert!((stats_ccw.area_m2 - stats_cw.area_m2).abs() < 1e-12, "CW and CCW areas must match to float precision");

    // Perimeter invariance
    assert!((stats_ccw.perimeter_m - stats_cw.perimeter_m).abs() < 1e-12, "Perimeter must be invariant to vertex direction");

    // PCA Azimuth invariance
    assert!(
        (stats_ccw.azimuth_deg - stats_cw.azimuth_deg).abs() < 1e-9,
        "PCA azimuth must be invariant to vertex direction: {} vs {}",
        stats_ccw.azimuth_deg,
        stats_cw.azimuth_deg
    );
}

// =========================================================================
// 2. POTTERY 3-POINT RIM CIRCUMCIRCLE FIT & BILATERAL REFLECTION STRESS TESTS
// =========================================================================

#[test]
fn test_stress_pottery_circle_fit_collinear_and_near_collinear_limit() {
    // 1. Exactly collinear points
    let p1 = Point::new(10.0, 10.0);
    let p2 = Point::new(20.0, 20.0);
    let p3 = Point::new(30.0, 30.0);

    let res_exact = fit_circle_3point(p1, p2, p3);
    match res_exact {
        Err(CircleFitError::CollinearPoints { area }) => {
            assert!(area < 1e-9, "collinear area must be < 1e-9");
        }
        other => panic!("Expected CollinearPoints, got {:?}", other),
    }

    // 2. Near-collinear points approaching the 1e-9 threshold
    // Let base segment be (0,0) to (2, 0). Height h above midpoint gives area A = 0.5 * 2 * h = h.
    let base1 = Point::new(0.0, 0.0);
    let base2 = Point::new(2.0, 0.0);

    // Case A: h = 1e-10 -> Area = 1e-10 < 1e-9 -> Must return Err(CollinearPoints)
    let mid_sub = Point::new(1.0, 1e-10);
    let res_sub = fit_circle_3point(base1, mid_sub, base2);
    assert!(matches!(res_sub, Err(CircleFitError::CollinearPoints { .. })));

    // Case B: h = 1e-8 -> Area = 1e-8 >= 1e-9 -> Must succeed without panic
    let mid_sup = Point::new(1.0, 1e-8);
    let res_sup = fit_circle_3point(base1, mid_sup, base2).expect("area 1e-8 must fit");
    assert!(res_sup.radius > 1e7, "near-collinear radius should be huge: {}", res_sup.radius);
    assert!(res_sup.center.y.is_finite());
    assert!(res_sup.center.x.is_finite());
}

#[test]
fn test_stress_pottery_circle_fit_coincident_and_non_finite() {
    let p1 = Point::new(50.0, 50.0);
    let p2 = Point::new(50.0, 50.0); // Coincident with p1
    let p3 = Point::new(80.0, 100.0);

    let res_coincident = fit_circle_3point(p1, p2, p3);
    assert!(matches!(res_coincident, Err(CircleFitError::DegeneratePoints)));

    // Extremely close points (< 1e-9 distance, EPSILON_DISTANCE threshold)
    let p2_close = Point::new(50.0 + 1e-10, 50.0);
    let res_close = fit_circle_3point(p1, p2_close, p3);
    assert!(matches!(res_close, Err(CircleFitError::DegeneratePoints)));

    // Distinct points (> 1e-9 distance) produce a valid fit without division by zero
    let p2_distinct = Point::new(50.0 + 1e-6, 50.0);
    let res_distinct = fit_circle_3point(p1, p2_distinct, p3);
    assert!(res_distinct.is_ok(), "Distance 1e-6 exceeds EPSILON_DISTANCE and fits");

    // Non-finite coordinates
    let nan_pt = Point::new(f64::NAN, 50.0);
    assert!(matches!(fit_circle_3point(nan_pt, p2, p3), Err(CircleFitError::InvalidCoordinates)));

    let inf_pt = Point::new(50.0, f64::INFINITY);
    assert!(matches!(fit_circle_3point(p1, inf_pt, p3), Err(CircleFitError::InvalidCoordinates)));
}

#[test]
fn test_stress_pottery_bilateral_reflection_and_section_cut_invariants() {
    let cl_x = 300.0;

    // Create an intricate right-side pottery outer profile (all x >= cl_x)
    let anchors = vec![
        Anchor {
            p: Point::new(392.0, 100.0), // Rim top
            h_in: Point::new(392.0, 95.0),
            h_out: Point::new(390.0, 105.0),
            kind: AnchorKind::Smooth,
        },
        Anchor {
            p: Point::new(385.0, 160.0), // Belly maximum diameter
            h_in: Point::new(388.0, 140.0),
            h_out: Point::new(380.0, 180.0),
            kind: AnchorKind::Smooth,
        },
        Anchor {
            p: Point::new(340.0, 250.0), // Base corner
            h_in: Point::new(350.0, 230.0),
            h_out: Point::new(330.0, 250.0),
            kind: AnchorKind::Corner,
        },
        Anchor {
            p: Point::new(300.0, 250.0), // Base at centerline
            h_in: Point::new(310.0, 250.0),
            h_out: Point::new(300.0, 250.0),
            kind: AnchorKind::Corner,
        },
    ];

    let path = PathData { subpaths: vec![SubPath { anchors, closed: false }] };

    // 1. Bilateral reflection across centerline
    let reflected = reflect_path_across_centerline(&path, cl_x);

    // Invariant A: All reflected points must lie on the left side (x <= cl_x)
    for sp in &reflected.subpaths {
        for a in &sp.anchors {
            assert!(a.p.x <= cl_x + 1e-9, "Reflected point must be <= cl_x: {} vs {}", a.p.x, cl_x);
            assert!(a.h_in.x <= cl_x + 1e-9, "Reflected h_in must be <= cl_x: {}", a.h_in.x);
            assert!(a.h_out.x <= cl_x + 1e-9, "Reflected h_out must be <= cl_x: {}", a.h_out.x);
        }
    }

    // Invariant B: Double reflection must recover original points exactly
    let double_reflected = reflect_path_across_centerline(&reflected, cl_x);
    assert_eq!(path.subpaths[0].anchors.len(), double_reflected.subpaths[0].anchors.len());
    for (orig, dbl) in path.subpaths[0].anchors.iter().zip(&double_reflected.subpaths[0].anchors) {
        assert!((orig.p.x - dbl.p.x).abs() < 1e-12, "Double reflection x mismatch");
        assert!((orig.p.y - dbl.p.y).abs() < 1e-12, "Double reflection y mismatch");
        assert!((orig.h_in.x - dbl.h_in.x).abs() < 1e-12, "Double reflection h_in x mismatch");
        assert!((orig.h_out.x - dbl.h_out.x).abs() < 1e-12, "Double reflection h_out x mismatch");
    }

    // Invariant C: Wall profile offset inwards for SectionSide::Right must NEVER cross centerline
    let offset_path = offset_wall_profile(&path, 20.0, cl_x, SectionSide::Right);
    for sp in &offset_path.subpaths {
        for a in &sp.anchors {
            assert!(a.p.x >= cl_x - 1e-9, "Section cut wall offset must not cross past centerline: {}", a.p.x);
        }
    }

    // Stress: Massive wall thickness (150 pt, far exceeding radius of 92 pt)
    let extreme_offset = offset_wall_profile(&path, 150.0, cl_x, SectionSide::Right);
    for sp in &extreme_offset.subpaths {
        for a in &sp.anchors {
            // Must be clamped to centerline, never penetrate left side
            assert!(a.p.x >= cl_x - 1e-9, "Extreme thickness offset penetrated centerline: {}", a.p.x);
            assert_eq!(a.p.x, cl_x, "Extreme thickness must clamp to centerline");
        }
    }
}

// =========================================================================
// 3. ELEVATION LINE GENERATION & 1D VOID CLIPPING STRESS TESTS
// =========================================================================

#[test]
fn test_stress_elevation_inverted_range_and_zero_intervals() {
    let mut ground = BezPath::new();
    ground.move_to(Point::new(100.0, 500.0));
    ground.line_to(Point::new(300.0, 520.0));
    ground.line_to(Point::new(500.0, 500.0));

    // 1. Inverted range (min > max)
    let spec_inverted = ElevationSpec {
        profile_id: vectorcraft_doc::NodeId(1),
        obstacle_ids: vec![],
        datum_m: 150.0,
        minor_interval_m: 0.1,
        major_interval_m: 0.5,
        minor_weight_pt: 0.2,
        major_weight_pt: 0.5,
        label_format: ElevationLabelFormat::MetersSuffix,
        label_placement: ElevationLabelPlacement::LeftEnd,
        label_font_size_pt: 8.0,
        range_mode: ElevationRangeMode::Explicit {
            min_elevation_m: 160.0,
            max_elevation_m: 150.0, // Inverted!
        },
        auto_update: true,
    };

    let out_inv = generate_elevation(&ground, &[], &spec_inverted, None);
    assert!(out_inv.segments.is_empty(), "Inverted elevation range must produce 0 segments");
    assert!(out_inv.labels.is_empty(), "Inverted elevation range must produce 0 labels");

    // 2. Zero minor interval: must not cause infinite loop or panic
    let mut spec_zero_interval = spec_inverted.clone();
    spec_zero_interval.minor_interval_m = 0.0;
    spec_zero_interval.range_mode = ElevationRangeMode::Explicit { min_elevation_m: 150.0, max_elevation_m: 151.0 };

    let out_zero = generate_elevation(&ground, &[], &spec_zero_interval, None);
    assert!(!out_zero.segments.is_empty(), "Zero minor interval is clamped safely by pathops");
}

#[test]
fn test_stress_elevation_1d_multi_polygon_void_obstacles() {
    // Ground profile at y = 500
    let mut ground = BezPath::new();
    ground.move_to(Point::new(0.0, 500.0));
    ground.line_to(Point::new(1000.0, 500.0));

    // 3 obstacles on the scanline at y = 300:
    // Obs 1: [100, 200]
    let mut obs1 = BezPath::new();
    obs1.move_to(Point::new(100.0, 250.0));
    obs1.line_to(Point::new(200.0, 250.0));
    obs1.line_to(Point::new(200.0, 350.0));
    obs1.line_to(Point::new(100.0, 350.0));
    obs1.close_path();

    // Obs 2: [300, 500]
    let mut obs2 = BezPath::new();
    obs2.move_to(Point::new(300.0, 250.0));
    obs2.line_to(Point::new(500.0, 250.0));
    obs2.line_to(Point::new(500.0, 350.0));
    obs2.line_to(Point::new(300.0, 350.0));
    obs2.close_path();

    // Obs 3: [700, 900]
    let mut obs3 = BezPath::new();
    obs3.move_to(Point::new(700.0, 250.0));
    obs3.line_to(Point::new(900.0, 250.0));
    obs3.line_to(Point::new(900.0, 350.0));
    obs3.line_to(Point::new(700.0, 350.0));
    obs3.close_path();

    let obstacles = vec![obs1, obs2, obs3];

    // Clip horizontal line at y = 300 across [0, 1000] with ceiling at y = 100
    let intervals = clip_elevation_scanline(300.0, &ground, &obstacles, 0.0, 1000.0, 100.0);

    // Initial open void is [0, 1000].
    // After subtracting [100, 200], [300, 500], [700, 900],
    // remaining intervals must be:
    // [0, 100], [200, 300], [500, 700], [900, 1000]
    assert_eq!(intervals.len(), 4, "Expected 4 void intervals, got {:?}", intervals);

    assert!((intervals[0].0 - 0.0).abs() < 1e-3 && (intervals[0].1 - 100.0).abs() < 1e-3);
    assert!((intervals[1].0 - 200.0).abs() < 1e-3 && (intervals[1].1 - 300.0).abs() < 1e-3);
    assert!((intervals[2].0 - 500.0).abs() < 1e-3 && (intervals[2].1 - 700.0).abs() < 1e-3);
    assert!((intervals[3].0 - 900.0).abs() < 1e-3 && (intervals[3].1 - 1000.0).abs() < 1e-3);

    // Test wide obstacle completely occluding everything: [0, 1000]
    let mut wide_obs = BezPath::new();
    wide_obs.move_to(Point::new(0.0, 200.0));
    wide_obs.line_to(Point::new(1000.0, 200.0));
    wide_obs.line_to(Point::new(1000.0, 400.0));
    wide_obs.line_to(Point::new(0.0, 400.0));
    wide_obs.close_path();

    let intervals_occluded = clip_elevation_scanline(300.0, &ground, &[wide_obs], 0.0, 1000.0, 100.0);
    assert!(intervals_occluded.is_empty(), "Completely occluded scanline must have 0 intervals");
}

// =========================================================================
// 4. UNDERLAY CALIBRATION & SYNTHETIC RESIDUALS STRESS TESTS
// =========================================================================

#[test]
fn test_stress_calibration_collinear_and_coincident_points() {
    // 1. Two-point calibration with coincident points
    let p = Point::new(100.0, 100.0);
    let res_coincident_2pt = fit_two_point(p, p, Some(10.0), None, None, AxisAlignment::Horizontal);
    assert!(matches!(res_coincident_2pt, Err(RegistrationError::DegeneratePoints)));

    // 2. Affine registration with collinear points: (0,0), (10,10), (20,20)
    let collinear_pairs = vec![
        PointPair { source: Point::new(0.0, 0.0), target: Point::new(100.0, 200.0) },
        PointPair { source: Point::new(10.0, 10.0), target: Point::new(110.0, 210.0) },
        PointPair { source: Point::new(20.0, 20.0), target: Point::new(120.0, 220.0) },
    ];

    let res_affine_collinear = fit_affine(&collinear_pairs, None);
    assert!(matches!(res_affine_collinear, Err(RegistrationError::CollinearPoints)), "Collinear control points must be rejected by fit_affine");

    // 3. fit_auto must catch CollinearPoints and gracefully fall back to similarity transform!
    let res_auto_fallback = fit_auto(&collinear_pairs, None);
    assert!(res_auto_fallback.is_ok(), "fit_auto must fall back to similarity transform on collinear points");
}

#[test]
fn test_stress_calibration_noisy_control_points_and_residuals() {
    // Ground truth: Scale 2.5, rotation 30 deg, translation (150, -80)
    let theta = 30.0_f64.to_radians();
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let scale = 2.5;
    let tx = 150.0;
    let ty = -80.0;

    let source_pts = vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0), Point::new(100.0, 80.0), Point::new(0.0, 80.0), Point::new(50.0, 40.0)];

    // Synthetic exact pairs
    let mut exact_pairs = Vec::new();
    for p in &source_pts {
        let x_tgt = scale * (cos_t * p.x - sin_t * p.y) + tx;
        let y_tgt = scale * (sin_t * p.x + cos_t * p.y) + ty;
        exact_pairs.push(PointPair { source: *p, target: Point::new(x_tgt, y_tgt) });
    }

    let exact_res = fit_affine(&exact_pairs, Some(100.0)).expect("exact fit");
    assert!(exact_res.rmse_pt < 1e-10, "Exact RMSE must be ~0");
    assert!((exact_res.scale.0 - 2.5).abs() < 1e-10);
    assert!((exact_res.rotation_deg - 30.0).abs() < 1e-10);

    // Add controlled synthetic residual noise to target points (dx = +0.5, dy = -0.5)
    let mut noisy_pairs = Vec::new();
    let noise = [(0.2, -0.1), (-0.3, 0.2), (0.1, 0.3), (-0.2, -0.2), (0.1, -0.1)];
    for (i, p) in source_pts.iter().enumerate() {
        let x_tgt = scale * (cos_t * p.x - sin_t * p.y) + tx + noise[i].0;
        let y_tgt = scale * (sin_t * p.x + cos_t * p.y) + ty + noise[i].1;
        noisy_pairs.push(PointPair { source: *p, target: Point::new(x_tgt, y_tgt) });
    }

    let noisy_res = fit_affine(&noisy_pairs, Some(100.0)).expect("noisy fit");
    assert!(noisy_res.rmse_pt > 0.0, "Noisy RMSE must be strictly positive");
    assert!(noisy_res.rmse_pt < 0.5, "RMSE must be within noise magnitude: {}", noisy_res.rmse_pt);
    assert_eq!(noisy_res.residuals.len(), 5);

    // Each residual item must have valid mm and pt metrics
    for r in &noisy_res.residuals {
        assert!(r.dist_pt.is_finite());
        assert!(r.dist_mm.is_finite());
        assert!(r.dist_m.is_some());
    }
}

// =========================================================================
// 5. 45°/135° DUAL GRAPH BIPARTITE COLORING STRESS TESTS
// =========================================================================

#[test]
fn test_stress_dual_graph_bipartite_chains_trees_and_even_cycles() {
    // 1. Linear chain P10 (0-1-2-...-9)
    let mut p10_adj = vec![Vec::new(); 10];
    for i in 0..9 {
        p10_adj[i].push(i + 1);
        p10_adj[i + 1].push(i);
    }
    let (colors_p10, conflicts_p10) = bipartite_2color_components(&p10_adj);
    assert_eq!(colors_p10.len(), 10);
    assert!(conflicts_p10.iter().all(|&c| !c), "Linear chain must have zero conflicts");
    // Verify alternating colors
    for i in 0..9 {
        assert_ne!(colors_p10[i], colors_p10[i + 1], "Adjacent nodes in chain must alternate");
    }

    // 2. Star graph K_{1, 5} (center 0 connected to 1,2,3,4,5) - Tree
    let mut star_adj = vec![Vec::new(); 6];
    for leaf in 1..=5 {
        star_adj[0].push(leaf);
        star_adj[leaf].push(0);
    }
    let (colors_star, conflicts_star) = bipartite_2color_components(&star_adj);
    assert!(conflicts_star.iter().all(|&c| !c), "Star tree must have zero conflicts");
    for leaf in 1..=5 {
        assert_ne!(colors_star[0], colors_star[leaf]);
    }

    // 3. Even cycle C6 (0-1-2-3-4-5-0)
    let mut c6_adj = vec![Vec::new(); 6];
    for i in 0..6 {
        let j = (i + 1) % 6;
        c6_adj[i].push(j);
        c6_adj[j].push(i);
    }
    let (colors_c6, conflicts_c6) = bipartite_2color_components(&c6_adj);
    assert!(conflicts_c6.iter().all(|&c| !c), "Even cycle C6 must have zero conflicts");
    for i in 0..6 {
        let j = (i + 1) % 6;
        assert_ne!(colors_c6[i], colors_c6[j]);
    }
}

#[test]
fn test_stress_dual_graph_odd_cycles_and_component_isolation() {
    // 1. Odd cycle C3 (Triangle / K3: 0-1, 1-2, 2-0)
    let c3_adj = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
    let (_colors_c3, conflicts_c3) = bipartite_2color_components(&c3_adj);
    assert!(conflicts_c3.iter().all(|&c| c), "All nodes in C3 odd cycle must be flagged with conflict");

    // 2. Odd cycle C5 (Pentagon: 0-1-2-3-4-0)
    let mut c5_adj = vec![Vec::new(); 5];
    for i in 0..5 {
        let j = (i + 1) % 5;
        c5_adj[i].push(j);
        c5_adj[j].push(i);
    }
    let (_colors_c5, conflicts_c5) = bipartite_2color_components(&c5_adj);
    assert!(conflicts_c5.iter().all(|&c| c), "All nodes in C5 odd cycle must be flagged with conflict");

    // 3. CRITICAL ISOLATION TEST: Disconnected components
    // Component A: C3 (Nodes 0, 1, 2 - Odd cycle, MUST have conflict = true)
    // Component B: P3 (Nodes 3, 4, 5 - Path 3-4-5, Bipartite, MUST have conflict = false)
    // Component C: Isolated vertex (Node 6, MUST have conflict = false)
    let mut mixed_adj = vec![Vec::new(); 7];
    // C3
    mixed_adj[0] = vec![1, 2];
    mixed_adj[1] = vec![0, 2];
    mixed_adj[2] = vec![0, 1];
    // P3
    mixed_adj[3] = vec![4];
    mixed_adj[4] = vec![3, 5];
    mixed_adj[5] = vec![4];
    // Node 6 is empty (isolated)

    let (colors_mixed, conflicts_mixed) = bipartite_2color_components(&mixed_adj);

    // Component A (0, 1, 2)
    assert!(conflicts_mixed[0], "Node 0 in C3 must have conflict");
    assert!(conflicts_mixed[1], "Node 1 in C3 must have conflict");
    assert!(conflicts_mixed[2], "Node 2 in C3 must have conflict");

    // Component B (3, 4, 5) MUST NOT BE CONTAMINATED BY COMPONENT A!
    assert!(!conflicts_mixed[3], "DEFECT REGRESSION: Node 3 in bipartite P3 was falsely contaminated by C3");
    assert!(!conflicts_mixed[4], "DEFECT REGRESSION: Node 4 in bipartite P3 was falsely contaminated by C3");
    assert!(!conflicts_mixed[5], "DEFECT REGRESSION: Node 5 in bipartite P3 was falsely contaminated by C3");
    assert_ne!(colors_mixed[3], colors_mixed[4]);
    assert_ne!(colors_mixed[4], colors_mixed[5]);

    // Component C (6)
    assert!(!conflicts_mixed[6], "Isolated node 6 must not have conflict");

    // Wrapper function test
    let (_assignments, has_conflict_global) = bipartite_2color(&mixed_adj);
    assert!(has_conflict_global, "Global conflict must be true because C3 is present");
}
