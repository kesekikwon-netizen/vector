//! Hatching commands (Trowel Archaeological CAD, design 2.1 §4, §7, §14):
//! - `hatch.set`: set hatch direction (45, 135, alt, none), paper spacing (mm), weight (pt), flip state.
//! - `hatch.cutFace` / `hatch.cut_face`: one-click "잘린 면" (0.8 pt border + 1.0mm/0.1pt Alternate hatch).
//! - `hatch.flip`: toggle 45° ↔ 135° flip on selected hatched faces.
//! - `hatch.get`: query hatch settings and odd-cycle conflict detection on selection.

use serde_json::{Value, json};
use vectorcraft_doc::hatch::{
    DEFAULT_CUT_BORDER_PT, DEFAULT_HATCH_SPACING_MM, DEFAULT_HATCH_WEIGHT_PT, HatchDirection, HatchStyle,
};
use vectorcraft_doc::{AppearanceItem, NodeId, NodeKind, StrokeLayer};
use vectorcraft_geom::BezPath;
use vectorcraft_pathops::hatch::solve_face_hatching;

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
            "{direction: \"45\"|\"135\"|\"alt\"|\"none\", spacing_mm?: f64, weight_pt?: f64, flipped?: bool, ids?: [u64]} set hatching on target paths",
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
        cmd!(
            "hatch.cut_face",
            "Cut Face",
            [],
            None,
            "{ids?: [u64]} alias for hatch.cutFace",
            has_doc,
            cut_face
        ),
        cmd!(
            "hatch.flip",
            "Flip Hatch Direction",
            [],
            None,
            "{ids?: [u64]} flip hatch direction (45° ↔ 135°) on target hatched paths",
            has_doc,
            flip
        ),
        cmd!(
            "hatch.get",
            "Get Hatch Info",
            [],
            None,
            "{id?: u64} get hatch style and conflict status for target path",
            has_doc,
            get
        ),
    ]
}

fn parse_direction(val: &str) -> Option<HatchDirection> {
    match val.trim().to_lowercase().as_str() {
        "45" | "deg45" | "45°" => Some(HatchDirection::Deg45),
        "135" | "deg135" | "135°" => Some(HatchDirection::Deg135),
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
                let flipped = flipped_param.unwrap_or_else(|| node.hatch.as_ref().map_or(false, |h| h.flipped));
                node.hatch = Some(HatchStyle {
                    direction,
                    spacing_mm,
                    weight_pt,
                    flipped,
                });
            }
            count += 1;
        }
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
            HatchDirection::Deg45 => "45",
            HatchDirection::Deg135 => "135",
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

fn check_conflicts(s: &Session, ids: &[NodeId]) -> Vec<u64> {
    let Ok(st) = s.doc() else { return Vec::new() };
    let mut faces = Vec::new();
    let mut styles = Vec::new();
    let mut valid_ids = Vec::new();

    let check_all = ids.len() <= 1;
    if check_all {
        st.doc.walk(|n| {
            if let Some(hatch) = &n.hatch {
                if hatch.direction == HatchDirection::Alternate {
                    let outline = match &n.kind {
                        NodeKind::Path { path, .. } => path.to_bezpath(),
                        NodeKind::Compound { children, .. } => {
                            let mut bp = BezPath::new();
                            for c in children {
                                if let Some(pd) = c.path_data() {
                                    bp.extend(pd.to_bezpath());
                                }
                            }
                            bp
                        }
                        _ => return,
                    };
                    if !outline.elements().is_empty() {
                        faces.push(outline);
                        styles.push(hatch.clone());
                        valid_ids.push(n.id.0);
                    }
                }
            }
        });
    } else {
        for id in ids {
            let Some(n) = st.doc.node(*id) else { continue };
            let Some(hatch) = &n.hatch else { continue };
            if hatch.direction != HatchDirection::Alternate {
                continue;
            }
            let outline = match &n.kind {
                NodeKind::Path { path, .. } => path.to_bezpath(),
                NodeKind::Compound { children, .. } => {
                    let mut bp = BezPath::new();
                    for c in children {
                        if let Some(pd) = c.path_data() {
                            bp.extend(pd.to_bezpath());
                        }
                    }
                    bp
                }
                _ => continue,
            };
            if outline.elements().is_empty() {
                continue;
            }
            faces.push(outline);
            styles.push(hatch.clone());
            valid_ids.push(id.0);
        }
    }

    if faces.len() < 3 {
        return Vec::new();
    }

    let results = solve_face_hatching(&faces, &styles, None);
    results
        .into_iter()
        .filter(|r| r.conflict)
        .filter_map(|r| valid_ids.get(r.face_index).copied())
        .collect()
}
