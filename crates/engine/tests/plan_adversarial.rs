//! Adversarial Stress Test Suite for Task J009 (Plan Tools & Statistical Readouts).
//!
//! Evaluates:
//! 1. Exact area calculation for canonical Gwangnyeong-ri 1 House 1 polygon (`HOUSE_1_POLYGON_METERS`)
//!    at 1:20 scale producing exactly `32.91 ㎡`.
//! 2. Self-intersecting polygons (figure-8), degenerate polygons (empty, 1 point, 2 points, collinear),
//!    and reverse-wound (clockwise vs counter-clockwise) polygons.
//! 3. Section cut line with zero length (p0 == p1) and reversed sightline (ViewDirection::Left vs Right).
//! 4. Spot elevation label formatting across positive, zero, negative heights (verifying 2 decimal places),
//!    and option flags (prefixPlus, suffixM).
//! 5. Depth marker generation across centimeter and meter formatting and symbol styles.
//! 6. Slope hachures with zero-length curves, single points, tight loops, and extreme curvatures.
//! 7. Engine commands (`trowel.sectionLine`, `trowel.spotElevation`, `trowel.depthMarker`,
//!    `trowel.slopeHachures`, `trowel.polygonStats`) under missing parameters, malformed JSON,
//!    and non-existent nodes.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, Point, Shape};
use serde_json::json;
use vectorcraft_doc::scale::ScaleConfig;
use vectorcraft_engine::Session;
use vectorcraft_pathops::plan::{
    DepthMarkerKind, DepthMarkerSpec, HOUSE_1_POLYGON_METERS, SectionLineSpec, SlopeHachureSpec, SpotElevationSpec, SpotMarkerKind, ViewDirection,
    compute_polygon_stats, generate_depth_marker_geometry, generate_section_line_geometry, generate_slope_hachures, generate_spot_elevation_geometry,
};

fn test_session_with_doc() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    s
}

// =========================================================================
// 1. Canonical House 1 Benchmark (J009 / §13)
// =========================================================================

#[test]
fn test_canonical_house_1_area_32_91_m2_exact() {
    let scale = ScaleConfig::new(20.0); // 1:20 scale
    let ppm = scale.scale.points_per_meter();

    let mut path = BezPath::new();
    let (first_x, first_y) = HOUSE_1_POLYGON_METERS[0];
    path.move_to(Point::new(first_x * ppm, -first_y * ppm));

    for &(x, y) in &HOUSE_1_POLYGON_METERS[1..] {
        path.line_to(Point::new(x * ppm, -y * ppm));
    }
    path.close_path();

    let stats = compute_polygon_stats(&path, &scale);
    assert_eq!(stats.format_area(), "32.91 ㎡");
    assert!((stats.area_m2 - 32.91).abs() < 0.005, "area difference must be within rounding threshold");

    // Also verify MBB dimensions and PCA azimuth are physically reasonable
    assert!(stats.major_axis_m > stats.minor_axis_m);
    assert!(stats.perimeter_m > 0.0);
    assert!(stats.azimuth_deg >= 0.0 && stats.azimuth_deg < 180.0);
    assert!(stats.azimuth_quadrant.starts_with('N'));
}

#[test]
fn test_engine_command_house_1_polygon_stats() {
    let mut s = test_session_with_doc();
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    let pts: Vec<[f64; 2]> = HOUSE_1_POLYGON_METERS.iter().map(|&(x, y)| [x * ppm, -y * ppm]).collect();

    let res = s.execute("trowel.polygonStats", &json!({ "points": pts })).expect("polygon stats");
    assert_eq!(res["formatted"]["area"].as_str().unwrap(), "32.91 ㎡");
    let area = res["area_m2"].as_f64().unwrap();
    assert!((area - 32.91).abs() < 0.005);
}

// =========================================================================
// 2. Degenerate, Self-Intersecting & Reverse-Wound Polygons
// =========================================================================

#[test]
fn test_degenerate_polygons_no_panic() {
    let scale = ScaleConfig::new(20.0);

    // Empty path
    let empty_path = BezPath::new();
    let stats = compute_polygon_stats(&empty_path, &scale);
    assert_eq!(stats.area_m2, 0.0);
    assert_eq!(stats.perimeter_m, 0.0);
    assert_eq!(stats.format_area(), "0.00 ㎡");

    // Single point path
    let mut single_pt = BezPath::new();
    single_pt.move_to(Point::new(50.0, 50.0));
    single_pt.close_path();
    let stats = compute_polygon_stats(&single_pt, &scale);
    assert_eq!(stats.area_m2, 0.0);
    assert_eq!(stats.perimeter_m, 0.0);

    // Two point segment
    let mut two_pts = BezPath::new();
    two_pts.move_to(Point::new(50.0, 50.0));
    two_pts.line_to(Point::new(100.0, 100.0));
    two_pts.close_path();
    let stats = compute_polygon_stats(&two_pts, &scale);
    assert_eq!(stats.area_m2, 0.0);
    assert_eq!(stats.perimeter_m, 0.0);

    // Three collinear points
    let mut collinear = BezPath::new();
    collinear.move_to(Point::new(10.0, 10.0));
    collinear.line_to(Point::new(20.0, 20.0));
    collinear.line_to(Point::new(30.0, 30.0));
    collinear.close_path();
    let stats = compute_polygon_stats(&collinear, &scale);
    assert!(stats.area_m2.abs() < 1e-6);
    assert!(stats.centroid_canvas.x.is_finite());
    assert!(stats.centroid_canvas.y.is_finite());
}

#[test]
fn test_reverse_wound_clockwise_polygon() {
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    // CCW triangle: (0,0) -> (10,0) -> (0,10) in meters
    let mut ccw = BezPath::new();
    ccw.move_to(Point::new(0.0 * ppm, 0.0 * ppm));
    ccw.line_to(Point::new(10.0 * ppm, 0.0 * ppm));
    ccw.line_to(Point::new(0.0 * ppm, -10.0 * ppm));
    ccw.close_path();
    let stats_ccw = compute_polygon_stats(&ccw, &scale);

    // CW triangle: (0,0) -> (0,10) -> (10,0) in meters
    let mut cw = BezPath::new();
    cw.move_to(Point::new(0.0 * ppm, 0.0 * ppm));
    cw.line_to(Point::new(0.0 * ppm, -10.0 * ppm));
    cw.line_to(Point::new(10.0 * ppm, 0.0 * ppm));
    cw.close_path();
    let stats_cw = compute_polygon_stats(&cw, &scale);

    // Area must be identical and positive (50 m2)
    assert!((stats_ccw.area_m2 - 50.0).abs() < 1e-4);
    assert!((stats_cw.area_m2 - 50.0).abs() < 1e-4);
    assert_eq!(stats_ccw.format_area(), "50.00 ㎡");
    assert_eq!(stats_cw.format_area(), "50.00 ㎡");

    // Centroids must match
    assert!((stats_ccw.centroid_canvas.x - stats_cw.centroid_canvas.x).abs() < 1e-4);
    assert!((stats_ccw.centroid_canvas.y - stats_cw.centroid_canvas.y).abs() < 1e-4);
}

#[test]
fn test_self_intersecting_figure_8_polygon() {
    let scale = ScaleConfig::new(20.0);
    let ppm = scale.scale.points_per_meter();

    // Symmetrical Figure-8: (0,0) -> (2,2) -> (2,0) -> (0,0) -> (-2,2) -> (-2,0)
    // Shoelace signed area cancels out to 0
    let mut fig8 = BezPath::new();
    fig8.move_to(Point::new(0.0, 0.0));
    fig8.line_to(Point::new(2.0 * ppm, -2.0 * ppm));
    fig8.line_to(Point::new(2.0 * ppm, 0.0));
    fig8.line_to(Point::new(0.0, 0.0));
    fig8.line_to(Point::new(-2.0 * ppm, -2.0 * ppm));
    fig8.line_to(Point::new(-2.0 * ppm, 0.0));
    fig8.close_path();

    let stats = compute_polygon_stats(&fig8, &scale);
    // Shoelace net area should be ~0.0
    assert!(stats.area_m2.abs() < 1e-4);
    assert!(stats.centroid_canvas.x.is_finite());
    assert!(stats.centroid_canvas.y.is_finite());
}

// =========================================================================
// 3. Section Cut Line Stress Tests
// =========================================================================

#[test]
fn test_section_cut_line_zero_length() {
    // p0 == p1: Zero length cut line must not divide by zero or panic
    let spec = SectionLineSpec {
        p0: Point::new(100.0, 100.0),
        p1: Point::new(100.0, 100.0),
        label_start: "B".to_string(),
        label_end: "B′".to_string(),
        view_direction: ViewDirection::Left,
        ..Default::default()
    };
    let geom = generate_section_line_geometry(&spec);
    assert!(!geom.cut_line.is_empty());
    assert!(!geom.arrows.is_empty());
    assert!(!geom.badges.is_empty());
    assert!(geom.label_start_pos.x.is_finite());
    assert!(geom.label_start_pos.y.is_finite());
    assert!(geom.label_end_pos.x.is_finite());
    assert!(geom.label_end_pos.y.is_finite());
}

#[test]
fn test_section_cut_line_view_direction_reversal() {
    let p0 = Point::new(50.0, 50.0);
    let p1 = Point::new(250.0, 50.0);

    let spec_left = SectionLineSpec { p0, p1, view_direction: ViewDirection::Left, arrow_length_pt: 14.0, ..Default::default() };
    let geom_left = generate_section_line_geometry(&spec_left);

    let spec_right = SectionLineSpec { p0, p1, view_direction: ViewDirection::Right, arrow_length_pt: 14.0, ..Default::default() };
    let geom_right = generate_section_line_geometry(&spec_right);

    // Left sightline normal for vector (200, 0) is (0, -1) -> arrow head points to y - 14
    // Right sightline normal for vector (200, 0) is (0, 1) -> arrow head points to y + 14
    let left_bbox = geom_left.arrows.bounding_box();
    let right_bbox = geom_right.arrows.bounding_box();

    assert!(left_bbox.min_y() < 50.0, "Left sightline arrows must point upward (decreasing y)");
    assert!(right_bbox.max_y() > 50.0, "Right sightline arrows must point downward (increasing y)");
}

// =========================================================================
// 4. Spot Elevation Label Formatting
// =========================================================================

#[test]
fn test_spot_elevation_positive_zero_negative_formatting() {
    let at = Point::new(10.0, 20.0);

    // 1. Positive elevation: must prefix with + and suffix with m
    let pos_spec = SpotElevationSpec { at, elevation_m: 152.35, prefix_plus: true, suffix_m: true, ..Default::default() };
    assert_eq!(generate_spot_elevation_geometry(&pos_spec).text, "+152.35m");

    // 2. Zero elevation: standard convention has no +, formatted to 2 decimals
    let zero_spec = SpotElevationSpec { at, elevation_m: 0.0, prefix_plus: true, suffix_m: true, ..Default::default() };
    assert_eq!(generate_spot_elevation_geometry(&zero_spec).text, "0.00m");

    // 3. Negative elevation: must retain minus sign
    let neg_spec = SpotElevationSpec { at, elevation_m: -12.5, prefix_plus: true, suffix_m: true, ..Default::default() };
    assert_eq!(generate_spot_elevation_geometry(&neg_spec).text, "-12.50m");

    // 4. Two decimal place rounding
    let round_spec = SpotElevationSpec { at, elevation_m: 45.6789, prefix_plus: true, suffix_m: true, ..Default::default() };
    assert_eq!(generate_spot_elevation_geometry(&round_spec).text, "+45.68m");

    // 5. Suffix and prefix disabled
    let no_unit_spec = SpotElevationSpec { at, elevation_m: 100.0, prefix_plus: false, suffix_m: false, ..Default::default() };
    assert_eq!(generate_spot_elevation_geometry(&no_unit_spec).text, "100.00");
}

#[test]
fn test_spot_elevation_marker_kinds() {
    let at = Point::new(10.0, 20.0);
    for kind in [SpotMarkerKind::Dot, SpotMarkerKind::Triangle, SpotMarkerKind::Crosshair] {
        let spec = SpotElevationSpec { at, elevation_m: 10.0, marker: kind, ..Default::default() };
        let geom = generate_spot_elevation_geometry(&spec);
        assert!(!geom.marker_path.is_empty());
        if kind == SpotMarkerKind::Dot {
            assert!(geom.marker_fill);
        } else {
            assert!(!geom.marker_fill);
        }
    }
}

// =========================================================================
// 5. Depth Marker Geometry & Formatting
// =========================================================================

#[test]
fn test_depth_marker_formatting_and_symbols() {
    let at = Point::new(40.0, 40.0);

    // Centimeter posthole concentric
    let cm_spec =
        DepthMarkerSpec { at, depth_val: -35.0, is_cm: true, symbol: DepthMarkerKind::PostholeConcentric, font_size_pt: 7.0, outer_radius_pt: 8.0 };
    let cm_geom = generate_depth_marker_geometry(&cm_spec);
    assert_eq!(cm_geom.text, "-35");
    assert!(!cm_geom.solid_symbol.is_empty());
    assert!(!cm_geom.dashed_symbol.is_empty());

    // Meter cross style
    let m_spec = DepthMarkerSpec { at, depth_val: -1.25, is_cm: false, symbol: DepthMarkerKind::Cross, font_size_pt: 8.0, outer_radius_pt: 10.0 };
    let m_geom = generate_depth_marker_geometry(&m_spec);
    assert_eq!(m_geom.text, "-1.25m");
    assert!(!m_geom.solid_symbol.is_empty());
    assert!(m_geom.dashed_symbol.is_empty());
}

// =========================================================================
// 6. Slope Hachures Stress Testing
// =========================================================================

#[test]
fn test_slope_hachures_zero_length_curves() {
    let spec = SlopeHachureSpec::default();

    // 0 points
    let empty = BezPath::new();
    assert!(generate_slope_hachures(&empty, None, &spec).is_empty());

    // 1 point
    let mut single = BezPath::new();
    single.move_to(Point::new(10.0, 10.0));
    assert!(generate_slope_hachures(&single, None, &spec).is_empty());

    // 2 identical points (length = 0)
    let mut zero_len = BezPath::new();
    zero_len.move_to(Point::new(10.0, 10.0));
    zero_len.line_to(Point::new(10.0, 10.0));
    assert!(generate_slope_hachures(&zero_len, None, &spec).is_empty());
}

#[test]
fn test_slope_hachures_extreme_curvatures_and_ticks() {
    let spec = SlopeHachureSpec { spacing_mm: 1.0, short_ratio: 0.5, default_width_pt: 15.0, ..Default::default() };

    // Extreme zigzag curve
    let mut zigzag = BezPath::new();
    zigzag.move_to(Point::new(0.0, 0.0));
    for i in 1..=50 {
        let y = if i % 2 == 0 { 0.0 } else { 50.0 };
        zigzag.line_to(Point::new(i as f64 * 5.0, y));
    }

    let hachures = generate_slope_hachures(&zigzag, None, &spec);
    assert!(!hachures.is_empty(), "Hachures must generate across sharp zigzag");

    // Verify all segment endpoints are finite
    for el in hachures.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => {
                assert!(p.x.is_finite());
                assert!(p.y.is_finite());
            }
            _ => {}
        }
    }

    // Tight closed circular loop as top path
    let mut loop_path = BezPath::new();
    loop_path.move_to(Point::new(100.0, 50.0));
    loop_path.quad_to(Point::new(150.0, 100.0), Point::new(100.0, 150.0));
    loop_path.quad_to(Point::new(50.0, 100.0), Point::new(100.0, 50.0));
    loop_path.close_path();

    let loop_hachures = generate_slope_hachures(&loop_path, None, &spec);
    assert!(!loop_hachures.is_empty(), "Loop hachures must generate around curved edge");
}

#[test]
fn test_slope_hachures_with_bottom_path_boundary() {
    let spec = SlopeHachureSpec { spacing_mm: 1.5, short_ratio: 0.5, default_width_pt: 30.0, ..Default::default() };

    let mut top = BezPath::new();
    top.move_to(Point::new(0.0, 0.0));
    top.line_to(Point::new(100.0, 0.0));

    let mut bottom = BezPath::new();
    bottom.move_to(Point::new(0.0, 20.0));
    bottom.line_to(Point::new(100.0, 20.0));

    let hachures = generate_slope_hachures(&top, Some(&bottom), &spec);
    assert!(!hachures.is_empty());

    // With bottom curve at y = 20, long ticks should terminate around y = 20
    let bbox = hachures.bounding_box();
    assert!(bbox.max_y() <= 20.5);
}

// =========================================================================
// 7. Engine Command Input Validation & Error Handling
// =========================================================================

#[test]
fn test_engine_trowel_commands_error_handling() {
    let mut s = test_session_with_doc();

    // 1. Missing p0 / p1 in section line
    let err = s.execute("trowel.sectionLine", &json!({})).unwrap_err();
    assert!(err.to_string().contains("p0") || err.to_string().contains("p1"));

    // 2. Missing elevation in spot elevation
    let err = s.execute("trowel.spotElevation", &json!({ "at": [10.0, 10.0] })).unwrap_err();
    assert!(err.to_string().contains("elevation"));

    // 3. Missing depth in depth marker
    let err = s.execute("trowel.depthMarker", &json!({ "at": [10.0, 10.0] })).unwrap_err();
    assert!(err.to_string().contains("depth"));

    // 4. Non-existent node in slope hachures
    let err = s.execute("trowel.slopeHachures", &json!({ "topId": 999999 })).unwrap_err();
    assert!(err.to_string().contains("not found"));

    // 5. Non-existent node in polygonStats
    let err = s.execute("trowel.polygonStats", &json!({ "id": 999999 })).unwrap_err();
    assert!(err.to_string().contains("not found"));
}
