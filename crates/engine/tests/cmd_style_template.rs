//! Integration tests for Style Template (.trowel-style) Diff & Merge and Dynamic Auto Legend (Task J018).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::style_template::TrowelStyleTemplate;
use vectorcraft_doc::{LayerColor, Legend, LineType, Node, NodeKind};
use vectorcraft_engine::Session;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    s
}

#[test]
fn test_style_template_export_and_diff() {
    let mut s = test_session();

    // Setup an archaeological layer with legend
    s.edit("Add Archaeological Layer", |doc, _| {
        let mut l = Node::layer(doc.alloc_id(), "주거지 외곽선", LayerColor::Preset(0));
        if let NodeKind::Layer { ref mut legend, .. } = l.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.8),
                line: LineType::Solid,
                favourite: true,
                sample: vectorcraft_doc::legend::LegendSample::Line,
                strata: None,
                group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
            }));
        }
        doc.layers.push(std::sync::Arc::new(l));
        Ok(())
    })
    .unwrap();

    // Export template
    let exp_res = s
        .execute(
            "style.exportTemplate",
            &json!({
                "name": "광령리 표준 양식",
                "author": "조사연구원",
                "organization": "문화유산연구원"
            }),
        )
        .unwrap();

    assert_eq!(exp_res["name"], "광령리 표준 양식");
    assert_eq!(exp_res["format"], "trowel-style");
    assert!(exp_res["rowCount"].as_u64().unwrap() >= 1);

    // Diff against default template
    let default_tmpl = TrowelStyleTemplate::default();
    let diff_res = s
        .execute(
            "style.diffTemplate",
            &json!({
                "template": default_tmpl
            }),
        )
        .unwrap();

    assert!(diff_res["items"].as_array().unwrap().len() > 0);
    assert!(diff_res["summaryKo"].as_str().unwrap().contains("개"));
}

#[test]
fn test_style_template_apply_merge_undo() {
    let mut s = test_session();

    let mut tmpl = TrowelStyleTemplate::default();
    tmpl.legend_rows = vec![
        vectorcraft_doc::style_template::TemplateLegendRow {
            name: "석축".to_string(),
            group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
            weight_pt: Some(0.5),
            line_type: LineType::Dashed,
            sample: vectorcraft_doc::legend::LegendSample::Line,
            favourite: true,
            strata_key: None,
            description: None,
            hatch_preset_ref: None,
        },
    ];

    // Apply template
    let apply_res = s
        .execute(
            "style.applyTemplate",
            &json!({
                "template": tmpl
            }),
        )
        .unwrap();

    assert_eq!(apply_res["appliedCount"].as_u64().unwrap(), 1);

    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name().trim() == "석축"));

    // Verify Undo works in one single step
    s.execute("edit.undo", &json!({})).unwrap();
    let doc_undone = &s.doc().unwrap().doc;
    assert!(!doc_undone.layers.iter().any(|l| l.display_name().trim() == "석축"));
}

#[test]
fn test_dynamic_legend_table_generation() {
    let mut s = test_session();

    // 1. Generate legend table with filter_unused = true on empty layers -> 0 entries
    let res = s
        .execute(
            "style.generateLegend",
            &json!({
                "at": [100.0, 100.0],
                "width": 150.0,
                "columns": 1,
                "filterUnused": true
            }),
        )
        .unwrap();

    assert_eq!(res["entryCount"].as_u64().unwrap(), 0);

    // 2. Add layer with drawn objects and legend
    s.edit("Add Layer and Lines", |doc, _| {
        let mut l = Node::layer(doc.alloc_id(), "유구선", LayerColor::Preset(1));
        if let NodeKind::Layer { ref mut legend, .. } = l.kind {
            *legend = Some(Box::new(Legend {
                weight: Some(0.5),
                line: LineType::Solid,
                favourite: true,
                sample: vectorcraft_doc::legend::LegendSample::Line,
                strata: None,
                group: Some(vectorcraft_doc::legend::LegendGroup::Plan),
            }));
        }

        // Add a line element to make layer "used"
        let line = Node::path(
            doc.alloc_id(),
            vectorcraft_geom::PathData::single(vectorcraft_geom::SubPath::polyline(
                &[kurbo::Point::new(0.0, 0.0), kurbo::Point::new(10.0, 10.0)],
                false,
            )),
            vectorcraft_doc::Appearance::default(),
        );

        if let Some(ch) = l.children_mut() {
            ch.push(std::sync::Arc::new(line));
        }
        doc.layers.push(std::sync::Arc::new(l));
        Ok(())
    })
    .unwrap();

    // Generate again with filter_unused = true -> exactly 1 entry
    let res2 = s
        .execute(
            "style.generateLegend",
            &json!({
                "at": [100.0, 100.0],
                "width": 150.0,
                "columns": 1,
                "filterUnused": true,
                "replaceExisting": true
            }),
        )
        .unwrap();

    assert_eq!(res2["entryCount"].as_u64().unwrap(), 1);

    // 3. Test in-place updateLegend command
    let update_res = s.execute("style.updateLegend", &json!({})).unwrap();
    assert_eq!(update_res["updated"].as_bool().unwrap(), true);
    assert_eq!(update_res["entryCount"].as_u64().unwrap(), 1);
}
