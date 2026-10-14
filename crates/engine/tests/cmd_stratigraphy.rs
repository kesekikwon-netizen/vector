//! Integration tests for Task J010 (Section & Stratigraphy Table).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::BezPath;
use serde_json::json;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Node, NodeId, NodeKind};
use vectorcraft_engine::Session;
use vectorcraft_geom::PathData;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 2000, "height": 1000})).unwrap();
    s
}

#[test]
fn test_stratigraphy_add_and_remove_layer() {
    let mut s = test_session();

    // 1. Add 3 layers
    let res1 = s
        .execute(
            "stratigraphy.addLayer",
            &json!({
                "colorName": "흑갈색",
                "munsell": "10YR 2/2",
                "texture": "사질점토",
                "inclusions": "목탄 소량",
                "notes": "표토층"
            }),
        )
        .unwrap();
    assert_eq!(res1["index"], 1);
    assert_eq!(res1["count"], 1);

    let res2 = s
        .execute(
            "stratigraphy.addLayer",
            &json!({
                "colorName": "암갈색",
                "munsell": "10YR 3/4",
                "texture": "점질토",
                "inclusions": "소토 입자",
                "notes": "주거지 폐기 뒤 퇴적"
            }),
        )
        .unwrap();
    assert_eq!(res2["index"], 2);
    assert_eq!(res2["count"], 2);

    let res3 = s
        .execute(
            "stratigraphy.addLayer",
            &json!({
                "colorName": "황갈색",
                "munsell": "10YR 4/6",
                "texture": "점토",
                "inclusions": "무유물",
                "notes": "생토층"
            }),
        )
        .unwrap();
    assert_eq!(res3["index"], 3);
    assert_eq!(res3["count"], 3);

    // 2. Remove layer 2
    let rem = s.execute("stratigraphy.removeLayer", &json!({ "index": 2 })).unwrap();
    assert_eq!(rem["removedIndex"], 2);
    assert_eq!(rem["count"], 2);

    let doc = &s.doc().unwrap().doc;
    let table = doc.stratigraphy_tables.first().unwrap();
    assert_eq!(table.layers.len(), 2);
    assert_eq!(table.layers[0].index, 1);
    assert_eq!(table.layers[1].index, 3);
}

#[test]
fn test_stratigraphy_circled_number_and_auto_table_sync() {
    let mut s = test_session();

    // Place circled layer number badge at (150, 250)
    let res = s
        .execute(
            "stratigraphy.circledNumber",
            &json!({
                "at": [150.0, 250.0],
                "number": 1,
                "style": "circled",
                "leaderTo": [120.0, 230.0],
                "addToTable": true,
            }),
        )
        .unwrap();

    assert_eq!(res["number"], 1);
    assert_eq!(res["badge"], "①");
    assert_eq!(res["layerCreated"], true);

    let badge_id = NodeId(res["id"].as_u64().unwrap());
    let doc = &s.doc().unwrap().doc;
    let badge_node = doc.node(badge_id).unwrap();
    assert!(badge_node.name.as_ref().unwrap().contains("①"));

    // Check table sync
    let table = doc.stratigraphy_tables.first().unwrap();
    assert_eq!(table.layers.len(), 1);
    assert_eq!(table.layers[0].badge_node_id, Some(badge_id));
}

#[test]
fn test_stratigraphy_generate_table_vector_geometry() {
    let mut s = test_session();

    // Add 2 layers
    s.execute("stratigraphy.addLayer", &json!({ "colorName": "흑갈색", "texture": "사질토" })).unwrap();
    s.execute("stratigraphy.addLayer", &json!({ "colorName": "황갈색", "texture": "점토" })).unwrap();

    // Generate table at (100, 400)
    let res = s
        .execute(
            "stratigraphy.generateTable",
            &json!({
                "at": [100.0, 400.0],
                "width": 450.0,
                "fontSizePt": 8.0,
            }),
        )
        .unwrap();

    let node_id = NodeId(res["nodeId"].as_u64().unwrap());
    let doc = &s.doc().unwrap().doc;
    let table_node = doc.node(node_id).unwrap();
    assert!(matches!(table_node.kind, NodeKind::Group { .. }));

    let table = doc.stratigraphy_tables.first().unwrap();
    let placement = table.placement.as_ref().unwrap();
    assert_eq!(placement.origin, kurbo::Point::new(100.0, 400.0));
    assert_eq!(placement.width_pt, 450.0);
}

#[test]
fn test_stratigraphy_live_paint_fill() {
    let mut s = test_session();

    // Create a closed bounding box (trench face) from 4 lines
    let mut trench = BezPath::new();
    trench.move_to((100.0, 100.0));
    trench.line_to((300.0, 100.0));
    trench.line_to((300.0, 200.0));
    trench.line_to((100.0, 200.0));
    trench.close_path();

    let pd = PathData::from_bezpath(&trench);
    let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 0.5);

    s.edit("Add Trench Line", |doc, _| {
        let id = doc.alloc_id();
        let node = Node::path(id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    })
    .unwrap();

    // Execute livePaintFill inside the box at (150, 150)
    let fill_res = s
        .execute(
            "stratigraphy.livePaintFill",
            &json!({
                "point": [150.0, 150.0],
                "fillStyle": {
                    "pattern": "sburnt"
                }
            }),
        )
        .unwrap();

    let face_id = NodeId(fill_res["faceId"].as_u64().unwrap());
    let doc = &s.doc().unwrap().doc;
    let face_node = doc.node(face_id).unwrap();
    assert!(face_node.name.as_ref().unwrap().contains("토층 칠"));
    assert!(fill_res["area"].as_f64().unwrap() > 0.0);
}

#[test]
fn test_stratigraphy_export_tsv() {
    let mut s = test_session();

    s.execute(
        "stratigraphy.addLayer",
        &json!({
            "colorName": "흑갈색",
            "munsell": "10YR 2/2",
            "texture": "사질점토",
            "inclusions": "목탄 소량",
            "notes": "표토층"
        }),
    )
    .unwrap();

    let tsv_res = s.execute("stratigraphy.exportTsv", &json!({})).unwrap();
    let tsv = tsv_res["tsv"].as_str().unwrap();
    assert!(tsv.contains("층위\t토색\t먼셀\t토성\t포함유물\t비고"));
    assert!(tsv.contains("①\t흑갈색\t10YR 2/2\t사질점토\t목탄 소량\t표토층"));
}

#[test]
fn test_stratigraphy_validation() {
    let mut s = test_session();

    // Empty table validation
    s.execute("stratigraphy.addLayer", &json!({ "colorName": "", "texture": "" })).unwrap();
    let val_res = s.execute("stratigraphy.validate", &json!({})).unwrap();
    assert_eq!(val_res["valid"], false);
    assert!(!val_res["warnings"].as_array().unwrap().is_empty());
}
