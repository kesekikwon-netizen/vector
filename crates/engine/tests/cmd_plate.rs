//! Integration tests for Report Plate Layout, Viewports, Title Block, and Vector Export (Task J012).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::plate::{PlateOrientation, PlateSize};
use vectorcraft_doc::scale::ScaleRatio;
use vectorcraft_engine::Session;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    s
}

#[test]
fn test_create_plate_sheet_sizes() {
    let mut s = test_session();

    // 1. Create 4×6배판 (Crown Quarto 188×257mm) Portrait
    let res = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "4x6",
                "orientation": "portrait",
                "name": "도판 1",
                "caption": "1호 주거지 평·단면도",
                "pageNumber": "57"
            }),
        )
        .unwrap();

    assert_eq!(res["size"], "4×6배판 (188×257 mm)");
    assert_eq!(res["orientation"], "portrait");
    let (w46, h46) = PlateSize::CrownQuarto4x6.dimensions_pt(PlateOrientation::Portrait);
    assert!((res["widthPt"].as_f64().unwrap() - w46).abs() < 1e-3);
    assert!((res["heightPt"].as_f64().unwrap() - h46).abs() < 1e-3);

    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.plate_sheets.len(), 1);
    assert_eq!(doc.plate_sheets[0].name, "도판 1");
    assert_eq!(doc.plate_sheets[0].caption, "1호 주거지 평·단면도");
    assert_eq!(doc.plate_sheets[0].page_number, "57");

    // 2. Create A4 Landscape
    let res_a4 = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "A4",
                "orientation": "landscape",
                "name": "도판 2",
            }),
        )
        .unwrap();

    assert_eq!(res_a4["size"], "A4 (210×297 mm)");
    assert_eq!(res_a4["orientation"], "landscape");
    let (w_a4, h_a4) = PlateSize::A4.dimensions_pt(PlateOrientation::Landscape);
    assert!((res_a4["widthPt"].as_f64().unwrap() - w_a4).abs() < 1e-3);
    assert!((res_a4["heightPt"].as_f64().unwrap() - h_a4).abs() < 1e-3);
}

#[test]
fn test_add_viewport_independent_scale() {
    let mut s = test_session();

    let _ = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "4x6",
                "orientation": "portrait",
            }),
        )
        .unwrap();

    // Add drawing viewport (source scale 1:20 -> target plate scale 1:60)
    let res = s
        .execute(
            "plate.addViewport",
            &json!({
                "name": "주거지 평면 뷰포트",
                "bounds": [40.0, 50.0, 300.0, 350.0],
                "scale": "1:60",
                "sourceScale": "1:20",
                "rotation": 0.0,
            }),
        )
        .unwrap();

    assert_eq!(res["scale"], "1:60");
    assert_eq!(res["sourceScale"], "1:20");

    let doc = &s.doc().unwrap().doc;
    let sh = &doc.plate_sheets[0];
    assert_eq!(sh.viewports.len(), 1);
    let vp = &sh.viewports[0];
    assert_eq!(vp.name, "주거지 평면 뷰포트");
    assert_eq!(vp.scale, ScaleRatio::new(60.0));
    assert_eq!(vp.source_scale, ScaleRatio::new(20.0));

    // Verify transformation ratio is 20 / 60 = 1/3
    let ratio = vp.zoom_factor();
    assert!((ratio - (1.0 / 3.0)).abs() < 1e-6);
}

#[test]
fn test_title_block_and_scale_bar_update() {
    let mut s = test_session();

    let _ = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "4x6",
                "orientation": "portrait",
            }),
        )
        .unwrap();

    // Update Title Block
    let _ = s
        .execute(
            "plate.updateTitleBlock",
            &json!({
                "drawingTitle": "1호 주거지 평·단면도",
                "projectTitle": "제주 애월 광령리 유적 발굴조사",
                "siteCode": "GLR-2026",
                "excavationTeam": "제주문화유산연구원",
                "director": "홍길동",
                "drawnBy": "김철수",
                "date": "2026-10-15",
                "scaleText": "S=1:60, 1:20",
                "crsDatum": "EPSG:5186 (Korea 2000)",
                "sheetNumber": "도판 12",
                "visible": true
            }),
        )
        .unwrap();

    // Update Scale Bar
    let sb_res = s
        .execute(
            "plate.updateScaleBar",
            &json!({
                "unit": 2.0,
                "count": 5,
                "scale": 60.0,
                "visible": true
            }),
        )
        .unwrap();

    assert!(sb_res["totalWidthPt"].as_f64().unwrap() > 0.0);

    // Update North Arrow
    let _ = s
        .execute(
            "plate.updateNorthArrow",
            &json!({
                "rotation": 30.0,
                "visible": true
            }),
        )
        .unwrap();

    let doc = &s.doc().unwrap().doc;
    let sh = &doc.plate_sheets[0];
    assert_eq!(sh.title_block.drawing_title, "1호 주거지 평·단면도");
    assert_eq!(sh.title_block.site_code, "GLR-2026");
    assert_eq!(sh.scale_bar.segment_unit_m, 2.0);
    assert_eq!(sh.scale_bar.segment_count, 5);
    assert_eq!(sh.north_arrow.rotation_deg, 30.0);
}

#[test]
fn test_plate_export_pdf_and_svg() {
    let mut s = test_session();

    let _ = s
        .execute(
            "plate.createSheet",
            &json!({
                "size": "4x6",
                "orientation": "portrait",
                "name": "도판 1",
            }),
        )
        .unwrap();

    let _ = s
        .execute(
            "plate.addViewport",
            &json!({
                "name": "도면 평면",
                "scale": 60.0,
                "sourceScale": 20.0,
            }),
        )
        .unwrap();

    // Export PDF
    let pdf_res = s.execute("plate.exportPdf", &json!({})).unwrap();
    assert!(pdf_res["bytesLen"].as_u64().unwrap() > 0);

    // Export SVG
    let svg_res = s.execute("plate.exportSvg", &json!({})).unwrap();
    assert!(svg_res["svgLen"].as_u64().unwrap() > 0);
}
