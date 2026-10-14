//! Integration tests for Task J015 (Automatic Horizontal Elevation Lines & 1D Clipping).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use kurbo::{BezPath, Point};
use serde_json::json;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Node, NodeId, NodeKind, ScaleConfig, ScaleRatio};
use vectorcraft_geom::PathData;
use vectorcraft_engine::Session;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 2000, "height": 1000})).unwrap();
    s
}

/// Creates a trench ground profile path in the document.
fn create_trench_profile(s: &mut Session) -> NodeId {
    let mut profile = BezPath::new();
    profile.move_to((100.0, 150.0));
    profile.line_to((200.0, 150.0));
    profile.line_to((250.0, 350.0));
    profile.line_to((450.0, 350.0));
    profile.line_to((500.0, 150.0));
    profile.line_to((600.0, 150.0));

    let pd = PathData::from_bezpath(&profile);
    let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0, 0, 0)), 1.0);

    let mut id = NodeId(0);
    s.edit("Add Profile", |doc, _| {
        id = doc.alloc_id();
        let node = Node::path(id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    }).unwrap();
    id
}

/// Creates an obstacle (stone) in the trench floor.
fn create_stone_obstacle(s: &mut Session) -> NodeId {
    let mut stone = BezPath::new();
    stone.move_to((300.0, 300.0));
    stone.line_to((400.0, 300.0));
    stone.line_to((400.0, 350.0));
    stone.line_to((300.0, 350.0));
    stone.close_path();

    let pd = PathData::from_bezpath(&stone);
    let app = Appearance::basic(Paint::solid(Color::rgb8(200, 200, 200)), Paint::solid(Color::rgb8(0, 0, 0)), 0.5);

    let mut id = NodeId(0);
    s.edit("Add Stone", |doc, _| {
        id = doc.alloc_id();
        let node = Node::path(id, pd, app);
        doc.insert(None, usize::MAX, node).unwrap();
        Ok(())
    }).unwrap();
    id
}

#[test]
fn test_elevation_create_stroke_weights_and_labels() {
    let mut s = test_session();
    let profile_id = create_trench_profile(&mut s);
    let stone_id = create_stone_obstacle(&mut s);

    // Set scale 1:20 on document with datum at 150.0m
    s.edit("Set Scale", |doc, _| {
        doc.scale = Some(ScaleConfig {
            scale: ScaleRatio::new(20.0),
            crs: Default::default(),
            canvas_origin: Point::new(0.0, 500.0),
            world_origin: (0.0, 150.0),
        });
        Ok(())
    }).unwrap();

    let res = s.execute("elevation.create", &json!({
        "profileId": profile_id.0,
        "obstacleIds": [stone_id.0],
        "datumM": 150.0,
        "minorIntervalM": 0.1,
        "majorIntervalM": 0.5,
        "minorWeightPt": 0.2,
        "majorWeightPt": 0.5,
    })).unwrap();

    let gid = NodeId(res["id"].as_u64().unwrap());
    let seg_count = res["segmentCount"].as_u64().unwrap();
    let lbl_count = res["labelCount"].as_u64().unwrap();

    assert!(seg_count > 0, "must generate line segments");
    assert!(lbl_count > 0, "must generate labels for major lines");

    let doc = &s.doc().unwrap().doc;
    let group = doc.node(gid).unwrap();
    assert!(group.elevation.is_some(), "must carry ElevationSpec");

    if let NodeKind::Group { children, .. } = &group.kind {
        // Verify child node attributes
        let mut has_major = false;
        let mut has_minor = false;
        let mut has_label = false;

        for c in children {
            if let NodeKind::Path { .. } = &c.kind {
                if let Some(st) = c.appearance.stroke() {
                    if (st.width - 0.5).abs() < 1e-4 {
                        has_major = true;
                    } else if (st.width - 0.2).abs() < 1e-4 {
                        has_minor = true;
                    }
                }
            } else if let NodeKind::Text(t) = &c.kind {
                has_label = true;
                let txt = t.plain_text();
                assert!(txt.ends_with('m'), "elevation label must end with 'm': got {txt}");
            }
        }
        assert!(has_major, "must have major lines with 0.5 pt stroke");
        assert!(has_minor, "must have minor lines with 0.2 pt stroke");
        assert!(has_label, "must have text labels");
    } else {
        panic!("expected group node");
    }
}

#[test]
fn test_elevation_set_and_expand() {
    let mut s = test_session();
    let profile_id = create_trench_profile(&mut s);

    let res = s.execute("elevation.create", &json!({
        "profileId": profile_id.0,
        "minorIntervalM": 0.2,
        "majorIntervalM": 1.0,
    })).unwrap();
    let gid = NodeId(res["id"].as_u64().unwrap());

    // Update options via elevation.set
    let set_res = s.execute("elevation.set", &json!({
        "id": gid.0,
        "minorIntervalM": 0.1,
        "majorIntervalM": 0.5,
    })).unwrap();
    assert_eq!(set_res["success"], true);

    // Expand elevation group
    let exp_res = s.execute("elevation.expand", &json!({
        "id": gid.0,
    })).unwrap();
    assert_eq!(exp_res["expanded"], true);

    let doc = &s.doc().unwrap().doc;
    let group = doc.node(gid).unwrap();
    assert!(group.elevation.is_none(), "expanded group must clear ElevationSpec");
    if let NodeKind::Group { children, .. } = &group.kind {
        assert!(!children.is_empty(), "children remain as plain paths and text");
    }
}

#[test]
fn test_dynamic_reclip_on_profile_edit() {
    let mut s = test_session();
    let profile_id = create_trench_profile(&mut s);

    let res = s.execute("elevation.create", &json!({
        "profileId": profile_id.0,
        "minorIntervalM": 0.1,
        "majorIntervalM": 0.5,
    })).unwrap();
    let gid = NodeId(res["id"].as_u64().unwrap());

    let initial_child_count = if let NodeKind::Group { children, .. } = &s.doc().unwrap().doc.node(gid).unwrap().kind {
        children.len()
    } else {
        0
    };

    // Edit profile path: deepen trench to y = 600
    s.edit("Deepen Trench", |doc, _| {
        let mut deepened = BezPath::new();
        deepened.move_to((100.0, 150.0));
        deepened.line_to((200.0, 150.0));
        deepened.line_to((250.0, 600.0));
        deepened.line_to((450.0, 600.0));
        deepened.line_to((500.0, 150.0));
        deepened.line_to((600.0, 150.0));

        let node = doc.node_mut(profile_id).unwrap();
        if let NodeKind::Path { path, .. } = &mut node.kind {
            *path = PathData::from_bezpath(&deepened);
        }
        Ok(())
    }).unwrap();

    // Verify elevation lines dynamically updated
    let doc = &s.doc().unwrap().doc;
    let group = doc.node(gid).unwrap();
    if let NodeKind::Group { children, .. } = &group.kind {
        assert!(children.len() > initial_child_count, "deepened trench must generate more elevation lines: before={}, after={}", initial_child_count, children.len());
    }
}
