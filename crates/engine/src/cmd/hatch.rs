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

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "hatch.set";
    let dir_str = str_param(p, "direction").ok_or_else(|| bad(C, "missing `direction` (\"45\", \"135\", \"alt\", \"none\")"))?;
    let direction = parse_direction(dir_str).ok_or_else(|| bad(C, format!("unknown direction `{dir_str}`")))?;

    let spacing_mm = p.get("spacing_mm").and_then(Value::as_f64).unwrap_or(DEFAULT_HATCH_SPACING_MM);
    let weight_pt = p.get("weight_pt").and_then(Value::as_f64).unwrap_or(DEFAULT_HATCH_WEIGHT_PT);
    let flipped_param = p.get("flipped").and_then(Value::as_bool);

    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({ "updated": 0, "conflicts": [] }));
    }

    let changed = edit_counted(s, "Set Hatch", |d| {
        let mut count = 0;
        for id in &ids {
            let Some(node) = d.node_mut(*id) else { continue };
            if !matches!(node.kind, NodeKind::Path { .. } | NodeKind::Compound { .. }) {
                continue;
            }
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
    let ids = targets(s, p)?;
    if ids.is_empty() {
        return Ok(json!({ "updated": 0 }));
    }

    let changed = edit_counted(s, "Cut Face", |d| {
        let mut count = 0;
        for id in &ids {
            let Some(node) = d.node_mut(*id) else { continue };
            if !matches!(node.kind, NodeKind::Path { .. } | NodeKind::Compound { .. }) {
                continue;
            }
            // 1. Set hatch to Cut Face (Alternate, 1.0mm, 0.1pt)
            node.hatch = Some(HatchStyle::cut_face());

            // 2. Set border stroke to 0.8 pt
            if let Some(st) = node.appearance.stroke_mut() {
                st.width = DEFAULT_CUT_BORDER_PT;
            } else {
                let st = StrokeLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), DEFAULT_CUT_BORDER_PT);
                node.appearance.items.push(AppearanceItem::Stroke(st));
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
        let mut count = 0;
        for id in &ids {
            let Some(node) = d.node_mut(*id) else { continue };
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
    let Some(first_id) = ids.first() else {
        return Ok(json!({
            "direction": "none",
            "spacing_mm": DEFAULT_HATCH_SPACING_MM,
            "weight_pt": DEFAULT_HATCH_WEIGHT_PT,
            "flipped": false,
            "has_conflict": false,
        }));
    };

    let doc = s.doc()?;
    let Some(node) = doc.doc.node(*first_id) else {
        return Ok(json!({
            "direction": "none",
            "spacing_mm": DEFAULT_HATCH_SPACING_MM,
            "weight_pt": DEFAULT_HATCH_WEIGHT_PT,
            "flipped": false,
            "has_conflict": false,
        }));
    };

    let conflicts = check_conflicts(s, &ids);
    let has_conflict = conflicts.contains(&first_id.0);

    if let Some(hatch) = &node.hatch {
        let dir_str = match hatch.direction {
            HatchDirection::Deg0 => "0",
            HatchDirection::Deg45 => "45",
            HatchDirection::Deg90 => "90",
            HatchDirection::Deg135 => "135",
            HatchDirection::Cross => "cross",
            HatchDirection::DiagonalCross => "diagonal_cross",
            HatchDirection::Alternate => "alt",
            HatchDirection::None => "none",
        };
        Ok(json!({
            "direction": dir_str,
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

fn update_document_hatch_directions(d: &mut vectorcraft_doc::Document) {
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
