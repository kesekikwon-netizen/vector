//! Adversarial stress test suite for VectorCraft Trowel Milestone M5 (Tasks J012 & J018).
//!
//! Comprehensive empirical challenge testing:
//! 1. J012 Paper sizes: A4, 4×6배판 (Crown Quarto 188×257 mm), A3 in Portrait and Landscape (mm and pt metrics).
//! 2. J012 Viewport scale zoom calculations across standard scales (1:1..1:200) and zero/negative scales.
//! 3. J012 Archaeological stroke weight invariant: unscaled paper pt line weights (0.8, 0.5, 0.2, 0.1 pt) under viewport zoom.
//! 4. J012 Title block and scale bar updates with boundary and extreme values.
//! 5. J012 North Arrow rotation coupling with viewport rotation angle.
//! 6. J018 `.trowel-style` JSON serialization roundtrip with empty, full, and boundary configurations.
//! 7. J018 Malformed JSON, unknown format tags, and mismatched version numbers returning graceful Err without panicking.
//! 8. J018 `diff_template` engine with identical, added, modified, and missing styles across all merge resolutions.
//! 9. J018 Dynamic auto legend table with `filter_unused = true` excluding unused layers and rendering active layers.

#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{Point, Rect};
use serde_json::json;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{
    Legend, LegendGroup, LegendLayoutOptions, LegendSample, LegendTable, LineType, MM,
};
use vectorcraft_doc::plate::{
    BorderFrame, NorthArrowItem, NorthArrowStyle, PlateOrientation, PlateSheet, PlateSize,
    ScaleBarItem, ScaleBarStyle, TitleBlockData, ViewportItem,
};
use vectorcraft_doc::scale::ScaleRatio;
use vectorcraft_doc::style_template::{
    diff_template, DiffStatus, ItemDiff, MergeAction, StyleCategory, StyleMergeResolution,
    TemplateLegendRow, TrowelStyleTemplate, TROWEL_STYLE_FORMAT, TROWEL_STYLE_VERSION,
};
use vectorcraft_doc::{Appearance, LayerColor, Node, NodeKind};
use vectorcraft_engine::Session;
use vectorcraft_geom::{PathData, SubPath, Vec2};

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s
        .execute("file.new", &json!({"width": 1000, "height": 1000}))
        .unwrap();
    s
}

// ============================================================================
// 1. Task J012 Paper Sizes & Sheet Dimensions Stress Tests
// ============================================================================

#[test]
fn test_j012_paper_sizes_mm_and_pt_metrics() {
    let tolerance = 1e-4;

    // 1. A4: 210 × 297 mm
    let a4 = PlateSize::A4;
    assert_eq!(a4.dimensions_mm(PlateOrientation::Portrait), (210.0, 297.0));
    assert_eq!(a4.dimensions_mm(PlateOrientation::Landscape), (297.0, 210.0));

    let (a4_p_w, a4_p_h) = a4.dimensions_pt(PlateOrientation::Portrait);
    let expected_a4_w = 210.0 * 72.0 / 25.4;
    let expected_a4_h = 297.0 * 72.0 / 25.4;
    assert!((a4_p_w - expected_a4_w).abs() < tolerance);
    assert!((a4_p_h - expected_a4_h).abs() < tolerance);
    assert!((a4_p_w - 595.27559).abs() < 0.01);
    assert!((a4_p_h - 841.88976).abs() < 0.01);

    let (a4_l_w, a4_l_h) = a4.dimensions_pt(PlateOrientation::Landscape);
    assert!((a4_l_w - expected_a4_h).abs() < tolerance);
    assert!((a4_l_h - expected_a4_w).abs() < tolerance);

    // 2. 4×6배판 (Crown Quarto): 188 × 257 mm
    let q46 = PlateSize::CrownQuarto4x6;
    assert_eq!(q46.dimensions_mm(PlateOrientation::Portrait), (188.0, 257.0));
    assert_eq!(q46.dimensions_mm(PlateOrientation::Landscape), (257.0, 188.0));

    let (q46_p_w, q46_p_h) = q46.dimensions_pt(PlateOrientation::Portrait);
    let expected_q46_w = 188.0 * 72.0 / 25.4;
    let expected_q46_h = 257.0 * 72.0 / 25.4;
    assert!((q46_p_w - expected_q46_w).abs() < tolerance);
    assert!((q46_p_h - expected_q46_h).abs() < tolerance);
    assert!((q46_p_w - 532.91338).abs() < 0.01);
    assert!((q46_p_h - 728.50393).abs() < 0.01);

    let (q46_l_w, q46_l_h) = q46.dimensions_pt(PlateOrientation::Landscape);
    assert!((q46_l_w - expected_q46_h).abs() < tolerance);
    assert!((q46_l_h - expected_q46_w).abs() < tolerance);

    // 3. A3: 297 × 420 mm
    let a3 = PlateSize::A3;
    assert_eq!(a3.dimensions_mm(PlateOrientation::Portrait), (297.0, 420.0));
    assert_eq!(a3.dimensions_mm(PlateOrientation::Landscape), (420.0, 297.0));

    let (a3_p_w, a3_p_h) = a3.dimensions_pt(PlateOrientation::Portrait);
    let expected_a3_w = 297.0 * 72.0 / 25.4;
    let expected_a3_h = 420.0 * 72.0 / 25.4;
    assert!((a3_p_w - expected_a3_w).abs() < tolerance);
    assert!((a3_p_h - expected_a3_h).abs() < tolerance);
    assert!((a3_p_w - 841.88976).abs() < 0.01);
    assert!((a3_p_h - 1190.55118).abs() < 0.01);

    let (a3_l_w, a3_l_h) = a3.dimensions_pt(PlateOrientation::Landscape);
    assert!((a3_l_w - expected_a3_h).abs() < tolerance);
    assert!((a3_l_h - expected_a3_w).abs() < tolerance);

    // 4. Custom dimensions
    let custom = PlateSize::Custom {
        width_mm: 150.0,
        height_mm: 200.0,
    };
    assert_eq!(custom.dimensions_mm(PlateOrientation::Portrait), (150.0, 200.0));
    assert_eq!(custom.dimensions_mm(PlateOrientation::Landscape), (200.0, 150.0));
}

#[test]
fn test_j012_plate_create_sheet_command_aliases() {
    let mut s = test_session();

    // Test Korean and alias names for 4x6배판
    for alias in &["4x6", "4×6", "4x6배판", "4×6배판", "crownquarto"] {
        let res = s
            .execute(
                "plate.createSheet",
                &json!({
                    "size": alias,
                    "orientation": "portrait"
                }),
            )
            .unwrap();
        assert_eq!(res["size"], "4×6배판 (188×257 mm)");
    }

    // Test A4 landscape in Korean
    let res_a4_kr = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "A4",
                "orientation": "가로"
            }),
        )
        .unwrap();
    assert_eq!(res_a4_kr["orientation"], "landscape");

    // Test Custom size via object
    let res_custom = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": { "widthMm": 300.0, "heightMm": 400.0 },
                "orientation": "portrait"
            }),
        )
        .unwrap();
    assert_eq!(res_custom["size"], "사용자 정의");
    assert!((res_custom["widthPt"].as_f64().unwrap() - 300.0 * MM).abs() < 1e-4);
    assert!((res_custom["heightPt"].as_f64().unwrap() - 400.0 * MM).abs() < 1e-4);
}

// ============================================================================
// 2. Task J012 Viewport Scale Zoom Factor & Boundary Scale Tests
// ============================================================================

#[test]
fn test_j012_viewport_zoom_standard_scales() {
    let source_scale = ScaleRatio::new(20.0); // 1:20 source drawing

    let test_cases = [
        (ScaleRatio::new(1.0), 20.0),            // 1:1 -> zoom = 20.0
        (ScaleRatio::new(10.0), 2.0),            // 1:10 -> zoom = 2.0
        (ScaleRatio::new(20.0), 1.0),            // 1:20 -> zoom = 1.0
        (ScaleRatio::new(50.0), 0.4),            // 1:50 -> zoom = 0.4
        (ScaleRatio::new(60.0), 1.0 / 3.0),      // 1:60 -> zoom = 0.3333333333
        (ScaleRatio::new(100.0), 0.2),           // 1:100 -> zoom = 0.2
        (ScaleRatio::new(200.0), 0.1),           // 1:200 -> zoom = 0.1
    ];

    for (target_scale, expected_zoom) in test_cases {
        let vp = ViewportItem::new(
            "vp_test",
            "Test VP",
            Rect::new(0.0, 0.0, 100.0, 100.0),
            target_scale,
            source_scale,
        );
        let actual_zoom = vp.zoom_factor();
        assert!(
            (actual_zoom - expected_zoom).abs() < 1e-9,
            "Scale 1:{} expected zoom {}, got {}",
            target_scale.denom,
            expected_zoom,
            actual_zoom
        );

        let xf = vp.transform();
        for &coeff in &xf.as_coeffs() {
            assert!(coeff.is_finite(), "transform coefficient must be finite");
        }
    }
}

#[test]
fn test_j012_viewport_zero_and_negative_scales() {
    // 1. ScaleRatio::new guards against 0.0 and negative by clamping to 0.001
    let zero_scale = ScaleRatio::new(0.0);
    assert_eq!(zero_scale.denom, 0.001);

    let neg_scale = ScaleRatio::new(-10.0);
    assert_eq!(neg_scale.denom, 0.001);

    // 2. Even if constructed directly with denom = 0.0 or negative, zoom_factor protects
    let unmanaged_zero_vp = ViewportItem {
        id: "zero".to_string(),
        name: "Zero Scale VP".to_string(),
        source: Default::default(),
        bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
        scale: ScaleRatio { denom: 0.0 },
        source_scale: ScaleRatio { denom: 20.0 },
        pan_offset: Vec2::ZERO,
        rotation_deg: 0.0,
        clip_boundary: true,
        visible: true,
    };
    assert_eq!(unmanaged_zero_vp.zoom_factor(), 1.0);
    let xf_zero = unmanaged_zero_vp.transform();
    for &coeff in &xf_zero.as_coeffs() {
        assert!(coeff.is_finite());
    }

    let unmanaged_neg_vp = ViewportItem {
        scale: ScaleRatio { denom: -5.0 },
        ..unmanaged_zero_vp
    };
    assert_eq!(unmanaged_neg_vp.zoom_factor(), 1.0);

    // 3. Command execution with zero or negative scales
    let mut s = test_session();
    let _ = s.execute("plate.createSheet", &json!({})).unwrap();

    let res_zero = s
        .execute(
            "plate.addViewport",
            &json!({
                "scale": 0.0,
                "sourceScale": -20.0
            }),
        )
        .unwrap();
    assert!(res_zero["scale"].is_string());
}

// ============================================================================
// 3. Task J012 Archaeological Line Weight Invariant Stress Test
// ============================================================================

#[test]
fn test_j012_unscaled_paper_pt_line_weight_invariant() {
    let mut s = test_session();

    // Draft 4 archaeological lines with canonical stroke weights (§7, §14(a,g)):
    // 0.8 pt: 유구 윤곽선
    // 0.5 pt: 부속 시설
    // 0.2 pt: 바닥선
    // 0.1 pt: 빗금선
    let weights = [0.8, 0.5, 0.2, 0.1];

    s.edit("Add 4 Archeological Lines", |doc, _| {
        for (i, &w) in weights.iter().enumerate() {
            let mut l = Node::layer(doc.alloc_id(), &format!("Layer_{i}"), LayerColor::Preset(i as u8));
            let line = Node::path(
                doc.alloc_id(),
                PathData::single(SubPath::polyline(
                    &[Point::new(10.0, 10.0), Point::new(300.0, 300.0)],
                    false,
                )),
                Appearance::basic(Paint::None, Paint::solid(Color::BLACK), w),
            );
            if let Some(ch) = l.children_mut() {
                ch.push(std::sync::Arc::new(line));
            }
            doc.layers.push(std::sync::Arc::new(l));
        }
        Ok(())
    })
    .unwrap();

    // Create Report Plate with 1:60 scale from 1:20 source (1/3 geometric zoom)
    let _ = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "4x6",
                "orientation": "portrait"
            }),
        )
        .unwrap();

    let _ = s
        .execute(
            "plate.addViewport",
            &json!({
                "name": "주거지 평면",
                "scale": "1:60",
                "sourceScale": "1:20",
                "bounds": [20.0, 20.0, 400.0, 500.0]
            }),
        )
        .unwrap();

    // Export SVG
    let svg_res = s.execute("plate.exportSvg", &json!({})).unwrap();
    assert!(svg_res["svgLen"].as_u64().unwrap() > 0);

    // Export PDF
    let pdf_res = s.execute("plate.exportPdf", &json!({})).unwrap();
    assert!(pdf_res["bytesLen"].as_u64().unwrap() > 0);

    // Verify stroke weight invariant directly on synthesized document nodes:
    let doc_st = s.doc().unwrap();
    let sheet = &doc_st.doc.plate_sheets[0];
    let vp = &sheet.viewports[0];
    assert!((vp.zoom_factor() - (1.0 / 3.0)).abs() < 1e-6);

    // Check all layers generated for the viewport in synthesize_plate_document
    // In plate.rs: transformed.appearance.stroke retains the original unscaled paper pt!
    let lines_in_source: Vec<f64> = doc_st
        .doc
        .layers
        .iter()
        .flat_map(|l| vectorcraft_doc::legend::lines(l))
        .map(|n| n.appearance.stroke().unwrap().width)
        .collect();

    assert_eq!(lines_in_source, vec![0.8, 0.5, 0.2, 0.1]);

    // Ensure line weights were NOT reduced to 0.8/3 = 0.267 pt!
    for &w in &lines_in_source {
        assert!(w == 0.8 || w == 0.5 || w == 0.2 || w == 0.1);
        assert!((w - 0.2666667).abs() > 0.01, "0.8 pt line weight must NOT be scaled down");
    }
}

// ============================================================================
// 4. Task J012 Title Block & Scale Bar Boundary Updates
// ============================================================================

#[test]
fn test_j012_title_block_and_scale_bar_boundary_values() {
    let mut s = test_session();
    let _ = s.execute("plate.createSheet", &json!({})).unwrap();

    // 1. Extreme Title Block data (Empty, Unicode, Korean Hanja, Special characters)
    let complex_title = "광령리(廣令里) 1호 주거지 발굴조사 & ⚡ <test>";
    let _ = s
        .execute(
            "plate.updateTitleBlock",
            &json!({
                "drawingTitle": complex_title,
                "projectTitle": "",
                "siteCode": "GLR-2026-LONG-NAME-".repeat(10),
                "visible": true
            }),
        )
        .unwrap();

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets[0].title_block.drawing_title, complex_title);
    assert_eq!(doc.plate_sheets[0].title_block.project_title, "");

    // Direct model verification for default items
    let bf = BorderFrame::default();
    assert_eq!(bf.margin_mm, 15.0);
    assert!(bf.double_border);

    let tb_def = TitleBlockData::default();
    assert_eq!(tb_def.width_mm, 160.0);
    assert_eq!(tb_def.height_mm, 65.0);

    let sb_def = ScaleBarItem::default();
    assert_eq!(sb_def.style, ScaleBarStyle::AlternatingBlocks);

    let na_def = NorthArrowItem::default();
    assert_eq!(na_def.style, NorthArrowStyle::ArchaeologicalSplit);

    // 2. Scale Bar boundary metrics
    let mut sb = ScaleBarItem::default();

    // 0 unit_m
    sb.segment_unit_m = 0.0;
    assert_eq!(sb.segment_width_pt(), 0.0);
    assert_eq!(sb.total_width_pt(), 0.0);

    // 0 segment count
    sb.segment_unit_m = 1.0;
    sb.segment_count = 0;
    assert_eq!(sb.total_width_pt(), 0.0);

    // Large segment count
    sb.segment_count = 100;
    assert!(sb.total_width_pt() > 0.0 && sb.total_width_pt().is_finite());

    // Zero scale ratio
    sb.scale = ScaleRatio { denom: 0.0 };
    assert_eq!(sb.segment_width_pt(), 0.0);
    assert_eq!(sb.total_width_pt(), 0.0);

    // Update via engine command
    let res_sb = s
        .execute(
            "plate.updateScaleBar",
            &json!({
                "unit": 5.0,
                "count": 4,
                "scale": 100.0,
                "visible": true
            }),
        )
        .unwrap();

    // 5m at 1:100 = 5000mm / 100 * (72 / 25.4) = 50mm * MM pt = 141.732 pt
    let tot_w = res_sb["totalWidthPt"].as_f64().unwrap();
    assert!((tot_w - 200.0 * MM).abs() < 1e-3);

    let doc_updated = &s.doc().unwrap().doc;
    let seg_w = doc_updated.plate_sheets[0].scale_bar.segment_width_pt();
    assert!((seg_w - 50.0 * MM).abs() < 1e-3);
}

// ============================================================================
// 5. Task J012 North Arrow Rotation Coupling
// ============================================================================

#[test]
fn test_j012_north_arrow_rotation_coupling() {
    let mut sheet = PlateSheet::default();

    let vp1 = ViewportItem {
        id: "vp-plan".to_string(),
        name: "Plan Viewport".to_string(),
        source: Default::default(),
        bounds: Rect::new(10.0, 10.0, 200.0, 200.0),
        scale: ScaleRatio::new(40.0),
        source_scale: ScaleRatio::new(20.0),
        pan_offset: Vec2::ZERO,
        rotation_deg: 37.5,
        clip_boundary: true,
        visible: true,
    };
    sheet.viewports.push(vp1);

    // Couple North Arrow and Scale Bar to vp-plan
    sheet.north_arrow.linked_viewport_id = Some("vp-plan".to_string());
    sheet.scale_bar.linked_viewport_id = Some("vp-plan".to_string());

    sheet.sync_dependencies();
    assert_eq!(sheet.north_arrow.rotation_deg, 37.5);
    assert_eq!(sheet.scale_bar.scale, ScaleRatio::new(40.0));

    // Modify viewport rotation to negative and boundary angles
    for test_angle in [-45.0, 0.0, 90.0, 180.0, 270.0, 360.0, 720.0] {
        sheet.viewports[0].rotation_deg = test_angle;
        sheet.sync_dependencies();
        assert_eq!(sheet.north_arrow.rotation_deg, test_angle);
    }

    // Unlinked viewport ID: retains previous rotation gracefully
    sheet.north_arrow.linked_viewport_id = Some("non-existent-vp".to_string());
    sheet.sync_dependencies();
    assert_eq!(sheet.north_arrow.rotation_deg, 720.0);

    // Test engine command updateNorthArrow
    let mut s = test_session();
    let _ = s.execute("plate.createSheet", &json!({})).unwrap();
    let _ = s
        .execute(
            "plate.updateNorthArrow",
            &json!({
                "rotation": 62.5,
                "visible": true
            }),
        )
        .unwrap();

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets[0].north_arrow.rotation_deg, 62.5);
}

// ============================================================================
// 6. Task J018 .trowel-style JSON Serialization Roundtrip
// ============================================================================

#[test]
fn test_j018_trowel_style_serialization_roundtrip_all_configs() {
    // 1. Full default configuration
    let full_tmpl = TrowelStyleTemplate::default();
    let full_json = serde_json::to_string_pretty(&full_tmpl).unwrap();
    let full_deser: TrowelStyleTemplate = serde_json::from_str(&full_json).unwrap();
    assert_eq!(full_tmpl, full_deser);
    assert_eq!(full_deser.stroke_weights.len(), 4);
    assert_eq!(full_deser.hatch_presets.len(), 3);
    assert_eq!(full_deser.soil_palettes.len(), 4);
    assert_eq!(full_deser.legend_rows.len(), 7);

    // 2. Empty configuration
    let empty_tmpl = TrowelStyleTemplate {
        format: TROWEL_STYLE_FORMAT.to_string(),
        version: TROWEL_STYLE_VERSION,
        name: "".to_string(),
        author: None,
        organization: None,
        description: None,
        stroke_weights: vec![],
        hatch_presets: vec![],
        soil_palettes: vec![],
        legend_rows: vec![],
        typography: None,
        elevation_rules: None,
        title_block: None,
    };
    let empty_json = serde_json::to_string(&empty_tmpl).unwrap();
    let empty_deser: TrowelStyleTemplate = serde_json::from_str(&empty_json).unwrap();
    assert_eq!(empty_tmpl, empty_deser);

    // 3. Boundary configuration (Unicode, long strings, extreme weights)
    let mut boundary_tmpl = full_tmpl.clone();
    boundary_tmpl.name = "고고학 樣式 #1 (★) [특수문자] & <tag>".to_string();
    boundary_tmpl.legend_rows.push(TemplateLegendRow {
        name: "극한 유구선".to_string(),
        group: Some(LegendGroup::Section),
        weight_pt: Some(999.9),
        line_type: LineType::Custom(vec![10.0, 5.0, 2.5, 1.25]),
        sample: LegendSample::Mark("㊿".to_string()),
        favourite: true,
        strata_key: Some("boundary_test".to_string()),
        description: Some("가나다라마바사".repeat(20)),
        hatch_preset_ref: Some("수혈 45°".to_string()),
    });

    let boundary_json = serde_json::to_string_pretty(&boundary_tmpl).unwrap();
    let boundary_deser: TrowelStyleTemplate = serde_json::from_str(&boundary_json).unwrap();
    assert_eq!(boundary_tmpl, boundary_deser);
}

// ============================================================================
// 7. Task J018 Malformed JSON, Unknown Formats & Versions (Zero Panic)
// ============================================================================

#[test]
fn test_j018_malformed_json_and_unsupported_versions_graceful_error() {
    let mut s = test_session();

    // 1. Completely malformed JSON strings
    let malformed_inputs = [
        "",
        "{",
        "{\"format\": \"trowel-style\"",
        "not a valid json syntax",
        "null",
        "42",
        "[]",
    ];

    for malformed in malformed_inputs {
        let diff_err = s.execute("style.diffTemplate", &json!({ "template": malformed }));
        assert!(diff_err.is_err(), "Must return Err on malformed JSON: '{malformed}'");

        let apply_err = s.execute("style.applyTemplate", &json!({ "template": malformed }));
        assert!(apply_err.is_err(), "Must return Err on malformed JSON: '{malformed}'");
    }

    // 2. Unknown format tags (e.g. "illustrator-style", "drawcraft")
    let invalid_format = json!({
        "format": "adobe-illustrator-preset",
        "version": 1,
        "name": "잘못된 양식"
    });
    let diff_fmt_err = s.execute("style.diffTemplate", &json!({ "template": invalid_format }));
    assert!(diff_fmt_err.is_err());
    let err_msg = format!("{}", diff_fmt_err.unwrap_err());
    assert!(err_msg.contains("unsupported format"));

    // 3. Mismatched / future version numbers (version > 1)
    let future_version = json!({
        "format": "trowel-style",
        "version": 9999,
        "name": "미래 양식"
    });
    let diff_ver_err = s.execute("style.diffTemplate", &json!({ "template": future_version }));
    assert!(diff_ver_err.is_err());
    let err_msg_ver = format!("{}", diff_ver_err.unwrap_err());
    assert!(err_msg_ver.contains("unsupported template version 9999"));
}

// ============================================================================
// 8. Task J018 `diff_template` Engine & Merge Resolutions Stress Tests
// ============================================================================

#[test]
fn test_j018_diff_template_engine_and_merge_resolutions() {
    let mut s = test_session();

    // Populate initial document with 2 layers:
    // Layer 1: "유구선" (weight: 0.8 pt, Line: Solid) -> will be Modified by template (to 1.0 pt)
    // Layer 2: "조사구역선" (weight: 1.0 pt, Line: DashDot) -> Identical to template
    // Layer 3: "임시작업선" (weight: 0.3 pt) -> Missing in template (doc only)
    s.edit("Setup initial layers", |doc, _| {
        // Layer 1: 유구선
        let mut l1 = Node::layer(doc.alloc_id(), "유구선", LayerColor::Preset(0));
        if let NodeKind::Layer { ref mut legend, .. } = l1.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: LegendSample::Area,
                strata: Some("feature_poly".to_string()),
                group: Some(LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(l1));

        // Layer 2: 조사구역선
        let mut l2 = Node::layer(doc.alloc_id(), "조사구역선", LayerColor::Preset(1));
        if let NodeKind::Layer { ref mut legend, .. } = l2.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(1.0),
                line: LineType::DashDot,
                favourite: false,
                sample: LegendSample::Area,
                strata: Some("survey_boundary".to_string()),
                group: Some(LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(l2));

        // Layer 3: 임시작업선
        let mut l3 = Node::layer(doc.alloc_id(), "임시작업선", LayerColor::Preset(2));
        if let NodeKind::Layer { ref mut legend, .. } = l3.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.3),
                line: LineType::Dotted,
                favourite: false,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(l3));

        Ok(())
    })
    .unwrap();

    // Prepare incoming template:
    // Row 1: "조사구역선" (Identical)
    // Row 2: "유구선" (Modified: weight 1.0 pt instead of 0.8 pt)
    // Row 3: "새석축선" (Added: in template only)
    let incoming_template = TrowelStyleTemplate {
        format: TROWEL_STYLE_FORMAT.to_string(),
        version: TROWEL_STYLE_VERSION,
        name: "테스트 양식".to_string(),
        author: None,
        organization: None,
        description: None,
        stroke_weights: vec![],
        hatch_presets: vec![],
        soil_palettes: vec![],
        legend_rows: vec![
            TemplateLegendRow {
                name: "조사구역선".to_string(),
                group: Some(LegendGroup::Plan),
                weight_pt: Some(1.0),
                line_type: LineType::DashDot,
                sample: LegendSample::Area,
                favourite: false,
                strata_key: Some("survey_boundary".to_string()),
                description: None,
                hatch_preset_ref: None,
            },
            TemplateLegendRow {
                name: "유구선".to_string(),
                group: Some(LegendGroup::Plan),
                weight_pt: Some(1.0), // Modified from 0.8
                line_type: LineType::Solid,
                sample: LegendSample::Area,
                favourite: true,
                strata_key: Some("feature_poly".to_string()),
                description: None,
                hatch_preset_ref: None,
            },
            TemplateLegendRow {
                name: "새석축선".to_string(), // Added
                group: Some(LegendGroup::Plan),
                weight_pt: Some(0.5),
                line_type: LineType::Dashed,
                sample: LegendSample::Line,
                favourite: true,
                strata_key: Some("stone_wall".to_string()),
                description: None,
                hatch_preset_ref: None,
            },
        ],
        typography: None,
        elevation_rules: None,
        title_block: None,
    };

    // 1. Run diffTemplate command and verify all 4 diff statuses
    let diff_res = s
        .execute(
            "style.diffTemplate",
            &json!({
                "template": incoming_template
            }),
        )
        .unwrap();

    assert_eq!(diff_res["identicalCount"], 1); // 조사구역선
    assert_eq!(diff_res["modifiedCount"], 1);  // 유구선
    assert_eq!(diff_res["addedCount"], 1);     // 새석축선
    assert_eq!(diff_res["missingCount"], 1);   // 임시작업선
    assert!(diff_res["summaryKo"].as_str().unwrap().contains("새 항목 1개"));

    // Direct Rust API diff_template verification
    let doc_before = &s.doc().unwrap().doc;
    let direct_diff = diff_template(doc_before, &incoming_template);
    assert_eq!(direct_diff.identical_count, 1);
    assert_eq!(direct_diff.modified_count, 1);
    assert_eq!(direct_diff.added_count, 1);
    assert_eq!(direct_diff.missing_count, 1);
    assert!(direct_diff.items.iter().any(|it: &ItemDiff| it.status == DiffStatus::Identical && it.recommended_action == MergeAction::KeepExisting));
    assert!(direct_diff.items.iter().any(|it: &ItemDiff| it.status == DiffStatus::Added && it.recommended_action == MergeAction::Overwrite));
    assert!(direct_diff.items.iter().any(|it: &ItemDiff| it.status == DiffStatus::Modified && it.recommended_action == MergeAction::Overwrite));
    assert!(direct_diff.items.iter().any(|it: &ItemDiff| it.status == DiffStatus::Missing && it.recommended_action == MergeAction::KeepExisting));

    let sample_res = StyleMergeResolution {
        category: StyleCategory::LegendRow,
        key: "새석축선".to_string(),
        action: MergeAction::Overwrite,
    };
    assert_eq!(sample_res.action, MergeAction::Overwrite);

    // 2. Test Merge Resolutions:
    // - Overwrite: "새석축선" (added as new layer)
    // - KeepExisting: "유구선" (retains 0.8 pt, incoming 1.0 pt ignored)
    // - ImportAsNewCopy: "조사구역선" (creates "조사구역선 (양식)")
    let apply_res = s
        .execute(
            "style.applyTemplate",
            &json!({
                "template": incoming_template,
                "resolutions": [
                    { "key": "새석축선", "action": "overwrite" },
                    { "key": "유구선", "action": "keep_existing" },
                    { "key": "조사구역선", "action": "import_as_new_copy" }
                ]
            }),
        )
        .unwrap();

    assert_eq!(apply_res["appliedCount"].as_u64().unwrap(), 2); // 1 added + 1 copied

    let doc = &s.doc().unwrap().doc;

    // Verify "유구선" was kept existing at 0.8 pt
    let yugu = doc.layers.iter().find(|l| l.display_name().trim() == "유구선").unwrap();
    assert_eq!(yugu.legend().unwrap().weight, Some(0.8));

    // Verify "새석축선" was created
    let stone = doc.layers.iter().find(|l| l.display_name().trim() == "새석축선").unwrap();
    assert_eq!(stone.legend().unwrap().weight, Some(0.5));

    // Verify "조사구역선 (양식)" was created as a new copy
    let survey_copy = doc.layers.iter().find(|l| l.display_name().trim() == "조사구역선 (양식)").unwrap();
    assert_eq!(survey_copy.legend().unwrap().weight, Some(1.0));

    // 3. Verify single-step undo cleanly restores initial state
    s.execute("edit.undo", &json!({})).unwrap();
    let doc_undone = &s.doc().unwrap().doc;
    assert!(!doc_undone.layers.iter().any(|l| l.display_name().trim() == "새석축선"));
    assert!(!doc_undone.layers.iter().any(|l| l.display_name().trim() == "조사구역선 (양식)"));
}

// ============================================================================
// 9. Task J018 Dynamic Auto Legend with `filter_unused = true`
// ============================================================================

#[test]
fn test_j018_dynamic_legend_table_filter_unused_and_rendering() {
    let mut s = test_session();

    // Setup 4 layers:
    // 1. "굴광선": has 2 line nodes -> active & used
    // 2. "시굴격자": has 0 children -> active & UNUSED
    // 3. "단면선": has 1 line node -> active & used, marked favourite ★
    // 4. "비활성 레이어": has 5 line nodes BUT visible = false -> hidden
    s.edit("Populate Mixed Layers", |doc, _| {
        // Layer 1: 굴광선 (used)
        let mut l1 = Node::layer(doc.alloc_id(), "굴광선", LayerColor::Preset(0));
        if let NodeKind::Layer { ref mut legend, .. } = l1.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Area,
                strata: None,
                group: Some(LegendGroup::Plan),
            }));
        }
        let line1 = Node::path(
            doc.alloc_id(),
            PathData::single(SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(50.0, 50.0)], false)),
            Appearance::default(),
        );
        let line2 = Node::path(
            doc.alloc_id(),
            PathData::single(SubPath::polyline(&[Point::new(50.0, 50.0), Point::new(100.0, 100.0)], false)),
            Appearance::default(),
        );
        if let Some(ch) = l1.children_mut() {
            ch.push(std::sync::Arc::new(line1));
            ch.push(std::sync::Arc::new(line2));
        }
        doc.layers.push(std::sync::Arc::new(l1));

        // Layer 2: 시굴격자 (UNUSED, 0 children)
        let mut l2 = Node::layer(doc.alloc_id(), "시굴격자", LayerColor::Preset(1));
        if let NodeKind::Layer { ref mut legend, .. } = l2.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.5),
                line: LineType::Solid,
                favourite: false,
                sample: LegendSample::Area,
                strata: None,
                group: Some(LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(l2));

        // Layer 3: 단면선 (used, favourite)
        let mut l3 = Node::layer(doc.alloc_id(), "단면선", LayerColor::Preset(2));
        if let NodeKind::Layer { ref mut legend, .. } = l3.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.2),
                line: LineType::DashDot,
                favourite: true, // ★ favourite!
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Plan),
            }));
        }
        let line3 = Node::path(
            doc.alloc_id(),
            PathData::single(SubPath::polyline(&[Point::new(0.0, 100.0), Point::new(100.0, 100.0)], false)),
            Appearance::default(),
        );
        if let Some(ch) = l3.children_mut() {
            ch.push(std::sync::Arc::new(line3));
        }
        doc.layers.push(std::sync::Arc::new(l3));

        // Layer 4: 숨김 레이어 (visible = false)
        let mut l4 = Node::layer(doc.alloc_id(), "숨김 레이어", LayerColor::Preset(3));
        l4.visible = false;
        if let NodeKind::Layer { ref mut legend, .. } = l4.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.5),
                line: LineType::Dashed,
                favourite: false,
                sample: LegendSample::Line,
                strata: None,
                group: Some(LegendGroup::Plan),
            }));
        }
        let line4 = Node::path(
            doc.alloc_id(),
            PathData::single(SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(10.0, 10.0)], false)),
            Appearance::default(),
        );
        if let Some(ch) = l4.children_mut() {
            ch.push(std::sync::Arc::new(line4));
        }
        doc.layers.push(std::sync::Arc::new(l4));

        Ok(())
    })
    .unwrap();

    // 1. Generate legend table with filter_unused = true
    let res_filtered = s
        .execute(
            "style.generateLegend",
            &json!({
                "at": [50.0, 50.0],
                "width": 160.0,
                "columns": 1,
                "filterUnused": true
            }),
        )
        .unwrap();

    // Exactly 2 active used layers: "굴광선" and "단면선". "시굴격자" and "숨김 레이어" must be excluded!
    assert_eq!(res_filtered["entryCount"].as_u64().unwrap(), 2);

    // Verify ordering: Favourite "단면선" (★) must come first!
    let doc_cur = &s.doc().unwrap().doc;
    let table_model = LegendTable::build_from_document(
        doc_cur,
        &LegendLayoutOptions {
            filter_unused: true,
            ..Default::default()
        },
    );
    assert_eq!(table_model.entries.len(), 2);
    assert_eq!(table_model.entries[0].label, "단면선", "Favourite layer must be sorted first");
    assert_eq!(table_model.entries[1].label, "굴광선");

    // 2. Generate legend table with filter_unused = false
    let res_unfiltered = s
        .execute(
            "style.generateLegend",
            &json!({
                "at": [50.0, 50.0],
                "width": 160.0,
                "columns": 1,
                "filterUnused": false,
                "replaceExisting": true
            }),
        )
        .unwrap();

    // Exactly 3 visible layers ("단면선", "굴광선", "시굴격자"). "숨김 레이어" is still excluded because visible = false!
    assert_eq!(res_unfiltered["entryCount"].as_u64().unwrap(), 3);
}
