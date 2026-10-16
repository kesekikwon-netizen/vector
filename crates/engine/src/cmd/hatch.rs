//! Hatching commands (Trowel Archaeological CAD, design 2.1 §4, §7, §14):
//! - `hatch.set`: set hatch direction (45, 135, alt, none), paper spacing (mm), weight (pt), flip state.
//! - `hatch.cutFace` / `hatch.cut_face`: one-click "잘린 면" (0.8 pt border + 1.0mm/0.1pt Alternate hatch).
//! - `hatch.flip`: toggle 45° ↔ 135° flip on selected hatched faces.
//! - `hatch.get`: query hatch settings and odd-cycle conflict detection on selection.

use serde_json::{Value, json};
use vectorcraft_doc::hatch::{DEFAULT_CUT_BORDER_PT, DEFAULT_HATCH_SPACING_MM, DEFAULT_HATCH_WEIGHT_PT, HatchDirection, HatchStyle};
use vectorcraft_doc::{AppearanceItem, NodeId, NodeKind, StrokeLayer};
use vectorcraft_pathops::hatch::resolve_document_hatching;

use super::overprint::edit_counted;
use super::*;
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "hatch.set",
            "Set Hatch",
            [],
            None,
            "{direction: \"0\"|\"45\"|\"90\"|\"135\"|\"cross\"|\"diagonal_cross\"|\"alt\"|\"none\", spacing_mm?: f64, weight_pt?: f64, flipped?: bool, ids?: [u64]} set hatching on target paths",
            has_doc,
            set
        ),
        cmd!(
            "hatch.cutFace",
            "Cut Face",
            [],
            None,
            "{ids?: [u64]} one-click Cut Face (0.8 pt border + 1.0mm/0.1pt Alternate hatch)",
            has_doc,
            cut_face
        ),
        cmd!("hatch.cut_face", "Cut Face", [], None, "{ids?: [u64]} alias for hatch.cutFace", has_doc, cut_face),
        cmd!(
            "hatch.flip",
            "Flip Hatch Direction",
            [],
            None,
            "{ids?: [u64]} flip hatch direction (45° ↔ 135°) on target hatched paths",
            has_doc,
            flip
        ),
        cmd!("hatch.get", "Get Hatch Info", [], None, "{id?: u64} get hatch style and conflict status for target path", has_doc, get),
    ]
}

fn parse_direction(val: &str) -> Option<HatchDirection> {
    match val.trim().to_lowercase().as_str() {
        "0" | "deg0" | "0°" => Some(HatchDirection::Deg0),
        "45" | "deg45" | "45°" => Some(HatchDirection::Deg45),
        "90" | "deg90" | "90°" => Some(HatchDirection::Deg90),
        "135" | "deg135" | "135°" => Some(HatchDirection::Deg135),
        "cross" | "cross_ortho" | "┼" | "십자" => Some(HatchDirection::Cross),
        "diagonal_cross" | "diag_cross" | "cross_diag" | "╳" | "사선십자" => Some(HatchDirection::DiagonalCross),
        "alt" | "alternate" | "번갈아" => Some(HatchDirection::Alternate),
        "none" | "없음" => Some(HatchDirection::None),
        _ => None,
    }
}

fn direction_to_str(dir: HatchDirection) -> &'static str {
    match dir {
        HatchDirection::Deg0 => "0",
        HatchDirection::Deg45 => "45",
        HatchDirection::Deg90 => "90",
        HatchDirection::Deg135 => "135",
        HatchDirection::Cross => "cross",
        HatchDirection::DiagonalCross => "diagonal_cross",
        HatchDirection::Alternate => "alt",
        HatchDirection::None => "none",
    }
}

/// Collects all hatchable leaf path/compound node IDs under `roots` (recursing into Groups,
/// skipping hidden Live Paint Sources and stroke-only Live Paint Edge nodes).
fn collect_hatchable_ids(d: &vectorcraft_doc::Document, roots: &[NodeId]) -> Vec<NodeId> {
    fn visit(n: &vectorcraft_doc::Node, out: &mut Vec<NodeId>) {
        if !n.visible || n.name.as_deref() == Some("Live Paint Sources") || n.name.as_deref() == Some("Edge") {
            return;
        }
        match &n.kind {
            NodeKind::Path { .. } | NodeKind::Compound { .. } => out.push(n.id),
            NodeKind::Group { children, .. } | NodeKind::Layer { children, .. } => {
                for c in children {
                    visit(c, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for id in roots {
        if let Some(n) = d.node(*id) {
            visit(n, &mut out);
        }
    }
    out.sort_unstable_by_key(|id| id.0);
    out.dedup();
    out
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "hatch.set";
    let dir_str = str_param(p, "direction").ok_or_else(|| bad(C, "missing `direction` (\"45\", \"135\", \"alt\", \"none\")"))?;
    let direction = parse_direction(dir_str).ok_or_else(|| bad(C, format!("unknown direction `{dir_str}`")))?;

    let spacing_mm = p.get("spacing_mm").and_then(Value::as_f64).unwrap_or(DEFAULT_HATCH_SPACING_MM);
    let weight_pt = p.get("weight_pt").and_then(Value::as_f64).unwrap_or(DEFAULT_HATCH_WEIGHT_PT);
    let flipped_param = p.get("flipped").and_then(Value::as_bool);

    s.default_hatch = direction;
    s.default_cut_face = false;

    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({ "updated": 0, "conflicts": [], "defaultDirection": direction_to_str(direction) }));
    }

    let changed = edit_counted(s, "Set Hatch", |d| {
        let leaf_ids = collect_hatchable_ids(d, &ids);
        let mut count = 0;
        for id in leaf_ids {
            let Some(node) = d.node_mut(id) else { continue };
            if direction == HatchDirection::None {
                node.hatch = None;
            } else {
                let flipped = flipped_param.unwrap_or_else(|| node.hatch.as_ref().is_some_and(|h| h.flipped));
                node.hatch = Some(HatchStyle { direction, spacing_mm, weight_pt, flipped, resolved_direction: None });
            }
            count += 1;
        }
        update_document_hatch_directions(d);
        Ok(count)
    })?;

    // Check for odd-cycle conflicts if Alternate
    let conflicts = check_conflicts(s, &ids);

    Ok(json!({
        "updated": changed,
        "conflicts": conflicts,
    }))
}

fn cut_face(s: &mut Session, p: &Value) -> Result<Value> {
    s.default_hatch = HatchDirection::Alternate;
    s.default_cut_face = true;

    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({ "updated": 0, "defaultDirection": "alt", "defaultCutFace": true }));
    }

    let changed = edit_counted(s, "Cut Face", |d| {
        let leaf_ids = collect_hatchable_ids(d, &ids);
        let mut count = 0;
        for id in leaf_ids {
            let Some(node) = d.node_mut(id) else { continue };
            // 1. Set hatch to Cut Face (Alternate, 1.0mm, 0.1pt)
            node.hatch = Some(HatchStyle::cut_face());

            // 2. Set border stroke to 0.8 pt (unless it's a fill-only Live Paint Face)
            if node.name.as_deref() != Some("Face") {
                if let Some(st) = node.appearance.stroke_mut() {
                    st.width = DEFAULT_CUT_BORDER_PT;
                } else {
                    let st = StrokeLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), DEFAULT_CUT_BORDER_PT);
                    node.appearance.items.push(AppearanceItem::Stroke(st));
                }
            }
            count += 1;
        }
        update_document_hatch_directions(d);
        Ok(count)
    })?;

    let conflicts = check_conflicts(s, &ids);

    Ok(json!({
        "updated": changed,
        "conflicts": conflicts,
    }))
}

fn flip(s: &mut Session, p: &Value) -> Result<Value> {
    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({ "updated": 0 }));
    }

    let changed = edit_counted(s, "Flip Hatch", |d| {
        let leaf_ids = collect_hatchable_ids(d, &ids);
        let mut count = 0;
        for id in leaf_ids {
            let Some(node) = d.node_mut(id) else { continue };
            if let Some(hatch) = &mut node.hatch {
                hatch.flipped = !hatch.flipped;
                count += 1;
            }
        }
        update_document_hatch_directions(d);
        Ok(count)
    })?;

    let conflicts = check_conflicts(s, &ids);

    Ok(json!({
        "updated": changed,
        "conflicts": conflicts,
    }))
}

fn get(s: &mut Session, p: &Value) -> Result<Value> {
    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({
            "direction": direction_to_str(s.default_hatch),
            "spacing_mm": DEFAULT_HATCH_SPACING_MM,
            "weight_pt": DEFAULT_HATCH_WEIGHT_PT,
            "flipped": false,
            "has_conflict": false,
        }));
    }

    let doc = s.doc()?;
    let leaf_ids = collect_hatchable_ids(&doc.doc, &ids);
    let target_node = leaf_ids
        .iter()
        .filter_map(|id| doc.doc.node(*id))
        .find(|n| n.hatch.is_some())
        .or_else(|| leaf_ids.first().and_then(|id| doc.doc.node(*id)));

    let Some(node) = target_node else {
        return Ok(json!({
            "direction": direction_to_str(s.default_hatch),
            "spacing_mm": DEFAULT_HATCH_SPACING_MM,
            "weight_pt": DEFAULT_HATCH_WEIGHT_PT,
            "flipped": false,
            "has_conflict": false,
        }));
    };

    let conflicts = check_conflicts(s, &ids);
    let has_conflict = conflicts.contains(&node.id.0);

    if let Some(hatch) = &node.hatch {
        Ok(json!({
            "direction": direction_to_str(hatch.direction),
            "spacing_mm": hatch.spacing_mm,
            "weight_pt": hatch.weight_pt,
            "flipped": hatch.flipped,
            "has_conflict": has_conflict,
        }))
    } else {
        Ok(json!({
            "direction": "none",
            "spacing_mm": DEFAULT_HATCH_SPACING_MM,
            "weight_pt": DEFAULT_HATCH_WEIGHT_PT,
            "flipped": false,
            "has_conflict": false,
        }))
    }
}

pub(crate) fn update_document_hatch_directions(d: &mut vectorcraft_doc::Document) {
    let res = resolve_document_hatching(d);
    for (id, base_dir) in res.base_dirs {
        if let Some(node) = d.node_mut(id)
            && let Some(hatch) = &mut node.hatch
        {
            hatch.resolved_direction = Some(base_dir);
        }
    }
}

fn check_conflicts(s: &Session, _ids: &[NodeId]) -> Vec<u64> {
    let Ok(st) = s.doc() else { return Vec::new() };
    let res = resolve_document_hatching(&st.doc);
    res.conflicts.into_iter().map(|id| id.0).collect()
}
