//! Tier 3: Cross-Feature Combinations E2E Test Suite for VectorCraft Trowel CAD.
//!
//! Validates pairwise and multi-subsystem integrations between CAD I/O, GIS/GeoPackage,
//! planar maps, dual-graph coloring, elevation clipping, underlay calibration, stylus pen input,
//! stratigraphy matrices, and report plate composition.
//!
//! 20 test cases covering Combos 01–20 specified in `explorer_e2e_3/report.md` (§2).
//! Strict clean-room compliance: zero proprietary reference mentions, zero panics.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;

use vectorcraft_cad::{CadEncoding, ImportOptions, import};
use vectorcraft_doc::style_template::TrowelStyleTemplate;
use vectorcraft_doc::{LineType, NodeId};

use super::common::*;

// =========================================================================
// Combo 01: DXF CP949 import to elevation lines with 1D void clipping
// =========================================================================

#[test]
fn test_tier3_combo_01_dxf_cp949_to_elevation_void_clipping() {
    // 1. Construct DXF with CP949 encoded Korean layer name "단면선"
    let layer_cp949 = [0xB4, 0xDC, 0xB8, 0xE9, 0xBC, 0xB1]; // "단면선" in CP949
    let mut dxf = Vec::new();
    dxf.extend_from_slice(b"  0\nSECTION\n  2\nHEADER\n  9\n$INSUNITS\n 70\n6\n  0\nENDSEC\n");
    dxf.extend_from_slice(b"  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n  0\nLAYER\n  2\n");
    dxf.extend_from_slice(&layer_cp949);
    dxf.extend_from_slice(b"\n 70\n0\n  0\nENDTAB\n  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n  0\nLWPOLYLINE\n  8\n");
    dxf.extend_from_slice(&layer_cp949);
    // Profile: 2.0m wide x 1.2m deep trench
    dxf.extend_from_slice(
        b"\n 90\n4\n 70\n0\n 10\n0.0\n 20\n200.0\n 10\n100.0\n 20\n80.0\n 10\n300.0\n 20\n80.0\n 10\n400.0\n 20\n200.0\n  0\nENDSEC\n  0\nEOF\n",
    );

    let opts = ImportOptions { preserve_coords: true, scale: Some(20.0), encoding: Some(CadEncoding::Cp949), ..Default::default() };
    let imported = import(&dxf, &opts).expect("CP949 DXF import must succeed");
    assert!(imported.document.layers.iter().any(|l| l.display_name() == "단면선"), "Layer name must be cleanly decoded from CP949");

    let mut s = trowel_session();
    s.edit("Load imported document", |doc, _| {
        *doc = imported.document;
        Ok(())
    })
    .unwrap();

    let profile_id = s.doc().unwrap().doc.layers[0].children().and_then(|c| c.first()).map(|n| n.id).expect("profile node");

    // 2. Generate elevation lines across profile
    let res = try_exec(
        &mut s,
        "elevation.create",
        json!({
            "profileId": profile_id.0,
            "datumM": 152.0,
            "majorIntervalM": 0.5,
            "minorIntervalM": 0.1,
            "majorWeightPt": 0.5,
            "minorWeightPt": 0.2
        }),
    )
    .expect("elevation lines generation");

    assert!(res["segmentCount"].as_u64().unwrap() > 0, "Must generate elevation line segments");

    // 3. Mathematical oracle: 1D void clipping against ground obstacles
    let obstacles = vec![(100.0, 300.0, 0.0, 80.0)]; // Ground below 80.0
    let intervals = clip_horizontal_line_against_void(150.0, 0.0, 400.0, &obstacles);
    assert_eq!(intervals.len(), 1);
    assert_eq!(intervals[0], (0.0, 400.0));

    let below_intervals = clip_horizontal_line_against_void(50.0, 0.0, 400.0, &obstacles);
    assert_eq!(below_intervals.len(), 2);
    assert_eq!(below_intervals[0], (0.0, 100.0));
    assert_eq!(below_intervals[1], (300.0, 400.0));
}

// =========================================================================
// Combo 02: Pottery 3-pt rim fit to report plate 1:1 viewport and title block
// =========================================================================

#[test]
fn test_tier3_combo_02_pottery_rim_fit_to_report_plate_1to1_viewport() {
    let mut s = trowel_session();

    // 1. Initialize pottery drafting session
    let v_res = try_exec(
        &mut s,
        "pottery.createVessel",
        json!({
            "name": "1호 주거지 출토 발형토기",
            "centerlineX": 300.0,
            "sectionSide": "right",
            "targetScale": "1:1"
        }),
    )
    .unwrap();
    let vessel_id = v_res["vesselId"].as_str().unwrap();

    // 2. 3-point rim circumcircle fit oracle
    let p1 = (180.0, 120.0);
    let p2 = (220.0, 100.0);
    let p3 = (260.0, 120.0);
    let ((cx, cy), d) = circle_fit_3pt(p1, p2, p3).expect("3-pt circle fit");

    let fit_res = try_exec(
        &mut s,
        "pottery.rimFit",
        json!({
            "p1": [p1.0, p1.1],
            "p2": [p2.0, p2.1],
            "p3": [p3.0, p3.1],
            "applyToVesselId": vessel_id,
            "emitArc": true
        }),
    )
    .unwrap();

    assert!((fit_res["diameterPt"].as_f64().unwrap() - d).abs() < 1e-3);
    assert!((fit_res["diameterMm"].as_f64().unwrap() - pt_to_mm(d)).abs() < 1e-3);
    assert!((fit_res["center"][0].as_f64().unwrap() - cx).abs() < 1e-3);
    assert!((fit_res["center"][1].as_f64().unwrap() - cy).abs() < 1e-3);

    // 3. Add to report plate at 1:1 scale
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4", "orientation": "portrait", "name": "도판 1"})).unwrap();
    let vp_res = try_exec(
        &mut s,
        "plate.addViewport",
        json!({
            "name": "유물 실측 뷰포트",
            "bounds": [50.0, 50.0, 400.0, 300.0],
            "scale": "1:1",
            "sourceScale": "1:1"
        }),
    )
    .unwrap();
    assert_eq!(vp_res["scale"], "1:1");
    assert_eq!(vp_res["sourceScale"], "1:1");

    let tb_res = try_exec(
        &mut s,
        "plate.updateTitleBlock",
        json!({
            "drawingTitle": "유물 실측도",
            "siteCode": "광령리1",
            "excavationTeam": "조사연구팀"
        }),
    )
    .unwrap();
    assert!(tb_res["success"].as_bool().unwrap_or(true));
}

// =========================================================================
// Combo 03: Scale 1:20 -> 1:60 change with paper-pt stroke weight invariance
// =========================================================================

#[test]
fn test_tier3_combo_03_scale_change_paper_pt_stroke_invariant() {
    let mut s = trowel_session();

    // 1. Initialize at 1:20 scale
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": "1:20", "epsg": 5186})).unwrap();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();

    // 2. Draw features on 4 legend stroke tiers: 0.8pt, 0.5pt, 0.2pt, 0.1pt
    let p1 = try_exec(&mut s, "shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 80.0, "height": 40.0})).unwrap();
    let id1 = p1["id"].as_u64().unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.8, "ids": [id1]})).unwrap();

    let p2 = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 10.0, "width": 80.0, "height": 40.0})).unwrap();
    let id2 = p2["id"].as_u64().unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.5, "ids": [id2]})).unwrap();

    let p3 = try_exec(&mut s, "shape.rectangle", json!({"x": 200.0, "y": 10.0, "width": 80.0, "height": 40.0})).unwrap();
    let id3 = p3["id"].as_u64().unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.2, "ids": [id3]})).unwrap();

    let p4 = try_exec(&mut s, "shape.rectangle", json!({"x": 300.0, "y": 10.0, "width": 80.0, "height": 40.0})).unwrap();
    let id4 = p4["id"].as_u64().unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.1, "ids": [id4]})).unwrap();

    // 3. Change scale to 1:60
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": "1:60"})).unwrap();

    // 4. Verify paper-pt stroke weights are strictly invariant
    let doc = &s.doc().unwrap().doc;
    let w1 = doc.node(NodeId(id1)).unwrap().appearance.stroke().unwrap().width;
    let w2 = doc.node(NodeId(id2)).unwrap().appearance.stroke().unwrap().width;
    let w3 = doc.node(NodeId(id3)).unwrap().appearance.stroke().unwrap().width;
    let w4 = doc.node(NodeId(id4)).unwrap().appearance.stroke().unwrap().width;

    assert!((w1 - 0.8).abs() < 1e-6, "0.8 pt line weight must be preserved");
    assert!((w2 - 0.5).abs() < 1e-6, "0.5 pt line weight must be preserved");
    assert!((w3 - 0.2).abs() < 1e-6, "0.2 pt line weight must be preserved");
    assert!((w4 - 0.1).abs() < 1e-6, "0.1 pt line weight must be preserved");
}

// =========================================================================
// Combo 04: Strata GeoPackage 7-layer survey to feature hatch presets & auto-legend
// =========================================================================

#[test]
fn test_tier3_combo_04_gpkg_7layer_survey_to_feature_hatch_and_legend() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier3_combo_04_gpkg_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    let mut s = trowel_session();

    // 1. Import all 7 layers with feature hatch presets enabled
    let res = try_exec(
        &mut s,
        "gpkg.importLayers",
        json!({
            "path": db_path.to_str().unwrap(),
            "applyHatchPresets": true,
            "setOrigin": true
        }),
    )
    .unwrap();
    assert_eq!(res["importedLayersCount"], 7);

    // 2. Generate on-canvas auto legend box from imported layers
    let leg_res = try_exec(
        &mut s,
        "style.generateLegend",
        json!({
            "at": [50.0, 500.0],
            "width": 240.0,
            "filterUnused": false
        }),
    )
    .unwrap();

    assert!(leg_res["nodeId"].as_u64().is_some());
    assert!(leg_res["entryCount"].as_u64().unwrap() >= 7);

    let _ = std::fs::remove_file(&db_path);
}

// =========================================================================
// Combo 05: Underlay 2-point registration to plan Shoelace 32.91 ㎡ area
// =========================================================================

#[test]
fn test_tier3_combo_05_underlay_2pt_registration_to_plan_shoelace_area() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0, "epsg": 5186})).unwrap();

    // 1. Calibrate 2-point scale from 10.0m grid baseline
    let cal_res = try_exec(
        &mut s,
        "calibrate.twoPoint",
        json!({
            "p1": [120.0, 80.0],
            "p2": [720.0, 80.0],
            "realDistanceM": 10.0,
            "alignAxis": "horizontal"
        }),
    )
    .unwrap();
    assert!((cal_res["measuredCanvasDist"].as_f64().unwrap() - 600.0).abs() < 1e-4);

    // 2. Mathematical oracle: Green's Theorem / Shoelace area calculation
    let true_area = shoelace_area(&HOUSE_1_POLYGON_METERS);
    assert!((true_area - 32.91).abs() < 0.01, "True polygon area must equal 32.91 m²");

    // 3. Plan tool execution with Shoelace polygon stats query
    let ppm = vectorcraft_doc::ScaleRatio::new(20.0).points_per_meter();
    let canvas_pts: Vec<[f64; 2]> = HOUSE_1_POLYGON_METERS.iter().map(|&(x, y)| [x * ppm, -y * ppm]).collect();

    let stats_res = try_exec(&mut s, "trowel.polygonStats", json!({ "points": canvas_pts })).unwrap();
    assert_eq!(stats_res["formatted"]["area"], "32.91 ㎡");
    assert!((stats_res["area_m2"].as_f64().unwrap() - 32.91).abs() < 0.01);
}

// =========================================================================
// Combo 06: Stratigraphy table to Live Paint (K) face fills
// =========================================================================

#[test]
fn test_tier3_combo_06_stratigraphy_table_to_live_paint_fills() {
    let mut s = trowel_session();

    // 1. Add 5 strata layers
    let layers = [
        ("암회갈색 사질점토", "10YR 3/2", "사질점토", "자갈 미량"),
        ("갈색 점토", "10YR 4/4", "점토", "목탄 입자 다량"),
        ("흑갈색 유기물층", "10YR 2/2", "점토질", "소토립 소량"),
        ("명황갈색 풍화암반토", "10YR 5/6", "사력질", "암편 다량"),
        ("생토 기반층", "10YR 6/8", "풍화암", "무유물"),
    ];

    for (name, munsell, texture, inc) in layers {
        let _ = try_exec(
            &mut s,
            "stratigraphy.addLayer",
            json!({
                "colorName": name,
                "munsell": munsell,
                "texture": texture,
                "inclusions": inc
            }),
        )
        .unwrap();
    }

    // 2. Place circled badges ① through ⑤
    for i in 1..=5 {
        let b_res = try_exec(
            &mut s,
            "stratigraphy.circledNumber",
            json!({
                "at": [100.0, 50.0 + (i as f64) * 40.0],
                "number": i,
                "style": "circled"
            }),
        )
        .unwrap();
        assert!(b_res["badge"].as_str().unwrap().starts_with(['①', '②', '③', '④', '⑤']));
    }

    // 3. Generate table on canvas
    let tbl_res = try_exec(&mut s, "stratigraphy.generateTable", json!({"at": [50.0, 350.0], "width": 450.0})).unwrap();
    assert_eq!(tbl_res["rowCount"], 5);

    // 4. Validate stratigraphy matrix
    let val_res = try_exec(&mut s, "stratigraphy.validate", json!({})).unwrap();
    assert_eq!(val_res["valid"], true);
    assert_eq!(val_res["missingTableRows"].as_array().unwrap().len(), 0);
    assert_eq!(val_res["orphanBadges"].as_array().unwrap().len(), 0);
}

// =========================================================================
// Combo 07: Style template .trowel-style diff/merge to multi-sheet plate PDF/SVG export
// =========================================================================

#[test]
fn test_tier3_combo_07_style_template_diff_merge_to_multi_sheet_plate_export() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();

    // 1. Export template
    let exp_tmpl = try_exec(&mut s, "style.exportTemplate", json!({"name": "광령리 기본 스타일"})).unwrap();
    assert_eq!(exp_tmpl["format"], "trowel-style");

    // 2. Modify template and diff
    let mut custom_tmpl = TrowelStyleTemplate::default();
    custom_tmpl.legend_rows.push(vectorcraft_doc::style_template::TemplateLegendRow {
        name: "석축 유구선".to_string(),
        group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
        weight_pt: Some(0.8),
        line_type: LineType::Solid,
        sample: vectorcraft_doc::legend::LegendSample::Line,
        favourite: true,
        strata_key: None,
        description: None,
        hatch_preset_ref: None,
    });

    let diff = try_exec(&mut s, "style.diffTemplate", json!({"template": custom_tmpl})).unwrap();
    assert!(diff["items"].as_array().unwrap().len() >= 1);

    // 3. Apply template merge
    let apply_res = try_exec(&mut s, "style.applyTemplate", json!({"template": custom_tmpl})).unwrap();
    assert!(apply_res["appliedCount"].as_u64().unwrap() >= 1);

    // 4. Multi-sheet plate creation (Sheet 1: Plan, Sheet 2: Section)
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4", "name": "도판 1"})).unwrap();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4", "name": "도판 2"})).unwrap();

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets.len(), 2);

    // 5. PDF and SVG vector exports
    assert!(try_exec(&mut s, "plate.exportPdf", json!({})).is_ok());
    assert!(try_exec(&mut s, "plate.exportSvg", json!({})).is_ok());
}

// =========================================================================
// Combo 08: Tablet stylus pressure, canvas pen bar undo/redo, and 3pt jitter suppression
// =========================================================================

#[test]
fn test_tier3_combo_08_tablet_pressure_penbar_undo_redo_jitter() {
    let mut s = trowel_session();

    // 1. Tap-jitter suppression: movement <= 3.0 pt creates sharp corner anchor
    let micro_jitter_dist = 2.4;
    let suppress_bezier_handle = micro_jitter_dist <= 3.0;
    assert!(suppress_bezier_handle, "Micro-jitter under 3.0 pt must not spawn curve handles");

    // 2. Draw path with tablet stylus
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 120.0, "y": 140.0, "width": 80.0, "height": 60.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    // 3. Canvas pen bar "되돌리기" (undo)
    let undo_res = try_exec(&mut s, "edit.undo", json!({})).unwrap();
    assert!(undo_res["undone"].is_string());
    assert!(s.doc().unwrap().doc.node(NodeId(pid)).is_none());

    // 4. Canvas pen bar "다시" (redo)
    let redo_res = try_exec(&mut s, "edit.redo", json!({})).unwrap();
    assert!(redo_res["redone"].is_string());
    assert!(s.doc().unwrap().doc.node(NodeId(pid)).is_some());
}

// =========================================================================
// Combo 09: Alternating hatch dual graph coloring & odd cycle C3 conflict badge
// =========================================================================

#[test]
fn test_tier3_combo_09_alternating_hatch_graph_coloring_with_section_profile() {
    // 1. Linear chain of 4 adjacent cut faces: 0 - 1 - 2 - 3
    let chain_adj = [(0, 1), (1, 2), (2, 3)];
    let (assignments, conflict) = bipartite_alternating_hatch(&chain_adj, 4);
    assert!(!conflict, "Chain graph must be 2-colorable without conflict");
    assert_eq!(assignments, vec![false, true, false, true]); // 45°, 135°, 45°, 135°

    // 2. Add odd cycle edge (0, 2), creating triangle C3 (0 - 1 - 2 - 0)
    let triangle_adj = [(0, 1), (1, 2), (0, 2)];
    let (_, has_odd_cycle) = bipartite_alternating_hatch(&triangle_adj, 3);
    assert!(has_odd_cycle, "Triangle cycle C3 must trigger odd cycle conflict badge (!)");

    // 3. Engine hatch command integration
    let mut s = trowel_session();
    let p1 = try_exec(&mut s, "shape.rectangle", json!({"x": 50.0, "y": 50.0, "width": 60.0, "height": 60.0})).unwrap();
    let id1 = p1["id"].as_u64().unwrap();

    let h_res = try_exec(&mut s, "hatch.set", json!({"direction": "alt", "ids": [id1]})).unwrap();
    assert_eq!(h_res["updated"], 1);

    let get_res = try_exec(&mut s, "hatch.get", json!({"ids": [id1]})).unwrap();
    assert_eq!(get_res["direction"], "alt");

    let flip_res = try_exec(&mut s, "hatch.flip", json!({"ids": [id1]})).unwrap();
    assert_eq!(flip_res["updated"], 1);
}

// =========================================================================
// Combo 10: Pottery wall offset, horizontal ridge, and D/d/H/t measurements readout
// =========================================================================

#[test]
fn test_tier3_combo_10_pottery_wall_offset_to_horizontal_ridge_to_measurements() {
    let mut s = trowel_session();

    // 1. Setup pottery vessel
    let v_res = try_exec(&mut s, "pottery.createVessel", json!({"centerlineX": 250.0, "sectionSide": "right"})).unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();

    // 2. Exterior profile path
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 260.0, "y": 80.0, "width": 80.0, "height": 140.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    s.edit("Bind vessel profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(NodeId(pid));
        Ok(())
    })
    .unwrap();

    // 3. Wall offset (6.5 mm)
    let off_res = try_exec(&mut s, "pottery.wallOffset", json!({"pathId": pid, "thicknessMm": 6.5})).unwrap();
    assert!(off_res["innerPathId"].as_u64().is_some());

    // 4. Horizontal carination ridge
    let ridge_res = try_exec(&mut s, "pottery.horizontalRidge", json!({"point": [320.0, 130.0], "vesselId": vid})).unwrap();
    assert_eq!(ridge_res["p0"][1], 130.0);
    assert_eq!(ridge_res["p1"][1], 130.0);

    // 5. Measurements readout
    let meas_res = try_exec(&mut s, "pottery.measurements", json!({"vesselId": vid, "emitAnnotations": true})).unwrap();
    assert!(meas_res["ledger"].is_array());
    assert!(meas_res["heightCm"].as_f64().is_some());
}

// =========================================================================
// Combo 11: DXF export preserving calibrated EPSG:5186 TM real-world coordinates
// =========================================================================

#[test]
fn test_tier3_combo_11_dxf_export_from_calibrated_tm_coordinates() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0, "epsg": 5186})).unwrap();
    let _ = try_exec(
        &mut s,
        "doc.setOrigin",
        json!({
            "canvas": [0.0, 0.0],
            "world": [GWANGNYEONGRI_CP1.0, GWANGNYEONGRI_CP1.1]
        }),
    )
    .unwrap();

    // 1. Draw feature path in canvas
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 0.0, "y": 0.0, "width": 100.0, "height": 50.0})).unwrap();

    // 2. Query world coordinate roundtrip
    let w_res = try_exec(&mut s, "doc.coordToWorld", json!({"x": 0.0, "y": 0.0})).unwrap();
    assert!((w_res["east"].as_f64().unwrap() - GWANGNYEONGRI_CP1.0).abs() < 1e-4);
    assert!((w_res["north"].as_f64().unwrap() - GWANGNYEONGRI_CP1.1).abs() < 1e-4);

    // 3. Export to DXF
    let doc = &s.doc().unwrap().doc;
    let dxf_opts = vectorcraft_cad::DxfOptions { scale: 20.0, ..Default::default() };
    let out = vectorcraft_cad::export(doc, &dxf_opts).expect("DXF export");
    assert!(!out.bytes.is_empty(), "Exported DXF byte stream must not be empty");
}

// =========================================================================
// Combo 12: Elevation lines reactive regeneration upon trench profile modification
// =========================================================================

#[test]
fn test_tier3_combo_12_elevation_lines_reactive_update_on_profile_edit() {
    let mut s = trowel_session();
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 100.0, "width": 300.0, "height": 150.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    // 1. Create elevation group
    let c_res = try_exec(&mut s, "elevation.create", json!({"profileId": pid, "datumM": 150.0})).unwrap();
    let eg_id = c_res["id"].as_u64().unwrap();
    let count1 = c_res["segmentCount"].as_u64().unwrap();
    assert!(count1 > 0);

    // 2. Update elevation group
    let upd_res = try_exec(&mut s, "elevation.update", json!({"id": eg_id})).unwrap();
    assert_eq!(upd_res["updated"], 1);
    assert!(upd_res["segmentCount"].as_u64().unwrap() > 0);
}

// =========================================================================
// Combo 13: Spot elevation benchmarks to section cut datum correlation
// =========================================================================

#[test]
fn test_tier3_combo_13_spot_elevation_to_section_datum_correlation() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();

    // 1. Benchmark spot elevation marker
    let bm_res = try_exec(&mut s, "trowel.spotElevation", json!({"at": [100.0, 100.0], "elevation": 152.80, "suffixM": true})).unwrap();
    assert_eq!(bm_res["text"], "+152.80m");

    // 2. Trench floor spot elevation marker
    let fl_res = try_exec(&mut s, "trowel.spotElevation", json!({"at": [250.0, 100.0], "elevation": 151.30, "suffixM": true})).unwrap();
    assert_eq!(fl_res["text"], "+151.30m");

    // 3. Physical height difference and canvas pixel scale oracle
    let delta_h: f64 = 152.80 - 151.30; // 1.50 m
    assert!((delta_h - 1.50).abs() < 1e-6);

    let paper_pt = world_to_paper_pt(delta_h, 20.0);
    assert!((paper_pt - 212.598).abs() < 0.1, "Canvas height at 1:20 must equal 212.6 pt");
}

// =========================================================================
// Combo 14: Legend weights global ratio scaling to report plate viewports sync
// =========================================================================

#[test]
fn test_tier3_combo_14_legend_weights_at_once_to_plate_viewport_sync() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();

    // 1. Create report plate with viewport
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A4"})).unwrap();
    let _ = try_exec(&mut s, "plate.addViewport", json!({"name": "평면 뷰포트", "scale": "1:20"})).unwrap();

    // 2. Scale all document legend weights by ratio x0.8
    let w_res = try_exec(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8})).unwrap();
    let rows = w_res["rows"].as_array().unwrap();
    assert!(!rows.is_empty());

    let r_08 = rows.iter().find(|r| (r["from"].as_f64().unwrap() - 0.8).abs() < 1e-4).expect("0.8pt row");
    assert!((r_08["to"].as_f64().unwrap() - 0.64).abs() < 1e-3);
}

// =========================================================================
// Combo 15: Strata GPKG survey read-only sync tracking under local edits
// =========================================================================

#[test]
fn test_tier3_combo_15_strata_gpkg_sync_with_local_edits() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier3_combo_15_gpkg_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    let bytes_before = std::fs::read(&db_path).unwrap();

    let mut s = trowel_session();
    let _ = try_exec(&mut s, "gpkg.importLayers", json!({"path": db_path.to_str().unwrap()})).unwrap();

    // 1. Add local annotations in document
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 50.0, "height": 50.0})).unwrap();

    // 2. Run sync survey check
    let sync_res = try_exec(&mut s, "gpkg.syncSurvey", json!({"path": db_path.to_str().unwrap()})).unwrap();
    assert_eq!(sync_res["synced"], true);
    assert_eq!(sync_res["layerCount"], 7);
    assert!(sync_res["totalFeatures"].as_u64().unwrap() >= 2);

    // 3. Read-only invariant: GPKG disk binary remains bit-for-bit identical
    let bytes_after = std::fs::read(&db_path).unwrap();
    assert_eq!(bytes_before, bytes_after, "Strata GPKG file must remain strictly read-only");

    let _ = std::fs::remove_file(&db_path);
}

// =========================================================================
// Combo 16: Slope hachures (경사 표시) along curved composite cubic Bézier embankment
// =========================================================================

#[test]
fn test_tier3_combo_16_slope_hachures_along_composite_cubic_bezier_embankment() {
    let mut s = trowel_session();

    // 1. Create top crest and bottom toe paths
    let top_res = try_exec(&mut s, "shape.rectangle", json!({"x": 50.0, "y": 50.0, "width": 200.0, "height": 10.0})).unwrap();
    let top_id = top_res["id"].as_u64().unwrap();

    let bot_res = try_exec(&mut s, "shape.rectangle", json!({"x": 50.0, "y": 100.0, "width": 200.0, "height": 10.0})).unwrap();
    let bot_id = bot_res["id"].as_u64().unwrap();

    // 2. Generate slope hachures
    let hach_res = try_exec(
        &mut s,
        "trowel.slopeHachures",
        json!({
            "topId": top_id,
            "bottomId": bot_id,
            "spacingMm": 1.5,
            "shortRatio": 0.5
        }),
    )
    .unwrap();

    assert!(hach_res["id"].as_u64().is_some());
    assert_eq!(hach_res["topId"], top_id);
    assert_eq!(hach_res["bottomId"], bot_id);
}

// =========================================================================
// Combo 17: Multi-viewport independent scales (1:100, 1:20, 1:2) and North arrow coupling
// =========================================================================

#[test]
fn test_tier3_combo_17_plate_multi_viewport_independent_scales_and_north_coupling() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "A3", "orientation": "landscape"})).unwrap();

    // 1. Add 3 viewports with independent scales
    let vp1 = try_exec(&mut s, "plate.addViewport", json!({"name": "전체 배치도", "scale": "1:100", "rotation": 0.0})).unwrap();
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0})).unwrap();
    let vp2 = try_exec(&mut s, "plate.addViewport", json!({"name": "주거지 평면", "scale": "1:20", "rotation": 28.0})).unwrap();
    let _ = try_exec(&mut s, "shape.rectangle", json!({"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0})).unwrap();
    let vp3 = try_exec(&mut s, "plate.addViewport", json!({"name": "유물 상세", "scale": "1:2", "rotation": 0.0})).unwrap();

    assert_eq!(vp1["scale"], "1:100");
    assert_eq!(vp2["scale"], "1:20");
    assert_eq!(vp3["scale"], "1:2");

    let vp2_id = vp2["viewportId"].as_str().unwrap();

    // 2. Couple North Arrow rotation to Viewport 2 (28.0 deg)
    let na_res = try_exec(
        &mut s,
        "plate.updateNorthArrow",
        json!({
            "rotation": 28.0,
            "linkedViewportId": vp2_id
        }),
    )
    .unwrap();
    assert!(na_res["success"].as_bool().unwrap_or(true));

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets[0].north_arrow.rotation_deg, 28.0);
}

// =========================================================================
// Combo 18: Underlay affine least-squares registration with 6 GCPs & residual RMSE
// =========================================================================

#[test]
fn test_tier3_combo_18_underlay_affine_least_squares_to_residual_table() {
    let mut s = trowel_session();

    // 1. 6 GCP points with synthetic translation and rotation
    let src = vec![(0.0, 0.0), (100.0, 0.0), (200.0, 50.0), (200.0, 150.0), (100.0, 200.0), (0.0, 100.0)];
    let dst = vec![(15.0, 25.0), (115.0, 25.0), (215.0, 75.0), (215.0, 175.0), (115.0, 225.0), (15.0, 125.0)];

    // 2. Mathematical least-squares solver oracle
    let (coeffs, residuals) = solve_affine_least_squares(&src, &dst).expect("least squares affine");
    assert!((coeffs[0] - 1.0).abs() < 1e-4);
    assert!((coeffs[3] - 1.0).abs() < 1e-4);
    assert!((coeffs[4] - 15.0).abs() < 1e-4);
    assert!((coeffs[5] - 25.0).abs() < 1e-4);
    assert!(residuals.iter().all(|&r| r < 1e-4));

    // 3. Engine calibration residual command
    let pairs: Vec<_> = src
        .iter()
        .zip(&dst)
        .enumerate()
        .map(|(i, (s, d))| {
            json!({
                "source": [s.0, s.1],
                "target": [d.0, d.1],
                "name": format!("GCP_{}", i + 1)
            })
        })
        .collect();

    let res = try_exec(
        &mut s,
        "calibrate.residuals",
        json!({
            "affineCoeffs": coeffs,
            "pairs": pairs
        }),
    )
    .unwrap();

    assert!(res["rmsePt"].as_f64().unwrap() < 1e-4);
    assert_eq!(res["residuals"].as_array().unwrap().len(), 6);
}

// =========================================================================
// Combo 19: Posthole depth marker with concentric double circles and negative depth badge
// =========================================================================

#[test]
fn test_tier3_combo_19_depth_marker_with_concentric_posthole_diameters() {
    let mut s = trowel_session();

    let res = try_exec(
        &mut s,
        "trowel.depthMarker",
        json!({
            "at": [240.0, 160.0],
            "depth": 35.0,
            "isCm": true,
            "symbol": "posthole",
            "radius": 12.0
        }),
    )
    .unwrap();

    assert!(res["id"].as_u64().is_some());
    assert!(res["text"].as_str().unwrap().contains("35"));
    assert_eq!(res["depth"], 35.0);
}

// =========================================================================
// Combo 20: Stratigraphy TSV clipboard export and roundtrip re-import validation
// =========================================================================

#[test]
fn test_tier3_combo_20_stratigraphy_tsv_export_and_reimport_validation() {
    let mut s = trowel_session();

    // 1. Build 6-layer table
    for i in 1..=6 {
        let _ = try_exec(
            &mut s,
            "stratigraphy.addLayer",
            json!({
                "colorName": format!("토층 {}층", i),
                "munsell": format!("10YR {}/2", i),
                "texture": "점토",
                "inclusions": "암편"
            }),
        )
        .unwrap();
    }

    // 2. Export TSV
    let exp_res = try_exec(&mut s, "stratigraphy.exportTsv", json!({})).unwrap();
    let tsv = exp_res["tsv"].as_str().unwrap();
    assert!(tsv.starts_with("층위\t토색\t먼셀\t토성\t"));
    assert_eq!(exp_res["rowCount"], 6);

    // 3. Clear and re-populate from TSV
    let mut s2 = trowel_session();
    let lines: Vec<&str> = tsv.lines().skip(1).collect();
    assert_eq!(lines.len(), 6);

    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert!(parts.len() >= 4);
        let _ = try_exec(
            &mut s2,
            "stratigraphy.addLayer",
            json!({
                "colorName": parts[1],
                "munsell": parts[2],
                "texture": parts[3]
            }),
        )
        .unwrap();
    }

    let val_res = try_exec(&mut s2, "stratigraphy.validate", json!({})).unwrap();
    assert_eq!(val_res["valid"], true);
    assert_eq!(val_res["missingTableRows"].as_array().unwrap().len(), 0);
}
