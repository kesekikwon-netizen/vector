//! Underlay image calibration and registration commands (Trowel design 2.1 §1.2, §3, §5, §13, §14, J008):
//! - `calibrate.twoPoint`: calibrate scale and rotation from 2 grid points on an underlay image.
//! - `calibrate.controlPoints` / `calibrate.affine`: least-squares similarity or affine fit against N control points.
//! - `calibrate.residuals`: compute residual error table and RMSE without modifying the document.
//! - `calibrate.apply`: bind and apply calibrated affine transform to target underlay image.

use kurbo::{Affine, Point};
use serde_json::{Value, json};
use vectorcraft_doc::legend::MM;
use vectorcraft_doc::{Document, Node, NodeId, NodeKind};
use vectorcraft_geom::registration::{
    AxisAlignment, PointPair, evaluate_residuals, fit_affine, fit_auto, fit_similarity,
    fit_two_point,
};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "calibrate.twoPoint",
            "Two-Point Scale Calibration",
            ["Trowel", "밑그림 맞추기", "모눈 2점 축척"],
            None,
            "{p1: [x,y]|{x,y}, p2: [x,y]|{x,y}, targetDist?: f64, realDistanceM?: f64, paperDistanceMm?: f64, gridCount?: f64, gridIntervalM?: f64, targetP1?: [x,y], targetP2?: [x,y], alignAxis?: 'horizontal'|'vertical'|'none', imageId?: u64, apply?: bool} calibrate underlay scale and rotation from 2 grid points",
            has_doc,
            calibrate_two_point
        ),
        cmd!(
            "calibrate.controlPoints",
            "Control Point Registration",
            ["Trowel", "밑그림 맞추기", "기준점 정합"],
            None,
            "{pairs: [{source: [u,v], target: [X,Y], name?: string}…], mode?: 'similarity'|'affine'|'auto', targetSpace?: 'canvas'|'world', sourceSpace?: 'canvas'|'pixel', imageId?: u64, apply?: bool} compute least-squares similarity or affine registration against control points",
            has_doc,
            calibrate_control_points
        ),
        cmd!(
            "calibrate.affine",
            "Control Point Registration (Alias)",
            ["Trowel", "밑그림 맞추기", "기준점 어파인"],
            None,
            "{pairs: [{source: [u,v], target: [X,Y], name?: string}…], mode?: 'similarity'|'affine'|'auto', targetSpace?: 'canvas'|'world', sourceSpace?: 'canvas'|'pixel', imageId?: u64, apply?: bool} compute least-squares similarity or affine registration against control points",
            has_doc,
            calibrate_control_points
        ),
        cmd!(
            query "calibrate.residuals",
            "Calibration Residuals",
            [],
            None,
            "{affineCoeffs: [a,b,c,d,e,f], pairs: [{source: [u,v], target: [X,Y], name?: string}…], targetSpace?: 'canvas'|'world'} compute residual error table and RMSE without modifying document",
            has_doc,
            calibrate_residuals
        ),
        cmd!(
            "calibrate.apply",
            "Apply Calibration Transform",
            ["Trowel", "밑그림 맞추기", "변환 적용"],
            None,
            "{affineCoeffs: [a,b,c,d,e,f], imageId?: u64, mode?: 'replace'|'multiply'} bind and apply calibrated affine transform to target underlay image",
            has_doc,
            calibrate_apply
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

fn parse_affine_coeffs(v: &Value) -> Option<Affine> {
    let arr = v.as_array()?;
    if arr.len() < 6 {
        return None;
    }
    let a = arr.first()?.as_f64()?;
    let b = arr.get(1)?.as_f64()?;
    let c = arr.get(2)?.as_f64()?;
    let d = arr.get(3)?.as_f64()?;
    let e = arr.get(4)?.as_f64()?;
    let f = arr.get(5)?.as_f64()?;
    Some(Affine::new([a, b, c, d, e, f]))
}

/// Locates the target underlay image node in the document.
fn find_underlay_image(doc: &Document, explicit_id: Option<NodeId>) -> Option<NodeId> {
    if let Some(id) = explicit_id {
        if matches!(doc.node(id).map(|n| &n.kind), Some(NodeKind::Image(_))) {
            return Some(id);
        }
    }
    // 1. Search layers for "밑그림" or starts with "밑그림 · " or is_template
    for layer in &doc.layers {
        let name = layer.name.as_deref().unwrap_or("");
        if name == "밑그림" || name.starts_with("밑그림 · ") || layer.is_template() {
            let mut found = None;
            layer.walk(&mut |n| {
                if found.is_none() && matches!(n.kind, NodeKind::Image(_)) {
                    found = Some(n.id);
                }
            });
            if found.is_some() {
                return found;
            }
        }
    }
    // 2. Search bottom layer
    if let Some(first_layer) = doc.layers.first() {
        let mut found = None;
        first_layer.walk(&mut |n| {
            if found.is_none() && matches!(n.kind, NodeKind::Image(_)) {
                found = Some(n.id);
            }
        });
        if found.is_some() {
            return found;
        }
    }
    // 3. Fallback: search whole document
    let mut any_image = None;
    doc.walk(|n| {
        if any_image.is_none() && matches!(n.kind, NodeKind::Image(_)) {
            any_image = Some(n.id);
        }
    });
    any_image
}

fn calibrate_two_point(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "calibrate.twoPoint";

    let p1 = p.get("p1").and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `p1`"))?;
    let p2 = p.get("p2").and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `p2`"))?;

    let ppm = s.doc()?.doc.scale.as_ref().map(|sc| sc.scale.points_per_meter()).unwrap_or(MM * 50.0); // 1:20 default

    let target_dist = if let Some(d) = p.get("targetDist").and_then(|v| v.as_f64()) {
        Some(d)
    } else if let Some(rm) = p.get("realDistanceM").and_then(|v| v.as_f64()) {
        Some(rm * ppm)
    } else if let Some(pmm) = p.get("paperDistanceMm").and_then(|v| v.as_f64()) {
        Some(pmm * MM)
    } else if let Some(count) = p.get("gridCount").and_then(|v| v.as_f64()) {
        let interval_m = p.get("gridIntervalM").and_then(|v| v.as_f64()).unwrap_or(1.0);
        Some(count * interval_m * ppm)
    } else {
        None
    };

    let target_p1 = p.get("targetP1").and_then(parse_point);
    let target_p2 = p.get("targetP2").and_then(parse_point);

    let align_axis = match p.get("alignAxis").and_then(|v| v.as_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("vertical") => AxisAlignment::Vertical,
        Some("none") => AxisAlignment::None,
        _ => AxisAlignment::Horizontal,
    };

    let res = fit_two_point(p1, p2, target_dist, target_p1, target_p2, align_axis)
        .map_err(|e| bad(C, e.to_string()))?;

    let apply = p.get("apply").and_then(|v| v.as_bool()).unwrap_or(false);
    let explicit_image_id = p.get("imageId").and_then(|v| v.as_u64()).map(NodeId);

    let mut applied = false;
    let mut bound_image_id = None;

    if apply {
        let xf = res.transform;
        s.edit("Two-Point Scale Calibration", |doc, _| {
            let img_id = find_underlay_image(doc, explicit_image_id)
                .ok_or_else(|| bad(C, "no underlay image node found to calibrate"))?;
            bound_image_id = Some(img_id.0);

            if let Some(Node { kind: NodeKind::Image(im), .. }) = doc.node_mut(img_id) {
                im.xf = xf * im.xf;
                applied = true;
            }
            Ok(json!({}))
        })?;
    } else {
        bound_image_id = find_underlay_image(&s.doc()?.doc, explicit_image_id).map(|id| id.0);
    }

    Ok(json!({
        "scale": res.scale,
        "rotationRad": res.rotation_rad,
        "rotationDeg": res.rotation_deg,
        "translation": [res.translation.x, res.translation.y],
        "affineCoeffs": res.transform.as_coeffs(),
        "measuredCanvasDist": res.measured_dist,
        "targetCanvasDist": res.target_dist,
        "applied": applied,
        "imageId": bound_image_id,
    }))
}

fn parse_pairs(cmd: &str, p: &Value, scale_cfg: Option<&vectorcraft_doc::scale::ScaleConfig>) -> Result<Vec<PointPair>> {
    let pairs_val = p.get("pairs").and_then(|v| v.as_array()).ok_or_else(|| bad(cmd, "missing `pairs` array"))?;
    let target_space = p.get("targetSpace").and_then(|v| v.as_str()).unwrap_or("canvas");
    let is_world = target_space.eq_ignore_ascii_case("world");

    let mut pairs = Vec::with_capacity(pairs_val.len());
    for item in pairs_val {
        let src = item.get("source").and_then(parse_point).ok_or_else(|| bad(cmd, "item missing `source` [u, v]"))?;
        let tgt_raw = item.get("target").and_then(parse_point).ok_or_else(|| bad(cmd, "item missing `target` [X, Y]"))?;

        let tgt = if is_world {
            let sc = scale_cfg.ok_or_else(|| bad(cmd, "world coordinates require document scale configuration"))?;
            sc.world_to_canvas(tgt_raw.x, tgt_raw.y)
        } else {
            tgt_raw
        };

        pairs.push(PointPair::new(src, tgt));
    }
    Ok(pairs)
}

fn calibrate_control_points(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "calibrate.controlPoints";

    let scale_cfg = s.doc()?.doc.scale.clone();
    let pairs = parse_pairs(C, p, scale_cfg.as_ref())?;
    let ppm = scale_cfg.as_ref().map(|sc| sc.scale.points_per_meter());

    let mode = p.get("mode").and_then(|v| v.as_str()).unwrap_or("auto");
    let res = match mode.to_ascii_lowercase().as_str() {
        "similarity" => fit_similarity(&pairs, ppm).map_err(|e| bad(C, e.to_string()))?,
        "affine" => fit_affine(&pairs, ppm).map_err(|e| bad(C, e.to_string()))?,
        _ => fit_auto(&pairs, ppm).map_err(|e| bad(C, e.to_string()))?,
    };

    let apply = p.get("apply").and_then(|v| v.as_bool()).unwrap_or(false);
    let source_space = p.get("sourceSpace").and_then(|v| v.as_str()).unwrap_or("canvas");
    let is_pixel = source_space.eq_ignore_ascii_case("pixel");
    let explicit_image_id = p.get("imageId").and_then(|v| v.as_u64()).map(NodeId);

    let mut applied = false;
    let mut bound_image_id = None;

    if apply {
        let xf = res.transform;
        s.edit("Control Point Registration", |doc, _| {
            let img_id = find_underlay_image(doc, explicit_image_id)
                .ok_or_else(|| bad(C, "no underlay image node found to calibrate"))?;
            bound_image_id = Some(img_id.0);

            if let Some(Node { kind: NodeKind::Image(im), .. }) = doc.node_mut(img_id) {
                if is_pixel {
                    im.xf = xf;
                } else {
                    im.xf = xf * im.xf;
                }
                applied = true;
            }
            Ok(json!({}))
        })?;
    } else {
        bound_image_id = find_underlay_image(&s.doc()?.doc, explicit_image_id).map(|id| id.0);
    }

    let residuals_json: Vec<Value> = res
        .residuals
        .iter()
        .map(|r| {
            json!({
                "index": r.index,
                "source": [r.source.x, r.source.y],
                "target": [r.target.x, r.target.y],
                "mapped": [r.mapped.x, r.mapped.y],
                "dx": r.dx,
                "dy": r.dy,
                "distPt": r.dist_pt,
                "distM": r.dist_m,
                "distMm": r.dist_mm,
            })
        })
        .collect();

    Ok(json!({
        "mode": mode,
        "affineCoeffs": res.affine_coeffs,
        "scale": [res.scale.0, res.scale.1],
        "rotationRad": res.rotation_rad,
        "rotationDeg": res.rotation_deg,
        "translation": [res.translation.x, res.translation.y],
        "residuals": residuals_json,
        "rmsePt": res.rmse_pt,
        "rmseM": res.rmse_m,
        "rmseMm": res.rmse_mm,
        "maxErrorPt": res.max_error_pt,
        "maxErrorIndex": res.max_error_index,
        "applied": applied,
        "imageId": bound_image_id,
    }))
}

fn calibrate_residuals(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "calibrate.residuals";

    let xf = p
        .get("affineCoeffs")
        .or_else(|| p.get("transform"))
        .and_then(parse_affine_coeffs)
        .ok_or_else(|| bad(C, "missing or invalid `affineCoeffs` [a, b, c, d, e, f]"))?;

    let scale_cfg = s.doc()?.doc.scale.clone();
    let pairs = parse_pairs(C, p, scale_cfg.as_ref())?;
    let ppm = scale_cfg.as_ref().map(|sc| sc.scale.points_per_meter());

    let res = evaluate_residuals(xf, &pairs, ppm).map_err(|e| bad(C, e.to_string()))?;

    let residuals_json: Vec<Value> = res
        .residuals
        .iter()
        .map(|r| {
            json!({
                "index": r.index,
                "source": [r.source.x, r.source.y],
                "target": [r.target.x, r.target.y],
                "mapped": [r.mapped.x, r.mapped.y],
                "dx": r.dx,
                "dy": r.dy,
                "distPt": r.dist_pt,
                "distM": r.dist_m,
                "distMm": r.dist_mm,
            })
        })
        .collect();

    Ok(json!({
        "affineCoeffs": res.affine_coeffs,
        "residuals": residuals_json,
        "rmsePt": res.rmse_pt,
        "rmseM": res.rmse_m,
        "rmseMm": res.rmse_mm,
        "maxErrorPt": res.max_error_pt,
        "maxErrorIndex": res.max_error_index,
    }))
}

fn calibrate_apply(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "calibrate.apply";

    let xf = p
        .get("affineCoeffs")
        .or_else(|| p.get("transform"))
        .and_then(parse_affine_coeffs)
        .ok_or_else(|| bad(C, "missing or invalid `affineCoeffs` [a, b, c, d, e, f]"))?;

    let mode = p.get("mode").and_then(|v| v.as_str()).unwrap_or("multiply");
    let is_replace = mode.eq_ignore_ascii_case("replace");
    let explicit_image_id = p.get("imageId").and_then(|v| v.as_u64()).map(NodeId);

    s.edit("Apply Calibration Transform", |doc, _| {
        let img_id = find_underlay_image(doc, explicit_image_id)
            .ok_or_else(|| bad(C, "no underlay image node found to calibrate"))?;

        if let Some(Node { kind: NodeKind::Image(im), .. }) = doc.node_mut(img_id) {
            if is_replace {
                im.xf = xf;
            } else {
                im.xf = xf * im.xf;
            }
            return Ok(json!({
                "applied": true,
                "imageId": img_id.0,
                "newAffineCoeffs": im.xf.as_coeffs(),
            }));
        }

        Err(bad(C, "failed to update image transform"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_doc::node::{ImageObject, LayerColor};

    fn setup_session_with_underlay() -> (Session, NodeId) {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).expect("new doc");
        let img_id = s.edit("Add Underlay", |doc, _| {
            let layer_id = doc.alloc_id();
            let mut layer = Node::layer(layer_id, "밑그림", LayerColor::Preset(0));
            if let NodeKind::Layer { template, .. } = &mut layer.kind {
                *template = true;
            }
            layer.locked = true;
            let img_id = doc.alloc_id();
            let img = Node::new(
                img_id,
                NodeKind::Image(ImageObject {
                    key: "test_img".to_string(),
                    width: 1000,
                    height: 1000,
                    xf: Affine::IDENTITY,
                    link: None,
                    placement: Default::default(),
                }),
            );
            doc.insert(None, usize::MAX, layer).unwrap();
            doc.insert(Some(layer_id), usize::MAX, img).unwrap();
            Ok(img_id)
        }).unwrap();
        (s, img_id)
    }

    #[test]
    fn test_two_point_calibration_cmd() {
        let (mut s, img_id) = setup_session_with_underlay();
        let res = s.execute(
            "calibrate.twoPoint",
            &json!({
                "p1": [10.0, 10.0],
                "p2": [110.0, 10.0],
                "targetDist": 200.0,
                "alignAxis": "horizontal",
                "apply": true
            }),
        ).expect("twoPoint cmd");

        assert_eq!(res["applied"], true);
        assert!((res["scale"].as_f64().unwrap() - 2.0).abs() < 1e-9);

        // Verify image node transform was updated
        let im_xf = match &s.doc().unwrap().doc.node(img_id).unwrap().kind {
            NodeKind::Image(im) => im.xf,
            _ => panic!("expected image"),
        };
        assert!((im_xf.as_coeffs()[0] - 2.0).abs() < 1e-9);

        // Undo and verify restored
        s.execute("edit.undo", &json!({})).expect("undo");
        let restored_xf = match &s.doc().unwrap().doc.node(img_id).unwrap().kind {
            NodeKind::Image(im) => im.xf,
            _ => panic!("expected image"),
        };
        assert!((restored_xf.as_coeffs()[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_control_points_affine_cmd() {
        let (mut s, _img_id) = setup_session_with_underlay();
        let ground_truth = Affine::new([1.05, 0.02, -0.01, 0.98, 50.0, -30.0]);
        let pts = [
            Point::new(10.0, 10.0),
            Point::new(200.0, 10.0),
            Point::new(200.0, 150.0),
            Point::new(10.0, 150.0),
        ];
        let pairs: Vec<Value> = pts
            .iter()
            .map(|&p| {
                let tgt = ground_truth * p;
                json!({
                    "source": [p.x, p.y],
                    "target": [tgt.x, tgt.y],
                })
            })
            .collect();

        let res = s.execute(
            "calibrate.controlPoints",
            &json!({
                "pairs": pairs,
                "mode": "affine",
                "apply": true,
            }),
        ).expect("controlPoints cmd");

        assert_eq!(res["applied"], true);
        assert!(res["rmsePt"].as_f64().unwrap() < 1e-9);
        assert_eq!(res["residuals"].as_array().unwrap().len(), 4);

        // Test calibrate.residuals query
        let query_res = s.execute(
            "calibrate.residuals",
            &json!({
                "affineCoeffs": res["affineCoeffs"],
                "pairs": pairs,
            }),
        ).expect("residuals query");
        assert!(query_res["rmsePt"].as_f64().unwrap() < 1e-9);
    }

    #[test]
    fn test_calibrate_apply_cmd() {
        let (mut s, img_id) = setup_session_with_underlay();
        let target_xf = [1.2, 0.0, 0.0, 1.2, 100.0, 50.0];
        let res = s.execute(
            "calibrate.apply",
            &json!({
                "affineCoeffs": target_xf,
                "mode": "replace",
            }),
        ).expect("calibrate.apply");

        assert_eq!(res["applied"], true);
        let im_xf = match &s.doc().unwrap().doc.node(img_id).unwrap().kind {
            NodeKind::Image(im) => im.xf,
            _ => panic!("expected image"),
        };
        assert!((im_xf.as_coeffs()[0] - 1.2).abs() < 1e-9);
    }
}
