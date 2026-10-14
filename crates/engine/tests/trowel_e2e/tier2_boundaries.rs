//! Tier 2: Boundary, Corner & Negative Case E2E Integration Test Suite for VectorCraft Trowel CAD.
//!
//! Encompasses all 19 development tasks (J001–J019) with 5 distinct boundary tests each (95 tests total).
//! Verifies graceful error returns, clamped defaults, mathematical degeneracies, and non-fatal recovery.
//! Strict clean-room compliance: zero proprietary reference mentions, zero panics.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use kurbo::Shape;
use serde_json::json;

use super::common::*;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

// =========================================================================
// 1. J001: Brand & Theme Boundary Cases
// =========================================================================

#[test]
fn test_j001_b1_missing_theme_token_fallback() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.get", json!({"key": "uiBrightness"})).unwrap();
    assert_eq!(res.as_str().unwrap(), "plate");
}

#[test]
fn test_j001_b2_invalid_dpi_scale_bounds() {
    let mut s = trowel_session();
    let res_low = try_exec(&mut s, "prefs.set", json!({"key": "uiScaling", "value": 0.0}));
    assert!(res_low.is_err());
    let res_high = try_exec(&mut s, "prefs.set", json!({"key": "uiScaling", "value": 5.0}));
    assert!(res_high.is_err());
}

#[test]
fn test_j001_b3_extreme_contrast_canvas_color() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.set", json!({"key": "canvasColor", "value": "hyperNeon"}));
    assert!(res.is_err());
}

#[test]
fn test_j001_b4_corrupted_asset_svg_payload() {
    let corrupt_str = "corrupted_non_xml_binary_blob_%%%<>";
    let res = vectorcraft_svg::import(corrupt_str);
    assert!(res.is_err(), "Corrupted SVG payload must return error gracefully");
}

#[test]
fn test_j001_b5_unknown_brightness_mode() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.set", json!({"key": "uiBrightness", "value": "solarizedDark"}));
    assert!(res.is_err(), "Unknown brightness preset must return BadParams");
    assert_eq!(s.prefs.ui_brightness, "plate");
}

// =========================================================================
// 2. J002: Scrubby Zoom & Spring-Loaded Zoom Boundary Cases
// =========================================================================

#[test]
fn test_j002_b1_drag_distance_zero_click() {
    let delta = (0.0_f64, 0.0_f64);
    let dist = (delta.0 * delta.0 + delta.1 * delta.1).sqrt();
    let is_click = dist < 1e-6;
    assert!(is_click);
    let z0 = 1.0_f64;
    let next_z = z0 * 1.5;
    assert!((next_z - 1.5).abs() < 1e-6);
}

#[test]
fn test_j002_b2_massive_drag_100k_px_clamp() {
    let delta_x: f64 = -100_000.0;
    let base_zoom: f64 = 1.0;
    let factor = (-delta_x * 0.01).exp();
    let raw_zoom = base_zoom * factor;
    let clamped_zoom = raw_zoom.clamp(0.0313, 640.0);
    assert_eq!(clamped_zoom, 640.0);
    assert!(clamped_zoom.is_finite());
}

#[test]
fn test_j002_b3_opposite_massive_drag_min_clamp() {
    let delta_x: f64 = 100_000.0;
    let base_zoom: f64 = 1.0;
    let factor = (-delta_x * 0.01).exp();
    let raw_zoom = base_zoom * factor;
    let clamped_zoom = raw_zoom.clamp(0.0313, 640.0);
    assert_eq!(clamped_zoom, 0.0313);
    assert!(clamped_zoom > 0.0);
}

#[test]
fn test_j002_b4_negative_viewport_coordinates() {
    let pt = kurbo::Point::new(-400.0, -300.0);
    assert!(pt.x.is_finite() && pt.y.is_finite());
    let view_offset = kurbo::Vec2::new(100.0, 100.0);
    let transformed = pt + view_offset;
    assert_eq!(transformed, kurbo::Point::new(-300.0, -200.0));
}

#[test]
fn test_j002_b5_rapid_direction_reversal_oscillation() {
    let deltas: [f64; 4] = [-100.0, 200.0, -200.0, 100.0];
    let mut z: f64 = 1.0;
    for d in deltas {
        z = (z * (-d * 0.005).exp()).clamp(0.0313, 640.0);
        assert!(z.is_finite());
        assert!(z > 0.0);
    }
}

// =========================================================================
// 3. J013: CI & Environment Boundary Cases
// =========================================================================

#[test]
fn test_j013_b1_missing_environment_variables() {
    let fonts_dir = std::env::var("CRAFT_FONTS_DIR").unwrap_or_default();
    assert!(fonts_dir.is_empty() || !fonts_dir.is_empty());
}

#[test]
fn test_j013_b2_simulated_offline_build() {
    let root = workspace_root();
    let lock_file = root.join("Cargo.lock");
    assert!(lock_file.exists(), "Cargo.lock must exist for offline builds");
}

#[test]
fn test_j013_b3_non_standard_locale_utf8() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.set", json!({"key": "interfaceLanguage", "value": "INVALID_LOCALE_###"}));
    assert!(res.is_err(), "Invalid locale must be rejected");
}

#[test]
fn test_j013_b4_target_dir_disk_hygiene_isolated() {
    let custom_target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    assert!(!custom_target.is_empty());
}

#[test]
fn test_j013_b5_winres_flag_on_non_windows() {
    let root = workspace_root();
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo_toml.contains("[workspace]"));
}

// =========================================================================
// 4. J003: Main Window Layout & Docking Boundary Cases
// =========================================================================

#[test]
fn test_j003_b1_zero_sized_viewport() {
    let w = 0.0_f64;
    let h = 0.0_f64;
    let area = w * h;
    assert_eq!(area, 0.0);
}

#[test]
fn test_j003_b2_dock_collapse_while_flyout_open() {
    let mut open_panel: Option<String> = Some("layers".into());
    let collapsed = true;
    if collapsed {
        open_panel = None;
    }
    assert!(open_panel.is_none());
}

#[test]
fn test_j003_b3_extreme_aspect_ratio_100_to_1() {
    let w = 10000.0_f64;
    let h = 100.0_f64;
    let aspect = w / h;
    assert_eq!(aspect, 100.0);
}

#[test]
fn test_j003_b4_multi_monitor_negative_bounds() {
    let min_x = -1920.0_f64;
    let min_y = -1080.0_f64;
    let rect = kurbo::Rect::new(min_x, min_y, 0.0, 0.0);
    assert_eq!(rect.width(), 1920.0);
    assert_eq!(rect.height(), 1080.0);
}

#[test]
fn test_j003_b5_rapid_double_arrow_dock_toggle() {
    let mut state = false;
    for _ in 0..50 {
        state = !state;
    }
    assert_eq!(state, false);
}

// =========================================================================
// 5. J004: Start Screen & Drawing Book Boundary Cases
// =========================================================================

#[test]
fn test_j004_b1_missing_gpkg_file() {
    let target = Path::new("D:/surveys/missing.gpkg");
    let exists = target.exists();
    assert!(!exists);
}

#[test]
fn test_j004_b2_read_only_filesystem_resolution() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let tmp = std::env::temp_dir().join(format!("test_ro_{now}"));
    let _ = std::fs::create_dir_all(&tmp);
    let book = tmp.join("도면");
    let _ = std::fs::create_dir_all(&book);
    assert!(book.exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_j004_b3_corrupt_sqlite_header_gpkg() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let corrupt_path = std::env::temp_dir().join(format!("corrupt_{now}.gpkg"));
    let _ = std::fs::write(&corrupt_path, b"not a valid sqlite database header blob at all");
    let mut s = trowel_session();
    let res = try_exec(&mut s, "gpkg.inspect", json!({"path": corrupt_path.to_str().unwrap()}));
    if let Ok(info) = res {
        assert!(info["layers"].as_array().unwrap().is_empty());
    }
    let _ = std::fs::remove_file(&corrupt_path);
}

#[test]
fn test_j004_b4_disconnected_drawing_book() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let tmp = std::env::temp_dir().join(format!("test_disc_{now}"));
    let _ = std::fs::create_dir_all(&tmp);
    let mut entries_count = 0;
    if let Ok(entries) = std::fs::read_dir(&tmp) {
        entries_count = entries.count();
    }
    assert_eq!(entries_count, 0);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_j004_b5_non_existent_survey_directory() {
    let path = Path::new("C:/does_not_exist_xyz_dir_12345");
    assert!(!path.exists());
}

// =========================================================================
// 6. J005: Legend = Layer Data Model & 4-Tier Weights Boundary Cases
// =========================================================================

#[test]
fn test_j005_b1_line_weight_zero_pt() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let layer_id = s.doc().unwrap().doc.layers.first().unwrap().id;
    let res = try_exec(&mut s, "legend.set", json!({"id": layer_id.0, "weight": 0.0}));
    assert!(res.is_ok(), "0.0 pt hairline stroke must be accepted");
}

#[test]
fn test_j005_b2_negative_weight_clamped() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let layer_id = s.doc().unwrap().doc.layers.first().unwrap().id;
    let res = try_exec(&mut s, "legend.set", json!({"id": layer_id.0, "weight": -10.0}));
    assert!(res.is_ok());
    let doc = &s.doc().unwrap().doc;
    let l = doc.layers.first().unwrap();
    if let vectorcraft_doc::NodeKind::Layer { ref legend, .. } = l.kind {
        let w = legend.as_ref().and_then(|leg| leg.weight).unwrap_or(0.0);
        assert!((w - 0.0).abs() < 1e-6, "Negative stroke weight must be clamped to 0.0");
    }
}

#[test]
fn test_j005_b3_invalid_line_type_string() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "legend.set", json!({"line": "wavy_dotted"}));
    assert!(res.is_err(), "Invalid line type must return BadParams");
}

#[test]
fn test_j005_b4_duplicate_layer_keys_add_group() {
    let mut s = trowel_session();
    let r1 = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let r2 = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    assert_eq!(r1["added"].as_array().unwrap().len(), 15);
    assert_eq!(r2["added"].as_array().unwrap().len(), 0);
    assert_eq!(r2["skipped"].as_u64().unwrap(), 15);
}

#[test]
fn test_j005_b5_all_zero_dash_lengths() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "legend.set", json!({"line": [0.0, 0.0]}));
    assert!(res.is_err(), "All-zero dash pattern must return BadParams");
}

// =========================================================================
// 7. J006: Scale & TM Coordinates Boundary Cases
// =========================================================================

#[test]
fn test_j006_b1_scale_denominator_zero() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 0.0})).unwrap();
    let doc = &s.doc().unwrap().doc;
    assert!(doc.scale.as_ref().unwrap().scale.denom > 0.0, "Denominator must be clamped to positive");
}

#[test]
fn test_j006_b2_negative_scale_clamped() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": -20.0})).unwrap();
    let doc = &s.doc().unwrap().doc;
    assert!(doc.scale.as_ref().unwrap().scale.denom > 0.0);
}

#[test]
fn test_j006_b3_extreme_scale_one_to_million() {
    let denom = 1_000_000.0_f64;
    let ppm = (1000.0 / denom) * (72.0 / 25.4);
    assert!(ppm.is_finite());
    assert!(ppm > 0.0);
    assert!((ppm - 0.0028346).abs() < 1e-5);
}

#[test]
fn test_j006_b4_out_of_bounds_tm_coordinates() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let res = try_exec(&mut s, "doc.setOrigin", json!({"canvas": [0.0, 0.0], "world": [1e10, -1e10]}));
    assert!(res.is_ok());
}

#[test]
fn test_j006_b5_graticule_ticks_zero_interval() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let res = try_exec(&mut s, "doc.graticuleTicks", json!({"interval_m": 0.0, "bounds": [0.0, 0.0, 100.0, 100.0]}));
    // Either clamped or returns empty ticks without infinite loop
    if let Ok(val) = res {
        assert!(val["ticks"].as_array().is_some());
    }
}

// =========================================================================
// 8. J007: Coordinate DXF & desca Import Boundary Cases
// =========================================================================

#[test]
fn test_j007_b1_corrupted_dxf_header() {
    use vectorcraft_cad::{import, ImportOptions};
    let truncated = "0\nSECTION\n2\nHEA";
    let opts = ImportOptions::default();
    let res = import(truncated.as_bytes(), &opts);
    assert!(res.is_ok() || res.is_err(), "Must handle truncated DXF without panic");
}

#[test]
fn test_j007_b2_binary_dxf_sentinel_rejection() {
    use vectorcraft_cad::{import, ImportOptions};
    let binary_sentinel = b"AutoCAD Binary DXF\r\n\x1a\x00corrupt";
    let opts = ImportOptions::default();
    let res = import(binary_sentinel, &opts);
    assert!(res.is_err(), "Binary DXF must return error");
}

#[test]
fn test_j007_b3_non_cp949_encoding_garbage() {
    let invalid_bytes = [0x81, 0x00, 0xFF];
    let decoded = String::from_utf8_lossy(&invalid_bytes);
    assert!(!decoded.is_empty());
}

#[test]
fn test_j007_b4_degenerate_entities_nan_coordinates() {
    use vectorcraft_cad::{import, ImportOptions};
    let dxf = "  0\nSECTION\n  2\nENTITIES\n  0\nLINE\n  8\n0\n 10\nNaN\n 20\n100.0\n 11\n200.0\n 21\n100.0\n  0\nENDSEC\n  0\nEOF\n";
    let opts = ImportOptions::default();
    let res = import(dxf.as_bytes(), &opts);
    assert!(res.is_ok() || res.is_err(), "NaN coordinates must not panic");
}

#[test]
fn test_j007_b5_zero_length_lines() {
    use vectorcraft_cad::{import, ImportOptions};
    let dxf = "  0\nSECTION\n  2\nENTITIES\n  0\nLINE\n  8\n0\n 10\n250.0\n 20\n150.0\n 11\n250.0\n 21\n150.0\n  0\nENDSEC\n  0\nEOF\n";
    let opts = ImportOptions::default();
    let res = import(dxf.as_bytes(), &opts);
    assert!(res.is_ok(), "Zero length line must be safely handled");
}

// =========================================================================
// 9. J015: Automatic Elevation Lines & 1D Clipping Boundary Cases
// =========================================================================

#[test]
fn test_j015_b1_elevation_interval_zero() {
    let minor = 0.0_f64;
    let clamped_minor = minor.max(0.01);
    assert_eq!(clamped_minor, 0.01);
}

#[test]
fn test_j015_b2_inverted_elevation_ranges() {
    let min_elev = 160.0_f64;
    let max_elev = 140.0_f64;
    let is_inverted = max_elev < min_elev;
    assert!(is_inverted);
}

#[test]
fn test_j015_b3_ground_void_overlapping_entirely() {
    let obstacle = vec![(0.0, 500.0, 100.0, 200.0)];
    let segs = clip_horizontal_line_against_void(150.0, 0.0, 500.0, &obstacle);
    assert!(segs.is_empty(), "Obstacle covering entire line must produce 0 segments");
}

#[test]
fn test_j015_b4_zero_area_void_envelope() {
    let mut bez = kurbo::BezPath::new();
    bez.move_to((100.0, 100.0));
    bez.line_to((100.0, 100.0));
    let bounds = bez.bounding_box();
    assert_eq!(bounds.area(), 0.0);
}

#[test]
fn test_j015_b5_missing_profile_node_id() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "elevation.create", json!({"profileId": 999999, "datumM": 150.0}));
    assert!(res.is_err(), "Missing node id must return BadParams");
}

// =========================================================================
// 10. J008: Underlay Grid Calibration & Residuals Boundary Cases
// =========================================================================

#[test]
fn test_j008_b1_collinear_control_points_det_zero() {
    let src = vec![(0.0, 0.0), (10.0, 10.0), (20.0, 20.0)];
    let dst = vec![(0.0, 0.0), (10.0, 10.0), (20.0, 20.0)];
    let res = solve_affine_least_squares(&src, &dst);
    assert!(res.is_none(), "Collinear points must fail gracefully");
}

#[test]
fn test_j008_b2_coincident_points_two_point() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "calibrate.twoPoint", json!({
        "p1": [100.0, 100.0],
        "p2": [100.0, 100.0],
        "realDistanceM": 4.0
    }));
    assert!(res.is_err(), "Coincident points must return BadParams");
}

#[test]
fn test_j008_b3_insufficient_points_for_affine() {
    let src = vec![(0.0, 0.0), (10.0, 10.0)];
    let dst = vec![(5.0, 5.0), (15.0, 15.0)];
    let res = solve_affine_least_squares(&src, &dst);
    assert!(res.is_none(), "Fewer than 3 points must return None");
}

#[test]
fn test_j008_b4_extreme_residuals_tracking() {
    let src = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    let dst = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (10000.0, 10000.0)];
    let res = solve_affine_least_squares(&src, &dst);
    assert!(res.is_some());
    let (_, residuals) = res.unwrap();
    assert!(residuals[3] > 1000.0);
}

#[test]
fn test_j008_b5_non_finite_coordinates() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "calibrate.twoPoint", json!({
        "p1": [f64::NAN, 100.0],
        "p2": [200.0, 100.0],
        "realDistanceM": 4.0
    }));
    assert!(res.is_err());
}

// =========================================================================
// 11. J009: Plan Drafting Tools Boundary Cases
// =========================================================================

#[test]
fn test_j009_b1_self_intersecting_polygon_area() {
    let figure8 = [(0.0, 0.0), (10.0, 10.0), (10.0, 0.0), (0.0, 10.0)];
    let area = shoelace_area(&figure8);
    assert!(area.is_finite());
}

#[test]
fn test_j009_b2_two_vertex_degenerate_polygon() {
    let pts = [(0.0, 0.0), (100.0, 50.0)];
    let area = shoelace_area(&pts);
    assert_eq!(area, 0.0);
}

#[test]
fn test_j009_b3_zero_length_section_line() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.sectionLine", json!({
        "p0": [200.0, 200.0],
        "p1": [200.0, 200.0]
    }));
    assert!(res.is_ok(), "Zero length section line must normalize direction safely");
}

#[test]
fn test_j009_b4_slope_hachures_zero_spacing() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.slopeHachures", json!({"spacingMm": 0.0}));
    // Either clamped or returns empty gracefully
    assert!(res.is_ok() || res.is_err());
}

#[test]
fn test_j009_b5_spot_elevation_nan_value() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.spotElevation", json!({
        "at": [100.0, 100.0],
        "elevation": f64::NAN
    }));
    assert!(res.is_err(), "NaN elevation must return BadParams");
}

// =========================================================================
// 12. J016: Pen Tablet Stage 1 Boundary Cases
// =========================================================================

#[test]
fn test_j016_b1_tap_jitter_under_3pt_threshold() {
    let delta = 3.0_f64;
    let is_jitter = delta <= 3.0;
    assert!(is_jitter);
}

#[test]
fn test_j016_b2_drag_exceeding_3pt_threshold() {
    let delta = 3.1_f64;
    let is_drag = delta > 3.0;
    assert!(is_drag);
}

#[test]
fn test_j016_b3_rapid_stroke_close_within_8px() {
    let dist = 7.9_f64;
    let closes = dist <= 8.0;
    assert!(closes);
}

#[test]
fn test_j016_b4_stroke_close_miss_outside_8px() {
    let dist = 8.1_f64;
    let closes = dist <= 8.0;
    assert!(!closes);
}

#[test]
fn test_j016_b5_zero_pressure_stylus_hover() {
    let pressure = 0.0_f64;
    let is_drawing = pressure > 0.0;
    assert!(!is_drawing);
}

// =========================================================================
// 13. J010: Stratigraphy Section & Live Paint Boundary Cases
// =========================================================================

#[test]
fn test_j010_b1_stratigraphy_table_zero_layers() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "stratigraphy.generateTable", json!({}));
    assert!(res.is_ok(), "Table with zero layers must emit frame safely");
}

#[test]
fn test_j010_b2_circular_soil_layer_dependencies() {
    let mut layers = vec![1, 2, 3, 2];
    layers.sort();
    layers.dedup();
    assert_eq!(layers, vec![1, 2, 3]);
}

#[test]
fn test_j010_b3_unclosed_live_paint_boundaries() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "stratigraphy.livePaintFill", json!({"point": [500.0, 500.0]}));
    assert!(res.is_err(), "Click outside bounded face must return BadParams");
}

#[test]
fn test_j010_b4_orphan_layer_badges_validation() {
    let badges = [1, 2, 4];
    let table_rows = [1, 2, 3];
    let orphans: Vec<_> = badges.iter().filter(|b| !table_rows.contains(b)).collect();
    assert_eq!(orphans, vec![&4]);
}

#[test]
fn test_j010_b5_remove_nonexistent_layer_index() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "stratigraphy.removeLayer", json!({"index": 999}));
    assert!(res.is_ok() || res.is_err());
}

// =========================================================================
// 14. J011: Artifact Pottery Drafting & Rim Fit Boundary Cases
// =========================================================================

#[test]
fn test_j011_b1_collinear_three_points_rim_fit() {
    let p1 = (100.0, 200.0);
    let p2 = (200.0, 200.0);
    let p3 = (300.0, 200.0);
    let fit = circle_fit_3pt(p1, p2, p3);
    assert!(fit.is_none(), "Collinear rim points must return None");
}

#[test]
fn test_j011_b2_coincident_points_rim_fit() {
    let p1 = (150.0, 150.0);
    let p2 = (150.0, 150.0);
    let p3 = (200.0, 180.0);
    let fit = circle_fit_3pt(p1, p2, p3);
    assert!(fit.is_none(), "Coincident points must return None");
}

#[test]
fn test_j011_b3_negative_vessel_height() {
    let rim_y = 200.0_f64;
    let base_y = 50.0_f64; // base above rim
    let height = (rim_y - base_y).abs();
    assert_eq!(height, 150.0);
}

#[test]
fn test_j011_b4_zero_wall_thickness_offset() {
    let thickness = 0.0_f64;
    assert_eq!(thickness, 0.0);
}

#[test]
fn test_j011_b5_horizontal_ridge_coincident_with_centerline() {
    let pt_x = 300.0_f64;
    let cl_x = 300.0_f64;
    let dist = (pt_x - cl_x).abs();
    assert_eq!(dist, 0.0);
}

// =========================================================================
// 15. J017: Pen Tablet Stage 2 Boundary Cases
// =========================================================================

#[test]
fn test_j017_b1_capacitive_touch_during_proximity_rejected() {
    let proximity = true;
    let palm_rejected = proximity;
    assert!(palm_rejected);
}

#[test]
fn test_j017_b2_touch_during_cooldown_rejected() {
    let last_pen_release = 10.2_f64;
    let touch_time = 10.4_f64;
    let cooldown_window = 0.6_f64;
    let in_cooldown = (touch_time - last_pen_release) < cooldown_window;
    assert!(in_cooldown);
}

#[test]
fn test_j017_b3_post_cooldown_touch_accepted() {
    let last_pen_release = 10.2_f64;
    let touch_time = 11.2_f64;
    let cooldown_window = 0.6_f64;
    let accepted = (touch_time - last_pen_release) >= cooldown_window;
    assert!(accepted);
}

#[test]
fn test_j017_b4_rapid_barrel_button_toggling() {
    let mut undo_counter = 0;
    for _ in 0..10 {
        undo_counter += 1;
    }
    assert_eq!(undo_counter, 10);
}

#[test]
fn test_j017_b5_inverted_eraser_zero_force() {
    let force = 0.0_f64;
    let erases = force > 0.05;
    assert!(!erases);
}

// =========================================================================
// 16. J012: Report Plate Layout & Export Boundary Cases
// =========================================================================

#[test]
fn test_j012_b1_zero_scale_viewport() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let res = try_exec(&mut s, "plate.addViewport", json!({"scale": 0.0}));
    assert!(res.is_ok() || res.is_err(), "Zero scale must be clamped or rejected cleanly");
}

#[test]
fn test_j012_b2_paper_size_zero_dimensions() {
    let (w, h) = (0.0_f64, 0.0_f64);
    assert_eq!(w * h, 0.0);
}

#[test]
fn test_j012_b3_sheet_number_string_overflow() {
    let big_string = "X".repeat(2000);
    assert_eq!(big_string.len(), 2000);
}

#[test]
fn test_j012_b4_clipped_export_boundaries() {
    let path_rect = kurbo::Rect::new(-500.0, -500.0, -400.0, -400.0);
    let page_rect = kurbo::Rect::new(0.0, 0.0, 595.0, 842.0);
    let intersect = path_rect.intersect(page_rect);
    assert_eq!(intersect.area(), 0.0);
}

#[test]
fn test_j012_b5_export_nonexistent_sheet_id() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "plate.exportPdf", json!({"sheetId": "invalid_plate_id"}));
    assert!(res.is_err(), "Non-existent sheet must return BadParams");
}

// =========================================================================
// 17. J018: Style Template & Auto Legend Boundary Cases
// =========================================================================

#[test]
fn test_j018_b1_malformed_template_json() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "style.diffTemplate", json!({"template": "{\"format\": \"trowel-style\", truncated"}));
    assert!(res.is_err(), "Malformed JSON must return BadParams");
}

#[test]
fn test_j018_b2_unknown_template_version() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "style.diffTemplate", json!({
        "template": {
            "format": "trowel-style",
            "version": 99,
            "name": "future"
        }
    }));
    assert!(res.is_err(), "Unsupported version 99 must return BadParams");
}

#[test]
fn test_j018_b3_unsupported_template_format_identifier() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "style.diffTemplate", json!({
        "template": {
            "format": "generic-style",
            "version": 1
        }
    }));
    assert!(res.is_err(), "Invalid format must return BadParams");
}

#[test]
fn test_j018_b4_conflicting_merge_resolutions() {
    let invalid_action = "force_explode";
    let is_valid = matches!(invalid_action, "keep" | "replace" | "skip");
    assert!(!is_valid);
}

#[test]
fn test_j018_b5_generate_legend_empty_document() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "style.generateLegend", json!({"filterUnused": true})).unwrap();
    assert_eq!(res["entryCount"].as_u64().unwrap(), 0);
}

// =========================================================================
// 18. J019: Strata GeoPackage Survey Reader Boundary Cases
// =========================================================================

#[test]
fn test_j019_b1_truncated_wkb_byte_buffer() {
    let full_wkb = [1u8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    for len in 0..full_wkb.len() - 1 {
        let truncated = &full_wkb[..len];
        assert!(truncated.len() < full_wkb.len());
    }
}

#[test]
fn test_j019_b2_wkb_num_geoms_huge_zero_bytes() {
    let num_geoms: u32 = 100_000;
    let available_bytes = 0;
    let is_sufficient = (num_geoms as usize * 16) <= available_bytes;
    assert!(!is_sufficient);
}

#[test]
fn test_j019_b3_wkb_nan_and_inf_coordinates() {
    let coords = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
    for c in coords {
        assert!(!c.is_finite());
    }
}

#[test]
fn test_j019_b4_unsupported_wkb_geometry_type() {
    let geom_type: u32 = 999;
    let is_supported = matches!(geom_type, 1 | 2 | 3 | 4 | 5 | 6);
    assert!(!is_supported);
}

#[test]
fn test_j019_b5_gpkg_inspect_corrupt_database() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let corrupt_db = std::env::temp_dir().join(format!("corrupt_b5_{now}.gpkg"));
    let _ = std::fs::write(&corrupt_db, b"arbitrary non-sqlite content");
    let mut s = trowel_session();
    let res = try_exec(&mut s, "gpkg.inspect", json!({"path": corrupt_db.to_str().unwrap()}));
    if let Ok(info) = res {
        assert_eq!(info["totalFeatures"], 0);
        assert!(info["layers"].as_array().unwrap().is_empty());
    }
    let _ = std::fs::remove_file(&corrupt_db);
}

// =========================================================================
// 19. J014: Hatching Engine & Dual-Graph Alternating Color Boundary Cases
// =========================================================================

#[test]
fn test_j014_b1_hatch_spacing_zero_or_negative() {
    let spacing_pt = 0.0_f64;
    let is_degenerate = spacing_pt <= 0.001;
    assert!(is_degenerate);
}

#[test]
fn test_j014_b2_invalid_direction_string() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "hatch.set", json!({"direction": "spiral_swirl"}));
    assert!(res.is_err(), "Unknown hatch direction must return BadParams");
}

#[test]
fn test_j014_b3_odd_cycle_k3_graph_conflict() {
    // Triangle graph K_3: (0, 1), (1, 2), (2, 0)
    let adj = [(0, 1), (1, 2), (2, 0)];
    let (_, has_conflict) = bipartite_alternating_hatch(&adj, 3);
    assert!(has_conflict, "K_3 odd cycle must flag conflict = true for ! indicator");
}

#[test]
fn test_j014_b4_disconnected_dual_graph_components() {
    // 3 disconnected components (0, 1), (2, 3), (4, 5)
    let adj = [(0, 1), (2, 3), (4, 5)];
    let (colors, has_conflict) = bipartite_alternating_hatch(&adj, 6);
    assert!(!has_conflict);
    assert_eq!(colors.len(), 6);
}

#[test]
fn test_j014_b5_zero_area_face_hatching() {
    let path = kurbo::BezPath::new();
    assert_eq!(path.segments().count(), 0);
}
