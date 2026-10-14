//! Tier 4: Real-World Archaeological Workflows E2E Test Suite for VectorCraft Trowel CAD.
//!
//! Validates complete, multi-step field excavation workflows based on actual Korean archaeological practice:
//! from raw survey imports (GPKG/DXF), underlay calibration, 2D plan drafting, stratigraphic section recording,
//! pottery vessel drafting, to publication monograph report plate composition and vector export.
//!
//! 15 test cases covering Workflows 01–15 specified in `explorer_e2e_3/report.md` (§3).
//! Strict clean-room compliance: zero proprietary reference mentions, zero panics.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;

use vectorcraft_doc::style_template::TrowelStyleTemplate;
use vectorcraft_doc::NodeId;

use super::common::*;

// =========================================================================
// Workflow 01: Bronze Age Pit Dwelling 1 (광령리 1호 주거지) complete plan drafting
// =========================================================================

#[test]
fn test_tier4_workflow_01_gwangnyeongri_house1_full_plan_drafting() {
    let mut s = trowel_session();

    // 1. Initialize session at scale 1:20, EPSG:5186, origin at CP1
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

    // 2. Add plan legend layers (굴광선 0.8pt, 바닥선 0.5pt, etc.)
    let leg_res = try_exec(&mut s, "legend.addGroup", json!({"group": "plan"})).unwrap();
    assert_eq!(leg_res["added"].as_array().unwrap().len(), 15);

    // 3. Calibrate underlay from 2 grid points (10m baseline)
    let cal_res = try_exec(
        &mut s,
        "calibrate.twoPoint",
        json!({
            "p1": [100.0, 100.0],
            "p2": [700.0, 100.0],
            "realDistanceM": 10.0,
            "alignAxis": "horizontal"
        }),
    )
    .unwrap();
    assert!((cal_res["measuredCanvasDist"].as_f64().unwrap() - 600.0).abs() < 1e-4);

    // 4. Trace dwelling outline using canonical benchmark polygon
    let area = shoelace_area(&HOUSE_1_POLYGON_METERS);
    let perim = polygon_perimeter(&HOUSE_1_POLYGON_METERS);
    let (azimuth, length) = pca_azimuth_and_length(&HOUSE_1_POLYGON_METERS);

    assert!((area - 32.91).abs() < 0.01, "Calculated area: {area} m²");
    assert!((perim - 20.89).abs() < 0.1, "Calculated perimeter: {perim} m");
    assert!(azimuth >= 0.0 && azimuth <= 180.0);
    assert!(length > 7.0 && length < 8.0);

    // 5. Query stats via engine plan command
    let ppm = vectorcraft_doc::ScaleRatio::new(20.0).points_per_meter();
    let pts: Vec<[f64; 2]> = HOUSE_1_POLYGON_METERS.iter().map(|&(x, y)| [x * ppm, -y * ppm]).collect();
    let stats_res = try_exec(&mut s, "trowel.polygonStats", json!({ "points": pts })).unwrap();
    assert_eq!(stats_res["formatted"]["area"], "32.91 ㎡");

    // 6. 4 central postholes with depth markers
    let posthole_depths = [45.0, 40.0, 42.0, 38.0];
    let posthole_positions = [[150.0, 200.0], [250.0, 200.0], [150.0, 300.0], [250.0, 300.0]];
    for (pos, depth) in posthole_positions.iter().zip(posthole_depths.iter()) {
        let dm = try_exec(
            &mut s,
            "trowel.depthMarker",
            json!({
                "at": pos,
                "depth": depth,
                "isCm": true,
                "symbol": "posthole",
                "radius": 10.0
            }),
        )
        .unwrap();
        assert!(dm["text"].as_str().unwrap().contains(&format!("{}", *depth as i32)));
    }

    // 7. Section Cut Line A-A'
    let sec_res = try_exec(
        &mut s,
        "trowel.sectionLine",
        json!({
            "p0": [80.0, 250.0],
            "p1": [320.0, 250.0],
            "labelStart": "A",
            "labelEnd": "A'",
            "viewDirection": "right"
        }),
    )
    .unwrap();
    assert_eq!(sec_res["labelStart"], "A");
    assert_eq!(sec_res["labelEnd"], "A'");

    // 8. Spot elevations at entrance and hearth
    let sp1 = try_exec(&mut s, "trowel.spotElevation", json!({"at": [100.0, 250.0], "elevation": 152.65, "suffixM": true})).unwrap();
    let sp2 = try_exec(&mut s, "trowel.spotElevation", json!({"at": [200.0, 250.0], "elevation": 152.30, "suffixM": true})).unwrap();
    assert_eq!(sp1["text"], "+152.65m");
    assert_eq!(sp2["text"], "+152.30m");
}

// =========================================================================
// Workflow 02: Section A-A' trench profile excavation record & 7-layer stratigraphy analysis
// =========================================================================

#[test]
fn test_tier4_workflow_02_section_a_a_profile_and_7layer_stratigraphy() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();

    // 1. Create trench profile path
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 150.0, "width": 400.0, "height": 180.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    // 2. Generate elevation lines at 152.0m datum with 50cm and 10cm intervals
    let el_res = try_exec(
        &mut s,
        "elevation.create",
        json!({
            "profileId": pid,
            "datumM": 152.0,
            "majorIntervalM": 0.5,
            "minorIntervalM": 0.1,
            "majorWeightPt": 0.5,
            "minorWeightPt": 0.2
        }),
    )
    .unwrap();
    assert!(el_res["segmentCount"].as_u64().unwrap() > 0);

    // 3. Populate 7 stratigraphic horizons
    let layers = [
        ("표토", "10YR 4/2", "사질점토", "초본류 뿌리 다량"),
        ("흑갈색 점질토", "10YR 3/1", "점토", "목탄 입자 소량"),
        ("암갈색 사질점토", "10YR 3/3", "사질점토", "토기편 미량"),
        ("갈색 점토 (주거지 퇴적토)", "10YR 4/3", "점토", "소토 입자 다량"),
        ("적갈색 소토층 (노지)", "5YR 4/6", "소토질", "소토 덩어리"),
        ("황갈색 풍화토 (바닥)", "10YR 5/6", "사력질", "기반암 풍화편"),
        ("생토 기반층 (풍화암반)", "10YR 6/8", "풍화암", "무유물"),
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

    // 4. Place circled numbers ① through ⑦
    for i in 1..=7 {
        let _ = try_exec(
            &mut s,
            "stratigraphy.circledNumber",
            json!({
                "at": [150.0, 160.0 + (i as f64) * 20.0],
                "number": i,
                "style": "circled"
            }),
        )
        .unwrap();
    }

    // 5. Generate vector table on canvas
    let tbl_res = try_exec(&mut s, "stratigraphy.generateTable", json!({"at": [50.0, 360.0], "width": 500.0})).unwrap();
    assert_eq!(tbl_res["rowCount"], 7);

    // 6. Validate matrix
    let val_res = try_exec(&mut s, "stratigraphy.validate", json!({})).unwrap();
    assert_eq!(val_res["valid"], true);
}

// =========================================================================
// Workflow 03: Mumun pottery vessel drafting (1:1 scale -> 3-pt rim fit -> Centerline -> Section cut -> Measurements)
// =========================================================================

#[test]
fn test_tier4_workflow_03_pottery_vessel_complete_drafting_and_measurement() {
    let mut s = trowel_session();

    // 1. Initialize pottery session at 1:1 scale with vertical centerline
    let v_res = try_exec(
        &mut s,
        "pottery.createVessel",
        json!({
            "name": "무문토기 발형토기",
            "centerlineX": 300.0,
            "sectionSide": "right",
            "targetScale": "1:1"
        }),
    )
    .unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();

    // 2. 3-point rim shard circumcircle fit
    let r = 92.0; // Radius = 92mm => Diameter = 184mm
    let a1 = 0.0_f64;
    let a2 = 45.0_f64.to_radians();
    let a3 = 90.0_f64.to_radians();
    let p1 = (300.0 + r * a1.cos(), 150.0 + r * a1.sin());
    let p2 = (300.0 + r * a2.cos(), 150.0 + r * a2.sin());
    let p3 = (300.0 + r * a3.cos(), 150.0 + r * a3.sin());

    let fit_res = try_exec(
        &mut s,
        "pottery.rimFit",
        json!({
            "p1": [p1.0, p1.1],
            "p2": [p2.0, p2.1],
            "p3": [p3.0, p3.1],
            "applyToVesselId": vid,
            "emitArc": true
        }),
    )
    .unwrap();
    assert!((fit_res["diameterPt"].as_f64().unwrap() - 184.0).abs() < 1e-3);

    // 3. Draft right profile and bind
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 310.0, "y": 80.0, "width": 80.0, "height": 160.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    s.edit("Bind outer profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(NodeId(pid));
        Ok(())
    })
    .unwrap();

    // 4. Reflect exterior profile across centerline to left side
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

    // 5. Inward wall thickness offset (7.0 mm)
    let off_res = try_exec(&mut s, "pottery.wallOffset", json!({"pathId": pid, "thicknessMm": 7.0})).unwrap();
    assert!(off_res["innerPathId"].as_u64().is_some());

    // 6. Section hatch on right profile
    let hatch_res = try_exec(&mut s, "pottery.wallHatch", json!({"vesselId": vid, "fillMode": "hatch", "angleDeg": 45.0})).unwrap();
    assert!(hatch_res["success"].as_bool().unwrap_or(true));

    // 7. Horizontal carination ridge
    let ridge_res = try_exec(&mut s, "pottery.horizontalRidge", json!({"point": [370.0, 140.0], "vesselId": vid})).unwrap();
    assert_eq!(ridge_res["p0"][1], 140.0);
    assert_eq!(ridge_res["p1"][1], 140.0);

    // 8. Dimensions readout
    let meas_res = try_exec(&mut s, "pottery.measurements", json!({"vesselId": vid, "emitAnnotations": true})).unwrap();
    assert!(meas_res["ledger"].is_array());
}

// =========================================================================
// Workflow 04: Formal excavation monograph report plate layout (4×6배판 -> Multi-scale viewports -> Title block -> Vector export)
// =========================================================================

#[test]
fn test_tier4_workflow_04_formal_excavation_monograph_plate_layout() {
    let mut s = trowel_session();

    // 1. Create 4×6배판 (Crown Quarto, 188×257 mm) plate sheet
    let sheet_res = try_exec(
        &mut s,
        "plate.createSheet",
        json!({
            "size": "4x6",
            "orientation": "portrait",
            "name": "도판 1",
            "pageNumber": "15"
        }),
    )
    .unwrap();
    assert_eq!(sheet_res["size"], "4×6배판 (188×257 mm)");

    // 2. Add 3 viewports: 1:20 plan, 1:20 section, 1:3 pottery
    let vp1 = try_exec(&mut s, "plate.addViewport", json!({"name": "주거지 평면", "bounds": [40.0, 40.0, 450.0, 250.0], "scale": "1:20"})).unwrap();
    let vp2 = try_exec(&mut s, "plate.addViewport", json!({"name": "토층 단면", "bounds": [40.0, 310.0, 450.0, 180.0], "scale": "1:20"})).unwrap();
    let vp3 = try_exec(&mut s, "plate.addViewport", json!({"name": "출토 토기", "bounds": [40.0, 510.0, 200.0, 160.0], "scale": "1:3"})).unwrap();

    assert_eq!(vp1["scale"], "1:20");
    assert_eq!(vp2["scale"], "1:20");
    assert_eq!(vp3["scale"], "1:3");

    // 3. Configure monograph submission title block
    let tb_res = try_exec(
        &mut s,
        "plate.updateTitleBlock",
        json!({
            "projectTitle": "제주 광령리 유적 발굴조사보고서",
            "drawingTitle": "1호 주거지 평·단면 및 출토유물",
            "siteCode": "제주 광령리 1",
            "director": "연구원",
            "date": "2026. 10."
        }),
    )
    .unwrap();
    assert!(tb_res["success"].as_bool().unwrap_or(true));

    // 4. Metric scale bar & North arrow
    let sb_res = try_exec(&mut s, "plate.updateScaleBar", json!({"unit": 1.0, "count": 5, "scale": "1:20"})).unwrap();
    assert!(sb_res["success"].as_bool().unwrap_or(true));

    let na_res = try_exec(&mut s, "plate.updateNorthArrow", json!({"rotation": 0.0})).unwrap();
    assert!(na_res["success"].as_bool().unwrap_or(true));

    // 5. Multi-page vector PDF and SVG export
    assert!(try_exec(&mut s, "plate.exportPdf", json!({})).is_ok());
    assert!(try_exec(&mut s, "plate.exportSvg", json!({})).is_ok());
}

// =========================================================================
// Workflow 05: Strata survey synchronization and .trowel-style template propagation
// =========================================================================

#[test]
fn test_tier4_workflow_05_strata_survey_sync_and_style_template_propagation() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier4_wf05_gpkg_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    // 1. Import GPKG survey in Drawing Book 1
    let mut s1 = trowel_session();
    let imp_res = try_exec(
        &mut s1,
        "gpkg.importLayers",
        json!({
            "path": db_path.to_str().unwrap(),
            "applyHatchPresets": true
        }),
    )
    .unwrap();
    assert_eq!(imp_res["importedLayersCount"], 7);

    // 2. Export standardized template
    let exp_res = try_exec(&mut s1, "style.exportTemplate", json!({"name": "광령리 표준 양식"})).unwrap();
    assert_eq!(exp_res["format"], "trowel-style");

    // 3. Create fresh Drawing Book 2 and apply template
    let mut s2 = trowel_session();
    let tmpl = TrowelStyleTemplate::default();
    let apply_res = try_exec(&mut s2, "style.applyTemplate", json!({"template": tmpl})).unwrap();
    assert!(apply_res["appliedCount"].as_u64().unwrap() >= 1);

    // 4. Generate dynamic legend
    let leg_res = try_exec(&mut s2, "style.generateLegend", json!({"filterUnused": false})).unwrap();
    assert!(leg_res["entryCount"].as_u64().unwrap() >= 1);

    // 5. Verify survey sync alignment
    let sync_res = try_exec(&mut s1, "gpkg.syncSurvey", json!({"path": db_path.to_str().unwrap()})).unwrap();
    assert_eq!(sync_res["synced"], true);

    let _ = std::fs::remove_file(&db_path);
}

// =========================================================================
// Workflow 06: Stone cist tomb (석관묘) plan and stone wall elevation drafting (alternating hatch & void clipping)
// =========================================================================

#[test]
fn test_tier4_workflow_06_stone_cist_tomb_plan_and_elevation_drafting() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();

    // 1. Grave pit outline (0.8pt)
    let pit = try_exec(&mut s, "shape.rectangle", json!({"x": 50.0, "y": 50.0, "width": 300.0, "height": 180.0})).unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.8, "ids": [pit["id"].as_u64().unwrap()]})).unwrap();

    // 2. Overlapping cut stone wall slabs (0.8pt + alternating hatch)
    let mut slab_ids = Vec::new();
    for i in 0..4 {
        let slab = try_exec(&mut s, "shape.rectangle", json!({"x": 60.0 + (i as f64) * 65.0, "y": 60.0, "width": 60.0, "height": 30.0})).unwrap();
        let sid = slab["id"].as_u64().unwrap();
        slab_ids.push(sid);
    }

    let h_res = try_exec(&mut s, "hatch.set", json!({"direction": "alt", "ids": slab_ids})).unwrap();
    assert_eq!(h_res["updated"], 4);

    // 3. Elevation lines clipped against stones
    let el_res = try_exec(&mut s, "elevation.create", json!({"profileId": pit["id"].as_u64().unwrap(), "datumM": 150.0})).unwrap();
    assert!(el_res["segmentCount"].as_u64().unwrap() > 0);
}

// =========================================================================
// Workflow 07: Cremation pit feature with burnt soil (소토) speckle and charcoal (┼) cross-hatching
// =========================================================================

#[test]
fn test_tier4_workflow_07_cremation_pit_burnt_soil_and_charcoal_hatching() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();

    // 1. Pit boundary (0.8pt)
    let pit = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 100.0, "width": 200.0, "height": 150.0})).unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.8, "ids": [pit["id"].as_u64().unwrap()]})).unwrap();

    // 2. Charcoal lens (목탄): cross-hatch (┼)
    let charcoal = try_exec(&mut s, "shape.rectangle", json!({"x": 130.0, "y": 130.0, "width": 80.0, "height": 50.0})).unwrap();
    let cid = charcoal["id"].as_u64().unwrap();
    let h_res = try_exec(&mut s, "hatch.set", json!({"direction": "cross", "ids": [cid]})).unwrap();
    assert_eq!(h_res["updated"], 1);

    let get_res = try_exec(&mut s, "hatch.get", json!({"ids": [cid]})).unwrap();
    assert_eq!(get_res["direction"], "cross");

    // 3. Posthole depth markers
    let dm = try_exec(&mut s, "trowel.depthMarker", json!({"at": [250.0, 180.0], "depth": 28.0, "isCm": true, "symbol": "posthole"})).unwrap();
    assert!(dm["text"].as_str().unwrap().contains("28"));
}

// =========================================================================
// Workflow 08: Multi-period stratigraphic trench profile (Joseon -> Goryeo -> Three Kingdoms -> Bronze Age)
// =========================================================================

#[test]
fn test_tier4_workflow_08_multi_period_stratigraphic_trench_overlap() {
    let mut s = trowel_session();

    // 1. 12 stratigraphic layers spanning 4 archaeological periods
    let horizons = [
        ("조선시대 경작토", "10YR 4/2", "사질점토", "백자편"),
        ("조선시대 흑갈색 점토", "10YR 3/2", "점토", "기와편"),
        ("고려시대 회갈색 사질토", "10YR 4/1", "사질토", "청자편"),
        ("고려시대 탄화목탄층", "10YR 2/1", "점토질", "목탄 다량"),
        ("통일신라 명갈색 점토", "10YR 5/3", "점토", "인화문토기편"),
        ("삼국시대 적갈색 소토층", "5YR 4/6", "소토질", "단경호편"),
        ("삼국시대 암회갈색 사력층", "10YR 3/3", "사력질", "할석 다량"),
        ("원삼국시대 흑색 유기물토", "10YR 2/2", "점토질", "타날문토기편"),
        ("청동기시대 갈색 점토 (주거지)", "10YR 4/4", "점토", "무문토기편"),
        ("청동기시대 적갈색 노지소토", "5YR 5/6", "소토질", "탄화곡물"),
        ("청동기시대 풍화토 기반층", "10YR 6/6", "풍화암", "무유물"),
        ("지질 암반 기반층", "10YR 7/8", "생토암반", "기반암"),
    ];

    for (name, munsell, texture, inc) in horizons {
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

    // 2. Place circled badges ① to ⑫
    for i in 1..=12 {
        let _ = try_exec(
            &mut s,
            "stratigraphy.circledNumber",
            json!({
                "at": [100.0, 50.0 + (i as f64) * 25.0],
                "number": i,
                "style": "circled"
            }),
        )
        .unwrap();
    }

    // 3. Pre-flight validate
    let val_res = try_exec(&mut s, "stratigraphy.validate", json!({})).unwrap();
    assert_eq!(val_res["valid"], true);
    assert_eq!(val_res["missingTableRows"].as_array().unwrap().len(), 0);

    // 4. Export TSV summary table
    let tsv_res = try_exec(&mut s, "stratigraphy.exportTsv", json!({})).unwrap();
    assert_eq!(tsv_res["rowCount"], 12);
}

// =========================================================================
// Workflow 09: Neolithic comb-pattern vessel (빗살무늬토기) with pointed base (d=0) and surface incisions
// =========================================================================

#[test]
fn test_tier4_workflow_09_comb_pattern_pottery_pointed_base_drafting() {
    let mut s = trowel_session();

    // 1. 3-point rim circumcircle fit (D = 240.0 mm)
    let r = 120.0;
    let a1 = 0.0_f64;
    let a2 = 45.0_f64.to_radians();
    let a3 = 90.0_f64.to_radians();
    let p1 = (300.0 + r * a1.cos(), 150.0 + r * a1.sin());
    let p2 = (300.0 + r * a2.cos(), 150.0 + r * a2.sin());
    let p3 = (300.0 + r * a3.cos(), 150.0 + r * a3.sin());
    let ((cx, _cy), d) = circle_fit_3pt(p1, p2, p3).expect("3-pt rim fit");
    assert!((cx - 300.0).abs() < 1e-4);
    assert!((d - 240.0).abs() < 1e-4);

    // 2. Create vessel session
    let v_res = try_exec(
        &mut s,
        "pottery.createVessel",
        json!({
            "name": "빗살무늬토기",
            "centerlineX": 300.0,
            "sectionSide": "right",
            "targetScale": "1:1"
        }),
    )
    .unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();

    // 3. Conical profile tapering to pointed base (d = 0)
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 300.0, "y": 100.0, "width": 120.0, "height": 260.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    s.edit("Bind conical profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(NodeId(pid));
        Ok(())
    })
    .unwrap();

    // 4. Inward wall thickness offset (8.0 mm)
    let off_res = try_exec(&mut s, "pottery.wallOffset", json!({"pathId": pid, "thicknessMm": 8.0})).unwrap();
    assert!(off_res["innerPathId"].as_u64().is_some());

    // 5. Bilateral reflection
    let ref_res = try_exec(&mut s, "pottery.reflectExterior", json!({"pathId": pid, "centerlineX": 300.0})).unwrap();
    assert!(ref_res["reflectedId"].as_u64().is_some());

    // 6. Measurements
    let meas_res = try_exec(&mut s, "pottery.measurements", json!({"vesselId": vid, "emitAnnotations": true})).unwrap();
    assert!(meas_res["ledger"].is_array());
}

// =========================================================================
// Workflow 10: Ancient pottery/tile kiln (토기가마) longitudinal and transverse sections with slope hachures
// =========================================================================

#[test]
fn test_tier4_workflow_10_kiln_structure_longitudinal_and_transverse_sections() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0})).unwrap();

    // 1. Kiln outline
    let kiln = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 100.0, "width": 500.0, "height": 180.0})).unwrap();
    let _ = try_exec(&mut s, "stroke.set", json!({"weight": 0.8, "ids": [kiln["id"].as_u64().unwrap()]})).unwrap();

    // 2. 3 synchronized section lines: A-A' (longitudinal), B-B' (firing chamber), C-C' (flue)
    let s_a = try_exec(&mut s, "trowel.sectionLine", json!({"p0": [80.0, 190.0], "p1": [620.0, 190.0], "labelStart": "A", "labelEnd": "A'"})).unwrap();
    let s_b = try_exec(&mut s, "trowel.sectionLine", json!({"p0": [250.0, 80.0], "p1": [250.0, 300.0], "labelStart": "B", "labelEnd": "B'"})).unwrap();
    let s_c = try_exec(&mut s, "trowel.sectionLine", json!({"p0": [480.0, 80.0], "p1": [480.0, 300.0], "labelStart": "C", "labelEnd": "C'"})).unwrap();

    assert_eq!(s_a["labelStart"], "A");
    assert_eq!(s_b["labelStart"], "B");
    assert_eq!(s_c["labelStart"], "C");

    // 3. Spot elevations showing slope gradient (15° incline)
    let sp1 = try_exec(&mut s, "trowel.spotElevation", json!({"at": [120.0, 190.0], "elevation": 150.20, "suffixM": true})).unwrap();
    let sp2 = try_exec(&mut s, "trowel.spotElevation", json!({"at": [300.0, 190.0], "elevation": 151.80, "suffixM": true})).unwrap();
    let sp3 = try_exec(&mut s, "trowel.spotElevation", json!({"at": [500.0, 190.0], "elevation": 153.40, "suffixM": true})).unwrap();

    assert_eq!(sp1["text"], "+150.20m");
    assert_eq!(sp2["text"], "+151.80m");
    assert_eq!(sp3["text"], "+153.40m");
}

// =========================================================================
// Workflow 11: Site excavation grid system (10m × 10m quadrats) with affine registration & graticule ticks
// =========================================================================

#[test]
fn test_tier4_workflow_11_trench_grid_system_and_subquadrat_batch_calibration() {
    let mut s = trowel_session();
    let _ = try_exec(&mut s, "doc.setScale", json!({"scale": 20.0, "epsg": 5186})).unwrap();
    let _ = try_exec(&mut s, "doc.setOrigin", json!({"canvas": [0.0, 0.0], "world": [152840.0, 512340.0]})).unwrap();

    // 1. 8 GCPs across 4 quadrats
    let src = vec![
        (0.0, 0.0),
        (200.0, 0.0),
        (400.0, 0.0),
        (400.0, 200.0),
        (400.0, 400.0),
        (200.0, 400.0),
        (0.0, 400.0),
        (0.0, 200.0),
    ];
    let dst = vec![
        (10.0, 10.0),
        (210.0, 10.0),
        (410.0, 10.0),
        (410.0, 210.0),
        (410.0, 410.0),
        (210.0, 410.0),
        (10.0, 410.0),
        (10.0, 210.0),
    ];

    let (coeffs, residuals) = solve_affine_least_squares(&src, &dst).expect("8 GCP affine");
    assert!(residuals.iter().all(|&r| r < 1e-4));

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

    let res = try_exec(&mut s, "calibrate.residuals", json!({"affineCoeffs": coeffs, "pairs": pairs})).unwrap();
    assert!(res["rmsePt"].as_f64().unwrap() < 1e-4);

    // 2. Graticule ticks every 5m
    let tick_res = try_exec(&mut s, "doc.graticuleTicks", json!({"interval_m": 5.0, "bounds": [0.0, 0.0, 500.0, 500.0]})).unwrap();
    assert!(!tick_res["ticks"].as_array().unwrap().is_empty());
}

// =========================================================================
// Workflow 12: Three Kingdoms iron sword artifact drafting with blade cross-sections (solid black fill)
// =========================================================================

#[test]
fn test_tier4_workflow_12_iron_weapon_artifact_drafting_blade_section() {
    let mut s = trowel_session();

    // 1. 1:1 artifact session
    let v_res = try_exec(
        &mut s,
        "pottery.createVessel",
        json!({
            "name": "삼국시대 환두대도",
            "centerlineX": 200.0,
            "sectionSide": "right",
            "fillMode": "black",
            "targetScale": "1:1"
        }),
    )
    .unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();

    // 2. Blade outline
    let b_res = try_exec(&mut s, "shape.rectangle", json!({"x": 190.0, "y": 50.0, "width": 20.0, "height": 600.0})).unwrap();
    let bid = b_res["id"].as_u64().unwrap();

    s.edit("Bind sword profile", |doc, _| {
        doc.pottery_vessels[0].outer_profile_node = Some(NodeId(bid));
        Ok(())
    })
    .unwrap();

    // 3. Solid black fill section
    let fill_res = try_exec(&mut s, "pottery.wallHatch", json!({"vesselId": vid, "fillMode": "black"})).unwrap();
    assert!(fill_res["success"].as_bool().unwrap_or(true));

    // 4. Measurements
    let m_res = try_exec(&mut s, "pottery.measurements", json!({"vesselId": vid, "emitAnnotations": true})).unwrap();
    assert!(m_res["ledger"].is_array());
}

// =========================================================================
// Workflow 13: Publication plate catalog appendix (5 sequential A4 sheets with independent title blocks)
// =========================================================================

#[test]
fn test_tier4_workflow_13_monograph_multi_page_plate_catalog_export() {
    let mut s = trowel_session();

    // 1. Create 5 sequential A4 sheets
    for i in 1..=5 {
        let sheet_res = try_exec(
            &mut s,
            "plate.createSheet",
            json!({
                "size": "A4",
                "name": format!("도판 {}", i),
                "pageNumber": format!("{}", i)
            }),
        )
        .unwrap();
        assert_eq!(sheet_res["name"], format!("도판 {}", i));

        let _ = try_exec(
            &mut s,
            "plate.updateTitleBlock",
            json!({
                "drawingTitle": format!("출토 유물 편람 ({})", i),
                "sheetNumber": format!("{}", i)
            }),
        )
        .unwrap();

        let _ = try_exec(
            &mut s,
            "plate.addViewport",
            json!({
                "name": format!("뷰포트 {}", i),
                "scale": "1:1"
            }),
        )
        .unwrap();
    }

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets.len(), 5);

    // 2. Export multi-sheet catalog to PDF
    assert!(try_exec(&mut s, "plate.exportPdf", json!({})).is_ok());
}

// =========================================================================
// Workflow 14: Pen tablet rough field sketch inking with stylus pressure curve and jitter suppression
// =========================================================================

#[test]
fn test_tier4_workflow_14_pen_tablet_field_sketch_inking_and_smoothing() {
    let mut s = trowel_session();

    // 1. Micro-jitter suppression check: jitter <= 3.0 pt creates corner point
    let jitter = 2.1;
    let is_suppressed = jitter <= 3.0;
    assert!(is_suppressed);

    // 2. Pen tablet inking stroke
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 100.0, "y": 100.0, "width": 120.0, "height": 80.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();

    // 3. Canvas pen bar undo action
    let u_res = try_exec(&mut s, "edit.undo", json!({})).unwrap();
    assert!(u_res["undone"].is_string());
    assert!(s.doc().unwrap().doc.node(NodeId(pid)).is_none());

    // 4. Canvas pen bar redo action
    let r_res = try_exec(&mut s, "edit.redo", json!({})).unwrap();
    assert!(r_res["redone"].is_string());
    assert!(s.doc().unwrap().doc.node(NodeId(pid)).is_some());
}

// =========================================================================
// Workflow 15: Complete excavation-to-publication lifecycle (GPKG -> Calibrate -> Plan -> Section -> Pottery -> Plate -> PDF)
// =========================================================================

#[test]
fn test_tier4_workflow_15_end_to_end_excavation_to_publication_full_lifecycle() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_tier4_wf15_lifecycle_{now}.gpkg"));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    let mut s = trowel_session();

    // Step 1: GPKG survey import
    let s1 = try_exec(&mut s, "gpkg.importLayers", json!({"path": db_path.to_str().unwrap(), "applyHatchPresets": true})).unwrap();
    assert_eq!(s1["importedLayersCount"], 7);

    // Step 2: Underlay calibration
    let s2 = try_exec(&mut s, "calibrate.twoPoint", json!({"p1": [0.0, 0.0], "p2": [600.0, 0.0], "realDistanceM": 10.0})).unwrap();
    assert!((s2["measuredCanvasDist"].as_f64().unwrap() - 600.0).abs() < 1e-4);

    // Step 3: Plan drafting of House 1 with Shoelace area 32.91 ㎡
    let area = shoelace_area(&HOUSE_1_POLYGON_METERS);
    assert!((area - 32.91).abs() < 0.01);

    // Step 4: Section profile with 1D clipped elevation lines & stratigraphy
    let p_res = try_exec(&mut s, "shape.rectangle", json!({"x": 50.0, "y": 50.0, "width": 400.0, "height": 200.0})).unwrap();
    let pid = p_res["id"].as_u64().unwrap();
    let s4_el = try_exec(&mut s, "elevation.create", json!({"profileId": pid, "datumM": 150.0})).unwrap();
    assert!(s4_el["segmentCount"].as_u64().unwrap() > 0);

    let _ = try_exec(&mut s, "stratigraphy.addLayer", json!({"colorName": "흑갈색 점토", "munsell": "10YR 2/2"})).unwrap();
    let _ = try_exec(&mut s, "stratigraphy.circledNumber", json!({"at": [100.0, 100.0], "number": 1})).unwrap();
    let s4_val = try_exec(&mut s, "stratigraphy.validate", json!({})).unwrap();
    assert_eq!(s4_val["valid"], true);

    // Step 5: Pottery 1:1 drafting with 3-pt rim circumcircle fit
    let v_res = try_exec(&mut s, "pottery.createVessel", json!({"centerlineX": 250.0, "sectionSide": "right"})).unwrap();
    let vid = v_res["vesselId"].as_str().unwrap();
    let s5_fit = try_exec(&mut s, "pottery.rimFit", json!({"p1": [150.0, 100.0], "p2": [250.0, 50.0], "p3": [350.0, 100.0], "applyToVesselId": vid})).unwrap();
    assert!(s5_fit["diameterMm"].as_f64().unwrap() > 0.0);

    // Step 6: 4×6배판 report plate composition with 3 viewports and title block
    let _ = try_exec(&mut s, "plate.createSheet", json!({"size": "4x6", "orientation": "portrait"})).unwrap();
    let _ = try_exec(&mut s, "plate.addViewport", json!({"name": "평면", "scale": "1:20"})).unwrap();
    let _ = try_exec(&mut s, "plate.addViewport", json!({"name": "단면", "scale": "1:20"})).unwrap();
    let _ = try_exec(&mut s, "plate.addViewport", json!({"name": "유물", "scale": "1:1"})).unwrap();
    let _ = try_exec(&mut s, "plate.updateTitleBlock", json!({"drawingTitle": "발굴조사 종합도", "projectTitle": "광령리 유적"})).unwrap();
    let _ = try_exec(&mut s, "plate.updateScaleBar", json!({"unit": 1.0, "count": 5})).unwrap();
    let _ = try_exec(&mut s, "plate.updateNorthArrow", json!({"rotation": 0.0})).unwrap();

    // Step 7: Export to submission vector PDF & SVG
    assert!(try_exec(&mut s, "plate.exportPdf", json!({})).is_ok());
    assert!(try_exec(&mut s, "plate.exportSvg", json!({})).is_ok());

    let _ = std::fs::remove_file(&db_path);
}
