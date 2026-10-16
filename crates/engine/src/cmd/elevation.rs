//! Automatic horizontal elevation lines commands (Trowel design 2.1 §4, §4-1, §14(b, c, d)).

use kurbo::{BezPath, Point, Shape as _};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::elevation::{ElevationLabelFormat, ElevationLabelPlacement, ElevationRangeMode, ElevationSpec};
use vectorcraft_doc::{Appearance, CharStyle, Document, Justify, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, SubPath};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "elevation.create",
            "Automatic Elevation Lines",
            ["Object", "해발선"],
            None,
            "{profileId?: u64, profileNodeId?: u64, obstacleIds?: [u64], obstacleNodeIds?: [u64], datumM?: f64, minorIntervalM?: f64, majorIntervalM?: f64, minorWeightPt?: f64, majorWeightPt?: f64, labelFormat?: string, labelPlacement?: string, labelFontSizePt?: f64, autoUpdate?: bool, minElevationM?: f64, maxElevationM?: f64} generate automatic horizontal elevation lines clipped to open void",
            has_doc,
            create
        ),
        cmd!(
            "elevation.update",
            "Update Elevation Lines",
            ["Object", "해발선 갱신"],
            None,
            "{id?: u64} recompute clipped elevation lines for target group or all in document",
            has_doc,
            update
        ),
        cmd!(
            "elevation.set",
            "Set Elevation Options",
            [],
            None,
            "{id?: u64, datumM?: f64, minorIntervalM?: f64, majorIntervalM?: f64, minorWeightPt?: f64, majorWeightPt?: f64, labelFormat?: string, labelPlacement?: string, labelFontSizePt?: f64, autoUpdate?: bool} update options of elevation group",
            has_doc,
            set
        ),
        cmd!(
            "elevation.expand",
            "Expand Elevation Lines",
            ["Object", "해발선 풀기"],
            None,
            "{id?: u64} ungroup live elevation group into discrete vector lines and labels",
            has_doc,
            expand
        ),
        cmd!(
            query "elevation.get",
            "Get Elevation Info",
            [],
            None,
            "{id?: u64} query elevation lines properties and generated line count",
            has_doc,
            get
        ),
    ]
}

/// Helper to extract BezPath geometry from a node.
pub(crate) fn get_bezpath_from_node(node: &Node) -> Option<BezPath> {
    match &node.kind {
        NodeKind::Path { path, .. } => Some(path.to_bezpath()),
        NodeKind::Compound { children, .. } => {
            let mut bp = BezPath::new();
            for c in children {
                if let Some(pd) = c.path_data() {
                    bp.extend(pd.to_bezpath());
                }
            }
            if bp.elements().is_empty() { None } else { Some(bp) }
        }
        NodeKind::Group { children, .. } => {
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

/// Parses an ElevationSpec from JSON params.
fn spec_from_params(cmd: &str, p: &Value, default_profile: Option<NodeId>) -> Result<ElevationSpec> {
    let profile_id = p
        .get("profileId")
        .or_else(|| p.get("profileNodeId"))
        .and_then(|v| v.as_u64())
        .map(NodeId)
        .or(default_profile)
        .ok_or_else(|| bad(cmd, "give profileId or select a profile path"))?;

    let obstacle_ids = p
        .get("obstacleIds")
        .or_else(|| p.get("obstacleNodeIds"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(NodeId)).collect())
        .unwrap_or_default();

    let datum_m = p.get("datumM").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let minor_interval_m = p.get("minorIntervalM").and_then(|v| v.as_f64()).unwrap_or(vectorcraft_doc::elevation::DEFAULT_MINOR_INTERVAL_M);
    let major_interval_m = p.get("majorIntervalM").and_then(|v| v.as_f64()).unwrap_or(vectorcraft_doc::elevation::DEFAULT_MAJOR_INTERVAL_M);
    let minor_weight_pt = p.get("minorWeightPt").and_then(|v| v.as_f64()).unwrap_or(vectorcraft_doc::elevation::DEFAULT_MINOR_WEIGHT_PT);
    let major_weight_pt = p.get("majorWeightPt").and_then(|v| v.as_f64()).unwrap_or(vectorcraft_doc::elevation::DEFAULT_MAJOR_WEIGHT_PT);
    let label_font_size_pt = p.get("labelFontSizePt").and_then(|v| v.as_f64()).unwrap_or(vectorcraft_doc::elevation::DEFAULT_LABEL_FONT_SIZE_PT);
    let auto_update = p.get("autoUpdate").and_then(|v| v.as_bool()).unwrap_or(true);

    let label_format = match p.get("labelFormat").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("elprefix") | Some("el") => ElevationLabelFormat::ElPrefix,
        Some("raw") => ElevationLabelFormat::Raw,
        _ => ElevationLabelFormat::MetersSuffix,
    };

    let label_placement = match p.get("labelPlacement").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("bothends") | Some("both") => ElevationLabelPlacement::BothEnds,
        Some("rightend") | Some("right") => ElevationLabelPlacement::RightEnd,
        _ => ElevationLabelPlacement::LeftEnd,
    };

    let fill_below_profile = p
        .get("fillBelowProfile")
        .and_then(|v| v.as_bool())
        .or_else(|| p.get("placement").and_then(|v| v.as_str()).map(|s| s.eq_ignore_ascii_case("below")))
        .unwrap_or(false);
    let red_datum_tick = p.get("redDatumTick").and_then(|v| v.as_bool()).unwrap_or(false);

    let range_mode = match (p.get("minElevationM").and_then(|v| v.as_f64()), p.get("maxElevationM").and_then(|v| v.as_f64())) {
        (Some(min_elevation_m), Some(max_elevation_m)) => ElevationRangeMode::Explicit { min_elevation_m, max_elevation_m },
        _ => ElevationRangeMode::Auto,
    };

    Ok(ElevationSpec {
        profile_id,
        obstacle_ids,
        datum_m,
        minor_interval_m,
        major_interval_m,
        minor_weight_pt,
        major_weight_pt,
        label_format,
        label_placement,
        label_font_size_pt,
        range_mode,
        auto_update,
        fill_below_profile,
        red_datum_tick,
    })
}

/// Generates child nodes for an elevation spec.
fn generate_children(doc: &mut Document, spec: &ElevationSpec) -> Result<(Vec<Arc<Node>>, usize, usize)> {
    let profile_node = doc.node(spec.profile_id).ok_or_else(|| bad("elevation", format!("profile node {} not found", spec.profile_id.0)))?;
    let profile_bp = get_bezpath_from_node(profile_node).ok_or_else(|| bad("elevation", "profile node has no path geometry"))?;

    let mut obstacle_bps = Vec::new();
    for &oid in &spec.obstacle_ids {
        if let Some(obs_node) = doc.node(oid)
            && let Some(bp) = get_bezpath_from_node(obs_node)
        {
            obstacle_bps.push(bp);
        }
    }

    let out = vectorcraft_pathops::generate_elevation(&profile_bp, &obstacle_bps, spec, doc.scale.as_ref());
    let mut children = Vec::with_capacity(out.segments.len() + out.labels.len() * 2);

    for seg in &out.segments {
        let cid = doc.alloc_id();
        let pd = PathData::single(SubPath::polyline(&[Point::new(seg.x0, seg.y), Point::new(seg.x1, seg.y)], false));
        let w = if seg.is_major { spec.major_weight_pt } else { spec.minor_weight_pt };
        let app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), w);
        let mut child = Node::path(cid, pd, app);
        child.name = Some(if seg.is_major { "주 해발선".to_string() } else { "해발선".to_string() });
        children.push(Arc::new(child));
    }

    let red_ref = Color::rgb8(210, 35, 90);
    for label in &out.labels {
        if spec.red_datum_tick {
            let tid = doc.alloc_id();
            let bar_r = kurbo::Rect::from_center_size(label.at, kurbo::Size::new(1.6, 6.5));
            let mut tick_bez = BezPath::new();
            tick_bez.extend(bar_r.to_path(0.1));
            let mut tnode = Node::path(tid, PathData::from_bezpath(&tick_bez), Appearance::basic(Paint::solid(red_ref), Paint::None, 0.0));
            tnode.name = Some("해발 붉은 눈금".to_string());
            children.push(Arc::new(tnode));
        }

        let cid = doc.alloc_id();
        let style = CharStyle { size: spec.label_font_size_pt, fill: Paint::solid(Color::rgb8(20, 20, 19)), ..Default::default() };
        let origin = if spec.red_datum_tick {
            if label.anchor_left {
                Point::new(label.at.x - 4.5, label.at.y + spec.label_font_size_pt * 0.35)
            } else {
                Point::new(label.at.x + 4.5, label.at.y + spec.label_font_size_pt * 0.35)
            }
        } else {
            Point::new(label.at.x + if label.anchor_left { 2.0 } else { -2.0 }, label.at.y - 2.0)
        };
        let mut t_obj = TextObject::point(origin, &label.text, style);
        if spec.red_datum_tick {
            t_obj.para.justify = if label.anchor_left { Justify::Right } else { Justify::Left };
        } else if !label.anchor_left {
            t_obj.para.justify = Justify::Right;
        }
        let mut child = Node::new(cid, NodeKind::Text(Box::new(t_obj)));
        child.name = Some("해발 숫자".to_string());
        children.push(Arc::new(child));
    }

    let seg_count = out.segments.len();
    let lbl_count = out.labels.len();
    Ok((children, seg_count, lbl_count))
}

/// Recomputes the children of an existing elevation group node.
pub(crate) fn recompute_elevation_children(doc: &mut Document, group_id: NodeId) -> Result<(usize, usize)> {
    let spec = match doc.node(group_id).and_then(|n| n.elevation.as_deref().cloned()) {
        Some(s) => s,
        None => return Err(bad("elevation", "node is not an elevation group")),
    };

    let (children, seg_count, lbl_count) = generate_children(doc, &spec)?;
    if let Some(NodeKind::Group { children: group_children, .. }) = doc.node_mut(group_id).map(|n| &mut n.kind) {
        *group_children = children;
    }
    Ok((seg_count, lbl_count))
}

fn create(s: &mut Session, p: &Value) -> Result<Value> {
    let explicit_profile = p.get("profileId").or_else(|| p.get("profileNodeId")).and_then(|v| v.as_u64()).map(NodeId);
    let target_parent = s.doc()?.target_parent()?;

    // Resolve or synthesize a continuous profile path when invoked from the UI without an explicit
    // profileId: the profile, or the ends of a datum baseline to draw for it. The baseline goes in
    // with the lines, in one undo step.
    let (default_profile, baseline) = if explicit_profile.is_some() {
        (explicit_profile, None)
    } else {
        let sel_ids = s.doc()?.selection.objects.clone();
        let mut best_sel: Option<(NodeId, kurbo::Rect)> = None;
        let mut combined_bb: Option<kurbo::Rect> = None;
        {
            let doc = &s.doc()?.doc;
            for &id in &sel_ids {
                if let Some(n) = doc.node(id)
                    && n.elevation.is_none()
                    && !matches!(
                        (&n.kind, n.name.as_deref()),
                        (NodeKind::Group { .. }, Some(nm))
                            if nm.contains("눈금")
                                || nm.contains("기호")
                                || nm.contains("단면")
                                || nm.contains("해발")
                                || nm.contains("방위")
                                || nm.contains("축척")
                                || nm.contains("토층")
                    )
                    && let Some(bp) = get_bezpath_from_node(n)
                {
                    let bb = bp.bounding_box();
                    let min_w = if matches!(n.kind, NodeKind::Group { .. }) { 25.0 } else { 1e-4 };
                    if bb.width() > min_w {
                        combined_bb = Some(match combined_bb {
                            Some(c) => c.union(bb),
                            None => bb,
                        });
                        if best_sel.is_none_or(|(_, b)| bb.width() > b.width()) {
                            best_sel = Some((id, bb));
                        }
                    }
                }
            }
        }

        if let (Some((best_id, best_bb)), Some(comb)) = (best_sel, combined_bb) {
            if sel_ids.len() > 1 && comb.width() > best_bb.width() * 1.4 {
                // Multiple scattered shapes selected: create a continuous horizontal datum baseline spanning their full width
                (None, Some([Point::new(comb.x0, comb.y1), Point::new(comb.x1, comb.y1)]))
            } else {
                (Some(best_id), None)
            }
        } else {
            // No path selected: look for top-level user paths on layers (ignoring generated groups)
            let mut best_doc: Option<(NodeId, kurbo::Rect)> = None;
            let mut all_user_bb: Option<kurbo::Rect> = None;
            {
                let doc = &s.doc()?.doc;
                for layer in &doc.layers {
                    if !layer.visible || layer.locked {
                        continue;
                    }
                    for child in layer.children().map_or(&[][..], |v| v.as_slice()) {
                        if child.elevation.is_some() {
                            continue;
                        }
                        if let NodeKind::Path { path, .. } = &child.kind {
                            let bb = path.to_bezpath().bounding_box();
                            if bb.width() > 10.0 {
                                all_user_bb = Some(match all_user_bb {
                                    Some(c) => c.union(bb),
                                    None => bb,
                                });
                                if best_doc.is_none_or(|(_, b)| bb.width() > b.width()) {
                                    best_doc = Some((child.id, bb));
                                }
                            }
                        }
                    }
                }
            }
            if let (Some((best_id, best_bb)), Some(comb)) = (best_doc, all_user_bb) {
                if comb.width() > best_bb.width() * 1.4 {
                    (None, Some([Point::new(comb.x0, comb.y1), Point::new(comb.x1, comb.y1)]))
                } else {
                    (Some(best_id), None)
                }
            } else if p.get("fillBelowProfile").is_some() || p.get("redDatumTick").is_some() {
                // 1-click UI button on a blank canvas: create a standard 220pt horizontal datum line at center
                let (cx, cy) =
                    p.get("at").and_then(|v| v.as_array()).and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?))).unwrap_or((300.0, 220.0));
                (None, Some([Point::new(cx - 110.0, cy), Point::new(cx + 110.0, cy)]))
            } else {
                (None, None)
            }
        }
    };

    s.edit("Automatic Elevation Lines", |doc, sel| {
        let mut profile = default_profile;
        if let Some(ends) = baseline {
            let pid = doc.alloc_id();
            let pd = PathData::single(SubPath::polyline(&ends, false));
            let mut pnode = Node::path(pid, pd, Appearance::basic(Paint::None, Paint::None, 0.0));
            pnode.name = Some("해발 기준선".to_string());
            doc.insert(target_parent, usize::MAX, pnode).map_err(|e| bad("elevation.create", e.to_string()))?;
            profile = Some(pid);
        }
        let spec = spec_from_params("elevation.create", p, profile)?;
        let (children, seg_count, lbl_count) = generate_children(doc, &spec)?;
        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some("해발선".to_string());
        group.elevation = Some(Box::new(spec));

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad("elevation.create", e.to_string()))?;
        sel.objects = vec![group_id];

        Ok(json!({
            "id": group_id.0,
            "segmentCount": seg_count,
            "labelCount": lbl_count,
        }))
    })
}

fn update(s: &mut Session, p: &Value) -> Result<Value> {
    let target_id = p.get("id").and_then(|v| v.as_u64()).map(NodeId);

    s.edit("Update Elevation Lines", |doc, _| {
        if let Some(id) = target_id {
            let (seg_count, lbl_count) = recompute_elevation_children(doc, id)?;
            Ok(json!({ "updated": 1, "segmentCount": seg_count, "labelCount": lbl_count }))
        } else {
            let mut elevation_groups = Vec::new();
            doc.walk(|n| {
                if n.elevation.is_some() {
                    elevation_groups.push(n.id);
                }
            });
            let mut count = 0;
            for id in elevation_groups {
                if recompute_elevation_children(doc, id).is_ok() {
                    count += 1;
                }
            }
            Ok(json!({ "updated": count }))
        }
    })
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p
        .get("id")
        .and_then(|v| v.as_u64())
        .map(NodeId)
        .or_else(|| s.doc().ok()?.selection.objects.first().copied())
        .ok_or_else(|| bad("elevation.set", "give id or select an elevation group"))?;

    s.edit("Set Elevation Options", |doc, _| {
        let mut spec =
            doc.node(id).and_then(|n| n.elevation.as_deref().cloned()).ok_or_else(|| bad("elevation.set", "node is not an elevation group"))?;

        if let Some(d) = p.get("datumM").and_then(|v| v.as_f64()) {
            spec.datum_m = d;
        }
        if let Some(m) = p.get("minorIntervalM").and_then(|v| v.as_f64()) {
            spec.minor_interval_m = m;
        }
        if let Some(m) = p.get("majorIntervalM").and_then(|v| v.as_f64()) {
            spec.major_interval_m = m;
        }
        if let Some(w) = p.get("minorWeightPt").and_then(|v| v.as_f64()) {
            spec.minor_weight_pt = w;
        }
        if let Some(w) = p.get("majorWeightPt").and_then(|v| v.as_f64()) {
            spec.major_weight_pt = w;
        }
        if let Some(sz) = p.get("labelFontSizePt").and_then(|v| v.as_f64()) {
            spec.label_font_size_pt = sz;
        }
        if let Some(u) = p.get("autoUpdate").and_then(|v| v.as_bool()) {
            spec.auto_update = u;
        }
        if let Some(fmt) = p.get("labelFormat").and_then(|v| v.as_str()).map(str::to_ascii_lowercase) {
            spec.label_format = match fmt.as_str() {
                "elprefix" | "el" => ElevationLabelFormat::ElPrefix,
                "raw" => ElevationLabelFormat::Raw,
                _ => ElevationLabelFormat::MetersSuffix,
            };
        }
        if let Some(pl) = p.get("labelPlacement").and_then(|v| v.as_str()).map(str::to_ascii_lowercase) {
            spec.label_placement = match pl.as_str() {
                "bothends" | "both" => ElevationLabelPlacement::BothEnds,
                _ => ElevationLabelPlacement::LeftEnd,
            };
        }

        if let Some(node) = doc.node_mut(id) {
            node.elevation = Some(Box::new(spec));
        }

        let (seg_count, lbl_count) = recompute_elevation_children(doc, id)?;
        Ok(json!({ "id": id.0, "success": true, "segmentCount": seg_count, "labelCount": lbl_count }))
    })
}

fn expand(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p
        .get("id")
        .and_then(|v| v.as_u64())
        .map(NodeId)
        .or_else(|| s.doc().ok()?.selection.objects.first().copied())
        .ok_or_else(|| bad("elevation.expand", "give id or select an elevation group"))?;

    s.edit("Expand Elevation Lines", |doc, _| {
        let node = doc.node_mut(id).ok_or_else(|| bad("elevation.expand", "node not found"))?;
        if node.elevation.is_none() {
            return Err(bad("elevation.expand", "node is not an elevation group"));
        }
        node.elevation = None;
        Ok(json!({ "id": id.0, "expanded": true }))
    })
}

fn get(s: &mut Session, p: &Value) -> Result<Value> {
    let id = p
        .get("id")
        .and_then(|v| v.as_u64())
        .map(NodeId)
        .or_else(|| s.doc().ok()?.selection.objects.first().copied())
        .ok_or_else(|| bad("elevation.get", "give id or select an elevation group"))?;

    let doc = &s.doc()?.doc;
    let node = doc.node(id).ok_or_else(|| bad("elevation.get", "node not found"))?;
    let spec = node.elevation.as_deref().ok_or_else(|| bad("elevation.get", "node is not an elevation group"))?;

    let child_count = if let NodeKind::Group { children, .. } = &node.kind { children.len() } else { 0 };

    Ok(json!({
        "id": id.0,
        "profileId": spec.profile_id.0,
        "obstacleIds": spec.obstacle_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
        "datumM": spec.datum_m,
        "minorIntervalM": spec.minor_interval_m,
        "majorIntervalM": spec.major_interval_m,
        "minorWeightPt": spec.minor_weight_pt,
        "majorWeightPt": spec.major_weight_pt,
        "autoUpdate": spec.auto_update,
        "childrenCount": child_count,
    }))
}

/// Dynamic update hook called from `Session::edit` to recompute elevation lines when profile
/// or obstacle paths change.
pub(crate) fn refresh(before: &Document, doc: &mut Document) {
    let mut targets = Vec::new();
    doc.walk(|n| {
        if let Some(spec) = &n.elevation
            && spec.auto_update
        {
            targets.push((n.id, spec.profile_id, spec.obstacle_ids.clone()));
        }
    });

    for (gid, pid, oids) in targets {
        let profile_changed = before.node(pid) != doc.node(pid);
        let mut obstacles_changed = false;
        for oid in oids {
            if before.node(oid) != doc.node(oid) {
                obstacles_changed = true;
                break;
            }
        }
        if profile_changed || obstacles_changed {
            let _ = recompute_elevation_children(doc, gid);
        }
    }
}
