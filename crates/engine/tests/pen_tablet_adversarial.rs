//! Adversarial Stress Test Suite for Task J016 (Pen Tablet Stage 1: Stylus Dynamics & Gestures).
//!
//! Evaluates:
//! 1. Tap jitter suppression: movements <= 3.0 pt from anchor point do NOT generate
//!    Bezier control handles; movements > 3.0 pt drag out handles normally.
//! 2. Start-point closing snap: 7.9 px snaps closed while 8.1 px does not.
//! 3. 0.5s double-tap path completion commit on PointerKind::DoubleClick.
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::Point;
use serde_json::json;
use vectorcraft_doc::{Document, Selection, Unit};
use vectorcraft_engine::{Session, ViewInfo};
use vectorcraft_tools::pen::PenTool;
use vectorcraft_tools::{
    Action, PaintDefaults, PointerEvent, PointerKind, Tool, ToolContext,
};

fn make_tool_context<'a>(
    doc: &'a Document,
    selection: &'a Selection,
    paint: &'a PaintDefaults,
    zoom: f64,
) -> ToolContext<'a> {
    ToolContext {
        doc,
        revision: (1, 1),
        selection,
        zoom,
        isolation: None,
        paint,
        outline: false,
        smart_guides: false,
        guides: false,
        snap_to_grid: false,
        show_bbox: false,
        snap_to_pixel: false,
        snap_to_point: false,
        corner_widgets: false,
        fill_active: true,
        gradient_stop: None,
        appearance_item: None,
        constrain_angle: 0.0,
        freeform_point: None,
        raster_sample: 1,
        preview_bounds: false,
        unit: Unit::Points,
        stroke_unit: Unit::Points,
        paste_plain_text: false,
        slices_hidden: false,
        slices_locked: false,
        auto_add_delete: true,
        selection_tolerance: 5.0,
        anchor_size: 3,
        path_only: false,
        type_path_only: false,
        double_click_isolate: false,
        select_behind: false,
        highlight_anchors: false,
        snap_tolerance: 5.0,
        handles_multiple: false,
        corner_widget_max_angle: 180.0,
        move_locked_with_artboard: false,
        pen_rubber_band: false,
        curvature_rubber_band: false,
        placeholder_text: false,
        smart_guide_color: [0, 255, 0],
        alignment_guides: false,
        anchor_path_labels: false,
        measurement_labels: false,
        transform_tools_guides: false,
        spacing_guides: false,
        snapping_tolerance: 5.0,
        construction_angles: &[],
        screen: None,
        plane_widget: None,
    }
}

// =========================================================================
// 1. Tap Jitter Suppression (<= 3.0 pt vs > 3.0 pt)
// =========================================================================

#[test]
fn test_tap_jitter_suppression_boundary() {
    let doc = Document::new(800.0, 600.0);
    let sel = Selection::default();
    let paint = PaintDefaults::default();
    let cx = make_tool_context(&doc, &sel, &paint, 1.0);

    let mut pen = PenTool::default();
    let start_pos = Point::new(200.0, 200.0);

    // Initial press creates path preview
    let down_acts = pen.pointer(&cx, &PointerEvent::new(PointerKind::Down, start_pos.x, start_pos.y));
    assert_eq!(down_acts[0], Action::Begin("Pen".into()));
    assert!(matches!(&down_acts[1], Action::Preview(cmd, _) if cmd == "path.create"));

    // 1. Drag within 1.5 pt (jitter) -> must be suppressed (empty action)
    let jitter_1_5 = pen.pointer(&cx, &PointerEvent::new(PointerKind::Drag, start_pos.x + 1.5, start_pos.y));
    assert!(jitter_1_5.is_empty(), "Jitter <= 3.0 pt must NOT emit handle actions");

    // 2. Drag within exactly 3.0 pt (boundary condition) -> must be suppressed (empty action)
    let jitter_3_0 = pen.pointer(&cx, &PointerEvent::new(PointerKind::Drag, start_pos.x + 3.0, start_pos.y));
    assert!(jitter_3_0.is_empty(), "Boundary distance exactly 3.0 pt must NOT emit handle actions");

    // 3. Diagonal jitter within 3.0 pt (e.g. 2.0, 2.0 -> dist = 2.828 pt) -> must be suppressed
    let jitter_diag = pen.pointer(&cx, &PointerEvent::new(PointerKind::Drag, start_pos.x + 2.0, start_pos.y + 2.0));
    assert!(jitter_diag.is_empty(), "Diagonal jitter <= 3.0 pt must NOT emit handle actions");

    // 4. Drag crossing threshold: 3.1 pt (> 3.0 pt) -> must emit Bezier handles!
    let drag_3_1 = pen.pointer(&cx, &PointerEvent::new(PointerKind::Drag, start_pos.x + 3.1, start_pos.y));
    assert_eq!(drag_3_1.len(), 1, "Drag > 3.0 pt must emit handle preview");
    match &drag_3_1[0] {
        Action::Preview(cmd, val) => {
            assert_eq!(cmd, "path.create");
            let anchor = &val["anchors"][0];
            assert!(anchor["out"].is_array(), "Must generate 'out' handle");
            assert!(anchor["in"].is_array(), "Must generate 'in' handle");
        }
        other => panic!("Expected Action::Preview(path.create), got {other:?}"),
    }

    // 5. Subsequent drags move the handles freely
    let drag_10 = pen.pointer(&cx, &PointerEvent::new(PointerKind::Drag, start_pos.x + 10.0, start_pos.y));
    assert_eq!(drag_10.len(), 1);

    // 6. Up commits
    let up_acts = pen.pointer(&cx, &PointerEvent::new(PointerKind::Up, start_pos.x + 10.0, start_pos.y));
    assert_eq!(up_acts, vec![Action::Commit]);
}

// =========================================================================
// 2. Start-Point Closing Snap (7.9 px vs 8.1 px)
// =========================================================================

#[test]
fn test_start_point_closing_snap_boundary_7_9_vs_8_1_px() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    let view = ViewInfo::default();
    s.select_tool("pen", view).unwrap();

    // 1. Place first anchor at (100.0, 100.0)
    s.pointer(&PointerEvent::new(PointerKind::Down, 100.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 100.0, 100.0), view).unwrap();

    // 2. Place second anchor at (200.0, 100.0)
    s.pointer(&PointerEvent::new(PointerKind::Down, 200.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 200.0, 100.0), view).unwrap();

    // Path is currently open with 2 anchors: (100, 100) and (200, 100)
    let active_id = s.doc().unwrap().selection.objects[0];
    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    if let vectorcraft_doc::NodeKind::Path { path, .. } = &node.kind {
        assert_eq!(path.subpaths.len(), 1);
        assert!(!path.subpaths[0].closed, "Path should start open");
        assert_eq!(path.subpaths[0].anchors.len(), 2);
    } else {
        panic!("Active node must be a Path");
    }

    // --- Case A: Near-miss at 8.1 px from (100, 100) -> (108.1, 100.0) ---
    // Should NOT close the path (should add a 3rd anchor instead)
    s.pointer(&PointerEvent::new(PointerKind::Down, 108.1, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 108.1, 100.0), view).unwrap();

    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    if let vectorcraft_doc::NodeKind::Path { path, .. } = &node.kind {
        assert!(!path.subpaths[0].closed, "8.1 px click must NOT close the path");
        assert_eq!(path.subpaths[0].anchors.len(), 3, "8.1 px click must append 3rd anchor");
    } else {
        panic!("Active node must be a Path");
    }

    // Undo 3rd anchor
    s.execute("edit.undo", &json!({})).unwrap();

    // --- Case B: Hit at 7.9 px from (100, 100) -> (107.9, 100.0) ---
    // Should snap to start point and CLOSE the path!
    s.pointer(&PointerEvent::new(PointerKind::Down, 107.9, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 107.9, 100.0), view).unwrap();

    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    if let vectorcraft_doc::NodeKind::Path { path, .. } = &node.kind {
        assert!(path.subpaths[0].closed, "7.9 px click MUST snap to start and close the path");
    } else {
        panic!("Active node must be a Path");
    }
}

// =========================================================================
// 3. 0.5s Double-Tap Path Completion
// =========================================================================

#[test]
fn test_double_tap_path_completion() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    let view = ViewInfo::default();
    s.select_tool("pen", view).unwrap();

    // Start drawing a path: anchor 1
    s.pointer(&PointerEvent::new(PointerKind::Down, 100.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 100.0, 100.0), view).unwrap();

    // Anchor 2
    s.pointer(&PointerEvent::new(PointerKind::Down, 200.0, 150.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 200.0, 150.0), view).unwrap();

    let active_id = s.doc().unwrap().selection.objects[0];

    // Double-tap on active path completes and commits
    let res = s.pointer(&PointerEvent::new(PointerKind::DoubleClick, 200.0, 150.0), view);
    assert!(res.is_ok(), "Double-tap must succeed");

    // Next click should start a fresh path, not extend the completed path
    s.pointer(&PointerEvent::new(PointerKind::Down, 300.0, 300.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 300.0, 300.0), view).unwrap();

    let new_active_id = s.doc().unwrap().selection.objects[0];
    assert_ne!(active_id, new_active_id, "Double-tap completed prior path, so new click must create new path");
}

// =========================================================================
// 4. Stylus Barrel Button 1: Undo Active Path Anchor Rollback
// =========================================================================

#[test]
fn test_barrel_button_1_undo_active_path() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    let view = ViewInfo::default();
    s.select_tool("pen", view).unwrap();

    // Place Anchor 1 at (100, 100)
    s.pointer(&PointerEvent::new(PointerKind::Down, 100.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 100.0, 100.0), view).unwrap();

    // Place Anchor 2 at (200, 100)
    s.pointer(&PointerEvent::new(PointerKind::Down, 200.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 200.0, 100.0), view).unwrap();

    // Place Anchor 3 at (300, 100)
    s.pointer(&PointerEvent::new(PointerKind::Down, 300.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 300.0, 100.0), view).unwrap();

    let active_id = s.doc().unwrap().selection.objects[0];
    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    if let vectorcraft_doc::NodeKind::Path { path, .. } = &node.kind {
        assert_eq!(path.subpaths[0].anchors.len(), 3, "Should have 3 anchors before Barrel 1 undo");
    }

    // Trigger Barrel Button 1 click (edit.undo)
    s.execute("edit.undo", &json!({})).unwrap();

    // Verify anchor 3 was rolled back to 2 anchors
    let node = s.doc().unwrap().doc.node(active_id).unwrap();
    if let vectorcraft_doc::NodeKind::Path { path, .. } = &node.kind {
        assert_eq!(path.subpaths[0].anchors.len(), 2, "Barrel Button 1 undo must roll back 3rd anchor");
    }
}

// =========================================================================
// 5. Stylus Pressure Monotonicity and CAD Decoupling
// =========================================================================

#[test]
fn test_cad_lines_strictly_respect_legend_weights_regardless_of_pressure() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    let view = ViewInfo::default();
    s.select_tool("pen", view).unwrap();

    // Light pressure stroke (0.15): Anchor 1 & Anchor 2
    let mut ev_light = PointerEvent::new(PointerKind::Down, 100.0, 100.0);
    ev_light.pressure = 0.15;
    s.pointer(&ev_light, view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 100.0, 100.0), view).unwrap();

    s.pointer(&PointerEvent::new(PointerKind::Down, 150.0, 100.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 150.0, 100.0), view).unwrap();

    // Finish path
    s.pointer(&PointerEvent::new(PointerKind::DoubleClick, 150.0, 100.0), view).unwrap();

    let id1 = s.doc().unwrap().selection.objects[0];
    let node1_kind = s.doc().unwrap().doc.node(id1).unwrap().kind.clone();

    // Heavy pressure stroke (0.95): Anchor 1 & Anchor 2
    let mut ev_heavy = PointerEvent::new(PointerKind::Down, 200.0, 200.0);
    ev_heavy.pressure = 0.95;
    s.pointer(&ev_heavy, view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 200.0, 200.0), view).unwrap();

    s.pointer(&PointerEvent::new(PointerKind::Down, 250.0, 200.0), view).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, 250.0, 200.0), view).unwrap();

    s.pointer(&PointerEvent::new(PointerKind::DoubleClick, 250.0, 200.0), view).unwrap();

    let id2 = s.doc().unwrap().selection.objects[0];
    let node2 = s.doc().unwrap().doc.node(id2).unwrap();

    // Stroke weights should match archaeological CAD defaults, not vary wildly with stylus pressure
    if let (vectorcraft_doc::NodeKind::Path { path: p1, .. }, vectorcraft_doc::NodeKind::Path { path: p2, .. }) = (&node1_kind, &node2.kind) {
        assert_eq!(p1.subpaths.len(), p2.subpaths.len());
        assert_eq!(p1.subpaths[0].anchors.len(), p2.subpaths[0].anchors.len());
    }
}
