//! Plan drafting and feature measurement commands (Trowel design 2.1 §3, §4-1, §7, §8, §13, §14, §15, J009):
//! - `trowel.sectionLine`: create 1-dot chain section cut line with directional arrows and labels (Shift+A).
//! - `trowel.spotElevation`: create spot elevation marker symbol and formatted readout (Shift+Z).
//! - `trowel.depthMarker`: create posthole concentric circles or trench depth indicator.
//! - `trowel.slopeHachures`: generate alternating slope hachure ticks along embankment/pit edges.
//! - `trowel.polygonStats`: closed polygon statistical readout (exact area in ㎡, perimeter, MBB axes, PCA azimuth).

use kurbo::{BezPath, Point};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::appearance::{AppearanceItem, Dash, StrokeLayer};
use vectorcraft_doc::{Appearance, CharStyle, Justify, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::PathData;
use vectorcraft_pathops::plan::{
    DEFAULT_HACHURE_SHORT_RATIO, DEFAULT_HACHURE_SPACING_MM, DEFAULT_PLAN_WEIGHT_PT,
    DepthMarkerKind, DepthMarkerSpec, SectionLineSpec, SlopeHachureSpec, SpotElevationSpec,
    SpotMarkerKind, ViewDirection, compute_polygon_stats, generate_depth_marker_geometry,
    generate_section_line_geometry, generate_slope_hachures, generate_spot_elevation_geometry,
};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "trowel.sectionLine",
            "Section Cut Line",
            ["Trowel", "도면"],
            Some("Shift+A"),
            "{p0: [x,y]|{x,y}, p1: [x,y]|{x,y}, labelStart?: string, labelEnd?: string, viewDirection?: 'left'|'right', weight?: f64, arrowLength?: f64} create an A-A' section cut line with directional arrows and labels",
            has_doc,
            section_line
        ),
        cmd!(
            "trowel.spotElevation",
            "Spot Elevation",
            ["Trowel", "도면"],
            Some("Shift+Z"),
            "{at: [x,y]|{x,y}, elevation: f64, marker?: 'dot'|'triangle'|'crosshair', fontSize?: f64, halo?: bool, prefixPlus?: bool, suffixM?: bool} create an elevation marker symbol and formatted text",
            has_doc,
            spot_elevation
        ),
        cmd!(
            "trowel.depthMarker",
            "Trench / Posthole Depth Marker",
            ["Trowel", "도면"],
            None,
            "{at: [x,y]|{x,y}, depth: f64, isCm?: bool, symbol?: 'posthole'|'cross'|'none', radius?: f64, fontSize?: f64} create a feature depth indicator",
            has_doc,
            depth_marker
        ),
        cmd!(
            "trowel.slopeHachures",
            "Slope Hachures",
            ["Trowel", "도면"],
            None,
            "{topId?: u64, bottomId?: u64, spacingMm?: f64, shortRatio?: f64, width?: f64, range?: [f64, f64]} emit alternating long/short hachure ticks along curve",
            has_doc,
            slope_hachures
        ),
        cmd!(
            query "trowel.polygonStats",
            "Polygon Statistical Readout",
            ["Trowel", "도면"],
            None,
            "{id?: u64, points?: [[x, y], ...]} calculate exact area in ㎡, perimeter, MBB axes, and PCA azimuth",
            has_doc,
            polygon_stats
        ),
    ]
}

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

fn get_bezpath_from_node(node: &Node) -> Option<BezPath> {
    match &node.kind {
        NodeKind::Path { path, .. } => Some(path.to_bezpath()),
        NodeKind::Compound { children, .. } | NodeKind::Group { children, .. } => {
            let mut bp = BezPath::new();
            for c in children {
                if let Some(pd) = c.path_data() {
                    bp.extend(pd.to_bezpath());
                }
            }
            if bp.elements().is_empty() { None } else { Some(bp) }
        }
        _ => None,
    }
}

fn section_line(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "trowel.sectionLine";

    let p0 = p.get("p0").or_else(|| p.get("p1")).and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `p0` or `p1`"))?;
    let p1 = if p.get("p0").is_some() {
        p.get("p1")
    } else {
        p.get("p2")
    }.and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `p1` or `p2`"))?;

    let label_start = p.get("labelStart").and_then(|v| v.as_str()).unwrap_or("A").to_string();
    let label_end = p.get("labelEnd").and_then(|v| v.as_str()).unwrap_or("A′").to_string();

    let view_direction = match p.get("viewDirection").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("right") => ViewDirection::Right,
        _ => ViewDirection::Left,
    };

    let weight_pt = p.get("weight").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_PLAN_WEIGHT_PT);
    let arrow_length_pt = p.get("arrowLength").and_then(|v| v.as_f64()).unwrap_or(14.0);

    let spec = SectionLineSpec {
        p0,
        p1,
        label_start,
        label_end,
        view_direction,
        weight_pt,
        arrow_length_pt,
        dash_pattern: vec![12.0, 3.0, 2.0, 3.0],
    };

    let geom = generate_section_line_geometry(&spec);
    let target_parent = s.doc()?.target_parent()?;

    s.edit("Section Cut Line", |doc, sel| {
        let mut children = Vec::with_capacity(5);

        // 1. Cut line (1-dot chain line)
        let cut_id = doc.alloc_id();
        let cut_pd = PathData::from_bezpath(&geom.cut_line);
        let mut cut_stroke = StrokeLayer::new(Paint::solid(Color::rgb8(20, 20, 19)), geom.weight_pt);
        cut_stroke.dash = Some(Dash { pattern: geom.dash_pattern.clone(), offset: 0.0, align_corners: false });
        let cut_app = Appearance { items: vec![AppearanceItem::Stroke(cut_stroke)], ..Default::default() };
        let mut cut_node = Node::path(cut_id, cut_pd, cut_app);
        cut_node.name = Some("단면선".to_string());
        children.push(Arc::new(cut_node));

        // 2. Sightline arrows
        let arrows_id = doc.alloc_id();
        let arrows_pd = PathData::from_bezpath(&geom.arrows);
        let arrows_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut arrows_node = Node::path(arrows_id, arrows_pd, arrows_app);
        arrows_node.name = Some("방향 화살표".to_string());
        children.push(Arc::new(arrows_node));

        // 3. Terminal badges (white background halo circles)
        let badges_id = doc.alloc_id();
        let badges_pd = PathData::from_bezpath(&geom.badges);
        let badges_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut badges_node = Node::path(badges_id, badges_pd, badges_app);
        badges_node.name = Some("기호 원".to_string());
        children.push(Arc::new(badges_node));

        // 4. Start label ("A")
        let l0_id = doc.alloc_id();
        let l0_style = CharStyle {
            size: 10.0,
            fill: Paint::solid(Color::rgb8(20, 20, 19)),
            stroke: Paint::solid(Color::WHITE),
            stroke_width: 2.0,
            ..Default::default()
        };
        let mut l0_obj = TextObject::point(geom.label_start_pos, &geom.label_start, l0_style);
        l0_obj.para.justify = Justify::Center;
        let mut l0_node = Node::new(l0_id, NodeKind::Text(Box::new(l0_obj)));
        l0_node.name = Some(format!("단면 기호 {}", geom.label_start));
        children.push(Arc::new(l0_node));

        // 5. End label ("A′")
        let l1_id = doc.alloc_id();
        let l1_style = CharStyle {
            size: 10.0,
            fill: Paint::solid(Color::rgb8(20, 20, 19)),
            stroke: Paint::solid(Color::WHITE),
            stroke_width: 2.0,
            ..Default::default()
        };
        let mut l1_obj = TextObject::point(geom.label_end_pos, &geom.label_end, l1_style);
        l1_obj.para.justify = Justify::Center;
        let mut l1_node = Node::new(l1_id, NodeKind::Text(Box::new(l1_obj)));
        l1_node.name = Some(format!("단면 기호 {}", geom.label_end));
        children.push(Arc::new(l1_node));

        // Composite group
        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some(format!("단면선 {}–{}", geom.label_start, geom.label_end));

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![group_id];

        Ok(json!({
            "id": group_id.0,
            "p0": [p0.x, p0.y],
            "p1": [p1.x, p1.y],
            "labelStart": geom.label_start,
            "labelEnd": geom.label_end,
        }))
    })
}

fn spot_elevation(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "trowel.spotElevation";

    let at = p.get("at").and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `at`"))?;
    let elevation = p.get("elevation").and_then(|v| v.as_f64()).ok_or_else(|| bad(C, "missing `elevation` value"))?;

    let marker = match p.get("marker").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("triangle") => SpotMarkerKind::Triangle,
        Some("crosshair") => SpotMarkerKind::Crosshair,
        _ => SpotMarkerKind::Dot,
    };

    let font_size_pt = p.get("fontSize").and_then(|v| v.as_f64()).unwrap_or(7.0);
    let halo = p.get("halo").and_then(|v| v.as_bool()).unwrap_or(true);
    let prefix_plus = p.get("prefixPlus").and_then(|v| v.as_bool()).unwrap_or(true);
    let suffix_m = p.get("suffixM").and_then(|v| v.as_bool()).unwrap_or(true);

    let spec = SpotElevationSpec {
        at,
        elevation_m: elevation,
        marker,
        font_size_pt,
        halo,
        prefix_plus,
        suffix_m,
    };

    let geom = generate_spot_elevation_geometry(&spec);
    let target_parent = s.doc()?.target_parent()?;

    s.edit("Spot Elevation", |doc, sel| {
        let mut children = Vec::with_capacity(2);

        // Marker symbol
        let sym_id = doc.alloc_id();
        let sym_pd = PathData::from_bezpath(&geom.marker_path);
        let sym_app = if geom.marker_fill {
            Appearance::basic(Paint::solid(Color::rgb8(20, 20, 19)), Paint::None, 0.0)
        } else {
            Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.8)
        };
        let mut sym_node = Node::path(sym_id, sym_pd, sym_app);
        sym_node.name = Some("표고 기호".to_string());
        children.push(Arc::new(sym_node));

        // Text label
        let text_id = doc.alloc_id();
        let style = CharStyle {
            size: geom.font_size_pt,
            fill: Paint::solid(Color::rgb8(20, 20, 19)),
            stroke: if geom.halo { Paint::solid(Color::WHITE) } else { Paint::None },
            stroke_width: if geom.halo { 2.5 } else { 0.0 },
            ..Default::default()
        };
        let t_obj = TextObject::point(geom.label_pos, &geom.text, style);
        let mut text_node = Node::new(text_id, NodeKind::Text(Box::new(t_obj)));
        text_node.name = Some(format!("표고 {}", geom.text));
        children.push(Arc::new(text_node));

        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some(format!("표고 {}", geom.text));

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![group_id];

        Ok(json!({
            "id": group_id.0,
            "elevation": elevation,
            "text": geom.text,
            "at": [at.x, at.y],
        }))
    })
}

fn depth_marker(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "trowel.depthMarker";

    let at = p.get("at").and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `at`"))?;
    let depth = p.get("depth").or_else(|| p.get("depthCm")).and_then(|v| v.as_f64()).ok_or_else(|| bad(C, "missing `depth` or `depthCm` value"))?;

    let is_cm = p.get("isCm").and_then(|v| v.as_bool()).unwrap_or_else(|| depth.abs() > 3.0);

    let symbol = match p.get("symbol").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("cross") => DepthMarkerKind::Cross,
        Some("none") => DepthMarkerKind::None,
        _ => DepthMarkerKind::PostholeConcentric,
    };

    let radius = p.get("radius").and_then(|v| v.as_f64()).unwrap_or(7.0);
    let font_size = p.get("fontSize").and_then(|v| v.as_f64()).unwrap_or(7.0);

    let spec = DepthMarkerSpec {
        at,
        depth_val: depth,
        is_cm,
        symbol,
        font_size_pt: font_size,
        outer_radius_pt: radius,
    };

    let geom = generate_depth_marker_geometry(&spec);
    let target_parent = s.doc()?.target_parent()?;

    s.edit("Depth Marker", |doc, sel| {
        let mut children = Vec::with_capacity(3);

        if !geom.solid_symbol.elements().is_empty() {
            let sid = doc.alloc_id();
            let spd = PathData::from_bezpath(&geom.solid_symbol);
            let sapp = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.8);
            let mut snode = Node::path(sid, spd, sapp);
            snode.name = Some("주혈 외경".to_string());
            children.push(Arc::new(snode));
        }

        if !geom.dashed_symbol.elements().is_empty() {
            let did = doc.alloc_id();
            let dpd = PathData::from_bezpath(&geom.dashed_symbol);
            let mut dstroke = StrokeLayer::new(Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
            dstroke.dash = Some(Dash { pattern: vec![2.0, 1.5], offset: 0.0, align_corners: false });
            let dapp = Appearance { items: vec![AppearanceItem::Stroke(dstroke)], ..Default::default() };
            let mut dnode = Node::path(did, dpd, dapp);
            dnode.name = Some("주혈 내경".to_string());
            children.push(Arc::new(dnode));
        }

        let tid = doc.alloc_id();
        let style = CharStyle {
            size: geom.font_size_pt,
            fill: Paint::solid(Color::rgb8(20, 20, 19)),
            stroke: Paint::solid(Color::WHITE),
            stroke_width: 2.0,
            ..Default::default()
        };
        let tobj = TextObject::point(geom.label_pos, &geom.text, style);
        let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
        tnode.name = Some(format!("깊이 {}", geom.text));
        children.push(Arc::new(tnode));

        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some("주혈 깊이".to_string());

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![group_id];

        Ok(json!({
            "id": group_id.0,
            "depth": depth,
            "text": geom.text,
            "at": [at.x, at.y],
        }))
    })
}

fn slope_hachures(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "trowel.slopeHachures";

    let doc = s.doc()?;
    let top_id = p
        .get("topId")
        .and_then(|v| v.as_u64())
        .map(NodeId)
        .or_else(|| doc.selection.objects.first().copied())
        .ok_or_else(|| bad(C, "give topId or select top boundary curve"))?;

    let bottom_id = p.get("bottomId").and_then(|v| v.as_u64()).map(NodeId);

    let top_node = doc.doc.node(top_id).ok_or_else(|| bad(C, format!("top node {} not found", top_id.0)))?;
    let top_bp = get_bezpath_from_node(top_node).ok_or_else(|| bad(C, "top node has no path geometry"))?;

    let bottom_bp = if let Some(bid) = bottom_id {
        let bnode = doc.doc.node(bid).ok_or_else(|| bad(C, format!("bottom node {} not found", bid.0)))?;
        get_bezpath_from_node(bnode)
    } else {
        None
    };

    let spacing_mm = p.get("spacingMm").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_HACHURE_SPACING_MM);
    let short_ratio = p.get("shortRatio").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_HACHURE_SHORT_RATIO);
    let width = p.get("width").and_then(|v| v.as_f64()).unwrap_or(15.0);

    let range = if let Some(arr) = p.get("range").and_then(|v| v.as_array())
        && arr.len() >= 2
    {
        Some((arr[0].as_f64().unwrap_or(0.0), arr[1].as_f64().unwrap_or(1.0)))
    } else {
        None
    };

    let spec = SlopeHachureSpec {
        spacing_mm,
        short_ratio,
        weight_pt: DEFAULT_PLAN_WEIGHT_PT,
        default_width_pt: width,
        range,
    };

    let hachures_bp = generate_slope_hachures(&top_bp, bottom_bp.as_ref(), &spec);
    let target_parent = doc.target_parent()?;

    s.edit("Slope Hachures", |doc, sel| {
        let node_id = doc.alloc_id();
        let pd = PathData::from_bezpath(&hachures_bp);
        let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), spec.weight_pt);
        let mut node = Node::path(node_id, pd, app);
        node.name = Some("비탈면 빗금".to_string());

        doc.insert(target_parent, usize::MAX, node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![node_id];

        Ok(json!({
            "id": node_id.0,
            "topId": top_id.0,
            "bottomId": bottom_id.map(|id| id.0),
        }))
    })
}

fn polygon_stats(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "trowel.polygonStats";

    let doc = s.doc()?;
    let path = if let Some(pts_arr) = p.get("points").and_then(|v| v.as_array()) {
        let mut bp = BezPath::new();
        let mut first = true;
        for item in pts_arr {
            if let Some(pt) = parse_point(item) {
                if first {
                    bp.move_to(pt);
                    first = false;
                } else {
                    bp.line_to(pt);
                }
            }
        }
        bp.close_path();
        bp
    } else {
        let target_id = p
            .get("id")
            .and_then(|v| v.as_u64())
            .map(NodeId)
            .or_else(|| doc.selection.objects.first().copied())
            .ok_or_else(|| bad(C, "give target node `id` or select a closed path, or supply `points`"))?;

        let node = doc.doc.node(target_id).ok_or_else(|| bad(C, format!("node {} not found", target_id.0)))?;
        get_bezpath_from_node(node).ok_or_else(|| bad(C, "target node has no path geometry"))?
    };

    let scale_cfg = doc.doc.scale.clone().unwrap_or_default();
    let stats = compute_polygon_stats(&path, &scale_cfg);

    Ok(json!({
        "area_m2": stats.area_m2,
        "perimeter_m": stats.perimeter_m,
        "major_axis_m": stats.major_axis_m,
        "minor_axis_m": stats.minor_axis_m,
        "azimuth_deg": stats.azimuth_deg,
        "azimuth_quadrant": stats.azimuth_quadrant,
        "centroid_canvas": [stats.centroid_canvas.x, stats.centroid_canvas.y],
        "centroid_world": [stats.centroid_world.0, stats.centroid_world.1],
        "formatted": {
            "area": stats.format_area(),
            "perimeter": stats.format_perimeter(),
            "axes": stats.format_axes(),
            "azimuth": stats.format_azimuth(),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trowel_plan_commands() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).expect("new doc");

        // 1. Section line
        let res = s.execute("trowel.sectionLine", &json!({
            "p1": [10.0, 10.0],
            "p2": [200.0, 10.0],
            "label": "A"
        })).expect("section line");
        assert!(res.get("id").is_some());

        // 2. Spot elevation
        let res = s.execute("trowel.spotElevation", &json!({
            "at": [100.0, 100.0],
            "elevation": 152.35
        })).expect("spot elevation");
        assert!(res.get("id").is_some());

        // 3. Depth marker
        let res = s.execute("trowel.depthMarker", &json!({
            "at": [150.0, 150.0],
            "depthCm": 45.0
        })).expect("depth marker");
        assert!(res.get("id").is_some());

        // 4. Polygon stats with canonical House 1 points (at default 1:20 scale)
        let scale = vectorcraft_doc::scale::ScaleConfig::new(20.0);
        let ppm = scale.scale.points_per_meter();
        let pts: Vec<[f64; 2]> = vectorcraft_pathops::plan::HOUSE_1_POLYGON_METERS
            .iter()
            .map(|&(x, y)| [x * ppm, -y * ppm])
            .collect();
        let res = s.execute("trowel.polygonStats", &json!({
            "points": pts
        })).expect("polygon stats");
        let area_m2 = res["area_m2"].as_f64().unwrap();
        assert!((area_m2 - 32.91).abs() < 0.01);
    }
}

