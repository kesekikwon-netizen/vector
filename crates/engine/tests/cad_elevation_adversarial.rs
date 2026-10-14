//! Adversarial Challenge & Stress-Test Suite for Tasks J007 (CAD DXF & desca) and J015 (Elevation Lines & 1D Clipping).
//!
//! Evaluates:
//! 1. J007: TM 200,000m / 500,000m real-world coordinate preservation without origin recentering,
//!    CP949 Korean layer names, multi-line MTEXT, mixed ASCII/CP949 decoding, and desca open vs closed SECTION_PROFILE classification.
//! 2. J015: 1D scanline void clipping boundary conditions (peak/valley touching, multiple obstacles,
//!    complex trench with postholes and stones, above-ceiling and below-trench empty emission),
//!    and reactive post-edit dynamic refresh on profile and obstacle geometry mutations.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::BezPath;
use serde_json::json;
use vectorcraft_cad::{CadEncoding, ImportOptions};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::LegendSample;
use vectorcraft_doc::{Appearance, Node, NodeId, NodeKind};
use vectorcraft_engine::Session;
use vectorcraft_geom::PathData;
use vectorcraft_pathops::elevation::clip_elevation_scanline;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 2000, "height": 1000})).unwrap();
    s
}

// =========================================================================
// PART 1: TASK J007 — CAD DXF COORDINATES, TEXT ENCODING & DESCA
// =========================================================================

/// Constructs a minimal valid ASCII DXF file containing the specified header,
/// layers, and entities.
fn make_dxf(header_vars: &[(i32, &str)], layers: &[&str], entities: &[(i32, &str)]) -> Vec<u8> {
    let mut out = String::new();
    out.push_str("  0\nSECTION\n  2\nHEADER\n");
    for (code, val) in header_vars {
        out.push_str(&format!("{code:>3}\n{val}\n"));
    }
    out.push_str("  0\nENDSEC\n");

    if !layers.is_empty() {
        out.push_str("  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n");
        for layer in layers {
            out.push_str(&format!("  0\nLAYER\n  2\n{layer}\n 70\n0\n 62\n7\n"));
        }
        out.push_str("  0\nENDTAB\n  0\nENDSEC\n");
    }

    out.push_str("  0\nSECTION\n  2\nENTITIES\n");
    for (code, val) in entities {
        out.push_str(&format!("{code:>3}\n{val}\n"));
    }
    out.push_str("  0\nENDSEC\n  0\nEOF\n");
    out.into_bytes()
}

/// 1.1 Stress test: DXF entities at large real-world TM coordinates (Easting 200,000m, Northing 500,000m)
/// must preserve floating point precision and NOT get recentered to origin when preserve_coords is true.
#[test]
fn test_j007_tm_coordinates_preservation_and_non_recentering() {
    let east0 = 200_000.12345;
    let north0 = 500_000.67890;
    let east1 = 200_050.54321;
    let north1 = 500_080.98765;

    // DXF in meters ($INSUNITS = 6)
    let header = [(9, "$INSUNITS"), (70, "6")];
    let entities = [(0, "LINE"), (8, "SURVEY_LINE"), (10, "200000.12345"), (20, "500000.67890"), (11, "200050.54321"), (21, "500080.98765")];
    let dxf_bytes = make_dxf(&header, &["SURVEY_LINE"], &entities);

    // Case A: preserve_coords = true (Scale 1:20)
    let opts_preserve = ImportOptions { preserve_coords: true, scale: Some(20.0), center: false, ..ImportOptions::default() };
    let imp_preserve = vectorcraft_cad::import(&dxf_bytes, &opts_preserve).expect("DXF import with preserve_coords should succeed");
    let doc_preserve = &imp_preserve.document;

    // Verify doc scale configuration exists
    let scale_cfg = doc_preserve.scale.as_ref().expect("scale config must be bound");
    assert_eq!(scale_cfg.scale.denom, 20.0);

    // Verify world origin preserves the large TM coordinate magnitude
    assert!((scale_cfg.world_origin.0 - 200_000.0).abs() < 1.0, "world_origin.0 must anchor near Easting 200,000");
    assert!((scale_cfg.world_origin.1 - 500_000.0).abs() < 1.0, "world_origin.1 must anchor near Northing 500,000");

    // Extract line geometry
    let mut flat_pts = Vec::new();
    doc_preserve.walk(|n| {
        if let NodeKind::Path { path, .. } = &n.kind {
            let bp = path.to_bezpath();
            for el in bp.elements() {
                match el {
                    kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => flat_pts.push(*p),
                    _ => {}
                }
            }
        }
    });
    assert_eq!(flat_pts.len(), 2, "must have start and end points");

    // Verify round-trip conversion canvas -> world recovers original TM coordinates with sub-millimeter precision (< 1e-4 m)
    let (rec_e0, rec_n0) = scale_cfg.canvas_to_world(flat_pts[0]);
    let (rec_e1, rec_n1) = scale_cfg.canvas_to_world(flat_pts[1]);

    assert!((rec_e0 - east0).abs() < 1e-4, "Recovered East0 error: {}", (rec_e0 - east0).abs());
    assert!((rec_n0 - north0).abs() < 1e-4, "Recovered North0 error: {}", (rec_n0 - north0).abs());
    assert!((rec_e1 - east1).abs() < 1e-4, "Recovered East1 error: {}", (rec_e1 - east1).abs());
    assert!((rec_n1 - north1).abs() < 1e-4, "Recovered North1 error: {}", (rec_n1 - north1).abs());

    // Case B: preserve_coords = false (normal CAD import centers art on artboard)
    let opts_standard = ImportOptions { preserve_coords: false, center: true, scale: Some(20.0), ..ImportOptions::default() };
    let imp_standard = vectorcraft_cad::import(&dxf_bytes, &opts_standard).expect("DXF import standard should succeed");
    assert!(imp_standard.document.scale.is_none(), "standard CAD import should not set TM scale config");
}

/// 1.2 Stress test: Korean layer names, multi-line text, and mixed ASCII/CP949 bytes decode cleanly without mojibake or panic.
#[test]
fn test_j007_korean_text_encoding_and_multiline_stress() {
    // "토층선_트렌치1" in CP949:
    // 토 = 0xC5 0xE4, 층 = 0xC3 0xFE, 선 = 0xBC 0xB1, _ = 0x5F, 트 = 0xC6 0xAE, 렌 = 0xB7 0xBB, 치 = 0xC4 0xA1, 1 = 0x31
    let layer_name_bytes = [0xC5, 0xE4, 0xC3, 0xFE, 0xBC, 0xB1, 0x5F, 0xC6, 0xAE, 0xB7, 0xBB, 0xC4, 0xA1, 0x31];

    // Construct DXF using raw bytes
    let mut raw_dxf = Vec::new();
    raw_dxf.extend_from_slice(b"  0\nSECTION\n  2\nHEADER\n  9\n$DWGCODEPAGE\n  3\nANSI_949\n  0\nENDSEC\n");
    raw_dxf.extend_from_slice(b"  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n  0\nLAYER\n  2\n");
    raw_dxf.extend_from_slice(&layer_name_bytes);
    raw_dxf.extend_from_slice(b"\n 70\n0\n 62\n1\n  0\nENDTAB\n  0\nENDSEC\n");
    raw_dxf.extend_from_slice(b"  0\nSECTION\n  2\nENTITIES\n");

    // Entity 1: Mixed ASCII + CP949 text: "P.H. 1호 토층 A-A' (EL. 152.5m)"
    // "호" = 0xC8 0xA3, " " = 0x20, "토층" = 0xC5 0xE4 0xC3 0xFE
    let mut mixed_text = Vec::new();
    mixed_text.extend_from_slice(b"P.H. 1");
    mixed_text.extend_from_slice(&[0xC8, 0xA3, 0x20, 0xC5, 0xE4, 0xC3, 0xFE]);
    mixed_text.extend_from_slice(b" A-A' (EL. 152.5m)");

    raw_dxf.extend_from_slice(b"  0\nTEXT\n  8\n");
    raw_dxf.extend_from_slice(&layer_name_bytes);
    raw_dxf.extend_from_slice(b"\n 10\n10.0\n 20\n10.0\n 40\n2.5\n  1\n");
    raw_dxf.extend_from_slice(&mixed_text);
    raw_dxf.extend_from_slice(b"\n");

    // Entity 2: Multiline MTEXT with Korean text: "1호 주거지\n북벽 토층\n축척 1:20"
    // "1호 주거지" = "1" + 0xC8 0xA3 + " " + 0xC1 0xD6 + 0xB0 0xC5 + 0xC1 0xF6
    // "북벽 토층" = 0xBA 0xCF + 0xBA 0xAE + " " + 0xC5 0xE4 + 0xC3 0xFE
    // "축척 1:20" = 0xC3 0xE0 + 0xC3 0xB4 + " 1:20"
    let mut mtext_bytes = Vec::new();
    mtext_bytes.extend_from_slice(b"1");
    mtext_bytes.extend_from_slice(&[0xC8, 0xA3, 0x20, 0xC1, 0xD6, 0xB0, 0xC5, 0xC1, 0xF6]);
    mtext_bytes.extend_from_slice(b"\\P");
    mtext_bytes.extend_from_slice(&[0xBA, 0xCF, 0xBA, 0xAE, 0x20, 0xC5, 0xE4, 0xC3, 0xFE]);
    mtext_bytes.extend_from_slice(b"\\P");
    mtext_bytes.extend_from_slice(&[0xC3, 0xE0, 0xC3, 0xB4]);
    mtext_bytes.extend_from_slice(b" 1:20");

    raw_dxf.extend_from_slice(b"  0\nMTEXT\n  8\n");
    raw_dxf.extend_from_slice(&layer_name_bytes);
    raw_dxf.extend_from_slice(b"\n 10\n20.0\n 20\n20.0\n 40\n3.0\n  1\n");
    raw_dxf.extend_from_slice(&mtext_bytes);
    raw_dxf.extend_from_slice(b"\n");

    raw_dxf.extend_from_slice(b"  0\nENDSEC\n  0\nEOF\n");

    // Import with CadEncoding::Cp949
    let opts = ImportOptions { encoding: Some(CadEncoding::Cp949), ..ImportOptions::default() };
    let imp = vectorcraft_cad::import(&raw_dxf, &opts).expect("CP949 DXF import must succeed without panic");
    let doc = &imp.document;

    // Check layer name was decoded cleanly without mojibake
    let target_layer = doc.layers.iter().find(|l| l.name.as_deref().unwrap_or("").contains("토층선"));
    assert!(
        target_layer.is_some(),
        "Layer name must decode to '토층선_트렌치1', found layers: {:?}",
        doc.layers.iter().map(|l| l.name.as_deref()).collect::<Vec<_>>()
    );
    assert_eq!(target_layer.unwrap().name.as_deref(), Some("토층선_트렌치1"));

    // Check text entities
    let mut texts: Vec<String> = Vec::new();
    doc.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });

    assert!(texts.iter().any(|t| t.contains("1호 토층")), "TEXT entity must contain '1호 토층': {:?}", texts);
    assert!(texts.iter().any(|t| t.contains("주거지")), "MTEXT entity must contain '주거지': {:?}", texts);
}

/// 1.3 Stress test: desca layers reclassification (open vs closed SECTION_PROFILE, LEVEL_*, SECTION_IMAGE, SECTION_AXIS).
#[test]
fn test_j007_desca_open_vs_closed_section_profile_reclassification() {
    // Create DXF with open polyline and closed polyline on SECTION_PROFILE layer
    let mut dxf_str = String::new();
    dxf_str.push_str("  0\nSECTION\n  2\nHEADER\n  0\nENDSEC\n");
    dxf_str.push_str("  0\nSECTION\n  2\nENTITIES\n");

    // Open polyline on SECTION_PROFILE (ground line: 70 = 0)
    dxf_str.push_str("  0\nLWPOLYLINE\n  8\nSECTION_PROFILE\n 90\n3\n 70\n0\n");
    dxf_str.push_str(" 10\n0.0\n 20\n100.0\n 10\n100.0\n 20\n50.0\n 10\n200.0\n 20\n100.0\n");

    // Closed polyline on SECTION_PROFILE (stone obstacle: 70 = 1)
    dxf_str.push_str("  0\nLWPOLYLINE\n  8\nSECTION_PROFILE\n 90\n4\n 70\n1\n");
    dxf_str.push_str(" 10\n50.0\n 20\n70.0\n 10\n70.0\n 20\n70.0\n 10\n70.0\n 20\n90.0\n 10\n50.0\n 20\n90.0\n");

    // LEVEL_10CM and LEVEL_50CM (desca original elevation lines)
    dxf_str.push_str("  0\nLINE\n  8\nLEVEL_10CM\n 10\n0.0\n 20\n60.0\n 11\n200.0\n 21\n60.0\n");
    dxf_str.push_str("  0\nLINE\n  8\nLEVEL_50CM\n 10\n0.0\n 20\n80.0\n 11\n200.0\n 21\n80.0\n");

    // LEVEL_TEXT with "EL. 152.5"
    dxf_str.push_str("  0\nTEXT\n  8\nLEVEL_TEXT\n 10\n0.0\n 20\n80.0\n 40\n2.5\n  1\nEL. 152.5\n");

    // SECTION_TITLE with "트렌치 1호 A-A' 1:20"
    dxf_str.push_str("  0\nTEXT\n  8\nSECTION_TITLE\n 10\n0.0\n 20\n120.0\n 40\n5.0\n  1\n트렌치 1호 A-A' 1:20\n");

    dxf_str.push_str("  0\nENDSEC\n  0\nEOF\n");

    let opts = ImportOptions { desca: true, ..ImportOptions::default() };
    let imp = vectorcraft_cad::import(dxf_str.as_bytes(), &opts).expect("desca DXF import must succeed");
    let doc = &imp.document;

    // Check title and scale extracted from SECTION_TITLE
    assert_eq!(doc.title, "트렌치 1호 A-A'");
    assert_eq!(doc.scale.as_ref().map(|s| s.scale.denom), Some(20.0));

    // Verify open polyline mapped to "단면선 (잘린 땅)"
    let ground_layer = doc.layers.iter().find(|l| l.name.as_deref() == Some("단면선 (잘린 땅)"));
    assert!(ground_layer.is_some(), "Open SECTION_PROFILE must map to '단면선 (잘린 땅)'");
    let gl = ground_layer.unwrap();
    assert!(gl.visible);
    if let NodeKind::Layer { legend, .. } = &gl.kind {
        let leg = legend.as_ref().expect("legend must be bound");
        assert_eq!(leg.weight, Some(0.8));
        assert_eq!(leg.sample, LegendSample::Line);
    }

    // Verify closed polyline mapped to "잘린 돌·유물·기와"
    let stone_layer = doc.layers.iter().find(|l| l.name.as_deref() == Some("잘린 돌·유물·기와"));
    assert!(stone_layer.is_some(), "Closed SECTION_PROFILE must map to '잘린 돌·유물·기와'");
    let sl = stone_layer.unwrap();
    assert!(sl.visible);
    if let NodeKind::Layer { legend, .. } = &sl.kind {
        let leg = legend.as_ref().expect("legend must be bound");
        assert_eq!(leg.weight, Some(0.8));
        assert_eq!(leg.sample, LegendSample::Area);
    }

    // Verify LEVEL_* lines mapped to "desca 원본" (hidden, template, non-printable)
    let desca_orig_layer = doc.layers.iter().find(|l| l.name.as_deref() == Some("desca 원본"));
    assert!(desca_orig_layer.is_some(), "LEVEL_* lines must map to 'desca 원본'");
    let dol = desca_orig_layer.unwrap();
    assert!(!dol.visible, "desca 원본 must be hidden");
    if let NodeKind::Layer { template, printable, .. } = &dol.kind {
        assert!(template, "desca 원본 must be template");
        assert!(!printable, "desca 원본 must be non-printable");
    }

    // Verify LEVEL_TEXT normalized to "152.5m"
    let text_layer = doc.layers.iter().find(|l| l.name.as_deref() == Some("해발 숫자"));
    assert!(text_layer.is_some(), "LEVEL_TEXT must map to '해발 숫자'");
    let mut texts = Vec::new();
    text_layer.unwrap().walk(&mut |n: &Node| {
        if let NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });
    assert!(texts.contains(&"152.5m".to_string()), "LEVEL_TEXT must normalize 'EL. 152.5' to '152.5m': got {:?}", texts);
}

// =========================================================================
// PART 2: TASK J015 — ELEVATION LINES & 1D VOID CLIPPING BOUNDARY TESTS
// =========================================================================

/// 2.1 Boundary stress: Scanline exactly touching profile peaks or valleys
#[test]
fn test_j015_scanline_touching_peaks_and_valleys() {
    // Ground profile with a sharp central peak at (200, 100) and valley at (400, 300)
    let mut profile = BezPath::new();
    profile.move_to((0.0, 200.0));
    profile.line_to((100.0, 200.0));
    profile.line_to((200.0, 100.0)); // Peak at y = 100
    profile.line_to((300.0, 200.0));
    profile.line_to((400.0, 300.0)); // Valley at y = 300
    profile.line_to((500.0, 200.0));
    profile.line_to((600.0, 200.0));

    let ceiling_y = 50.0;
    let x_min = 0.0;
    let x_max = 600.0;

    // Case 1: Scanline exactly at peak (y = 100.0)
    let res_peak = clip_elevation_scanline(100.0, &profile, &[], x_min, x_max, ceiling_y);
    assert!(!res_peak.is_empty(), "Scanline at peak must emit open void");
    let total_len: f64 = res_peak.iter().map(|(a, b)| b - a).sum();
    assert!((total_len - 600.0).abs() < 1.0, "Total void width should span [0, 600], got {total_len}");

    // Case 2: Scanline exactly at valley floor (y = 300.0)
    let res_valley = clip_elevation_scanline(300.0, &profile, &[], x_min, x_max, ceiling_y);
    // Valley touches at exactly (400, 300).
    // Zero-length or sub-threshold tangent segment (< 0.5 pt) must be dropped
    for (a, b) in &res_valley {
        assert!(*b - *a >= 0.5, "Any emitted segment must be >= 0.5 pt");
    }

    // Case 3: Scanline slightly above valley floor (y = 290.0)
    let res_above_valley = clip_elevation_scanline(290.0, &profile, &[], x_min, x_max, ceiling_y);
    assert_eq!(res_above_valley.len(), 1, "Must emit single small interval inside valley notch");
    assert!(res_above_valley[0].0 > 380.0 && res_above_valley[0].1 < 420.0);
}

/// 2.2 Boundary stress: Scanline cutting through multiple obstacle polygons (disjoint segments)
#[test]
fn test_j015_multiple_obstacle_cutting_disjoint_segments() {
    // Wide trench at y = 200 spanning x = 0..1000
    let mut profile = BezPath::new();
    profile.move_to((0.0, 200.0));
    profile.line_to((1000.0, 200.0));

    // 3 stone obstacles spaced along the trench floor:
    // Stone 1: [150, 250], y = 120..200
    // Stone 2: [350, 450], y = 120..200
    // Stone 3: [600, 750], y = 120..200
    let make_rect = |x0: f64, x1: f64| {
        let mut p = BezPath::new();
        p.move_to((x0, 120.0));
        p.line_to((x1, 120.0));
        p.line_to((x1, 200.0));
        p.line_to((x0, 200.0));
        p.close_path();
        p
    };

    let s1 = make_rect(150.0, 250.0);
    let s2 = make_rect(350.0, 450.0);
    let s3 = make_rect(600.0, 750.0);
    let obstacles = vec![s1, s2, s3];

    // Scanline at y = 150.0 cutting through all 3 stones
    let res = clip_elevation_scanline(150.0, &profile, &obstacles, 0.0, 1000.0, 50.0);

    // Expect 4 disjoint void segments: [0, 150], [250, 350], [450, 600], [750, 1000]
    assert_eq!(res.len(), 4, "Must split into exactly 4 segments around 3 obstacles: got {:?}", res);
    assert!((res[0].0 - 0.0).abs() < 1e-3 && (res[0].1 - 150.0).abs() < 1e-3);
    assert!((res[1].0 - 250.0).abs() < 1e-3 && (res[1].1 - 350.0).abs() < 1e-3);
    assert!((res[2].0 - 450.0).abs() < 1e-3 && (res[2].1 - 600.0).abs() < 1e-3);
    assert!((res[3].0 - 750.0).abs() < 1e-3 && (res[3].1 - 1000.0).abs() < 1e-3);

    // Scanline above the stones at y = 100.0: uncut span [0, 1000]
    let res_above = clip_elevation_scanline(100.0, &profile, &obstacles, 0.0, 1000.0, 50.0);
    assert_eq!(res_above.len(), 1);
    assert!((res_above[0].0 - 0.0).abs() < 1e-3 && (res_above[0].1 - 1000.0).abs() < 1e-3);
}

/// 2.3 Boundary stress: Scanline completely above ceiling or completely below trench bottom
#[test]
fn test_j015_completely_above_ceiling_and_completely_below_bottom() {
    let mut profile = BezPath::new();
    profile.move_to((100.0, 200.0));
    profile.line_to((200.0, 200.0));
    profile.line_to((250.0, 400.0)); // Trench floor at y = 400
    profile.line_to((350.0, 400.0));
    profile.line_to((400.0, 200.0));
    profile.line_to((500.0, 200.0));

    let ceiling_y = 100.0;
    let x_min = 100.0;
    let x_max = 500.0;

    // Scanline above ceiling at y = 50.0: MUST emit 0 segments
    let res_above = clip_elevation_scanline(50.0, &profile, &[], x_min, x_max, ceiling_y);
    assert!(res_above.is_empty(), "Scanline above ceiling must emit 0 segments, got {:?}", res_above);

    // Scanline below trench floor at y = 450.0: MUST emit 0 segments
    let res_below = clip_elevation_scanline(450.0, &profile, &[], x_min, x_max, ceiling_y);
    assert!(res_below.is_empty(), "Scanline below trench bottom must emit 0 segments, got {:?}", res_below);
}

/// 2.4 Deep trench with dual postholes and cut stones (complex archaeological section)
#[test]
fn test_j015_deep_trench_with_postholes_and_cut_stones() {
    // Section geometry:
    // Ground at y = 150.
    // Trench steps down to y = 300 (floor).
    // Posthole 1 at x = 320..360 reaches y = 450.
    // Posthole 2 at x = 640..680 reaches y = 450.
    let mut profile = BezPath::new();
    profile.move_to((0.0, 150.0));
    profile.line_to((200.0, 150.0));
    profile.line_to((250.0, 300.0));
    profile.line_to((320.0, 300.0));
    profile.line_to((320.0, 450.0));
    profile.line_to((360.0, 450.0));
    profile.line_to((360.0, 300.0));
    profile.line_to((640.0, 300.0));
    profile.line_to((640.0, 450.0));
    profile.line_to((680.0, 450.0));
    profile.line_to((680.0, 300.0));
    profile.line_to((750.0, 300.0));
    profile.line_to((800.0, 150.0));
    profile.line_to((1000.0, 150.0));

    // Cut stone resting on trench floor between postholes: x = 450..550, y = 240..300
    let mut stone = BezPath::new();
    stone.move_to((450.0, 240.0));
    stone.line_to((550.0, 240.0));
    stone.line_to((550.0, 300.0));
    stone.line_to((450.0, 300.0));
    stone.close_path();

    let obstacles = vec![stone];
    let ceiling_y = 50.0;

    // 1. Level in postholes below trench floor: y = 380.0
    let res_postholes = clip_elevation_scanline(380.0, &profile, &obstacles, 0.0, 1000.0, ceiling_y);
    assert_eq!(res_postholes.len(), 2, "Must emit exactly two intervals inside postholes");
    assert!((res_postholes[0].0 - 320.0).abs() < 1e-3 && (res_postholes[0].1 - 360.0).abs() < 1e-3);
    assert!((res_postholes[1].0 - 640.0).abs() < 1e-3 && (res_postholes[1].1 - 680.0).abs() < 1e-3);

    // 2. Level in trench cutting through stone: y = 270.0
    // Trench spans from left slope (x ≈ 240) to right slope (x ≈ 760), split by stone [450, 550]
    let res_trench = clip_elevation_scanline(270.0, &profile, &obstacles, 0.0, 1000.0, ceiling_y);
    assert_eq!(res_trench.len(), 2, "Trench level must be split by cut stone into 2 intervals");
    assert!(res_trench[0].1 <= 450.0 + 1e-3);
    assert!(res_trench[1].0 >= 550.0 - 1e-3);
}

/// 2.5 Dynamic update: Editing ground profile or obstacles triggers automatic re-clipping via Session::edit hook
#[test]
fn test_j015_dynamic_refresh_on_profile_and_obstacle_edit() {
    let mut s = test_session();

    // 1. Create initial profile
    let mut profile = BezPath::new();
    profile.move_to((100.0, 150.0));
    profile.line_to((200.0, 150.0));
    profile.line_to((250.0, 300.0));
    profile.line_to((450.0, 300.0));
    profile.line_to((500.0, 150.0));
    profile.line_to((600.0, 150.0));

    let pd_prof = PathData::from_bezpath(&profile);
    let app_prof = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 1.0);
    let mut prof_id = NodeId(0);
    s.edit("Add Profile", |doc, _| {
        prof_id = doc.alloc_id();
        let node = Node::path(prof_id, pd_prof, app_prof);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    })
    .unwrap();

    // 2. Create obstacle stone at x = 300..400, y = 250..300
    let mut stone = BezPath::new();
    stone.move_to((300.0, 250.0));
    stone.line_to((400.0, 250.0));
    stone.line_to((400.0, 300.0));
    stone.line_to((300.0, 300.0));
    stone.close_path();

    let pd_stone = PathData::from_bezpath(&stone);
    let app_stone = Appearance::basic(Paint::solid(Color::rgb8(180, 180, 180)), Paint::None, 0.0);
    let mut stone_id = NodeId(0);
    s.edit("Add Stone", |doc, _| {
        stone_id = doc.alloc_id();
        let node = Node::path(stone_id, pd_stone, app_stone);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    })
    .unwrap();

    // 3. Create elevation lines group with autoUpdate: true
    let res = s
        .execute(
            "elevation.create",
            &json!({
                "profileId": prof_id.0,
                "obstacleIds": [stone_id.0],
                "datumM": 150.0,
                "minorIntervalM": 0.1,
                "majorIntervalM": 0.5,
                "autoUpdate": true,
            }),
        )
        .unwrap();

    let gid = NodeId(res["id"].as_u64().unwrap());
    let initial_count = res["segmentCount"].as_u64().unwrap();
    assert!(initial_count > 0);

    // 4. Mutation 1: Deepen profile trench floor from y = 300 to y = 500
    s.edit("Deepen Profile", |doc, _| {
        let mut deepened = BezPath::new();
        deepened.move_to((100.0, 150.0));
        deepened.line_to((200.0, 150.0));
        deepened.line_to((250.0, 500.0));
        deepened.line_to((450.0, 500.0));
        deepened.line_to((500.0, 150.0));
        deepened.line_to((600.0, 150.0));

        let node = doc.node_mut(prof_id).unwrap();
        if let NodeKind::Path { path, .. } = &mut node.kind {
            *path = PathData::from_bezpath(&deepened);
        }
        Ok(())
    })
    .unwrap();

    // Verify elevation group was refreshed automatically by post-edit hook
    let doc1 = &s.doc().unwrap().doc;
    let group1 = doc1.node(gid).unwrap();
    let count_after_deepening = match &group1.kind {
        NodeKind::Group { children, .. } => children.iter().filter(|c| matches!(c.kind, NodeKind::Path { .. })).count(),
        _ => panic!("Expected group"),
    };
    assert!(
        count_after_deepening > initial_count as usize,
        "Deepening trench must increase segment count: initial={initial_count}, new={count_after_deepening}"
    );

    // 5. Mutation 2: Remove or shrink obstacle stone
    s.edit("Shrink Stone", |doc, _| {
        let mut shrunk = BezPath::new();
        shrunk.move_to((340.0, 280.0));
        shrunk.line_to((360.0, 280.0));
        shrunk.line_to((360.0, 300.0));
        shrunk.line_to((340.0, 300.0));
        shrunk.close_path();

        let node = doc.node_mut(stone_id).unwrap();
        if let NodeKind::Path { path, .. } = &mut node.kind {
            *path = PathData::from_bezpath(&shrunk);
        }
        Ok(())
    })
    .unwrap();

    // Verify elevation group refreshed again
    let doc2 = &s.doc().unwrap().doc;
    let group2 = doc2.node(gid).unwrap();
    assert!(group2.elevation.is_some());

    // 6. Test with autoUpdate = false: edits should NOT refresh
    s.execute(
        "elevation.set",
        &json!({
            "id": gid.0,
            "autoUpdate": false,
        }),
    )
    .unwrap();

    let count_before_freeze = match &s.doc().unwrap().doc.node(gid).unwrap().kind {
        NodeKind::Group { children, .. } => children.len(),
        _ => 0,
    };

    s.edit("Further Deepen with AutoUpdate Off", |doc, _| {
        let mut deepened_more = BezPath::new();
        deepened_more.move_to((100.0, 150.0));
        deepened_more.line_to((200.0, 150.0));
        deepened_more.line_to((250.0, 800.0));
        deepened_more.line_to((450.0, 800.0));
        deepened_more.line_to((500.0, 150.0));
        deepened_more.line_to((600.0, 150.0));

        let node = doc.node_mut(prof_id).unwrap();
        if let NodeKind::Path { path, .. } = &mut node.kind {
            *path = PathData::from_bezpath(&deepened_more);
        }
        Ok(())
    })
    .unwrap();

    let count_after_freeze_edit = match &s.doc().unwrap().doc.node(gid).unwrap().kind {
        NodeKind::Group { children, .. } => children.len(),
        _ => 0,
    };
    assert_eq!(count_before_freeze, count_after_freeze_edit, "When autoUpdate is false, post-edit hook must NOT recompute");

    // Explicit manual update recomputes
    let upd_res = s.execute("elevation.update", &json!({ "id": gid.0 })).unwrap();
    assert_eq!(upd_res["updated"], 1);
    let count_after_manual = match &s.doc().unwrap().doc.node(gid).unwrap().kind {
        NodeKind::Group { children, .. } => children.len(),
        _ => 0,
    };
    assert!(count_after_manual > count_before_freeze, "Manual elevation.update must refresh");
}

/// 2.6 Boundary stress: Overlapping obstacles and completely occluded voids
#[test]
fn test_j015_overlapping_and_fully_occluding_obstacles() {
    let mut profile = BezPath::new();
    profile.move_to((0.0, 200.0));
    profile.line_to((600.0, 200.0));

    // Obstacle A: [150, 250], y = 120..200
    let mut obs_a = BezPath::new();
    obs_a.move_to((150.0, 120.0));
    obs_a.line_to((250.0, 120.0));
    obs_a.line_to((250.0, 200.0));
    obs_a.line_to((150.0, 200.0));
    obs_a.close_path();

    // Obstacle B (overlapping A): [200, 350], y = 120..200
    let mut obs_b = BezPath::new();
    obs_b.move_to((200.0, 120.0));
    obs_b.line_to((350.0, 120.0));
    obs_b.line_to((350.0, 200.0));
    obs_b.line_to((200.0, 200.0));
    obs_b.close_path();

    let res = clip_elevation_scanline(150.0, &profile, &[obs_a, obs_b], 0.0, 600.0, 50.0);
    // Combined obstacle spans [150, 350].
    // Void [0, 600] minus [150, 350] must give [0, 150] and [350, 600].
    assert_eq!(res.len(), 2, "Overlapping obstacles must merge cleanly without extra fragments: got {:?}", res);
    assert!((res[0].0 - 0.0).abs() < 1e-3 && (res[0].1 - 150.0).abs() < 1e-3);
    assert!((res[1].0 - 350.0).abs() < 1e-3 && (res[1].1 - 600.0).abs() < 1e-3);

    // Obstacle C completely covering trench void [0, 600]: [ -50, 650 ]
    let mut obs_c = BezPath::new();
    obs_c.move_to((-50.0, 100.0));
    obs_c.line_to((650.0, 100.0));
    obs_c.line_to((650.0, 200.0));
    obs_c.line_to((-50.0, 200.0));
    obs_c.close_path();

    let res_occluded = clip_elevation_scanline(150.0, &profile, &[obs_c], 0.0, 600.0, 50.0);
    assert!(res_occluded.is_empty(), "Completely covered void must emit 0 segments, got {:?}", res_occluded);
}

/// 2.7 Boundary stress: Ground profile specified from right to left (reverse orientation)
#[test]
fn test_j015_reversed_ground_profile_right_to_left() {
    // Ground profile running from x = 600 down to x = 0 (right to left)
    let mut rev_profile = BezPath::new();
    rev_profile.move_to((600.0, 100.0));
    rev_profile.line_to((400.0, 100.0));
    rev_profile.line_to((300.0, 300.0)); // V-trench bottom
    rev_profile.line_to((200.0, 100.0));
    rev_profile.line_to((0.0, 100.0));

    let ceiling_y = 0.0;
    let x_min = 0.0;
    let x_max = 600.0;

    // Scanline at y = 200 (halfway in trench)
    let res = clip_elevation_scanline(200.0, &rev_profile, &[], x_min, x_max, ceiling_y);
    assert_eq!(res.len(), 1, "Reversed profile must produce single trench span");
    assert!((res[0].0 - 250.0).abs() < 1.0, "Left slope at 250: got {}", res[0].0);
    assert!((res[0].1 - 350.0).abs() < 1.0, "Right slope at 350: got {}", res[0].1);
}

/// 1.4 Stress test: Negative TM coordinates and desca meter/unit parsing matrix
#[test]
fn test_j007_negative_tm_coordinates_and_unit_parsing() {
    // DXF with negative coordinates (e.g. southern hemisphere or local grid origin)
    let header = [(9, "$INSUNITS"), (70, "6")];
    let entities = [(0, "LINE"), (8, "NEG_SURVEY"), (10, "-200000.5"), (20, "-500000.5"), (11, "-200010.5"), (21, "-500020.5")];
    let dxf_bytes = make_dxf(&header, &["NEG_SURVEY"], &entities);

    let opts = ImportOptions { preserve_coords: true, scale: Some(20.0), ..ImportOptions::default() };
    let imp = vectorcraft_cad::import(&dxf_bytes, &opts).expect("Negative TM import must succeed");
    let scale_cfg = imp.document.scale.as_ref().expect("scale config must exist");

    assert!(scale_cfg.world_origin.0 <= -200000.0);
    assert!(scale_cfg.world_origin.1 <= -500000.0);

    // Verify desca helper functions
    assert_eq!(vectorcraft_cad::desca::normalize_elevation_text("EL. 152.5"), "152.5m");
    assert_eq!(vectorcraft_cad::desca::normalize_elevation_text("el 152.5m"), "152.5m");
    assert_eq!(vectorcraft_cad::desca::normalize_elevation_text("EL. 78.0M"), "78.0m");
    assert_eq!(vectorcraft_cad::desca::parse_meter_string("10cm"), Some(0.1));
    assert_eq!(vectorcraft_cad::desca::parse_meter_string("50CM"), Some(0.5));
    assert_eq!(vectorcraft_cad::desca::parse_meter_string("1.0 m"), Some(1.0));
    assert_eq!(vectorcraft_cad::desca::parse_meter_string("152.5"), Some(152.5));
}
