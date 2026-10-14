//! Artifact Pottery drafting engine commands (Trowel design 2.1 §4-3, §5, §14(3), §15(2), J011):
//! - `pottery.createVessel`: setup new pottery vessel drafting session and vertical centerline.
//! - `pottery.rimFit`: 3-point circumcircle fit R = abc/4A, diameter D = 2R, rim preservation %.
//! - `pottery.reflectExterior`: bilateral reflection of outer profile across centerline.
//! - `pottery.wallHatch`: construct closed section polygon and generate 45° 0.1 pt vector hatch.
//! - `pottery.measurements`: calculate D, d, H, t in mm/cm and emit sky-blue dimension annotations.
//! - `pottery.wallOffset`: generate inward normal offset wall profile.
//! - `pottery.horizontalRidge`: connect inflection points horizontally to centerline.

use kurbo::{BezPath, Point};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::appearance::{AppearanceItem, Dash, StrokeLayer};
use vectorcraft_doc::pottery::{
    MM, PotteryMeasurements, PotteryVessel, SectionFillMode, SectionSide, VesselType,
};
use vectorcraft_doc::{Appearance, CharStyle, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::PathData;

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "pottery.createVessel",
            "Create Pottery Vessel",
            ["Trowel", "유물"],
            None,
            "{name?: string, centerlineX: f64, sectionSide?: 'right'|'left', fillMode?: 'none'|'black'|'hatch', targetScale?: '1:1'|'1:2'|'1:3'|'1:4', vesselType?: string, findNumber?: string, feature?: string} initialize pottery vessel drafting session with vertical centerline",
            has_doc,
            create_vessel
        ),
        cmd!(
            "pottery.rimFit",
            "3-Point Rim Circle Fit",
            ["Trowel", "유물"],
            None,
            "{p1: [x,y]|{x,y}, p2: [x,y]|{x,y}, p3: [x,y]|{x,y}, applyToVesselId?: string, emitArc?: bool} fit circumcircle through 3 rim shard points with R=abc/4A and D=2R",
            has_doc,
            rim_fit
        ),
        cmd!(
            "pottery.reflectExterior",
            "Reflect Exterior Profile",
            ["Trowel", "유물"],
            None,
            "{pathId?: u64, centerlineX?: f64, vesselId?: string, targetSide?: 'left'|'right'} bilateral reflection of vessel outer profile across centerline",
            has_doc,
            reflect_exterior
        ),
        cmd!(
            "pottery.wallHatch",
            "Generate Wall Hatching",
            ["Trowel", "유물"],
            None,
            "{vesselId: string, fillMode?: 'none'|'black'|'hatch', spacingMm?: f64, weightPt?: f64, angleDeg?: f64, floorThicknessMm?: f64} build section polygon and generate 45° vector hatching",
            has_doc,
            wall_hatch
        ),
        cmd!(
            "pottery.measurements",
            "Pottery Measurements Readout",
            ["Trowel", "유물"],
            None,
            "{vesselId: string, emitAnnotations?: bool} compute vessel dimensions (D, d, H, t) and emit sky-blue annotations",
            has_doc,
            measurements
        ),
        cmd!(
            "pottery.wallOffset",
            "Offset Wall Thickness",
            ["Trowel", "유물"],
            None,
            "{pathId: u64, thicknessMm: f64, centerlineX?: f64, vesselId?: string} generate inward wall profile offset",
            has_doc,
            wall_offset
        ),
        cmd!(
            "pottery.horizontalRidge",
            "Horizontal Inflection Ridge",
            ["Trowel", "유물"],
            None,
            "{point: [x,y]|{x,y}, centerlineX?: f64, vesselId?: string} connect inflection point horizontally to vertical centerline",
            has_doc,
            horizontal_ridge
        ),
    ]
}

const C: &str = "pottery";

fn parse_point(v: &Value) -> Option<Point> {
    if let Some(arr) = v.as_array()
        && arr.len() >= 2
    {
        let x = arr.first()?.as_f64()?;
        let y = arr.get(1)?.as_f64()?;
        return Some(Point::new(x, y));
    }
    if let Some(obj) = v.as_object() {
        let x = obj.get("x")?.as_f64()?;
        let y = obj.get("y")?.as_f64()?;
        return Some(Point::new(x, y));
    }
    None
}



fn create_vessel(s: &mut Session, p: &Value) -> Result<Value> {
    let centerline_x = p
        .get("centerlineX")
        .and_then(Value::as_f64)
        .ok_or_else(|| bad(C, "centerlineX is required"))?;
    let name = p.get("name").and_then(Value::as_str).unwrap_or("심발").to_string();
    let find_number = p.get("findNumber").and_then(Value::as_str).unwrap_or("1").to_string();
    let feature = p.get("feature").and_then(Value::as_str).unwrap_or("1호 주거지").to_string();
    let target_scale = p.get("targetScale").and_then(Value::as_str).unwrap_or("1:3").to_string();

    let section_side = match p.get("sectionSide").and_then(Value::as_str) {
        Some("left") => SectionSide::Left,
        _ => SectionSide::Right,
    };

    let fill_mode = match p.get("fillMode").and_then(Value::as_str) {
        Some("black") => SectionFillMode::Black,
        Some("hatch") => SectionFillMode::Hatch45,
        _ => SectionFillMode::None,
    };

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Create Pottery Vessel", |doc, sel| {
        let cl_id = doc.alloc_id();

        // Centerline geometry: 1-dot chain line (0.2 pt)
        let mut cl_bez = BezPath::new();
        cl_bez.move_to(Point::new(centerline_x, 50.0));
        cl_bez.line_to(Point::new(centerline_x, 750.0));

        let cl_pd = PathData::from_bezpath(&cl_bez);
        let mut cl_stroke = StrokeLayer::new(Paint::solid(Color::rgb8(20, 20, 19)), 0.2);
        cl_stroke.dash = Some(Dash {
            pattern: vec![12.0, 3.0, 2.0, 3.0],
            offset: 0.0,
            align_corners: false,
        });

        let cl_app = Appearance {
            items: vec![AppearanceItem::Stroke(cl_stroke)],
            ..Default::default()
        };

        let mut cl_node = Node::path(cl_id, cl_pd, cl_app);
        cl_node.name = Some("유물 중심선".to_string());

        doc.insert(target_parent, usize::MAX, cl_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![cl_id];

        let vessel_id = format!("vessel-{}", doc.pottery_vessels.len() + 1);
        let vessel = PotteryVessel {
            id: vessel_id.clone(),
            name,
            vessel_type: VesselType::DeepBowl,
            period: "청동기시대".to_string(),
            find_number,
            feature,
            elevation_m: None,
            preservation_note: "완형".to_string(),
            centerline_x,
            section_side,
            fill_mode,
            target_scale,
            centerline_node: Some(cl_id),
            outer_profile_node: None,
            inner_profile_node: None,
            reflected_node: None,
            section_polygon_node: None,
            hatch_lines_node: None,
            ridge_nodes: Vec::new(),
            pattern_nodes: Vec::new(),
            measurements: PotteryMeasurements::default(),
        };

        doc.pottery_vessels.push(vessel);

        Ok(json!({
            "vesselId": vessel_id,
            "centerlineX": centerline_x,
            "centerlineNodeId": cl_id.0,
        }))
    })
}

fn rim_fit(s: &mut Session, p: &Value) -> Result<Value> {
    let p1_val = p.get("p1").ok_or_else(|| bad(C, "p1 is required"))?;
    let p2_val = p.get("p2").ok_or_else(|| bad(C, "p2 is required"))?;
    let p3_val = p.get("p3").ok_or_else(|| bad(C, "p3 is required"))?;

    let p1 = parse_point(p1_val).ok_or_else(|| bad(C, "invalid p1"))?;
    let p2 = parse_point(p2_val).ok_or_else(|| bad(C, "invalid p2"))?;
    let p3 = parse_point(p3_val).ok_or_else(|| bad(C, "invalid p3"))?;

    let fit = vectorcraft_geom::fit_circle_3point(p1, p2, p3)
        .map_err(|e| bad(C, format!("circle fit failed: {e}")))?;

    let apply_vessel_id = p.get("applyToVesselId").and_then(Value::as_str);
    let emit_arc = p.get("emitArc").and_then(Value::as_bool).unwrap_or(false);

    let target_parent = s.doc()?.target_parent()?;

    s.edit("3-Point Rim Circle Fit", |doc, sel| {
        let mut emitted_arc_id = None;

        if emit_arc {
            let arc_id = doc.alloc_id();
            let mut bez = BezPath::new();
            bez.move_to(p1);
            bez.quad_to(p2, p3);
            let pd = PathData::from_bezpath(&bez);
            let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(106, 155, 204)), 0.5);
            let mut arc_node = Node::path(arc_id, pd, app);
            arc_node.name = Some("구경 맞춤 호".to_string());
            doc.insert(target_parent, usize::MAX, arc_node).map_err(|e| bad(C, e.to_string()))?;
            sel.objects = vec![arc_id];
            emitted_arc_id = Some(arc_id.0);
        }

        if let Some(v_id) = apply_vessel_id {
            if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == v_id) {
                vessel.measurements.rim_diameter_mm = Some(fit.diameter_mm());
                vessel.measurements.rim_preservation_percent = Some(fit.rim_preservation_percent);
            }
        }

        Ok(json!({
            "center": [fit.center.x, fit.center.y],
            "radiusPt": fit.radius,
            "diameterPt": fit.diameter,
            "radiusMm": fit.radius * (25.4 / 72.0),
            "diameterMm": fit.diameter_mm(),
            "diameterCm": fit.diameter_cm(),
            "arcCoverageDeg": fit.arc_span_deg,
            "rimPreservationPercent": fit.rim_preservation_percent,
            "emittedArcId": emitted_arc_id,
        }))
    })
}

fn reflect_exterior(s: &mut Session, p: &Value) -> Result<Value> {
    let vessel_id = p.get("vesselId").and_then(Value::as_str);
    let path_id_val = p.get("pathId").and_then(Value::as_u64).map(NodeId);

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Reflect Exterior Profile", |doc, sel| {
        let (source_id, centerline_x) = if let Some(v_id) = vessel_id {
            let vessel = doc.pottery_vessels.iter().find(|v| v.id == v_id)
                .ok_or_else(|| bad(C, format!("vessel {v_id} not found")))?;
            let src = path_id_val.or(vessel.outer_profile_node)
                .ok_or_else(|| bad(C, "vessel has no outer_profile_node and no pathId given"))?;
            (src, vessel.centerline_x)
        } else {
            let src = path_id_val.ok_or_else(|| bad(C, "pathId is required when vesselId is omitted"))?;
            let cl_x = p.get("centerlineX").and_then(Value::as_f64).unwrap_or(300.0);
            (src, cl_x)
        };

        let node = doc.node(source_id).ok_or_else(|| bad(C, format!("source node {:?} not found", source_id)))?;
        let orig_pd = match &node.kind {
            NodeKind::Path { path, .. } => path,
            _ => return Err(bad(C, "source node is not a path")),
        };

        let reflected_pd = vectorcraft_pathops::reflect_path_across_centerline(orig_pd, centerline_x);

        let ref_id = doc.alloc_id();
        let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.8);
        let mut ref_node = Node::path(ref_id, reflected_pd, app);
        ref_node.name = Some("외형 입면선 (반쪽 대칭)".to_string());

        doc.insert(target_parent, usize::MAX, ref_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![ref_id];

        if let Some(v_id) = vessel_id {
            if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == v_id) {
                vessel.reflected_node = Some(ref_id);
            }
        }

        Ok(json!({
            "sourceId": source_id.0,
            "reflectedId": ref_id.0,
            "centerlineX": centerline_x,
        }))
    })
}

fn wall_hatch(s: &mut Session, p: &Value) -> Result<Value> {
    let vessel_id = p
        .get("vesselId")
        .and_then(Value::as_str)
        .ok_or_else(|| bad(C, "vesselId is required"))?;

    let fill_mode_str = p.get("fillMode").and_then(Value::as_str).unwrap_or("hatch");
    let fill_mode = match fill_mode_str {
        "black" => SectionFillMode::Black,
        "none" => SectionFillMode::None,
        _ => SectionFillMode::Hatch45,
    };

    let spacing_mm = p.get("spacingMm").and_then(Value::as_f64).unwrap_or(1.0);
    let weight_pt = p.get("weightPt").and_then(Value::as_f64).unwrap_or(0.1);
    let angle_deg = p.get("angleDeg").and_then(Value::as_f64).unwrap_or(45.0);
    let floor_thickness_mm = p.get("floorThicknessMm").and_then(Value::as_f64).unwrap_or(10.0);
    let floor_thickness_pt = floor_thickness_mm * MM;

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Generate Wall Hatching", |doc, sel| {
        let (outer_node_id, inner_node_id, centerline_x) = {
            let vessel = doc.pottery_vessels.iter().find(|v| v.id == vessel_id)
                .ok_or_else(|| bad(C, format!("vessel {vessel_id} not found")))?;
            let out_id = vessel.outer_profile_node
                .ok_or_else(|| bad(C, "vessel has no outer profile path"))?;
            (out_id, vessel.inner_profile_node, vessel.centerline_x)
        };

        let outer_pd = match doc.node(outer_node_id).map(|n| &n.kind) {
            Some(NodeKind::Path { path, .. }) => path.clone(),
            _ => return Err(bad(C, "outer profile node is not a path")),
        };

        let inner_pd = if let Some(in_id) = inner_node_id {
            match doc.node(in_id).map(|n| &n.kind) {
                Some(NodeKind::Path { path, .. }) => path.clone(),
                _ => PathData::default(),
            }
        } else {
            PathData::default()
        };

        let section_poly = vectorcraft_pathops::build_section_polygon(&outer_pd, &inner_pd, centerline_x, floor_thickness_pt);

        let poly_id = doc.alloc_id();
        let (poly_app, hatch_node_id) = match fill_mode {
            SectionFillMode::Black => {
                let app = Appearance::basic(Paint::solid(Color::rgb8(20, 20, 19)), Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
                (app, None)
            }
            SectionFillMode::Hatch45 => {
                let poly_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);

                let hatch_lines = vectorcraft_pathops::generate_section_hatch(&section_poly, spacing_mm, weight_pt, angle_deg);
                let hatch_pd = PathData::from_bezpath(&hatch_lines);
                let hatch_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), weight_pt);

                let h_id = doc.alloc_id();
                let mut h_node = Node::path(h_id, hatch_pd, hatch_app);
                h_node.name = Some("단면 빗금 (45°)".to_string());
                doc.insert(target_parent, usize::MAX, h_node).map_err(|e| bad(C, e.to_string()))?;

                (poly_app, Some(h_id))
            }
            SectionFillMode::None => {
                let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
                (app, None)
            }
        };

        let mut poly_node = Node::path(poly_id, section_poly, poly_app);
        poly_node.name = Some("단면 외곽선".to_string());
        doc.insert(target_parent, usize::MAX, poly_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![poly_id];

        if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == vessel_id) {
            vessel.section_polygon_node = Some(poly_id);
            vessel.hatch_lines_node = hatch_node_id;
            vessel.fill_mode = fill_mode;
        }

        Ok(json!({
            "vesselId": vessel_id,
            "sectionPolygonId": poly_id.0,
            "hatchId": hatch_node_id.map(|id| id.0),
            "fillMode": fill_mode_str,
        }))
    })
}

fn measurements(s: &mut Session, p: &Value) -> Result<Value> {
    let vessel_id = p
        .get("vesselId")
        .and_then(Value::as_str)
        .ok_or_else(|| bad(C, "vesselId is required"))?;
    let emit_annotations = p.get("emitAnnotations").and_then(Value::as_bool).unwrap_or(false);

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Pottery Measurements Readout", |doc, sel| {
        let (outer_node_id, inner_node_id, centerline_x, rim_fit_d) = {
            let vessel = doc.pottery_vessels.iter().find(|v| v.id == vessel_id)
                .ok_or_else(|| bad(C, format!("vessel {vessel_id} not found")))?;
            let out_id = vessel.outer_profile_node
                .ok_or_else(|| bad(C, "vessel has no outer profile path"))?;
            let rim_fit_pt = vessel.measurements.rim_diameter_mm.map(|d_mm| d_mm * MM);
            (out_id, vessel.inner_profile_node, vessel.centerline_x, rim_fit_pt)
        };

        let outer_pd = match doc.node(outer_node_id).map(|n| &n.kind) {
            Some(NodeKind::Path { path, .. }) => path.clone(),
            _ => return Err(bad(C, "outer profile node is not a path")),
        };

        let inner_pd = inner_node_id.and_then(|in_id| {
            match doc.node(in_id).map(|n| &n.kind) {
                Some(NodeKind::Path { path, .. }) => Some(path.clone()),
                _ => None,
            }
        });

        let mut m = vectorcraft_pathops::compute_vessel_measurements(centerline_x, &outer_pd, inner_pd.as_ref(), rim_fit_d);

        // Keep existing rim fit preservation percent if present
        if let Some(vessel) = doc.pottery_vessels.iter().find(|v| v.id == vessel_id) {
            if m.rim_preservation_percent.is_none() {
                m.rim_preservation_percent = vessel.measurements.rim_preservation_percent;
            }
        }

        let ledger = m.format_ledger();

        if emit_annotations {
            let mut annot_children = Vec::new();
            let blue_paint = Paint::solid(Color::rgb8(106, 155, 204));

            // Dimension labels
            if let Some(rim_cm) = m.rim_diameter_cm() {
                let lbl_id = doc.alloc_id();
                let char_style = CharStyle {
                    size: 8.0,
                    fill: blue_paint.clone(),
                    ..Default::default()
                };
                let txt_obj = TextObject::point(Point::new(centerline_x, 40.0), &format!("구경 {:.1}cm", rim_cm), char_style);
                let txt_node = Node::new(lbl_id, NodeKind::Text(Box::new(txt_obj)));
                annot_children.push(Arc::new(txt_node));
            }

            if let Some(h_cm) = m.height_cm() {
                let lbl_id = doc.alloc_id();
                let char_style = CharStyle {
                    size: 8.0,
                    fill: blue_paint.clone(),
                    ..Default::default()
                };
                let txt_obj = TextObject::point(Point::new(centerline_x - 40.0, 150.0), &format!("기고 {:.1}cm", h_cm), char_style);
                let txt_node = Node::new(lbl_id, NodeKind::Text(Box::new(txt_obj)));
                annot_children.push(Arc::new(txt_node));
            }

            if !annot_children.is_empty() {
                let annot_group_id = doc.alloc_id();
                let mut annot_group = Node::new(annot_group_id, NodeKind::Group { children: annot_children, clip: false });
                annot_group.name = Some("계측선 및 수치".to_string());
                doc.insert(target_parent, usize::MAX, annot_group).map_err(|e| bad(C, e.to_string()))?;
                sel.objects = vec![annot_group_id];
            }
        }

        if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == vessel_id) {
            vessel.measurements = m.clone();
        }

        Ok(json!({
            "vesselId": vessel_id,
            "rimDiameterMm": m.rim_diameter_mm,
            "rimDiameterCm": m.rim_diameter_cm(),
            "baseDiameterMm": m.base_diameter_mm,
            "baseDiameterCm": m.base_diameter_cm(),
            "heightMm": m.height_mm,
            "heightCm": m.height_cm(),
            "wallThicknessMm": m.wall_thickness_mm,
            "wallThicknessCm": m.wall_thickness_cm(),
            "maxBellyDiameterMm": m.max_belly_diameter_mm,
            "ledger": [ledger.0, ledger.1, ledger.2, ledger.3],
        }))
    })
}

fn wall_offset(s: &mut Session, p: &Value) -> Result<Value> {
    let path_id = p.get("pathId").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(C, "pathId is required"))?;
    let thickness_mm = p.get("thicknessMm").and_then(Value::as_f64).ok_or_else(|| bad(C, "thicknessMm is required"))?;
    let thickness_pt = thickness_mm * MM;

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Offset Wall Thickness", |doc, sel| {
        let node = doc.node(path_id).ok_or_else(|| bad(C, format!("node {:?} not found", path_id)))?;
        let orig_pd = match &node.kind {
            NodeKind::Path { path, .. } => path,
            _ => return Err(bad(C, "node is not a path")),
        };

        let (centerline_x, side) = if let Some(v_id) = p.get("vesselId").and_then(Value::as_str) {
            let v = doc.pottery_vessels.iter().find(|v| v.id == v_id)
                .ok_or_else(|| bad(C, format!("vessel {v_id} not found")))?;
            (v.centerline_x, v.section_side)
        } else {
            let cl_x = p.get("centerlineX").and_then(Value::as_f64).unwrap_or(300.0);
            (cl_x, SectionSide::Right)
        };

        let inner_pd = vectorcraft_pathops::offset_wall_profile(orig_pd, thickness_pt, centerline_x, side);

        let inner_id = doc.alloc_id();
        let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut inner_node = Node::path(inner_id, inner_pd, app);
        inner_node.name = Some("내면선 (기벽 두께)".to_string());

        doc.insert(target_parent, usize::MAX, inner_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![inner_id];

        if let Some(v_id) = p.get("vesselId").and_then(Value::as_str) {
            if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == v_id) {
                vessel.inner_profile_node = Some(inner_id);
            }
        }

        Ok(json!({
            "innerPathId": inner_id.0,
            "thicknessMm": thickness_mm,
        }))
    })
}

fn horizontal_ridge(s: &mut Session, p: &Value) -> Result<Value> {
    let pt_val = p.get("point").ok_or_else(|| bad(C, "point is required"))?;
    let point = parse_point(pt_val).ok_or_else(|| bad(C, "invalid point coordinates"))?;

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Horizontal Inflection Ridge", |doc, sel| {
        let centerline_x = if let Some(v_id) = p.get("vesselId").and_then(Value::as_str) {
            let v = doc.pottery_vessels.iter().find(|v| v.id == v_id)
                .ok_or_else(|| bad(C, format!("vessel {v_id} not found")))?;
            v.centerline_x
        } else {
            p.get("centerlineX").and_then(Value::as_f64).unwrap_or(300.0)
        };

        let line = vectorcraft_pathops::generate_horizontal_ridge(point, centerline_x);
        let mut bez = BezPath::new();
        bez.move_to(line.p0);
        bez.line_to(line.p1);

        let ridge_id = doc.alloc_id();
        let pd = PathData::from_bezpath(&bez);
        let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut ridge_node = Node::path(ridge_id, pd, app);
        ridge_node.name = Some("수평 능선".to_string());

        doc.insert(target_parent, usize::MAX, ridge_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![ridge_id];

        if let Some(v_id) = p.get("vesselId").and_then(Value::as_str) {
            if let Some(vessel) = doc.pottery_vessels.iter_mut().find(|v| v.id == v_id) {
                vessel.ridge_nodes.push(ridge_id);
            }
        }

        Ok(json!({
            "ridgeId": ridge_id.0,
            "p0": [line.p0.x, line.p0.y],
            "p1": [line.p1.x, line.p1.y],
        }))
    })
}
