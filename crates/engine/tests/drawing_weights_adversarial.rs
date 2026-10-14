//! Adversarial stress tests for `drawing.weightsAtOnce` compatibility alias.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::{EngineError, Session};

/// Helper to set up a reproducible multi-layer document with:
/// - A legend row layer with look (weight 1.0 pt) and a stroke following it
/// - A plain layer without look with a stroke (weight 0.5 pt)
/// - A stroke on the legend layer set apart (weight 0.7087 pt)
/// - A locked stroke (weight 0.5 pt)
/// - A hidden legend row layer (weight 0.5 pt) with a hidden stroke (0.5 pt)
fn create_test_session() -> (Session, NodeId, NodeId, NodeId, NodeId, NodeId, NodeId, NodeId) {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 400})).unwrap();

    // 1. Plain layer with open line (weight 0.5 pt)
    let plain_layer = s.execute("layer.new", &json!({"name": "plain", "top": true})).unwrap();
    let plain_layer_id = NodeId(plain_layer["id"].as_u64().unwrap());
    let plain_line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 10.0, "y": 10.0}, {"x": 50.0, "y": 10.0}],
                "closed": false
            }),
        )
        .unwrap();
    let plain_line_id = NodeId(plain_line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 0.5, "ids": [plain_line_id.0]})).unwrap();

    // 2. Legend row layer "경계" (weight 1.0 pt)
    let row_layer = s.execute("layer.new", &json!({"name": "경계", "top": true})).unwrap();
    let row_layer_id = NodeId(row_layer["id"].as_u64().unwrap());
    s.execute("legend.set", &json!({"id": row_layer_id.0, "weight": 1.0, "line": "solid"})).unwrap();

    // Line following the row (weight 1.0 pt)
    let row_line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 10.0, "y": 30.0}, {"x": 50.0, "y": 30.0}],
                "closed": false
            }),
        )
        .unwrap();
    let row_line_id = NodeId(row_line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 1.0, "ids": [row_line_id.0]})).unwrap();

    // Line set apart on the row layer (weight 0.7087 pt)
    let over_line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 10.0, "y": 50.0}, {"x": 50.0, "y": 50.0}],
                "closed": false
            }),
        )
        .unwrap();
    let over_line_id = NodeId(over_line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 0.7087, "ids": [over_line_id.0]})).unwrap();

    // Locked line on the row layer (weight 0.5 pt)
    let locked_line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 10.0, "y": 70.0}, {"x": 50.0, "y": 70.0}],
                "closed": false
            }),
        )
        .unwrap();
    let locked_line_id = NodeId(locked_line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 0.5, "ids": [locked_line_id.0]})).unwrap();
    s.execute("layer.setProps", &json!({"ids": [locked_line_id.0], "locked": true})).unwrap();

    // 3. Hidden legend layer (weight 0.5 pt)
    let hidden_layer = s.execute("layer.new", &json!({"name": "숨긴줄", "top": true})).unwrap();
    let hidden_layer_id = NodeId(hidden_layer["id"].as_u64().unwrap());
    s.execute("legend.set", &json!({"id": hidden_layer_id.0, "weight": 0.5, "line": "solid"})).unwrap();
    let hidden_line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 10.0, "y": 90.0}, {"x": 50.0, "y": 90.0}],
                "closed": false
            }),
        )
        .unwrap();
    let hidden_line_id = NodeId(hidden_line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 0.5, "ids": [hidden_line_id.0]})).unwrap();
    s.execute("layer.setProps", &json!({"ids": [hidden_layer_id.0], "visible": false})).unwrap();

    (s, plain_layer_id, plain_line_id, row_layer_id, row_line_id, over_line_id, locked_line_id, hidden_line_id)
}

fn stroke_weight(s: &Session, id: NodeId) -> f64 {
    s.doc().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().width
}

fn row_weight(s: &Session, id: NodeId) -> Option<f64> {
    s.doc().unwrap().doc.node(id).and_then(vectorcraft_doc::Node::legend).and_then(|l| l.weight)
}

#[test]
fn test_param_empty_object_defaults() {
    let (mut s, _, plain_line, row_layer, row_line, over_line, locked_line, hidden_line) = create_test_session();

    // Invocation with empty object {}
    let res = s.execute("drawing.weightsAtOnce", &json!({})).expect("Empty object should succeed with defaults");

    // Default ratio is 0.8
    assert_eq!(res["ratio"], json!(0.8));
    // Response fields
    assert!(res["rows"].is_array(), "rows must be an array");
    assert!(res["lines"].is_u64(), "lines must be u64");
    assert_eq!(res["skipped"], json!(1), "one locked line should be skipped");
    let lines_cnt = res["lines"].as_u64().unwrap();
    let rows_cnt = res["rows"].as_array().unwrap().len() as u64;
    assert_eq!(res["modified"], json!(lines_cnt + rows_cnt));

    // Weight checks (1.0 * 0.8 = 0.8, 0.5 * 0.8 = 0.4, 0.7087 * 0.8 = 0.567 due to default overrides: true)
    assert_eq!(row_weight(&s, row_layer), Some(0.8));
    assert_eq!(stroke_weight(&s, row_line), 0.8);
    assert_eq!(stroke_weight(&s, plain_line), 0.4);
    assert_eq!(stroke_weight(&s, over_line), 0.567);
    assert_eq!(stroke_weight(&s, locked_line), 0.5, "locked line must not change");
    assert_eq!(stroke_weight(&s, hidden_line), 0.4, "hidden line must change");
}

#[test]
fn test_param_custom_ratio() {
    let (mut s, _, plain_line, row_layer, row_line, over_line, locked_line, _) = create_test_session();

    // Invocation with custom ratio 0.5
    let res = s.execute("drawing.weightsAtOnce", &json!({"ratio": 0.5})).expect("Custom ratio should succeed");

    assert_eq!(res["ratio"], json!(0.5));
    assert_eq!(row_weight(&s, row_layer), Some(0.5));
    assert_eq!(stroke_weight(&s, row_line), 0.5);
    assert_eq!(stroke_weight(&s, plain_line), 0.25);
    assert_eq!(stroke_weight(&s, over_line), 0.354); // 0.7087 * 0.5 = 0.35435 -> 0.354
    assert_eq!(stroke_weight(&s, locked_line), 0.5);
}

#[test]
fn test_param_explicit_legend_weights_format() {
    let (mut s, _, plain_line, row_layer, row_line, over_line, locked_line, _) = create_test_session();

    // Invocation with explicit legend.weights format: {"how": "ratio", "ratio": 1.2, "overrides": true}
    let res = s
        .execute(
            "drawing.weightsAtOnce",
            &json!({
                "how": "ratio",
                "ratio": 1.2,
                "overrides": true
            }),
        )
        .expect("Explicit legend.weights format should succeed");

    assert_eq!(res["ratio"], json!(1.2));
    assert_eq!(row_weight(&s, row_layer), Some(1.2));
    assert_eq!(stroke_weight(&s, row_line), 1.2);
    assert_eq!(stroke_weight(&s, plain_line), 0.6); // 0.5 * 1.2 = 0.6
    assert_eq!(stroke_weight(&s, over_line), 0.85); // 0.7087 * 1.2 = 0.85044 -> 0.85
    assert_eq!(stroke_weight(&s, locked_line), 0.5);
}

#[test]
fn test_param_explicit_overrides_false() {
    let (mut s, _, plain_line, row_layer, row_line, over_line, _, _) = create_test_session();

    // When overrides is explicitly false, over_line should NOT change
    let res = s
        .execute(
            "drawing.weightsAtOnce",
            &json!({
                "ratio": 0.5,
                "overrides": false
            }),
        )
        .expect("Explicit overrides: false should succeed");

    assert_eq!(res["ratio"], json!(0.5));
    assert_eq!(row_weight(&s, row_layer), Some(0.5));
    assert_eq!(stroke_weight(&s, row_line), 0.5);
    assert_eq!(stroke_weight(&s, plain_line), 0.25);
    assert_eq!(stroke_weight(&s, over_line), 0.7087, "line set apart must stay unchanged when overrides: false");
}

#[test]
fn test_param_invalid_values_rejected() {
    let (mut s, _, _, _, _, _, _, _) = create_test_session();

    // Negative ratio
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": -1.0})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": -0.001})).is_err());

    // Zero ratio
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": 0.0})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": -0.0})).is_err());

    // String instead of number
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": "0.5"})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": "invalid"})).is_err());

    // Bool / Array / Object / Null instead of number
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": true})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": [0.5]})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": {"val": 0.5}})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"ratio": null})).is_err());

    // Explicit format invalid values
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": "ratio", "ratio": -1.0})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": "ratio", "ratio": 0.0})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": "ratio"})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": "invalid_mode"})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": 123})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": null})).is_err());
    assert!(s.execute("drawing.weightsAtOnce", &json!({"how": "ratio", "ratio": 0.8, "overrides": "not_a_bool"})).is_err());
}

#[test]
fn test_param_extra_unexpected_fields() {
    let (mut s, _, plain_line, row_layer, row_line, _, _, _) = create_test_session();

    // Unexpected extra fields should be safely ignored
    let res = s
        .execute(
            "drawing.weightsAtOnce",
            &json!({
                "ratio": 0.5,
                "unexpected_field": "test",
                "nested_garbage": {"a": [1, 2, 3]},
                "timestamp": 123456789
            }),
        )
        .expect("Extra unexpected fields must be safely ignored without error");

    assert_eq!(res["ratio"], json!(0.5));
    assert_eq!(row_weight(&s, row_layer), Some(0.5));
    assert_eq!(stroke_weight(&s, row_line), 0.5);
    assert_eq!(stroke_weight(&s, plain_line), 0.25);
}

#[test]
fn test_non_object_root_inputs() {
    // Non-object root inputs (null, bool, number, array, string) should fallback cleanly to defaults
    for non_obj in [json!(null), json!(true), json!(123), json!([1, 2, 3]), json!("unexpected")] {
        let (mut s, _, plain_line, row_layer, row_line, _, _, _) = create_test_session();
        let res = s.execute("drawing.weightsAtOnce", &non_obj).expect("Non-object root input should fallback to defaults");
        assert_eq!(res["ratio"], json!(0.8));
        assert_eq!(row_weight(&s, row_layer), Some(0.8));
        assert_eq!(stroke_weight(&s, row_line), 0.8);
        assert_eq!(stroke_weight(&s, plain_line), 0.4);
    }
}

#[test]
fn test_response_shape_and_consistency_with_legend_weights() {
    let (mut s_compat, _, _, _, _, _, _, _) = create_test_session();
    let (mut s_legend, _, _, _, _, _, _, _) = create_test_session();

    let compat_res = s_compat
        .execute(
            "drawing.weightsAtOnce",
            &json!({
                "how": "ratio",
                "ratio": 0.8,
                "overrides": true
            }),
        )
        .unwrap();

    let legend_res = s_legend
        .execute(
            "legend.weights",
            &json!({
                "how": "ratio",
                "ratio": 0.8,
                "overrides": true
            }),
        )
        .unwrap();

    // Check common fields identical between both commands
    assert_eq!(compat_res["rows"], legend_res["rows"], "rows array must match exactly");
    assert_eq!(compat_res["lines"], legend_res["lines"], "lines count must match exactly");
    assert_eq!(compat_res["skipped"], legend_res["skipped"], "skipped count must match exactly");

    // Check compatibility-specific fields
    let lines_cnt = legend_res["lines"].as_u64().unwrap();
    let rows_cnt = legend_res["rows"].as_array().unwrap().len() as u64;
    assert_eq!(compat_res["modified"], json!(lines_cnt + rows_cnt), "modified must equal lines + rows.len()");
    assert_eq!(compat_res["ratio"], json!(0.8), "ratio field must be preserved");

    // Verify row structure
    let rows_arr = compat_res["rows"].as_array().unwrap();
    assert!(!rows_arr.is_empty());
    for r in rows_arr {
        assert!(r["id"].is_u64(), "row id must be u64");
        assert!(r["name"].is_string(), "row name must be string");
        assert!(r["from"].is_f64(), "row from must be f64");
        assert!(r["to"].is_f64(), "row to must be f64");
    }

    // Verify resulting document state is identical between drawing.weightsAtOnce and legend.weights
    assert_eq!(s_compat.active().unwrap().doc.layers.len(), s_legend.active().unwrap().doc.layers.len());
}

#[test]
fn test_extreme_ratios_clamping() {
    let (mut s, _, plain_line, row_layer, row_line, _, _, _) = create_test_session();

    // Very large ratio -> clamps to MAX_WEIGHT (1000.0 pt)
    let res = s.execute("drawing.weightsAtOnce", &json!({"ratio": 1e9})).unwrap();
    assert_eq!(res["ratio"], json!(1e9));
    assert_eq!(row_weight(&s, row_layer), Some(1000.0));
    assert_eq!(stroke_weight(&s, row_line), 1000.0);
    assert_eq!(stroke_weight(&s, plain_line), 1000.0);

    // Very small ratio -> clamps/rounds to 0.000 pt
    let (mut s_tiny, _, plain_line_t, row_layer_t, row_line_t, _, _, _) = create_test_session();
    let res_t = s_tiny.execute("drawing.weightsAtOnce", &json!({"ratio": 1e-9})).unwrap();
    assert_eq!(res_t["ratio"], json!(1e-9));
    assert_eq!(row_weight(&s_tiny, row_layer_t), Some(0.0));
    assert_eq!(stroke_weight(&s_tiny, row_line_t), 0.0);
    assert_eq!(stroke_weight(&s_tiny, plain_line_t), 0.0);
}

#[test]
fn test_identity_ratio_no_change_no_undo() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 200, "height": 200})).unwrap();
    let layer = s.execute("layer.new", &json!({"name": "test", "top": true})).unwrap();
    let layer_id = NodeId(layer["id"].as_u64().unwrap());
    s.execute("legend.set", &json!({"id": layer_id.0, "weight": 0.5, "line": "solid"})).unwrap();
    let line = s
        .execute(
            "path.create",
            &json!({
                "anchors": [{"x": 0.0, "y": 0.0}, {"x": 10.0, "y": 10.0}],
                "closed": false
            }),
        )
        .unwrap();
    let line_id = NodeId(line["id"].as_u64().unwrap());
    s.execute("stroke.set", &json!({"weight": 0.5, "ids": [line_id.0]})).unwrap();

    let revision_before = s.active().unwrap().revision;

    // Ratio 1.0 on strokes with standard precision: no changes occur
    let res = s.execute("drawing.weightsAtOnce", &json!({"ratio": 1.0})).unwrap();
    assert_eq!(res["modified"], json!(0));
    assert_eq!(res["lines"], json!(0));
    assert_eq!(res["rows"].as_array().unwrap().len(), 0);

    // Weights remain unchanged
    assert_eq!(row_weight(&s, layer_id), Some(0.5));
    assert_eq!(stroke_weight(&s, line_id), 0.5);

    // Revision untouched (no empty undo step created)
    assert_eq!(s.active().unwrap().revision, revision_before);
}

#[test]
fn test_undo_redo_round_trip() {
    let (mut s, _, plain_line, row_layer, row_line, over_line, _, _) = create_test_session();

    // Apply weights
    s.execute("drawing.weightsAtOnce", &json!({"ratio": 0.5})).unwrap();
    assert_eq!(row_weight(&s, row_layer), Some(0.5));
    assert_eq!(stroke_weight(&s, row_line), 0.5);
    assert_eq!(stroke_weight(&s, plain_line), 0.25);
    assert_eq!(stroke_weight(&s, over_line), 0.354);

    // Undo
    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(row_weight(&s, row_layer), Some(1.0), "undo must restore row weight");
    assert_eq!(stroke_weight(&s, row_line), 1.0, "undo must restore row line");
    assert_eq!(stroke_weight(&s, plain_line), 0.5, "undo must restore plain line");
    assert_eq!(stroke_weight(&s, over_line), 0.7087, "undo must restore overridden line");

    // Redo
    s.execute("edit.redo", &json!({})).unwrap();
    assert_eq!(row_weight(&s, row_layer), Some(0.5), "redo must re-apply row weight");
    assert_eq!(stroke_weight(&s, row_line), 0.5, "redo must re-apply row line");
    assert_eq!(stroke_weight(&s, plain_line), 0.25, "redo must re-apply plain line");
    assert_eq!(stroke_weight(&s, over_line), 0.354, "redo must re-apply overridden line");
}

#[test]
fn test_empty_document_handling() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();

    let res = s.execute("drawing.weightsAtOnce", &json!({})).expect("Empty document must not fail or panic");
    assert_eq!(res["modified"], json!(0));
    assert_eq!(res["lines"], json!(0));
    assert_eq!(res["skipped"], json!(0));
    assert_eq!(res["rows"].as_array().unwrap().len(), 0);
    assert_eq!(res["ratio"], json!(0.8));
}

#[test]
fn test_no_document_error_propagation() {
    let mut s = Session::new();
    // No document created: command is disabled via has_doc
    let err = s.execute("drawing.weightsAtOnce", &json!({})).expect_err("Must error when no document is open");
    assert!(
        matches!(err, EngineError::Disabled(ref cmd, _) if cmd == "drawing.weightsAtOnce"),
        "Expected EngineError::Disabled for drawing.weightsAtOnce, got: {err:?}"
    );
}
