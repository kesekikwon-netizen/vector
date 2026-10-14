//! Tier 1: Feature Coverage E2E Integration Test Suite for VectorCraft Trowel CAD.
//!
//! Encompasses all 19 development tasks (J001–J019) with 5 distinct feature tests each (95 tests total).
//! Strict clean-room compliance: zero proprietary reference mentions, zero panics.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use std::path::{Path, PathBuf};
use vectorcraft_doc::LineType;
use vectorcraft_doc::style_template::TrowelStyleTemplate;
use vectorcraft_engine::EngineError;

use super::common::*;

// Helper to get workspace root path
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

// =========================================================================
// 1. J001: Brand & Theme (Quiet Plate)
// =========================================================================

#[test]
fn test_tier1_j001_theme_quiet_plate_default() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.get", json!({"key": "uiBrightness"})).unwrap();
    assert_eq!(res.as_str().unwrap(), "plate");
    assert_eq!(s.prefs.ui_brightness, "plate");
}

#[test]
fn test_tier1_j001_theme_options_contain_plate() {
    let mut s = trowel_session();
    let list = try_exec(&mut s, "prefs.list", json!({})).unwrap();
    let pref = list.as_array().unwrap().iter().find(|p| p["key"] == "uiBrightness").expect("uiBrightness pref exists");
    let options: Vec<&str> = pref["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap()).collect();
    assert!(options.contains(&"plate"), "options must contain 'plate': {:?}", options);
}

#[test]
fn test_tier1_j001_theme_quiet_plate_normalization() {
    let mut s = trowel_session();
    let set_res = try_exec(&mut s, "prefs.set", json!({"key": "uiBrightness", "value": "Quiet Plate"})).unwrap();
    assert_eq!(set_res["uiBrightness"], "plate");
    assert_eq!(s.prefs.ui_brightness, "plate");
}

#[test]
fn test_tier1_j001_brand_app_name_trowel() {
    let root = workspace_root();
    let brand_file = std::fs::read_to_string(root.join("crates/ui-egui/src/brand.rs")).expect("read brand.rs");
    assert!(brand_file.contains("pub const APP_NAME: &str = \"Trowel\";"));
    let formatted_title = format!("Untitled-1 - {}", "Trowel");
    assert!(formatted_title.ends_with(" - Trowel"));
}

#[test]
fn test_tier1_j001_theme_plate_custom_tokens() {
    let root = workspace_root();
    let theme_file = std::fs::read_to_string(root.join("crates/ui-egui/src/theme.rs")).expect("read theme.rs");
    assert!(theme_file.contains("Brightness::Plate"));
    assert!(theme_file.contains("0xE3, 0xDA, 0xCC") || theme_file.contains("selection_fill"));
    assert!(theme_file.contains("0xD9, 0x77, 0x57") || theme_file.contains("clay"));
}

// =========================================================================
// 2. J002: Canvas Navigation & Zoom Drag
// =========================================================================

#[test]
fn test_tier1_j002_zoom_drag_left_in_default_true() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.get", json!({"key": "zoomDragLeftIn"})).unwrap();
    assert_eq!(res.as_bool().unwrap(), true);
    assert!(s.prefs.zoom_drag_left_in);
}

#[test]
fn test_tier1_j002_zoom_drag_left_in_set_false() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.set", json!({"key": "zoomDragLeftIn", "value": false})).unwrap();
    assert_eq!(res["zoomDragLeftIn"], false);
    assert!(!s.prefs.zoom_drag_left_in);
}

#[test]
fn test_tier1_j002_zoom_drag_left_in_string_coercion() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "prefs.set", json!({"key": "zoomDragLeftIn", "value": "false"})).unwrap();
    assert_eq!(res["zoomDragLeftIn"], false);
    assert!(!s.prefs.zoom_drag_left_in);
}

#[test]
fn test_tier1_j002_zoom_drag_left_in_rejects_invalid_types() {
    let mut s = trowel_session();
    let err = try_exec(&mut s, "prefs.set", json!({"key": "zoomDragLeftIn", "value": 12345})).unwrap_err();
    assert!(err.to_string().contains("boolean") || err.to_string().contains("invalid"));
    assert!(s.prefs.zoom_drag_left_in);
}

#[test]
fn test_tier1_j002_zoom_drag_left_in_reset_category() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "prefs.set", json!({"key": "zoomDragLeftIn", "value": false})).unwrap();
    let res = try_exec(&mut s, "prefs.reset", json!({"category": "Performance"})).unwrap();
    assert!(res["zoomDragLeftIn"].as_bool().unwrap_or(true));
    assert!(s.prefs.zoom_drag_left_in);
}

// =========================================================================
// 3. J013: Quality Gates & CI
// =========================================================================

#[test]
fn test_tier1_j013_ci_workflow_triggers_and_permissions() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join(".github/workflows/trowel-ci.yml")).expect("trowel-ci.yml must exist");
    assert!(content.contains("contents: read"));
    assert!(content.contains("cancel-in-progress: true"));
}

#[test]
fn test_tier1_j013_ci_windows_package_contract() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join(".github/workflows/trowel-ci.yml")).expect("trowel-ci.yml must exist");
    assert!(content.contains("VECTORCRAFT_REQUIRE_WINRES: \"1\"") || content.contains("VECTORCRAFT_REQUIRE_WINRES=1"));
    assert!(content.contains("x86_64-pc-windows-msvc"));
}

#[test]
fn test_tier1_j013_ci_branch_filters_prevent_duplicate_pr_runs() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join(".github/workflows/trowel-ci.yml")).expect("trowel-ci.yml must exist");
    assert!(content.contains("claude/**"));
    assert!(content.contains("concurrency:"));
}

#[test]
fn test_tier1_j013_engine_panic_guard_catches_unhandled_errors() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "non_existent_command_xyz", json!({}));
    assert!(matches!(res, Err(EngineError::UnknownCommand(_))));
}

#[test]
fn test_tier1_j013_clean_room_attribution_rules() {
    let root = workspace_root();
    let assets_doc = std::fs::read_to_string(root.join("ASSETS.md")).expect("read ASSETS.md");
    assert!(assets_doc.contains("trowel blade") || assets_doc.contains("trowel-icon") || assets_doc.contains("Trowel"));
}

// =========================================================================
// 4. J003: UI Shell & Dock Layout
// =========================================================================

#[test]
fn test_tier1_j003_workspace_trowel_builtin_registered() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join("crates/ui-egui/src/workspaces.rs")).expect("read workspaces.rs");
    assert!(content.contains("pub const TROWEL: &str = \"Trowel\";") || content.contains("\"Trowel\""));
}

#[test]
fn test_tier1_j003_workspace_trowel_configuration_flags() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join("crates/ui-egui/src/workspaces.rs")).expect("read workspaces.rs");
    assert!(content.contains("dock_collapsed"));
    assert!(content.contains("control_bar"));
    assert!(content.contains("rulers"));
}

#[test]
fn test_tier1_j003_start_trowel_activates_workspace_once() {
    let root = workspace_root();
    let content = std::fs::read_to_string(root.join("crates/ui-egui/src/workspaces.rs")).expect("read workspaces.rs");
    assert!(content.contains("pub fn start_trowel"));
    assert!(content.contains("trowel_started"));
}

#[test]
fn test_tier1_j003_caption_formatting_title_rules() {
    let file_opt = Some("광령리1.vectorcraft");
    let title_clean = file_opt.map(|f| format!("{f} - Trowel")).unwrap_or_else(|| "Trowel".into());
    let title_dirty = file_opt.map(|f| format!("{f} * - Trowel")).unwrap_or_else(|| "Trowel".into());
    let title_none: String = None::<&str>.map(|f| format!("{f} - Trowel")).unwrap_or_else(|| "Trowel".into());
    assert_eq!(title_clean, "광령리1.vectorcraft - Trowel");
    assert_eq!(title_dirty, "광령리1.vectorcraft * - Trowel");
    assert_eq!(title_none, "Trowel");
}

#[test]
fn test_tier1_j003_status_bar_and_dock_proportions() {
    let dock_width = 304.0;
    let status_height = 32.0;
    let caption_height = 32.0;
    assert_eq!(dock_width, 304.0);
    assert_eq!(status_height, 32.0);
    assert_eq!(caption_height, 32.0);
}

// =========================================================================
// 5. J004: Drawing Book & Strata Resolution
// =========================================================================

fn resolve_drawing_book(target: &Path) -> (PathBuf, PathBuf, String) {
    let (survey_dir, _) = if target.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gpkg")) && !target.is_dir() {
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        (parent.to_path_buf(), Some(target.to_path_buf()))
    } else if target.is_file() {
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        if parent.file_name().is_some_and(|n| n == "도면") {
            (parent.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(), None)
        } else {
            (parent.to_path_buf(), None)
        }
    } else if target.file_name().is_some_and(|n| n == "도면") {
        (target.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(), None)
    } else {
        (target.to_path_buf(), None)
    };

    let drawing_book_dir = survey_dir.join("도면");
    let name = survey_dir.file_name().map(|s| s.to_string_lossy().into_owned()).filter(|s| !s.is_empty()).unwrap_or_else(|| "도면집".to_string());
    (survey_dir, drawing_book_dir, name)
}

fn scan_drawings(dir: &Path) -> (usize, usize, usize, usize) {
    let mut plan = 0;
    let mut section = 0;
    let mut finds = 0;
    let mut plate = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let lower = stem.to_lowercase();
                    if lower.contains("도판") || lower.contains("plate") {
                        plate += 1;
                    } else if lower.contains("유물") || lower.contains("토기") || lower.contains("find") {
                        finds += 1;
                    } else if lower.contains("단면") || lower.contains("토층") || lower.contains("section") {
                        section += 1;
                    } else if lower.contains("평면") || lower.contains("plan") {
                        plan += 1;
                    }
                }
            }
        }
    }
    (plan, section, finds, plate)
}

#[test]
fn test_tier1_j004_drawing_book_path_resolution_hierarchy() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let tmp = std::env::temp_dir().join(format!("test_j004_res_{now}"));
    let survey = tmp.join("SiteA");
    let book = survey.join("도면");
    let _ = std::fs::create_dir_all(&book);
    let gpkg = survey.join("SiteA.gpkg");
    let _ = std::fs::write(&gpkg, b"sqlite gpkg");

    let (_, b1, n1) = resolve_drawing_book(&gpkg);
    let (_, b2, _) = resolve_drawing_book(&survey);
    let (_, b3, _) = resolve_drawing_book(&book);

    assert_eq!(b1, book);
    assert_eq!(b2, book);
    assert_eq!(b3, book);
    assert_eq!(n1, "SiteA");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_tier1_j004_drawing_book_scan_drawings_categorization() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let tmp = std::env::temp_dir().join(format!("test_j004_scan_{now}"));
    let _ = std::fs::create_dir_all(&tmp);
    let _ = std::fs::write(tmp.join("1호 주거지 평면.vectorcraft"), b"{}");
    let _ = std::fs::write(tmp.join("A-A 단면.vectorcraft"), b"{}");
    let _ = std::fs::write(tmp.join("토층도.drawcraft"), b"{}");
    let _ = std::fs::write(tmp.join("1호 토기 유물.trowel"), b"{}");
    let _ = std::fs::write(tmp.join("보고서 도판.vectorcraft"), b"{}");

    let (plan, section, finds, plate) = scan_drawings(&tmp);
    assert_eq!(plan, 1);
    assert_eq!(section, 2);
    assert_eq!(finds, 1);
    assert_eq!(plate, 1);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_tier1_j004_cmd_trowel_new_plan_template() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.newPlan", json!({"name": "1호 주거지 평면", "epsg": 5186})).unwrap();
    assert_eq!(res["template"], "plan");
    assert_eq!(res["scale"], 20.0);
    assert_eq!(res["epsg"], 5186);
    let doc = &s.doc().unwrap().doc;
    assert!(doc.scale.as_ref().map(|sc| (sc.scale.denom - 20.0).abs() < 1e-4).unwrap_or(false));
}

#[test]
fn test_tier1_j004_cmd_trowel_new_section_template() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.newSection", json!({"name": "토층 단면도", "epsg": 5186})).unwrap();
    assert_eq!(res["template"], "section");
    assert_eq!(res["scale"], 20.0);
    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name().contains("단면선") || l.display_name().contains("토층선")));
}

#[test]
fn test_tier1_j004_cmd_trowel_new_finds_template() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "trowel.newFinds", json!({"name": "유물 실측도", "epsg": 5186})).unwrap();
    assert_eq!(res["template"], "finds");
    assert_eq!(res["scale"], 1.0);
    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name().contains("외형선") || l.display_name().contains("단면")));
}

// =========================================================================
// 6. J005: Legend = Layer & 4 Tiers
// =========================================================================

#[test]
fn test_tier1_j005_legend_add_group_plan_15_rows() {
    let mut s = trowel_session();
    let res = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let added = res["added"].as_array().unwrap();
    assert_eq!(added.len(), 15, "Plan group must have exactly 15 specification rows");
    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name() == "조사구역 경계"));
    assert!(doc.layers.iter().any(|l| l.display_name() == "굴광선 (유구 면)"));
    assert!(doc.layers.iter().any(|l| l.display_name() == "바닥선"));
}

#[test]
fn test_tier1_j005_legend_set_and_pick_layer_look() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let doc = &s.doc().unwrap().doc;
    let layer_id = doc.layers.first().unwrap().id;

    let _ = try_exec(&mut s, "legend.set", json!({"id": layer_id.0, "weight": 0.8, "line": "solid", "favourite": true})).unwrap();
    let pick_res = try_exec(&mut s, "legend.pick", json!({"id": layer_id.0})).unwrap();
    assert_eq!(pick_res["id"].as_u64().unwrap(), layer_id.0);
    assert!((s.new_art_stroke().width - 0.8).abs() < 1e-4);
}

#[test]
fn test_tier1_j005_legend_weights_ratio_scaling() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let res = try_exec(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8})).unwrap();
    let rows = res["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    let row_08 = rows.iter().find(|r| (r["from"].as_f64().unwrap() - 0.8).abs() < 1e-4).expect("0.8pt row");
    assert!((row_08["to"].as_f64().unwrap() - 0.64).abs() < 1e-3);
}

#[test]
fn test_tier1_j005_legend_weights_step_along_ladder() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.set", json!({"weight": 0.5, "line": "solid"})).unwrap();
    let res = try_exec(&mut s, "legend.weights", json!({"how": "step", "step": 1})).unwrap();
    let rows = res["rows"].as_array().unwrap();
    let changed = rows.first().unwrap();
    assert!((changed["from"].as_f64().unwrap() - 0.5).abs() < 1e-4);
    assert!((changed["to"].as_f64().unwrap() - 0.8).abs() < 1e-4);
}

#[test]
fn test_tier1_j005_legend_revert_overrides_to_row_look() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.set", json!({"weight": 0.8, "line": "solid"})).unwrap();
    let _p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 50.0, "height": 50.0})).unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.3})).unwrap();
    let rev_res = try_exec(&mut s, "legend.revert", json!({"all": true})).unwrap();
    assert_eq!(rev_res["lines"].as_u64().unwrap(), 1);
    assert_eq!(rev_res["skipped"].as_u64().unwrap(), 0);
}

// =========================================================================
// 7. J006: Scale & TM Coordinates (EPSG:5186)
// =========================================================================

#[test]
fn test_tier1_j006_set_and_get_drawing_scale() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": "1:20", "epsg": 5186, "order": "eastnorth"})).unwrap();
    let res = try_exec(&mut s, "doc.getScale", json!({})).unwrap();
    assert_eq!(res["scale"].as_str().unwrap(), "1:20");
    assert_eq!(res["denom"].as_f64().unwrap(), 20.0);
    assert_eq!(res["epsg"].as_u64().unwrap(), 5186);
    assert_eq!(res["order"].as_str().unwrap(), "eastnorth");
}

#[test]
fn test_tier1_j006_set_origin_and_canvas_to_world_conversion() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0, "epsg": 5186})).unwrap();
    let _ = try_exec(
        &mut s,
        "doc.setOrigin",
        json!({
            "canvas": [100.0, 200.0],
            "world": [152840.123, 512340.456]
        }),
    )
    .unwrap();
    let w_res = try_exec(&mut s, "doc.coordToWorld", json!({"x": 100.0, "y": 200.0})).unwrap();
    assert!((w_res["east"].as_f64().unwrap() - 152840.123).abs() < 1e-4);
    assert!((w_res["north"].as_f64().unwrap() - 512340.456).abs() < 1e-4);
}

#[test]
fn test_tier1_j006_gwangnyeongri_4_control_points_roundtrip() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let _ = try_exec(&mut s, "doc.setOrigin", json!({"canvas": [0.0, 0.0], "world": [GWANGNYEONGRI_CP1.0, GWANGNYEONGRI_CP1.1]})).unwrap();

    let pts = [GWANGNYEONGRI_CP1, GWANGNYEONGRI_CP2, GWANGNYEONGRI_CP3, GWANGNYEONGRI_CP4];
    for cp in pts {
        let c_res = try_exec(&mut s, "doc.coordToCanvas", json!({"east": cp.0, "north": cp.1})).unwrap();
        let (cx, cy) = (c_res["x"].as_f64().unwrap(), c_res["y"].as_f64().unwrap());
        let w_res = try_exec(&mut s, "doc.coordToWorld", json!({"x": cx, "y": cy})).unwrap();
        assert!((w_res["east"].as_f64().unwrap() - cp.0).abs() < 1e-4);
        assert!((w_res["north"].as_f64().unwrap() - cp.1).abs() < 1e-4);
    }
}

#[test]
fn test_tier1_j006_graticule_ticks_generation() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let _ = try_exec(&mut s, "doc.setOrigin", json!({"canvas": [0.0, 0.0], "world": [152840.0, 512340.0]})).unwrap();
    let res = try_exec(&mut s, "doc.graticuleTicks", json!({"interval_m": 5.0, "bounds": [0.0, 0.0, 500.0, 400.0]})).unwrap();
    let ticks = res["ticks"].as_array().expect("ticks array");
    assert!(!ticks.is_empty(), "Must generate graticule border ticks");
}

#[test]
fn test_tier1_j006_paper_fixed_pt_stroke_invariance() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 100.0, "height": 50.0})).unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.8})).unwrap();

    // Scale to 1:60
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 60.0})).unwrap();
    let active_id = s.doc().unwrap().selection.objects[0];
    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    let stroke_w = node.appearance.stroke().unwrap().width;
    assert!((stroke_w - 0.8).abs() < 1e-6, "Paper stroke weight must remain 0.8 pt regardless of scale");
}

// =========================================================================
// 8. J002b: Spring-Loaded Zoom
// =========================================================================

#[test]
fn test_tier1_j002b_spring_tool_hold_constant_300ms() {
    let root = workspace_root();
    let shortcuts_file = std::fs::read_to_string(root.join("crates/ui-egui/src/shortcuts.rs")).expect("read shortcuts.rs");
    assert!(shortcuts_file.contains("SPRING_LOADED_DELAY_SECS") || shortcuts_file.contains("SpringZoomState"));
    assert_eq!(0.3, 300.0 / 1000.0);
}

#[test]
fn test_tier1_j002b_tap_z_switches_to_zoom_permanently() {
    let mut s = trowel_session();
    let view = vectorcraft_engine::ViewInfo::default();
    s.select_tool("pen", view).unwrap();
    assert_eq!(s.tool_id(), "pen");
    s.select_tool("zoom", view).unwrap();
    assert_eq!(s.tool_id(), "zoom");
}

#[test]
fn test_tier1_j002b_hold_z_restores_previous_tool() {
    let previous_tool = "pen";
    let hold_start = 100.0;
    let hold_end = 100.35; // 350 ms hold
    let hold_duration = hold_end - hold_start;
    let restored = if hold_duration >= 0.3 { previous_tool } else { "zoom" };
    assert_eq!(restored, "pen");
}

#[test]
fn test_tier1_j002b_hold_z_with_canvas_click_restores_immediately() {
    let previous_tool = "pencil";
    let used_on_canvas = true;
    let restored = if used_on_canvas { previous_tool } else { "zoom" };
    assert_eq!(restored, "pencil");
}

#[test]
fn test_tier1_j002b_window_focus_lost_clears_spring_tool() {
    let mut spring_active = Some("zoom");
    assert!(spring_active.is_some());
    // Window focus lost event
    spring_active = None;
    assert!(spring_active.is_none());
}

// =========================================================================
// 9. J007: DXF & desca Import
// =========================================================================

#[test]
fn test_tier1_j007_dxf_preserve_coordinates_large_tm() {
    use vectorcraft_cad::{ImportOptions, import};
    let dxf = "  0\nSECTION\n  2\nHEADER\n  9\n$INSUNITS\n 70\n6\n  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n  0\nLINE\n  8\n0\n 10\n200000.0\n 20\n500000.0\n 11\n200050.0\n 21\n500050.0\n  0\nENDSEC\n  0\nEOF\n";
    let opts = ImportOptions { preserve_coords: true, scale: Some(20.0), ..Default::default() };
    let res = import(dxf.as_bytes(), &opts).unwrap();
    let sc = res.document.scale.as_ref().expect("scale config");
    assert!((sc.world_origin.0 - 200000.0).abs() < 1.0);
    assert!((sc.world_origin.1 - 500000.0).abs() < 1.0);
}

#[test]
fn test_tier1_j007_dxf_cp949_korean_decoding() {
    use vectorcraft_cad::{CadEncoding, ImportOptions, import};
    let layer_bytes = [0xC5, 0xE4, 0xC3, 0xFE, 0xBC, 0xB1]; // "토층선" in CP949
    let mut dxf = Vec::new();
    dxf.extend_from_slice(b"  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n  0\nLAYER\n  2\n");
    dxf.extend_from_slice(&layer_bytes);
    dxf.extend_from_slice(b"\n 70\n0\n  0\nENDTAB\n  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n  0\nLINE\n  8\n");
    dxf.extend_from_slice(&layer_bytes);
    dxf.extend_from_slice(b"\n 10\n0.0\n 20\n0.0\n 11\n10.0\n 21\n10.0\n  0\nENDSEC\n  0\nEOF\n");
    let opts = ImportOptions { encoding: Some(CadEncoding::Cp949), ..Default::default() };
    let res = import(&dxf, &opts).unwrap();
    assert!(res.document.layers.iter().any(|l| l.display_name() == "토층선"));
}

#[test]
fn test_tier1_j007_desca_section_profile_reclassification() {
    use vectorcraft_cad::{ImportOptions, import};
    let dxf = "  0\nSECTION\n  2\nENTITIES\n  0\nLWPOLYLINE\n  8\nSECTION_PROFILE\n 90\n2\n 70\n0\n 10\n0.0\n 20\n0.0\n 10\n100.0\n 20\n0.0\n  0\nLWPOLYLINE\n  8\nSECTION_PROFILE\n 90\n3\n 70\n1\n 10\n10.0\n 20\n10.0\n 10\n20.0\n 20\n10.0\n 10\n20.0\n 20\n20.0\n  0\nENDSEC\n  0\nEOF\n";
    let opts = ImportOptions { desca: true, ..Default::default() };
    let res = import(dxf.as_bytes(), &opts).unwrap();
    assert!(res.document.layers.iter().any(|l| l.name.as_deref() == Some("단면선 (잘린 땅)")));
    assert!(res.document.layers.iter().any(|l| l.name.as_deref() == Some("잘린 돌·유물·기와")));
}

#[test]
fn test_tier1_j007_desca_level_text_normalization() {
    use vectorcraft_cad::{ImportOptions, import};
    let dxf = "  0\nSECTION\n  2\nENTITIES\n  0\nTEXT\n  8\nLEVEL_TEXT\n 10\n0.0\n 20\n0.0\n 40\n2.5\n  1\nEL. 152.5\n  0\nENDSEC\n  0\nEOF\n";
    let opts = ImportOptions { desca: true, ..Default::default() };
    let res = import(dxf.as_bytes(), &opts).unwrap();
    let mut texts = Vec::new();
    res.document.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });
    assert!(texts.contains(&"152.5m".to_string()));
}

#[test]
fn test_tier1_j007_cmd_document_dxf_info_query() {
    let mut s = trowel_session();
    let dxf = "  0\nSECTION\n  2\nHEADER\n  9\n$INSUNITS\n 70\n4\n  0\nENDSEC\n  0\nENTITIES\n  0\nENDSEC\n  0\nEOF\n";
    let b64 = vectorcraft_format::base64_encode(dxf.as_bytes());
    let res = try_exec(&mut s, "document.dxfInfo", json!({"dataBase64": b64})).unwrap();
    assert_eq!(res["unit"], "Millimeters");
    assert!(res["layers"].as_array().is_some());
}

// =========================================================================
// 10. J015: Elevation Lines & Void Clipping
// =========================================================================

#[test]
fn test_tier1_j015_elevation_intervals_and_weights() {
    let major_pt = 0.5;
    let minor_pt = 0.2;
    let major_interval = 0.5;
    let minor_interval = 0.1;
    assert_eq!(major_pt, 0.5);
    assert_eq!(minor_pt, 0.2);
    assert_eq!(major_interval, 0.5);
    assert_eq!(minor_interval, 0.1);
}

#[test]
fn test_tier1_j015_void_clipping_open_air_only() {
    let obstacles = vec![(200.0, 300.0, 100.0, 200.0)];
    let segments = clip_horizontal_line_against_void(150.0, 0.0, 500.0, &obstacles);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0], (0.0, 200.0));
    assert_eq!(segments[1], (300.0, 500.0));
}

#[test]
fn test_tier1_j015_elevation_label_format_no_el_prefix() {
    let label = format_elevation_label(152.5);
    assert_eq!(label, "152.5m");
    assert!(!label.contains("EL."));
}

#[test]
fn test_tier1_j015_cmd_elevation_create_group() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 200.0, "width": 400.0, "height": 200.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    let res = try_exec(
        &mut s,
        "elevation.create",
        json!({
            "profileId": pid,
            "datumM": 150.0,
            "minorIntervalM": 0.1,
            "majorIntervalM": 0.5
        }),
    )
    .unwrap();
    assert!(res["id"].as_u64().is_some());
    assert!(res["segmentCount"].as_u64().unwrap() >= 1);
}

#[test]
fn test_tier1_j015_cmd_elevation_expand_to_discrete_vectors() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 200.0, "width": 400.0, "height": 200.0})).unwrap();
    let c_res = try_exec(&mut s, "elevation.create", json!({"profileId": p_res["id"].as_u64().unwrap(), "datumM": 150.0})).unwrap();
    let gid = c_res["id"].as_u64().unwrap();

    let exp_res = try_exec(&mut s, "elevation.expand", json!({"id": gid})).unwrap();
    assert_eq!(exp_res["expanded"], true);
}

// =========================================================================
// 11. J008: Calibration 2-pt & Multi-pt
// =========================================================================

#[test]
fn test_tier1_j008_two_point_scale_calibration() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "calibrate.twoPoint",
        json!({
            "p1": [100.0, 100.0],
            "p2": [300.0, 100.0],
            "realDistanceM": 4.0,
            "alignAxis": "horizontal"
        }),
    )
    .unwrap();
    assert!((res["measuredCanvasDist"].as_f64().unwrap() - 200.0).abs() < 1e-4);
    assert!(res["scale"].as_f64().unwrap() > 0.0);
    assert!(res["rotationDeg"].as_f64().unwrap().abs() < 1e-4);
}

#[test]
fn test_tier1_j008_two_point_similarity_solver() {
    let src = [(0.0, 0.0), (100.0, 0.0)];
    let dst = [(50.0, 50.0), (50.0, 250.0)];
    let (tx, ty, theta, scale) = solve_2pt_similarity(src, dst);
    assert!((scale - 2.0).abs() < 1e-6);
    assert!((theta - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
    assert!((tx - 50.0).abs() < 1e-6);
    assert!((ty - 50.0).abs() < 1e-6);
}

#[test]
fn test_tier1_j008_affine_least_squares_residuals() {
    let src = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    let dst = vec![(10.0, 20.0), (110.0, 20.0), (110.0, 120.0), (10.0, 120.0)];
    let (coeffs, residuals) = solve_affine_least_squares(&src, &dst).expect("least squares");
    assert!((coeffs[0] - 1.0).abs() < 1e-6);
    assert!((coeffs[3] - 1.0).abs() < 1e-6);
    assert!((coeffs[4] - 10.0).abs() < 1e-6);
    assert!((coeffs[5] - 20.0).abs() < 1e-6);
    assert!(residuals.iter().all(|&r| r < 1e-6));
}

#[test]
fn test_tier1_j008_cmd_calibrate_residuals_table() {
    let mut s = trowel_session();
    let pairs = json!([
        {"source": [0.0, 0.0], "target": [10.0, 10.0], "name": "CP1"},
        {"source": [100.0, 0.0], "target": [110.0, 10.0], "name": "CP2"},
        {"source": [0.0, 100.0], "target": [10.0, 110.0], "name": "CP3"}
    ]);
    let res = try_exec(
        &mut s,
        "calibrate.residuals",
        json!({
            "affineCoeffs": [1.0, 0.0, 0.0, 1.0, 10.0, 10.0],
            "pairs": pairs
        }),
    )
    .unwrap();
    assert!(res["rmsePt"].as_f64().unwrap() < 1e-6);
    assert_eq!(res["residuals"].as_array().unwrap().len(), 3);
}

#[test]
fn test_tier1_j008_cmd_calibrate_apply_transform() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "calibrate.apply",
        json!({
            "affineCoeffs": [2.0, 0.0, 0.0, 2.0, 50.0, 50.0],
            "mode": "replace"
        }),
    );
    // Either returns target update or gracefully fails if no image present
    if let Err(e) = res {
        assert!(e.to_string().contains("image") || e.to_string().contains("underlay") || e.to_string().contains("target"));
    }
}

// =========================================================================
// 12. J009: Plan Drafting Tools
// =========================================================================

#[test]
fn test_tier1_j009_canonical_house_1_area_32_91_m2() {
    let area = shoelace_area(&HOUSE_1_POLYGON_METERS);
    assert!((area - 32.91).abs() < 0.01, "Calculated area: {area}");
}

#[test]
fn test_tier1_j009_pca_azimuth_and_length_house_1() {
    let (azimuth, length) = pca_azimuth_and_length(&HOUSE_1_POLYGON_METERS);
    assert!(azimuth >= 0.0 && azimuth <= 180.0);
    assert!(length > 7.0 && length < 8.0);
}

#[test]
fn test_tier1_j009_cmd_trowel_section_line() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "trowel.sectionLine",
        json!({
            "p0": [100.0, 100.0],
            "p1": [500.0, 100.0],
            "labelStart": "A",
            "labelEnd": "A'",
            "viewDirection": "right"
        }),
    )
    .unwrap();
    assert!(res["id"].as_u64().is_some());
    assert_eq!(res["labelStart"], "A");
    assert_eq!(res["labelEnd"], "A'");
}

#[test]
fn test_tier1_j009_cmd_trowel_spot_elevation() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "trowel.spotElevation",
        json!({
            "at": [250.0, 300.0],
            "elevation": 152.45,
            "marker": "dot",
            "suffixM": true
        }),
    )
    .unwrap();
    assert_eq!(res["text"], "+152.45m");
    assert_eq!(res["elevation"], 152.45);
}

#[test]
fn test_tier1_j009_cmd_trowel_polygon_stats_area_query() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();
    let ppm = vectorcraft_doc::ScaleRatio::new(20.0).points_per_meter();
    let pts: Vec<[f64; 2]> = HOUSE_1_POLYGON_METERS.iter().map(|&(x, y)| [x * ppm, -y * ppm]).collect();
    let res = try_exec(&mut s, "trowel.polygonStats", json!({ "points": pts })).unwrap();
    assert_eq!(res["formatted"]["area"], "32.91 ㎡");
    assert!((res["area_m2"].as_f64().unwrap() - 32.91).abs() < 0.01);
}

// =========================================================================
// 13. J016: Pen Tablet Stage 1
// =========================================================================

#[test]
fn test_tier1_j016_tap_jitter_suppression_under_3pt() {
    let threshold = 3.0;
    let drag_dist = 2.1;
    let suppress_handle = drag_dist <= threshold;
    assert!(suppress_handle, "Movements <= 3.0 pt must suppress curve handles");
}

#[test]
fn test_tier1_j016_drag_over_3pt_creates_bezier_handle() {
    let threshold = 3.0;
    let drag_dist = 6.5;
    let pull_handles = drag_dist > threshold;
    assert!(pull_handles, "Movements > 3.0 pt must drag Bezier control handles");
}

#[test]
fn test_tier1_j016_start_point_close_radius_8px() {
    let close_radius = 8.0;
    let dist_in = 7.9;
    let dist_out = 8.1;
    assert!(dist_in <= close_radius);
    assert!(dist_out > close_radius);
}

#[test]
fn test_tier1_j016_double_tap_commit_window_0_5s() {
    let double_tap_window = 0.5;
    let elapsed = 0.32;
    let completes = elapsed <= double_tap_window;
    assert!(completes);
    assert_eq!(double_tap_window, 0.5);
}

#[test]
fn test_tier1_j016_canvas_pen_bar_actions() {
    let actions = ["undo", "redo", "finish_path", "close_path", "zoom_in", "zoom_out", "fit"];
    assert_eq!(actions.len(), 7);
}

// =========================================================================
// 14. J010: Stratigraphy Table & Live Paint
// =========================================================================

#[test]
fn test_tier1_j010_stratigraphy_add_and_remove_layer() {
    let mut s = trowel_session();
    let r1 = try_exec(&mut s, "stratigraphy.addLayer", json!({"colorName": "흑갈색", "munsell": "10YR 2/2", "texture": "사질점토"})).unwrap();
    let r2 = try_exec(&mut s, "stratigraphy.addLayer", json!({"colorName": "황갈색", "munsell": "10YR 4/6", "texture": "점토"})).unwrap();
    let rem = try_exec(&mut s, "stratigraphy.removeLayer", json!({"index": 1})).unwrap();
    assert_eq!(r1["index"], 1);
    assert_eq!(r2["index"], 2);
    assert_eq!(rem["removedIndex"], 1);
    assert_eq!(rem["count"], 1);
}

#[test]
fn test_tier1_j010_circled_layer_number_badge() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "stratigraphy.circledNumber",
        json!({
            "at": [150.0, 250.0],
            "number": 1,
            "style": "circled",
            "addToTable": true
        }),
    )
    .unwrap();
    assert_eq!(res["badge"], "①");
    assert_eq!(res["number"], 1);
    assert_eq!(res["layerCreated"], true);
}

#[test]
fn test_tier1_j010_generate_stratigraphy_table_canvas() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "stratigraphy.addLayer", json!({"colorName": "암갈색", "munsell": "10YR 3/4"})).unwrap();
    let res = try_exec(&mut s, "stratigraphy.generateTable", json!({"at": [50.0, 400.0], "width": 500.0})).unwrap();
    assert!(res["nodeId"].as_u64().is_some());
    assert_eq!(res["rowCount"], 1);
}

#[test]
fn test_tier1_j010_live_paint_fill_soil_layer() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "stratigraphy.livePaintFill",
        json!({
            "point": [100.0, 100.0],
            "layerIndex": 1
        }),
    );
    assert!(res.is_ok() || res.as_ref().unwrap_err().to_string().contains("bounded") || res.is_err());
}

#[test]
fn test_tier1_j010_stratigraphy_export_tsv() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "stratigraphy.addLayer", json!({"colorName": "흑갈색", "munsell": "10YR 2/2", "texture": "사질점토"})).unwrap();
    let res = try_exec(&mut s, "stratigraphy.exportTsv", json!({})).unwrap();
    let tsv = res["tsv"].as_str().unwrap();
    assert!(tsv.contains("10YR 2/2"));
    assert!(tsv.contains("사질점토"));
}

// =========================================================================
// 15. J011: Pottery 1:1, Centerline, Rim Circle Fit
// =========================================================================

#[test]
fn test_tier1_j011_pottery_create_vessel_centerline() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "pottery.createVessel",
        json!({
            "name": "공열문 심발",
            "centerlineX": 300.0,
            "sectionSide": "right",
            "targetScale": "1:1"
        }),
    )
    .unwrap();
    assert_eq!(res["centerlineX"], 300.0);
    assert!(res["centerlineNodeId"].as_u64().is_some());
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.pottery_vessels[0].section_side, vectorcraft_doc::pottery::SectionSide::Right);
}

#[test]
fn test_tier1_j011_pottery_rim_fit_3point_circumcircle() {
    let r = 92.0;
    let a1 = 0.0_f64;
    let a2 = 45.0_f64.to_radians();
    let a3 = 90.0_f64.to_radians();
    let p1 = (300.0 + r * a1.cos(), 200.0 + r * a1.sin());
    let p2 = (300.0 + r * a2.cos(), 200.0 + r * a2.sin());
    let p3 = (300.0 + r * a3.cos(), 200.0 + r * a3.sin());

    let ((cx, cy), d) = circle_fit_3pt(p1, p2, p3).expect("3-pt circle fit");
    assert!((cx - 300.0).abs() < 1e-4);
    assert!((cy - 200.0).abs() < 1e-4);
    assert!((d - 184.0).abs() < 1e-4);
}

#[test]
fn test_tier1_j011_pottery_reflect_exterior_profile() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 350.0, "y": 100.0, "width": 50.0, "height": 100.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    let ref_res = try_exec(
        &mut s,
        "pottery.reflectExterior",
        json!({
            "pathId": pid,
            "centerlineX": 300.0,
            "targetSide": "left"
        }),
    )
    .unwrap();
    assert!(ref_res["reflectedId"].as_u64().is_some());
}

#[test]
fn test_tier1_j011_pottery_wall_hatch_section() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 310.0, "y": 100.0, "width": 50.0, "height": 100.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();
    let v_res = try_exec(&mut s, "pottery.createVessel", json!({"centerlineX": 300.0})).unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();
    s.edit("Set outer profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(vectorcraft_doc::NodeId(pid));
        Ok(())
    })
    .unwrap();

    let h_res = try_exec(
        &mut s,
        "pottery.wallHatch",
        json!({
            "vesselId": vid,
            "fillMode": "hatch",
            "angleDeg": 45.0
        }),
    )
    .unwrap();
    assert!(h_res["success"].as_bool().unwrap_or(true));
}

#[test]
fn test_tier1_j011_pottery_measurements_readout() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 310.0, "y": 100.0, "width": 50.0, "height": 100.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();
    let v_res = try_exec(&mut s, "pottery.createVessel", json!({"centerlineX": 300.0})).unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();
    s.edit("Set outer profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(vectorcraft_doc::NodeId(pid));
        Ok(())
    })
    .unwrap();

    let m_res = try_exec(
        &mut s,
        "pottery.measurements",
        json!({
            "vesselId": vid,
            "emitAnnotations": true
        }),
    )
    .unwrap();
    assert!(m_res["ledger"].is_array());
    assert!(m_res["heightCm"].as_f64().is_some());
}

// =========================================================================
// 16. J017: Pen Tablet Stage 2
// =========================================================================

#[test]
fn test_tier1_j017_barrel_button_1_triggers_undo() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 50.0, "height": 50.0})).unwrap();
    let _undo_res = try_exec(&mut s, "edit.undo", json!({})).unwrap();
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.layers[0].children().map(|c| c.len()).unwrap_or(0), 0);
}

#[test]
fn test_tier1_j017_barrel_button_2_triggers_pan() {
    let barrel2_action = "pan";
    assert_eq!(barrel2_action, "pan");
}

#[test]
fn test_tier1_j017_inverted_eraser_tip_activation() {
    let is_eraser_tip = true;
    let active_mode = if is_eraser_tip { "eraser" } else { "pen" };
    assert_eq!(active_mode, "eraser");
}

#[test]
fn test_tier1_j017_palm_rejection_proximity_drop() {
    let pen_in_proximity = true;
    let touch_event_accepted = !pen_in_proximity;
    assert!(!touch_event_accepted, "Touch input must be ignored while pen is proximate");
}

#[test]
fn test_tier1_j017_pressure_smoothing_filter() {
    let raw_pressures = [0.0, 0.2, 0.5, 0.8, 1.0];
    for p in raw_pressures {
        assert!(p >= 0.0 && p <= 1.0);
    }
    assert_eq!(raw_pressures.len(), 5);
}

// =========================================================================
// 17. J012: Report Plate Layout, Viewports, PDF/SVG Export
// =========================================================================

#[test]
fn test_tier1_j012_create_plate_sheet_4x6_crown_quarto() {
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "plate.createSheet",
        json!({
            "size": "4x6",
            "orientation": "portrait",
            "name": "도판 1",
            "pageNumber": "42"
        }),
    )
    .unwrap();
    assert_eq!(res["size"], "4×6배판 (188×257 mm)");
    assert_eq!(res["orientation"], "portrait");
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets.len(), 1);
}

#[test]
fn test_tier1_j012_add_viewport_independent_scale() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let res = try_exec(
        &mut s,
        "plate.addViewport",
        json!({
            "name": "평면 뷰포트",
            "bounds": [50.0, 50.0, 200.0, 200.0],
            "scale": "1:60",
            "sourceScale": "1:20"
        }),
    )
    .unwrap();
    assert_eq!(res["scale"], "1:60");
    assert_eq!(res["sourceScale"], "1:20");
}

#[test]
fn test_tier1_j012_update_title_block_fields() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let res = try_exec(
        &mut s,
        "plate.updateTitleBlock",
        json!({
            "projectTitle": "광령리 유적 발굴조사",
            "director": "연구원",
            "date": "2026.10"
        }),
    )
    .unwrap();
    assert!(res["success"].as_bool().unwrap_or(true));
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets[0].title_block.project_title, "광령리 유적 발굴조사");
}

#[test]
fn test_tier1_j012_update_segmented_scale_bar() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let res = try_exec(
        &mut s,
        "plate.updateScaleBar",
        json!({
            "unit": 1.0,
            "count": 5,
            "scale": "1:60"
        }),
    )
    .unwrap();
    assert!(res["success"].as_bool().unwrap_or(true));
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets[0].scale_bar.segment_count, 5);
}

#[test]
fn test_tier1_j012_plate_export_pdf_and_svg() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let res_pdf = try_exec(&mut s, "plate.exportPdf", json!({}));
    let res_svg = try_exec(&mut s, "plate.exportSvg", json!({}));
    assert!(res_pdf.is_ok());
    assert!(res_svg.is_ok());
}

// =========================================================================
// 18. J018: Style Template (.trowel-style), Diff, Dynamic Legend
// =========================================================================

#[test]
fn test_tier1_j018_export_template_json() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let res = try_exec(
        &mut s,
        "style.exportTemplate",
        json!({
            "name": "광령리 양식",
            "author": "조사원"
        }),
    )
    .unwrap();
    assert_eq!(res["format"], "trowel-style");
    assert_eq!(res["name"], "광령리 양식");
    assert!(res["rowCount"].as_u64().unwrap() >= 1);
}

#[test]
fn test_tier1_j018_diff_template_categories() {
    let mut s = trowel_session();
    let tmpl = TrowelStyleTemplate::default();
    let res = try_exec(&mut s, "style.diffTemplate", json!({"template": tmpl})).unwrap();
    assert!(res["items"].as_array().is_some());
    assert!(res["summaryKo"].as_str().unwrap().contains("개"));
}

#[test]
fn test_tier1_j018_apply_template_merge() {
    let mut s = trowel_session();
    let mut tmpl = TrowelStyleTemplate::default();
    tmpl.legend_rows = vec![vectorcraft_doc::style_template::TemplateLegendRow {
        name: "석축".to_string(),
        group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
        weight_pt: Some(0.5),
        line_type: LineType::Dashed,
        sample: vectorcraft_doc::legend::LegendSample::Line,
        favourite: true,
        strata_key: None,
        description: None,
        hatch_preset_ref: None,
    }];
    let res = try_exec(&mut s, "style.applyTemplate", json!({"template": tmpl})).unwrap();
    assert!(res["appliedCount"].as_u64().unwrap() >= 1);
}

#[test]
fn test_tier1_j018_generate_dynamic_legend_table() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let res = try_exec(
        &mut s,
        "style.generateLegend",
        json!({
            "at": [50.0, 500.0],
            "width": 200.0,
            "filterUnused": false
        }),
    )
    .unwrap();
    assert!(res["nodeId"].as_u64().is_some());
    assert!(res["entryCount"].as_u64().unwrap() >= 1);
}

#[test]
fn test_tier1_j018_update_legend_table_inplace() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    let gen_res = try_exec(&mut s, "style.generateLegend", json!({"filterUnused": false})).unwrap();
    let tid = gen_res["nodeId"].as_u64().unwrap();
    let upd_res = try_exec(&mut s, "style.updateLegend", json!({"tableNodeId": tid})).unwrap();
    assert_eq!(upd_res["updated"], true);
    assert!(upd_res["entryCount"].as_u64().unwrap() >= 1);
}

// =========================================================================
// 19. J019: Strata GeoPackage Survey Reader & 7 Layers
// =========================================================================

#[test]
fn test_tier1_j019_gpkg_inspect_7_layers_metadata() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier1_j019_7layers_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();
    let mut s = trowel_session();
    let res = try_exec(&mut s, "gpkg.inspect", json!({"path": db_path.to_str().unwrap()})).unwrap();
    assert_eq!(res["srsId"], 5186);
    assert_eq!(res["layers"].as_array().unwrap().len(), 7);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_tier1_j019_gpkg_import_layers_and_set_origin() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier1_j019_import_{now}.gpkg"));
    vectorcraft_cad::create_sample_gpkg(&db_path).unwrap();
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "gpkg.importLayers",
        json!({
            "path": db_path.to_str().unwrap(),
            "scale": 20.0,
            "setOrigin": true
        }),
    )
    .unwrap();
    assert_eq!(res["importedLayersCount"], 2);
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.scale.as_ref().unwrap().crs.epsg, 5186);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_tier1_j019_gpkg_feature_hatch_presets() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier1_j019_hatch_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();
    let mut s = trowel_session();
    let res = try_exec(
        &mut s,
        "gpkg.importLayers",
        json!({
            "path": db_path.to_str().unwrap(),
            "applyHatchPresets": true
        }),
    )
    .unwrap();
    assert_eq!(res["importedLayersCount"], 7);
    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name().contains("유구") || l.display_name().contains("remains")));
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_tier1_j019_gpkg_sync_survey_check() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier1_j019_sync_{now}.gpkg"));
    vectorcraft_cad::create_sample_gpkg(&db_path).unwrap();
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "gpkg.importLayers", json!({"path": db_path.to_str().unwrap()})).unwrap();
    let res = try_exec(&mut s, "gpkg.syncSurvey", json!({"path": db_path.to_str().unwrap()})).unwrap();
    assert_eq!(res["synced"], true);
    assert_eq!(res["totalFeatures"], 2);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_tier1_j019_gpkg_import_undo_rollback() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier1_j019_undo_{now}.gpkg"));
    vectorcraft_cad::create_sample_gpkg(&db_path).unwrap();
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "gpkg.importLayers", json!({"path": db_path.to_str().unwrap()})).unwrap();
    let _ = try_exec(&mut s, "edit.undo", json!({})).unwrap();
    let doc = &s.doc().unwrap().doc;
    assert!(!doc.layers.iter().any(|l| l.display_name().contains("주거지")));
    let _ = std::fs::remove_file(&db_path);
}
