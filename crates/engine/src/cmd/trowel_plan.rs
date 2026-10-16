//! Plan drafting and feature measurement commands (Trowel design 2.1 §3, §4-1, §7, §8, §13, §14, §15, J009):
//! - `trowel.sectionLine`: create 1-dot chain section cut line with directional arrows and labels (Shift+A).
//! - `trowel.spotElevation`: create spot elevation marker symbol and formatted readout (Shift+Z).
//! - `trowel.depthMarker`: create posthole concentric circles or trench depth indicator.
//! - `trowel.slopeHachures`: generate alternating slope hachure ticks along embankment/pit edges.
//! - `trowel.polygonStats`: closed polygon statistical readout (exact area in ㎡, perimeter, MBB axes, PCA azimuth).

use kurbo::{BezPath, Point, Shape as _};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::appearance::{AppearanceItem, Dash, StrokeLayer};
use vectorcraft_doc::{Appearance, CharStyle, Justify, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::PathData;
use vectorcraft_pathops::plan::{
    DEFAULT_HACHURE_SHORT_RATIO, DEFAULT_HACHURE_SPACING_MM, DEFAULT_PLAN_WEIGHT_PT, DepthMarkerKind, DepthMarkerSpec, SectionLineSpec,
    SlopeHachureSpec, SpotElevationSpec, SpotMarkerKind, ViewDirection, compute_polygon_stats, generate_depth_marker_geometry,
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
        cmd!(
            "trowel.insertScaleBar",
            "Insert Archaeological Scale Bar",
            ["Trowel", "도면"],
            None,
            "{at?: [x,y]|{x,y}, scale?: u32, style?: 'report'|'parallel', lengthM?: f64} insert a 1-click vector scale bar (1/10, 1/20, 1/30, 1/40, 1/50, 1/60, 1/80, 1/100, 1/200) in report style (left 5 subdivisions + right solid bar) or parallel line style",
            has_doc,
            insert_scale_bar
        ),
        cmd!(
            "trowel.insertSectionMarker",
            "Insert Section Marker or Datum Tick",
            ["Trowel", "도면"],
            None,
            "{at?: [x,y]|{x,y}, label?: string, orientation?: 'horizontal'|'vertical', spanPt?: f64, kind?: 'cut'|'datum', elevationM?: f64} insert section cut markers (A-A', B-B', a-a', b-b' with red reference bar and sightline check-arrow) or section datum tick (┣ 93.0m)",
            has_doc,
            insert_section_marker
        ),
        cmd!(
            "trowel.insertNorthArrow",
            "Insert Archaeological North Arrow",
            ["Trowel", "도면"],
            None,
            "{at?: [x,y]|{x,y}, rotationDeg?: f64, sizePt?: f64} insert an archaeological north arrow (half-filled needle + crossbar + N) at a given rotation angle",
            has_doc,
            insert_north_arrow
        ),
        cmd!(
            "trowel.insertFieldStamp",
            "Insert Field Text/Symbol Stamp",
            ["Trowel", "도면"],
            None,
            "{at?: [x,y]|{x,y}, kind?: 'disturbance'|'excavation_limit'|'grid_labels'|'spec_note', text?: string, fontSizePt?: f64} insert standard 6pt/7pt field stamps (교란, 조사경계, 격자 A1~B2, 현장 규격 주기)",
            has_doc,
            insert_field_stamp
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

    let p0 = p
        .get("p0")
        .or_else(|| p.get("start"))
        .or_else(|| p.get("p1"))
        .and_then(parse_point)
        .ok_or_else(|| bad(C, "missing or invalid `p0` or `p1`"))?;
    let p1 = if p.get("p0").is_some() || p.get("start").is_some() { p.get("p1").or_else(|| p.get("end")) } else { p.get("p2") }
        .and_then(parse_point)
        .ok_or_else(|| bad(C, "missing or invalid `p1` or `p2`"))?;

    let label_start = p.get("labelStart").and_then(|v| v.as_str()).unwrap_or("A").to_string();
    let label_end = p.get("labelEnd").and_then(|v| v.as_str()).unwrap_or("A′").to_string();

    let view_direction = match p.get("viewDirection").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("right") => ViewDirection::Right,
        _ => ViewDirection::Left,
    };

    let weight_pt = p.get("weight").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_PLAN_WEIGHT_PT);
    let arrow_length_pt = p.get("arrowLength").and_then(|v| v.as_f64()).unwrap_or(14.0);

    let spec =
        SectionLineSpec { p0, p1, label_start, label_end, view_direction, weight_pt, arrow_length_pt, dash_pattern: vec![12.0, 3.0, 2.0, 3.0] };

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
    let elevation = p
        .get("elevation")
        .or_else(|| p.get("elevation_m"))
        .or_else(|| p.get("elevationM"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| bad(C, "missing `elevation` value"))?;

    let marker = match p.get("marker").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("triangle") => SpotMarkerKind::Triangle,
        Some("crosshair") => SpotMarkerKind::Crosshair,
        _ => SpotMarkerKind::Dot,
    };

    let font_size_pt = p.get("fontSize").and_then(|v| v.as_f64()).unwrap_or(7.0);
    let halo = p.get("halo").and_then(|v| v.as_bool()).unwrap_or(true);
    let prefix_plus = p.get("prefixPlus").and_then(|v| v.as_bool()).unwrap_or(true);
    let suffix_m = p.get("suffixM").and_then(|v| v.as_bool()).unwrap_or(true);

    let spec = SpotElevationSpec { at, elevation_m: elevation, marker, font_size_pt, halo, prefix_plus, suffix_m };

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

    let spec = DepthMarkerSpec { at, depth_val: depth, is_cm, symbol, font_size_pt: font_size, outer_radius_pt: radius };

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

    let spec = SlopeHachureSpec { spacing_mm, short_ratio, weight_pt: DEFAULT_PLAN_WEIGHT_PT, default_width_pt: width, range };

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

fn format_meters(m: f64) -> String {
    if (m - m.round()).abs() < 1e-4 {
        format!("{}", m.round() as i64)
    } else if (m * 10.0 - (m * 10.0).round()).abs() < 1e-4 {
        format!("{:.1}", m)
    } else {
        format!("{:.2}", m)
    }
}

fn insert_scale_bar(s: &mut Session, p: &Value) -> Result<Value> {
    let default_scale = s.doc()?.doc.scale.as_ref().map(|cfg| cfg.scale.denom.round().max(1.0) as u32).unwrap_or(40);
    let scale = p.get("scale").and_then(Value::as_u64).map(|u| (u as u32).max(1)).unwrap_or(default_scale);
    let style = p.get("style").and_then(Value::as_str).unwrap_or("report");
    let default_total_m = match scale {
        1..=15 => 0.5,
        16..=45 => 1.0,
        46..=85 => 2.0,
        86..=150 => 4.0,
        _ => 10.0,
    };
    let total_m = p.get("lengthM").and_then(Value::as_f64).filter(|v| v.is_finite() && *v > 0.0).unwrap_or(default_total_m);
    let origin = p.get("at").and_then(parse_point).unwrap_or(Point::new(360.0, 500.0));

    // Paper width in points: (total_m * 1000 mm / scale) * (72 / 25.4)
    const PT_PER_MM: f64 = 72.0 / 25.4;
    let bar_mm = (total_m * 1000.0) / (scale as f64);
    let bar_w = (bar_mm * PT_PER_MM).clamp(36.0, 320.0);
    let bar_h = 3.6;

    let target_parent = s.doc()?.target_parent()?;
    let black = Color::rgb8(20, 20, 19);

    s.edit("Insert Archaeological Scale Bar", |doc, sel| {
        let mut children = Vec::new();
        let half_w = bar_w * 0.5;
        let sub_w = half_w / 5.0;

        if style == "parallel" {
            // Style B (PDF 1 parallel lines with 5 left subdivisions + (1/scale) on right)
            let mut bez = BezPath::new();
            // Top & bottom parallel lines
            bez.move_to(origin);
            bez.line_to(Point::new(origin.x + bar_w, origin.y));
            bez.move_to(Point::new(origin.x, origin.y + bar_h));
            bez.line_to(Point::new(origin.x + bar_w, origin.y + bar_h));
            // End & midpoint vertical ticks
            for &dx in &[0.0, half_w, bar_w] {
                bez.move_to(Point::new(origin.x + dx, origin.y - 2.0));
                bez.line_to(Point::new(origin.x + dx, origin.y + bar_h));
            }
            // 5 left subdivisions
            for i in 1..5 {
                let dx = (i as f64) * sub_w;
                bez.move_to(Point::new(origin.x + dx, origin.y));
                bez.line_to(Point::new(origin.x + dx, origin.y + bar_h));
            }
            let frame_id = doc.alloc_id();
            let mut frame_node = Node::path(frame_id, PathData::from_bezpath(&bez), Appearance::basic(Paint::None, Paint::solid(black), 0.3));
            frame_node.name = Some("평행 눈금선".to_string());
            children.push(Arc::new(frame_node));

            let lbl_id = doc.alloc_id();
            let cs = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let lbl_text = format!("(1/{scale})");
            let lbl_obj = TextObject::point(Point::new(origin.x + bar_w + 4.0, origin.y + bar_h), &lbl_text, cs);
            let mut lbl_node = Node::new(lbl_id, NodeKind::Text(Box::new(lbl_obj)));
            lbl_node.name = Some(format!("축척 (1/{scale})"));
            children.push(Arc::new(lbl_node));
        } else {
            // Style A (PDF 2 report style: left half 5 subdivisions, right half solid black bar,
            // with 0 / half / total_m (1/scale) labels at 6pt above)
            let mut frame_bez = BezPath::new();
            let rect = kurbo::Rect::from_origin_size(origin, kurbo::Size::new(bar_w, bar_h));
            frame_bez.extend(rect.to_path(0.1));
            for &dx in &[0.0, half_w, bar_w] {
                frame_bez.move_to(Point::new(origin.x + dx, origin.y - 2.4));
                frame_bez.line_to(Point::new(origin.x + dx, origin.y + bar_h));
            }
            for i in 1..5 {
                let dx = (i as f64) * sub_w;
                frame_bez.move_to(Point::new(origin.x + dx, origin.y - 1.2));
                frame_bez.line_to(Point::new(origin.x + dx, origin.y + bar_h));
            }
            let frame_id = doc.alloc_id();
            let mut frame_node =
                Node::path(frame_id, PathData::from_bezpath(&frame_bez), Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(black), 0.3));
            frame_node.name = Some("축척 외곽선".to_string());
            children.push(Arc::new(frame_node));

            // Right half solid black bar + alternating left sub-blocks
            let mut fill_bez = BezPath::new();
            let right_rect = kurbo::Rect::from_origin_size(Point::new(origin.x + half_w, origin.y), kurbo::Size::new(half_w, bar_h));
            fill_bez.extend(right_rect.to_path(0.1));
            for i in (0..5).step_by(2) {
                let sub_rect = kurbo::Rect::from_origin_size(Point::new(origin.x + (i as f64) * sub_w, origin.y), kurbo::Size::new(sub_w, bar_h));
                fill_bez.extend(sub_rect.to_path(0.1));
            }
            let fill_id = doc.alloc_id();
            let mut fill_node = Node::path(fill_id, PathData::from_bezpath(&fill_bez), Appearance::basic(Paint::solid(black), Paint::None, 0.0));
            fill_node.name = Some("축척 채움".to_string());
            children.push(Arc::new(fill_node));

            // 6pt labels above: "0", "{half}", "{total}m (1/{scale})"
            let half_str = format_meters(total_m * 0.5);
            let end_str = format!("{}m (1/{scale})", format_meters(total_m));
            for (dx, txt, justify) in [(0.0, "0".to_string(), Justify::Center), (half_w, half_str, Justify::Center), (bar_w, end_str, Justify::Left)]
            {
                let tid = doc.alloc_id();
                let cs = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
                let tx = if justify == Justify::Left { origin.x + dx - 3.0 } else { origin.x + dx };
                let mut tobj = TextObject::point(Point::new(tx, origin.y - 4.0), &txt, cs);
                tobj.para.justify = justify;
                let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
                tnode.name = Some(format!("눈금 {txt}"));
                children.push(Arc::new(tnode));
            }
        }

        let gid = doc.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children, clip: false });
        group.name = Some(format!("축척 막대 (1/{scale})"));
        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("trowel.insertScaleBar", e.to_string()))?;
        sel.objects = vec![gid];

        Ok(json!({
            "id": gid.0,
            "scale": scale,
            "style": style,
            "totalM": total_m,
            "barWidthPt": bar_w,
        }))
    })
}

fn insert_section_marker(s: &mut Session, p: &Value) -> Result<Value> {
    let origin = p.get("at").and_then(parse_point).unwrap_or(Point::new(200.0, 220.0));
    let kind = p.get("kind").and_then(Value::as_str).unwrap_or("cut");
    let orientation = p.get("orientation").and_then(Value::as_str).unwrap_or("horizontal");
    let label_base = p.get("label").and_then(Value::as_str).unwrap_or("A").trim();
    let label_start = if label_base.is_empty() { "A".to_string() } else { label_base.to_string() };
    let label_end = format!("{label_start}′");
    let span = p.get("spanPt").and_then(Value::as_f64).filter(|v| v.is_finite() && *v > 10.0).unwrap_or(180.0);
    let elev_m = p.get("elevationM").and_then(Value::as_f64).filter(|v| v.is_finite()).unwrap_or(93.0);

    let target_parent = s.doc()?.target_parent()?;
    let red_ref = Color::rgb8(210, 35, 90);
    let black = Color::rgb8(20, 20, 19);

    s.edit("Insert Section Marker", |doc, sel| {
        let mut children = Vec::new();

        if kind == "datum" {
            // Korean field standard Section Elevation Benchmark:
            // Black reference line (14pt, 0.2pt) + Filled Red Bar ┃ (1.6 x 6.5 pt) + 6pt elevation text ("93.0m")
            let mut line_bez = BezPath::new();
            let (bar_rect, text_pos) = if orientation == "vertical" {
                line_bez.move_to(origin);
                line_bez.line_to(Point::new(origin.x, origin.y + 14.0));
                (kurbo::Rect::from_center_size(origin, kurbo::Size::new(6.5, 1.6)), Point::new(origin.x + 5.5, origin.y + 2.2))
            } else {
                line_bez.move_to(Point::new(origin.x - 14.0, origin.y));
                line_bez.line_to(origin);
                (kurbo::Rect::from_center_size(origin, kurbo::Size::new(1.6, 6.5)), Point::new(origin.x + 4.5, origin.y + 2.2))
            };

            let ref_id = doc.alloc_id();
            let mut ref_node = Node::path(ref_id, PathData::from_bezpath(&line_bez), Appearance::basic(Paint::None, Paint::solid(black), 0.2));
            ref_node.name = Some("해발 보조선".to_string());
            children.push(Arc::new(ref_node));

            let mut bar_bez = BezPath::new();
            bar_bez.extend(bar_rect.to_path(0.1));
            let tid = doc.alloc_id();
            let mut tnode = Node::path(tid, PathData::from_bezpath(&bar_bez), Appearance::basic(Paint::solid(red_ref), Paint::None, 0.0));
            tnode.name = Some("해발 기준 눈금".to_string());
            children.push(Arc::new(tnode));

            let lbl_str = format!("{elev_m:.1}m");
            let lid = doc.alloc_id();
            let cs = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let mut lobj = TextObject::point(text_pos, &lbl_str, cs);
            lobj.para.justify = Justify::Left;
            let mut lnode = Node::new(lid, NodeKind::Text(Box::new(lobj)));
            lnode.name = Some(format!("해발 {lbl_str}"));
            children.push(Arc::new(lnode));

            let gid = doc.alloc_id();
            let mut group = Node::new(gid, NodeKind::Group { children, clip: false });
            group.name = Some(format!("단면 해발 눈금 ({lbl_str})"));
            doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("trowel.insertSectionMarker", e.to_string()))?;
            sel.objects = vec![gid];
            return Ok(json!({ "id": gid.0, "kind": "datum", "elevationM": elev_m }));
        }

        // Section Cut Markers (A-A', B-B', a-a', b-b') with Red Reference Block ■ + Collinear Sightline Check-Arrow:
        // Horizontal: Left is `∠───■` (with `A` above shaft), Right is mirror-symmetric `■───⦣` (with `A′` above shaft).
        // Vertical: Top is vertical red block `■` with shaft extending up and barb folding right, Bottom is mirror-symmetric.
        let mut red_bars = BezPath::new();
        let mut arrows = BezPath::new();

        if orientation == "vertical" {
            // Top endpoint (e.g. "B") at origin: red block extends inward (downward), shaft extends upward
            let pt0 = origin;
            let bar0 = kurbo::Rect::new(pt0.x - 1.4, pt0.y, pt0.x + 1.4, pt0.y + 8.5);
            red_bars.extend(bar0.to_path(0.1));
            arrows.move_to(Point::new(pt0.x - 1.4, pt0.y));
            arrows.line_to(Point::new(pt0.x - 1.4, pt0.y - 16.0));
            arrows.line_to(Point::new(pt0.x + 3.8, pt0.y - 10.5));

            let lid0 = doc.alloc_id();
            let cs0 = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let mut lobj0 = TextObject::point(Point::new(pt0.x + 3.5, pt0.y - 2.5), &label_start, cs0);
            lobj0.para.justify = Justify::Left;
            let mut lnode0 = Node::new(lid0, NodeKind::Text(Box::new(lobj0)));
            lnode0.name = Some(format!("기호 {label_start}"));
            children.push(Arc::new(lnode0));

            // Bottom endpoint (e.g. "B′") at origin.y + span: red block extends inward (upward), shaft extends downward
            let pt1 = Point::new(origin.x, origin.y + span);
            let bar1 = kurbo::Rect::new(pt1.x - 1.4, pt1.y - 8.5, pt1.x + 1.4, pt1.y);
            red_bars.extend(bar1.to_path(0.1));
            arrows.move_to(Point::new(pt1.x - 1.4, pt1.y));
            arrows.line_to(Point::new(pt1.x - 1.4, pt1.y + 16.0));
            arrows.line_to(Point::new(pt1.x + 3.8, pt1.y + 10.5));

            let lid1 = doc.alloc_id();
            let cs1 = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let mut lobj1 = TextObject::point(Point::new(pt1.x + 3.5, pt1.y + 7.5), &label_end, cs1);
            lobj1.para.justify = Justify::Left;
            let mut lnode1 = Node::new(lid1, NodeKind::Text(Box::new(lobj1)));
            lnode1.name = Some(format!("기호 {label_end}"));
            children.push(Arc::new(lnode1));
        } else {
            // Left endpoint (e.g. "A") at origin: `∠───■`
            let pt0 = origin;
            let bar0 = kurbo::Rect::new(pt0.x, pt0.y - 1.4, pt0.x + 8.5, pt0.y + 1.4);
            red_bars.extend(bar0.to_path(0.1));
            arrows.move_to(Point::new(pt0.x, pt0.y + 1.4));
            arrows.line_to(Point::new(pt0.x - 16.0, pt0.y + 1.4));
            arrows.line_to(Point::new(pt0.x - 10.5, pt0.y - 3.8));

            let lid0 = doc.alloc_id();
            let cs0 = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let mut lobj0 = TextObject::point(Point::new(pt0.x - 6.0, pt0.y - 1.5), &label_start, cs0);
            lobj0.para.justify = Justify::Center;
            let mut lnode0 = Node::new(lid0, NodeKind::Text(Box::new(lobj0)));
            lnode0.name = Some(format!("기호 {label_start}"));
            children.push(Arc::new(lnode0));

            // Right endpoint (e.g. "A′") at origin.x + span: mirror-symmetric `■───⦣`
            let pt1 = Point::new(origin.x + span, origin.y);
            let bar1 = kurbo::Rect::new(pt1.x - 8.5, pt1.y - 1.4, pt1.x, pt1.y + 1.4);
            red_bars.extend(bar1.to_path(0.1));
            arrows.move_to(Point::new(pt1.x, pt1.y + 1.4));
            arrows.line_to(Point::new(pt1.x + 16.0, pt1.y + 1.4));
            arrows.line_to(Point::new(pt1.x + 10.5, pt1.y - 3.8));

            let lid1 = doc.alloc_id();
            let cs1 = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
            let mut lobj1 = TextObject::point(Point::new(pt1.x + 6.0, pt1.y - 1.5), &label_end, cs1);
            lobj1.para.justify = Justify::Center;
            let mut lnode1 = Node::new(lid1, NodeKind::Text(Box::new(lobj1)));
            lnode1.name = Some(format!("기호 {label_end}"));
            children.push(Arc::new(lnode1));
        }

        let bar_id = doc.alloc_id();
        let mut bar_node = Node::path(bar_id, PathData::from_bezpath(&red_bars), Appearance::basic(Paint::solid(red_ref), Paint::None, 0.0));
        bar_node.name = Some("단면 붉은 기준점".to_string());
        children.push(Arc::new(bar_node));

        let arr_id = doc.alloc_id();
        let mut arr_node = Node::path(arr_id, PathData::from_bezpath(&arrows), Appearance::basic(Paint::None, Paint::solid(black), 0.35));
        arr_node.name = Some("시선 화살표".to_string());
        children.push(Arc::new(arr_node));

        let gid = doc.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children, clip: false });
        group.name = Some(format!("단면 기호 {label_start}–{label_end}"));
        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("trowel.insertSectionMarker", e.to_string()))?;
        sel.objects = vec![gid];

        Ok(json!({
            "id": gid.0,
            "label": format!("{label_start}–{label_end}"),
            "orientation": orientation,
        }))
    })
}

fn insert_north_arrow(s: &mut Session, p: &Value) -> Result<Value> {
    let center = p.get("at").and_then(parse_point).unwrap_or(Point::new(420.0, 140.0));
    let rot_deg = p.get("rotationDeg").and_then(Value::as_f64).filter(|v| v.is_finite()).unwrap_or(0.0);
    let h = p.get("sizePt").and_then(Value::as_f64).filter(|v| v.is_finite() && *v > 8.0).unwrap_or(36.0);

    let rad = rot_deg.to_radians();
    let (sin_a, cos_a) = rad.sin_cos();
    let rot = |dx: f64, dy: f64| -> Point { Point::new(center.x + dx * cos_a - dy * sin_a, center.y + dx * sin_a + dy * cos_a) };

    let target_parent = s.doc()?.target_parent()?;
    let black = Color::rgb8(20, 20, 19);

    s.edit("Insert Archaeological North Arrow", |doc, sel| {
        let mut children = Vec::new();
        let tip = rot(0.0, -h * 0.5);
        let base_mid = rot(0.0, -h * 0.05);
        let wing_l = rot(-h * 0.14, h * 0.04);
        let wing_r = rot(h * 0.14, h * 0.04);
        let shaft_bot = rot(0.0, h * 0.5);
        let cross_l = rot(-h * 0.16, h * 0.22);
        let cross_r = rot(h * 0.16, h * 0.22);

        // Left open wing
        let mut left_bez = BezPath::new();
        left_bez.move_to(tip);
        left_bez.line_to(wing_l);
        left_bez.line_to(base_mid);
        left_bez.close_path();
        let lid = doc.alloc_id();
        let mut lnode = Node::path(lid, PathData::from_bezpath(&left_bez), Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(black), 0.3));
        lnode.name = Some("방위표 왼쪽 날개".to_string());
        children.push(Arc::new(lnode));

        // Right filled black wing
        let mut right_bez = BezPath::new();
        right_bez.move_to(tip);
        right_bez.line_to(wing_r);
        right_bez.line_to(base_mid);
        right_bez.close_path();
        let rid = doc.alloc_id();
        let mut rnode = Node::path(rid, PathData::from_bezpath(&right_bez), Appearance::basic(Paint::solid(black), Paint::solid(black), 0.3));
        rnode.name = Some("방위표 오른쪽 날개".to_string());
        children.push(Arc::new(rnode));

        // Central shaft + crossbar
        let mut shaft_bez = BezPath::new();
        shaft_bez.move_to(tip);
        shaft_bez.line_to(shaft_bot);
        shaft_bez.move_to(cross_l);
        shaft_bez.line_to(cross_r);
        let sid = doc.alloc_id();
        let mut snode = Node::path(sid, PathData::from_bezpath(&shaft_bez), Appearance::basic(Paint::None, Paint::solid(black), 0.35));
        snode.name = Some("방위표 축".to_string());
        children.push(Arc::new(snode));

        // "N" label above tip
        let n_pos = rot(0.0, -h * 0.64);
        let nid = doc.alloc_id();
        let cs = CharStyle { size: 7.0, fill: Paint::solid(black), ..Default::default() };
        let mut nobj = TextObject::point(Point::new(n_pos.x, n_pos.y + 2.5), "N", cs);
        nobj.para.justify = Justify::Center;
        let mut nnode = Node::new(nid, NodeKind::Text(Box::new(nobj)));
        nnode.name = Some("N".to_string());
        children.push(Arc::new(nnode));

        let gid = doc.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children, clip: false });
        group.name = Some(format!("방위표 (N {rot_deg:.0}°)"));
        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("trowel.insertNorthArrow", e.to_string()))?;
        sel.objects = vec![gid];

        Ok(json!({
            "id": gid.0,
            "rotationDeg": rot_deg,
        }))
    })
}

fn insert_field_stamp(s: &mut Session, p: &Value) -> Result<Value> {
    let origin = p.get("at").and_then(parse_point).unwrap_or(Point::new(240.0, 260.0));
    let kind = p.get("kind").and_then(Value::as_str).unwrap_or("disturbance");
    let custom_text = p.get("text").and_then(Value::as_str);
    let font_size = p.get("fontSizePt").and_then(Value::as_f64).filter(|v| v.is_finite() && *v > 1.0).unwrap_or(6.0);

    let target_parent = s.doc()?.target_parent()?;
    let black = Color::rgb8(20, 20, 19);

    s.edit("Insert Field Stamp", |doc, sel| {
        let mut children = Vec::new();
        let group_title = match kind {
            "excavation_limit" => {
                let mut bez = BezPath::new();
                bez.move_to(Point::new(origin.x - 24.0, origin.y + 4.0));
                bez.line_to(Point::new(origin.x + 24.0, origin.y + 4.0));
                let mut stroke = StrokeLayer::new(Paint::solid(black), 0.4);
                stroke.dash = Some(Dash { pattern: vec![4.0, 2.0], offset: 0.0, align_corners: false });
                let app = Appearance { items: vec![AppearanceItem::Stroke(stroke)], ..Default::default() };
                let lid = doc.alloc_id();
                let mut lnode = Node::path(lid, PathData::from_bezpath(&bez), app);
                lnode.name = Some("조사경계 선".to_string());
                children.push(Arc::new(lnode));

                let tid = doc.alloc_id();
                let cs = CharStyle { size: font_size, fill: Paint::solid(black), ..Default::default() };
                let mut tobj = TextObject::point(origin, custom_text.unwrap_or("조사경계"), cs);
                tobj.para.justify = Justify::Center;
                let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
                tnode.name = Some("조사경계".to_string());
                children.push(Arc::new(tnode));
                "조사경계 표기".to_string()
            }
            "grid_labels" => {
                let labels = ["A1", "A2", "A3", "B1", "B2", "B3", "C1", "C2", "C3"];
                for (idx, lbl) in labels.iter().enumerate() {
                    let col = (idx % 3) as f64;
                    let row = (idx / 3) as f64;
                    let tid = doc.alloc_id();
                    let cs = CharStyle { size: font_size, fill: Paint::solid(black), ..Default::default() };
                    let tobj = TextObject::point(Point::new(origin.x + col * 28.0, origin.y + row * 16.0), lbl, cs);
                    let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
                    tnode.name = Some((*lbl).to_string());
                    children.push(Arc::new(tnode));
                }
                "격자 기호 (A1~C4)".to_string()
            }
            "spec_note" => {
                let lines = [
                    "글자체: 윤고딕, 글자포인트: 6p, 토층설명: 7p",
                    "평면외곽선 0.6pt / 내선(바닥) 0.3pt / 이중굴광 가운데선 0.4pt(파선)",
                    "석곽-평면외곽선 0.5pt / 석곽-내선 0.2pt / 유물외곽 0.3pt / 내선 0.2pt / 깨진면 0.15pt",
                    "단면선 0.8pt / 단면빗금 0.2pt / 해발선 0.3pt (0.5단위 0.5pt)",
                ];
                for (i, line) in lines.iter().enumerate() {
                    let tid = doc.alloc_id();
                    let cs = CharStyle { size: 6.0, fill: Paint::solid(black), ..Default::default() };
                    let tobj = TextObject::point(Point::new(origin.x, origin.y + (i as f64) * 10.0), line, cs);
                    let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
                    tnode.name = Some(format!("규격 주기 {}", i + 1));
                    children.push(Arc::new(tnode));
                }
                "현장 도면 규격 주기".to_string()
            }
            _ => {
                let lbl = custom_text.unwrap_or("교란");
                let tid = doc.alloc_id();
                let cs = CharStyle { size: font_size, fill: Paint::solid(black), ..Default::default() };
                let mut tobj = TextObject::point(origin, lbl, cs);
                tobj.para.justify = Justify::Center;
                let mut tnode = Node::new(tid, NodeKind::Text(Box::new(tobj)));
                tnode.name = Some(lbl.to_string());
                children.push(Arc::new(tnode));
                format!("{lbl} 표기")
            }
        };

        let gid = doc.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children, clip: false });
        group.name = Some(group_title);
        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("trowel.insertFieldStamp", e.to_string()))?;
        sel.objects = vec![gid];

        Ok(json!({
            "id": gid.0,
            "kind": kind,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trowel_plan_commands() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).expect("new doc");

        // 1. Section line
        let res = s
            .execute(
                "trowel.sectionLine",
                &json!({
                    "p1": [10.0, 10.0],
                    "p2": [200.0, 10.0],
                    "label": "A"
                }),
            )
            .expect("section line");
        assert!(res.get("id").is_some());

        // 2. Spot elevation
        let res = s
            .execute(
                "trowel.spotElevation",
                &json!({
                    "at": [100.0, 100.0],
                    "elevation": 152.35
                }),
            )
            .expect("spot elevation");
        assert!(res.get("id").is_some());

        // 3. Depth marker
        let res = s
            .execute(
                "trowel.depthMarker",
                &json!({
                    "at": [150.0, 150.0],
                    "depthCm": 45.0
                }),
            )
            .expect("depth marker");
        assert!(res.get("id").is_some());

        // 4. Polygon stats with canonical House 1 points (at default 1:20 scale)
        let scale = vectorcraft_doc::scale::ScaleConfig::new(20.0);
        let ppm = scale.scale.points_per_meter();
        let pts: Vec<[f64; 2]> = vectorcraft_pathops::plan::HOUSE_1_POLYGON_METERS.iter().map(|&(x, y)| [x * ppm, -y * ppm]).collect();
        let res = s
            .execute(
                "trowel.polygonStats",
                &json!({
                    "points": pts
                }),
            )
            .expect("polygon stats");
        let area_m2 = res["area_m2"].as_f64().unwrap();
        assert!((area_m2 - 32.91).abs() < 0.01);
    }

    #[test]
    fn test_field_report_generators() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).expect("new doc");

        // 1. Scale bars (1/40 report bar & 1/200 parallel bar)
        let sb1 = s.execute("trowel.insertScaleBar", &json!({ "scale": 40, "style": "report" })).expect("scale bar 1/40");
        assert_eq!(sb1["scale"].as_u64(), Some(40));
        let sb2 = s.execute("trowel.insertScaleBar", &json!({ "scale": 200, "style": "parallel" })).expect("scale bar 1/200");
        assert_eq!(sb2["scale"].as_u64(), Some(200));

        // 2. Section markers (A-A', B-B', and ┣ 93.0m datum tick)
        let sm1 = s
            .execute("trowel.insertSectionMarker", &json!({ "label": "A", "orientation": "horizontal", "kind": "cut" }))
            .expect("section marker A-A'");
        assert_eq!(sm1["label"].as_str(), Some("A–A′"));
        let sm2 = s
            .execute("trowel.insertSectionMarker", &json!({ "kind": "datum", "orientation": "horizontal", "elevationM": 93.0 }))
            .expect("datum tick 93.0m");
        assert_eq!(sm2["kind"].as_str(), Some("datum"));

        // 3. Rotated North Arrow
        let na = s.execute("trowel.insertNorthArrow", &json!({ "rotationDeg": -30.0 })).expect("north arrow");
        assert_eq!(na["rotationDeg"].as_f64(), Some(-30.0));

        // 4. Field stamps (교란, 조사경계, 격자, 규격 주기)
        for kind in ["disturbance", "excavation_limit", "grid_labels", "spec_note"] {
            let st = s.execute("trowel.insertFieldStamp", &json!({ "kind": kind })).expect("field stamp");
            assert_eq!(st["kind"].as_str(), Some(kind));
        }

        // 5. Field standard 12 stroke presets (0.15..0.8 pt)
        let fp = s.execute("legend.addFieldPresets", &json!({})).expect("field presets");
        assert_eq!(fp["added"].as_array().map(Vec::len), Some(12));

        // 6. Roman numeral (I..XX) & 7pt inline stratigraphy block
        assert_eq!(vectorcraft_doc::stratigraphy::to_ascii_roman(1), "I");
        assert_eq!(vectorcraft_doc::stratigraphy::to_ascii_roman(19), "XIX");
        assert_eq!(vectorcraft_doc::stratigraphy::to_ascii_roman(20), "XX");
        assert_eq!(vectorcraft_doc::stratigraphy::to_ascii_roman(40), "XL");
        s.execute("stratigraphy.setStyle", &json!({ "style": "roman" })).expect("set roman style");
        let tbl = s.execute("stratigraphy.generateTable", &json!({ "format": "inline", "fontSizePt": 7.0 })).expect("inline 7pt table");
        assert!(tbl["nodeId"].as_u64().is_some());
    }
}
