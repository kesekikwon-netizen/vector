//! Adversarial stress test suite for VectorCraft Trowel Milestone M4 (Tasks J010 & J011).
//!
//! Comprehensive empirical challenge testing:
//! 1. J011 Pottery Circumcircle Fitting (unit circle, 184mm benchmark, collinear, coincident, NaN/Inf, small arc, large coordinates)
//! 2. J011 Pottery Geometry (bilateral reflection handle inversion & Bézier continuity, wall thickness offset zero/negative/overlapping, 45° section hatch)
//! 3. J010 Stratigraphy Table & Numbering (boundaries 1-50, >50, Roman 1-10, TSV roundtrip, live paint disjoint/open/exterior, validation diagnostics)

#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, ParamCurve, PathSeg, Point};
use serde_json::json;
use vectorcraft_doc::pottery::SectionSide;
use vectorcraft_doc::stratigraphy::{NumberingStyle, SoilDescription, StratigraphyFillStyle, StratigraphyLayer, StratigraphyTable};
use vectorcraft_doc::{Appearance, Node, NodeId};
use vectorcraft_engine::Session;
use vectorcraft_geom::{Anchor, AnchorKind, CircleFitError, PathData, SubPath, fit_circle_3point};

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 2000, "height": 1000})).unwrap();
    s
}

// ============================================================================
// 1. J011 Circle Fitting Stress Tests
// ============================================================================

#[test]
fn test_circle_fit_unit_circle_and_orientations() {
    // Standard unit circle: (1, 0), (0, 1), (-1, 0)
    let p1 = Point::new(1.0, 0.0);
    let p2 = Point::new(0.0, 1.0);
    let p3 = Point::new(-1.0, 0.0);

    let fit = fit_circle_3point(p1, p2, p3).expect("unit circle fit should succeed");
    assert!((fit.center.x).abs() < 1e-12, "center x must be 0");
    assert!((fit.center.y).abs() < 1e-12, "center y must be 0");
    assert!((fit.radius - 1.0).abs() < 1e-12, "radius must be 1.0");
    assert!((fit.diameter - 2.0).abs() < 1e-12, "diameter must be 2.0");
    assert!((fit.arc_span_deg - 180.0).abs() < 1e-6, "arc span must be 180 deg");
    assert!((fit.rim_preservation_percent - 50.0).abs() < 1e-6, "rim preservation must be 50%");

    // Clockwise traversal: (-1, 0), (0, 1), (1, 0)
    let fit_cw = fit_circle_3point(p3, p2, p1).expect("clockwise fit should succeed");
    assert!((fit_cw.center.x).abs() < 1e-12);
    assert!((fit_cw.center.y).abs() < 1e-12);
    assert!((fit_cw.radius - 1.0).abs() < 1e-12);
    assert!((fit_cw.diameter - 2.0).abs() < 1e-12);
    assert!((fit_cw.arc_span_deg - 180.0).abs() < 1e-6);

    // 90-degree quadrant span: (1, 0), (cos 45°, sin 45°), (0, 1)
    let ang_mid = 45.0_f64.to_radians();
    let p_mid = Point::new(ang_mid.cos(), ang_mid.sin());
    let fit_quad = fit_circle_3point(p1, p_mid, p2).expect("quadrant fit should succeed");
    assert!((fit_quad.radius - 1.0).abs() < 1e-12);
    assert!((fit_quad.arc_span_deg - 90.0).abs() < 1e-6, "arc span must be 90 deg");
    assert!((fit_quad.rim_preservation_percent - 25.0).abs() < 1e-6, "preservation must be 25%");
}

#[test]
fn test_circle_fit_archaeological_benchmark_gwangnyeongri_3() {
    // Benchmark: Gwangnyeong-ri 1 House 1 pottery ③
    // Rim diameter D = 18.4 cm = 184.0 mm, radius R = 92.0 mm
    let r_mm = 92.0;

    let ang1 = 15.0_f64.to_radians();
    let ang2 = 45.0_f64.to_radians();
    let ang3 = 75.0_f64.to_radians();

    let p1 = Point::new(r_mm * ang1.cos(), r_mm * ang1.sin());
    let p2 = Point::new(r_mm * ang2.cos(), r_mm * ang2.sin());
    let p3 = Point::new(r_mm * ang3.cos(), r_mm * ang3.sin());

    let fit = fit_circle_3point(p1, p2, p3).expect("archaeological benchmark fit should succeed");
    assert!((fit.radius - 92.0).abs() < 1e-6, "radius must be 92.0 mm");
    assert!((fit.diameter - 184.0).abs() < 1e-6, "diameter must be 184.0 mm");
    assert!((fit.arc_span_deg - 60.0).abs() < 1e-6, "arc span must be 60 degrees");
    assert!((fit.rim_preservation_percent - (60.0 / 360.0 * 100.0)).abs() < 1e-6);
}

#[test]
fn test_circle_fit_collinear_points_stress() {
    // 1. Horizontal collinear
    let p1 = Point::new(10.0, 50.0);
    let p2 = Point::new(30.0, 50.0);
    let p3 = Point::new(90.0, 50.0);
    let err = fit_circle_3point(p1, p2, p3).unwrap_err();
    assert!(matches!(err, CircleFitError::CollinearPoints { .. }));

    // 2. Vertical collinear
    let p1 = Point::new(100.0, 20.0);
    let p2 = Point::new(100.0, 150.0);
    let p3 = Point::new(100.0, 300.0);
    let err = fit_circle_3point(p1, p2, p3).unwrap_err();
    assert!(matches!(err, CircleFitError::CollinearPoints { .. }));

    // 3. Diagonal collinear
    let p1 = Point::new(-50.0, -50.0);
    let p2 = Point::new(0.0, 0.0);
    let p3 = Point::new(150.0, 150.0);
    let err = fit_circle_3point(p1, p2, p3).unwrap_err();
    assert!(matches!(err, CircleFitError::CollinearPoints { .. }));

    // 4. Near-collinear under threshold (Area < 1e-9)
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(100.0, 1e-12);
    let p3 = Point::new(200.0, 0.0);
    let err = fit_circle_3point(p1, p2, p3).unwrap_err();
    assert!(matches!(err, CircleFitError::CollinearPoints { .. }));
}

#[test]
fn test_circle_fit_degenerate_and_non_finite() {
    // Coincident p1 == p2
    let p = Point::new(42.0, 77.0);
    let p3 = Point::new(100.0, 120.0);
    assert_eq!(fit_circle_3point(p, p, p3), Err(CircleFitError::DegeneratePoints));

    // Coincident p2 == p3
    assert_eq!(fit_circle_3point(p3, p, p), Err(CircleFitError::DegeneratePoints));

    // Coincident p1 == p3
    assert_eq!(fit_circle_3point(p, p3, p), Err(CircleFitError::DegeneratePoints));

    // All coincident
    assert_eq!(fit_circle_3point(p, p, p), Err(CircleFitError::DegeneratePoints));

    // Non-finite NaN
    assert_eq!(fit_circle_3point(Point::new(f64::NAN, 0.0), p, p3), Err(CircleFitError::InvalidCoordinates));
    assert_eq!(fit_circle_3point(p, Point::new(0.0, f64::NAN), p3), Err(CircleFitError::InvalidCoordinates));
    assert_eq!(fit_circle_3point(p, p3, Point::new(f64::NAN, f64::NAN)), Err(CircleFitError::InvalidCoordinates));

    // Non-finite Infinity
    assert_eq!(fit_circle_3point(Point::new(f64::INFINITY, 0.0), p, p3), Err(CircleFitError::InvalidCoordinates));
    assert_eq!(fit_circle_3point(p, Point::new(0.0, f64::NEG_INFINITY), p3), Err(CircleFitError::InvalidCoordinates));
}

#[test]
fn test_circle_fit_small_arc_and_large_coordinates() {
    // 1. Small arc span (sub-millimeter shard: 0.92 mm arc on 184 mm rim)
    let r_mm: f64 = 92.0;
    let ang1: f64 = 0.0;
    let ang2: f64 = 0.005; // 0.286 deg
    let ang3: f64 = 0.010; // 0.573 deg
    let p1 = Point::new(r_mm * ang1.cos(), r_mm * ang1.sin());
    let p2 = Point::new(r_mm * ang2.cos(), r_mm * ang2.sin());
    let p3 = Point::new(r_mm * ang3.cos(), r_mm * ang3.sin());

    let fit = fit_circle_3point(p1, p2, p3).expect("small arc span should fit cleanly");
    assert!((fit.radius - 92.0).abs() < 0.05, "radius should recover ~92mm");
    assert!((fit.diameter - 184.0).abs() < 0.1, "diameter should recover ~184mm");
    assert!((fit.arc_span_deg - 0.010_f64.to_degrees()).abs() < 1e-4);

    // 2. Large coordinates (CAD / Korean TM coordinate range X ~ 500,000, Y ~ 200,000)
    let cx = 500_000.0;
    let cy = 200_000.0;
    let r = 100.0;
    let p1 = Point::new(cx + r, cy);
    let p2 = Point::new(cx, cy + r);
    let p3 = Point::new(cx - r, cy);

    let fit_large = fit_circle_3point(p1, p2, p3).expect("large coordinate fit should succeed");
    assert!((fit_large.center.x - cx).abs() < 1e-6);
    assert!((fit_large.center.y - cy).abs() < 1e-6);
    assert!((fit_large.radius - 100.0).abs() < 1e-6);
    assert!((fit_large.diameter - 200.0).abs() < 1e-6);
}

// ============================================================================
// 2. J011 Pottery Pathops & Geometry Stress Tests
// ============================================================================

#[test]
fn test_pottery_bilateral_reflection_bezier_curve_continuity() {
    let cl_x = 300.0;

    // Create a smooth cubic Bézier profile on the right of centerline
    // Segment from anchor 0 to anchor 1
    let a0 = Anchor { p: Point::new(340.0, 100.0), h_in: Point::new(330.0, 90.0), h_out: Point::new(350.0, 110.0), kind: AnchorKind::Smooth };
    let a1 = Anchor { p: Point::new(380.0, 160.0), h_in: Point::new(370.0, 140.0), h_out: Point::new(390.0, 180.0), kind: AnchorKind::Smooth };
    let a2 = Anchor { p: Point::new(330.0, 240.0), h_in: Point::new(340.0, 220.0), h_out: Point::new(320.0, 260.0), kind: AnchorKind::Smooth };

    let path = PathData { subpaths: vec![SubPath { anchors: vec![a0, a1, a2], closed: false }] };

    let reflected = vectorcraft_pathops::pottery::reflect_path_across_centerline(&path, cl_x);
    let r_sp = &reflected.subpaths[0];
    assert_eq!(r_sp.anchors.len(), 3);

    // Reversed order: r_sp[0] is reflection of a2, r_sp[2] is reflection of a0
    let r0 = &r_sp.anchors[0];
    let r1 = &r_sp.anchors[1];
    let r2 = &r_sp.anchors[2];

    // Verification of coordinate reflection: 2 * 300 - x
    assert_eq!(r0.p, Point::new(2.0 * cl_x - a2.p.x, a2.p.y)); // 270, 240
    assert_eq!(r0.h_in, Point::new(2.0 * cl_x - a2.h_out.x, a2.h_out.y)); // was h_out
    assert_eq!(r0.h_out, Point::new(2.0 * cl_x - a2.h_in.x, a2.h_in.y)); // was h_in

    assert_eq!(r1.p, Point::new(2.0 * cl_x - a1.p.x, a1.p.y)); // 220, 160
    assert_eq!(r2.p, Point::new(2.0 * cl_x - a0.p.x, a0.p.y)); // 260, 100

    // Rigorous curve sampling test:
    // Any point on original curve at t must match point on reflected curve at (1 - t) mirrored horizontally!
    let bez_orig = path.to_bezpath();
    let bez_refl = reflected.to_bezpath();

    let segs_orig: Vec<PathSeg> = bez_orig.segments().collect();
    let segs_refl: Vec<PathSeg> = bez_refl.segments().collect();
    assert_eq!(segs_orig.len(), 2);
    assert_eq!(segs_refl.len(), 2);

    for seg_idx in 0..2 {
        let orig_seg = segs_orig[seg_idx];
        let refl_seg = segs_refl[1 - seg_idx]; // reversed order

        for step in 0..=10 {
            let t = step as f64 / 10.0;
            let pt_orig = orig_seg.eval(t);
            let pt_refl = refl_seg.eval(1.0 - t);

            let expected_refl_x = 2.0 * cl_x - pt_orig.x;
            assert!(
                (pt_refl.x - expected_refl_x).abs() < 1e-6,
                "Bézier horizontal symmetry failed at seg {} t {}: refl.x={}, expected={}",
                seg_idx,
                t,
                pt_refl.x,
                expected_refl_x
            );
            assert!(
                (pt_refl.y - pt_orig.y).abs() < 1e-6,
                "Bézier vertical alignment failed at seg {} t {}: refl.y={}, orig.y={}",
                seg_idx,
                t,
                pt_refl.y,
                pt_orig.y
            );
        }
    }
}

#[test]
fn test_pottery_wall_thickness_offset_edge_cases() {
    let cl_x = 300.0;

    // Profile path: 3 anchors
    let a0 = Anchor::corner(Point::new(350.0, 100.0));
    let a1 = Anchor::corner(Point::new(360.0, 150.0));
    let a2 = Anchor::corner(Point::new(340.0, 200.0));
    let path = PathData { subpaths: vec![SubPath { anchors: vec![a0, a1, a2], closed: false }] };

    // 1. Zero thickness offset
    let off_zero = vectorcraft_pathops::pottery::offset_wall_profile(&path, 0.0, cl_x, SectionSide::Right);
    let sp_zero = &off_zero.subpaths[0];
    assert_eq!(sp_zero.anchors.len(), 3);
    assert!((sp_zero.anchors[0].p.x - 350.0).abs() < 1e-6);
    assert!((sp_zero.anchors[1].p.x - 360.0).abs() < 1e-6);
    assert!((sp_zero.anchors[2].p.x - 340.0).abs() < 1e-6);

    // 2. Negative thickness offset (expands outwards rather than inwards)
    let off_neg = vectorcraft_pathops::pottery::offset_wall_profile(&path, -10.0, cl_x, SectionSide::Right);
    let sp_neg = &off_neg.subpaths[0];
    assert_eq!(sp_neg.anchors.len(), 3);
    // Outward moves away from centerline
    assert!(sp_neg.anchors[0].p.x > 350.0);

    // 3. Excessive thickness clamped to centerline
    // If thickness is 100 pt, 350 - 100 = 250 < 300, so it must clamp to 300
    let off_excess = vectorcraft_pathops::pottery::offset_wall_profile(&path, 100.0, cl_x, SectionSide::Right);
    let sp_excess = &off_excess.subpaths[0];
    for a in &sp_excess.anchors {
        assert!(a.p.x >= cl_x, "offset must not penetrate past centerline");
    }

    // 4. Looping / self-overlapping curve
    let a_loop0 = Anchor::corner(Point::new(350.0, 100.0));
    let a_loop1 = Anchor::corner(Point::new(380.0, 150.0));
    let a_loop2 = Anchor::corner(Point::new(330.0, 150.0));
    let a_loop3 = Anchor::corner(Point::new(360.0, 200.0));
    let loop_path = PathData { subpaths: vec![SubPath { anchors: vec![a_loop0, a_loop1, a_loop2, a_loop3], closed: false }] };
    let off_loop = vectorcraft_pathops::pottery::offset_wall_profile(&loop_path, 8.0, cl_x, SectionSide::Right);
    assert_eq!(off_loop.subpaths[0].anchors.len(), 4, "loop offset must produce valid anchors");
}

#[test]
fn test_pottery_section_hatch_angles_and_line_emission() {
    // Construct a section polygon (box on right side of centerline)
    let p0 = Anchor::corner(Point::new(300.0, 100.0));
    let p1 = Anchor::corner(Point::new(350.0, 100.0));
    let p2 = Anchor::corner(Point::new(350.0, 200.0));
    let p3 = Anchor::corner(Point::new(300.0, 200.0));

    let poly = PathData { subpaths: vec![SubPath { anchors: vec![p0, p1, p2, p3], closed: true }] };

    // 45° hatch lines
    let hatch_45 = vectorcraft_pathops::pottery::generate_section_hatch(&poly, 1.0, 0.1, 45.0);
    assert!(!hatch_45.is_empty(), "45° hatch lines must be emitted");
    let segs_45: Vec<_> = hatch_45.segments().collect();
    assert!(segs_45.len() > 10, "1.0mm spacing in 50x100pt box should produce multiple lines");

    // 135° hatch lines
    let hatch_135 = vectorcraft_pathops::pottery::generate_section_hatch(&poly, 1.0, 0.1, 135.0);
    assert!(!hatch_135.is_empty(), "135° hatch lines must be emitted");

    // Empty polygon
    let empty_poly = PathData::default();
    let hatch_empty = vectorcraft_pathops::pottery::generate_section_hatch(&empty_poly, 1.0, 0.1, 45.0);
    assert!(hatch_empty.is_empty(), "empty polygon must yield empty hatch");
}

// ============================================================================
// 3. J010 Stratigraphy Table & Live Paint Stress Tests
// ============================================================================

#[test]
fn test_stratigraphy_numbering_styles_boundary_adversarial() {
    // Circled boundary conditions:
    // 1..=20: 0x2460 + n - 1 (①..⑳)
    assert_eq!(NumberingStyle::Circled.format(1), "①");
    assert_eq!(NumberingStyle::Circled.format(2), "②");
    assert_eq!(NumberingStyle::Circled.format(20), "⑳");

    // 21..=35: 0x3251 + n - 21 (㉑..㉟)
    assert_eq!(NumberingStyle::Circled.format(21), "㉑");
    assert_eq!(NumberingStyle::Circled.format(35), "㉟");

    // 36..=50: 0x32B1 + n - 36 (㊱..㊿)
    assert_eq!(NumberingStyle::Circled.format(36), "㊱");
    assert_eq!(NumberingStyle::Circled.format(50), "㊿");

    // Boundaries > 50: format as "(N)"
    assert_eq!(NumberingStyle::Circled.format(51), "(51)");
    assert_eq!(NumberingStyle::Circled.format(99), "(99)");
    assert_eq!(NumberingStyle::Circled.format(1000), "(1000)");

    // Boundary n = 0
    assert_eq!(NumberingStyle::Circled.format(0), "(0)");

    // Roman numerals:
    assert_eq!(NumberingStyle::Roman.format(1), "Ⅰ");
    assert_eq!(NumberingStyle::Roman.format(4), "Ⅳ");
    assert_eq!(NumberingStyle::Roman.format(9), "Ⅸ");
    assert_eq!(NumberingStyle::Roman.format(10), "Ⅹ");
    assert_eq!(NumberingStyle::Roman.format(11), "11");
    assert_eq!(NumberingStyle::Roman.format(0), "0");

    // Arabic numbers:
    assert_eq!(NumberingStyle::Arabic.format(0), "0");
    assert_eq!(NumberingStyle::Arabic.format(1), "1");
    assert_eq!(NumberingStyle::Arabic.format(12345), "12345");
}

#[test]
fn test_stratigraphy_tsv_export_roundtrip_and_empty_fields() {
    // 1. Completely empty table
    let empty_table = StratigraphyTable {
        id: "empty".into(),
        title: "빈 토층표".into(),
        section_line_id: None,
        numbering_style: NumberingStyle::Circled,
        layers: Vec::new(),
        placement: None,
    };
    let empty_tsv = empty_table.to_tsv();
    assert_eq!(empty_tsv, "층위\t토색\t먼셀\t토성\t포함유물\t비고\n");

    // 2. Table with empty description strings
    let table_with_empty_row = StratigraphyTable {
        id: "sparse".into(),
        title: "부분 기록 표".into(),
        section_line_id: None,
        numbering_style: NumberingStyle::Circled,
        layers: vec![
            StratigraphyLayer {
                index: 1,
                custom_badge: None,
                description: SoilDescription::default(), // all empty strings
                fill_style: StratigraphyFillStyle::None,
                badge_node_id: None,
                face_node_id: None,
            },
            StratigraphyLayer {
                index: 52, // > 50
                custom_badge: Some("유구바닥".into()),
                description: SoilDescription {
                    color_name: "명황갈색".into(),
                    munsell: "10YR 6/6".into(),
                    texture: "".into(),
                    inclusions: "".into(),
                    notes: "소토 반점 다량".into(),
                },
                fill_style: StratigraphyFillStyle::None,
                badge_node_id: None,
                face_node_id: None,
            },
        ],
        placement: None,
    };

    let tsv = table_with_empty_row.to_tsv();
    let lines: Vec<&str> = tsv.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "층위\t토색\t먼셀\t토성\t포함유물\t비고");
    assert_eq!(lines[1], "①\t\t\t\t\t");
    assert_eq!(lines[2], "유구바닥\t명황갈색\t10YR 6/6\t\t\t소토 반점 다량");
}

#[test]
fn test_stratigraphy_live_paint_fill_adversarial() {
    let mut s = test_session();

    // 1. Live paint fill with NO curves in document
    let err_no_curves = s
        .execute(
            "stratigraphy.livePaintFill",
            &json!({
                "point": [150.0, 150.0],
            }),
        )
        .unwrap_err();
    assert!(err_no_curves.to_string().contains("no vector curves available"));

    // 2. Open / disjoint lines that do not form a closed face
    let mut open_line1 = BezPath::new();
    open_line1.move_to((100.0, 100.0));
    open_line1.line_to((300.0, 100.0));

    let mut open_line2 = BezPath::new();
    open_line2.move_to((100.0, 200.0));
    open_line2.line_to((300.0, 200.0));

    s.edit("Add Open Lines", |doc, _| {
        let id1 = doc.alloc_id();
        let n1 = Node::path(id1, PathData::from_bezpath(&open_line1), Appearance::default());
        doc.insert(None, usize::MAX, n1).unwrap();

        let id2 = doc.alloc_id();
        let n2 = Node::path(id2, PathData::from_bezpath(&open_line2), Appearance::default());
        doc.insert(None, usize::MAX, n2).unwrap();
        Ok(())
    })
    .unwrap();

    let err_unclosed = s
        .execute(
            "stratigraphy.livePaintFill",
            &json!({
                "point": [150.0, 150.0],
            }),
        )
        .unwrap_err();
    assert!(err_unclosed.to_string().contains("does not lie inside any bounded planar face"));

    // 3. Add closed box and test click OUTSIDE bounded face
    let mut closed_box = BezPath::new();
    closed_box.move_to((100.0, 100.0));
    closed_box.line_to((200.0, 100.0));
    closed_box.line_to((200.0, 200.0));
    closed_box.line_to((100.0, 200.0));
    closed_box.close_path();

    s.edit("Add Closed Box", |doc, _| {
        let id = doc.alloc_id();
        let n = Node::path(id, PathData::from_bezpath(&closed_box), Appearance::default());
        doc.insert(None, usize::MAX, n).unwrap();
        Ok(())
    })
    .unwrap();

    // Click outside at (500, 500)
    let err_exterior = s
        .execute(
            "stratigraphy.livePaintFill",
            &json!({
                "point": [500.0, 500.0],
            }),
        )
        .unwrap_err();
    assert!(err_exterior.to_string().contains("does not lie inside any bounded planar face"));

    // Click INSIDE at (150, 150) with hatch fill
    let fill_hatch = s
        .execute(
            "stratigraphy.livePaintFill",
            &json!({
                "point": [150.0, 150.0],
                "fillStyle": {
                    "angleDeg": 45.0,
                    "spacingMm": 1.0,
                    "weightPt": 0.2
                }
            }),
        )
        .unwrap();
    assert_eq!(fill_hatch["isHatch"], true);
    assert!(fill_hatch["faceId"].as_u64().is_some());
    assert!((fill_hatch["area"].as_f64().unwrap() - 10000.0).abs() < 1e-3);
}

#[test]
fn test_stratigraphy_validate_diagnostics_adversarial() {
    let mut s = test_session();

    // 1. Initial empty document validation
    let val0 = s.execute("stratigraphy.validate", &json!({})).unwrap();
    assert_eq!(val0["valid"], false);
    assert!(val0["warnings"].as_array().unwrap()[0].as_str().unwrap().contains("정의되어 있지 않습니다"));

    // 2. Add layer with empty description
    s.execute(
        "stratigraphy.addLayer",
        &json!({
            "colorName": "",
            "texture": "",
        }),
    )
    .unwrap();

    let val1 = s.execute("stratigraphy.validate", &json!({})).unwrap();
    assert_eq!(val1["valid"], false);
    let missing_rows = val1["missingTableRows"].as_array().unwrap();
    assert_eq!(missing_rows.len(), 1);
    assert_eq!(missing_rows[0], 1);

    // 3. Add layer with invalid/deleted badge node ID
    let ghost_id = NodeId(88888);
    s.edit("Inject Ghost Badge Node", |doc, _| {
        doc.stratigraphy_tables[0].layers[0].badge_node_id = Some(ghost_id);
        doc.stratigraphy_tables[0].layers[0].description.color_name = "흑갈색".into();
        doc.stratigraphy_tables[0].layers[0].description.texture = "점토".into();
        Ok(())
    })
    .unwrap();

    let val2 = s.execute("stratigraphy.validate", &json!({})).unwrap();
    assert_eq!(val2["valid"], false);
    let warnings = val2["warnings"].as_array().unwrap();
    assert!(warnings.iter().any(|w| w.as_str().unwrap().contains("도면에서 누락되었습니다")));

    // 4. Place an orphan badge node on canvas that references a number not in table
    s.edit("Add Orphan Badge Node", |doc, _| {
        let orphan_id = doc.alloc_id();
        let mut n = Node::path(orphan_id, PathData::default(), Appearance::default());
        n.name = Some("토층 번호 99".to_string());
        doc.insert(None, usize::MAX, n).unwrap();
        Ok(())
    })
    .unwrap();

    let val3 = s.execute("stratigraphy.validate", &json!({})).unwrap();
    assert_eq!(val3["valid"], false);
    let orphans = val3["orphanBadges"].as_array().unwrap();
    assert!(orphans.iter().any(|o| o == 99));
}
