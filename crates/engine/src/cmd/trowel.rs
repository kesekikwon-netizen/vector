//! Trowel archaeological project drawing commands (design 2.1 §2, §7, §14(1)):
//! - `trowel.newPlan`: 1:20 A3 landscape feature plan drawing with plan legend group.
//! - `trowel.newSection`: 1:20 A3 landscape stratigraphy section drawing with section legend group.
//! - `trowel.newFinds`: 1:1 A4 portrait artifact drawing with finds legend group.
//! - `trowel.newSitePlan`: 1:100 A3 landscape site layout drawing with plan legend group.
//! - `trowel.newPlate`: A4 portrait report plate drawing.

use serde_json::{Value, json};

use super::*;
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "trowel.newPlan",
            "New Feature Plan",
            ["Trowel"],
            None,
            "{name?: string, epsg?: u32} create a 1:20 A3 landscape archaeological plan drawing with plan legend group",
            always,
            new_plan
        ),
        cmd!(
            "trowel.newSection",
            "New Section",
            ["Trowel"],
            None,
            "{name?: string, epsg?: u32} create a 1:20 A3 landscape archaeological section drawing with section legend group",
            always,
            new_section
        ),
        cmd!(
            "trowel.newFinds",
            "New Artifact Drawing",
            ["Trowel"],
            None,
            "{name?: string, epsg?: u32} create a 1:1 A4 portrait artifact drawing with finds legend group",
            always,
            new_finds
        ),
        cmd!(
            "trowel.newSitePlan",
            "New Site Layout Plan",
            ["Trowel"],
            None,
            "{name?: string, epsg?: u32} create a 1:100 A3 landscape archaeological site layout drawing with plan legend group",
            always,
            new_site_plan
        ),
        cmd!(
            "trowel.newPlate",
            "New Report Plate",
            ["Trowel"],
            None,
            "{name?: string, epsg?: u32} create an A4 portrait report plate drawing",
            always,
            new_plate
        ),
    ]
}

fn str_param<'a>(p: &'a Value, key: &str) -> Option<&'a str> {
    p.get(key).and_then(Value::as_str)
}

fn u32_or(p: &Value, key: &str, default: u32) -> u32 {
    p.get(key).and_then(Value::as_u64).map(|n| n as u32).unwrap_or(default)
}

/// Remove the leftover empty `"Layer 1"` from `file.new`, set default fill to `None`,
/// and pick the first favourite legend row so the user can draw immediately.
fn finish_template_setup(s: &mut Session) -> Result<()> {
    let _ = s.execute("paint.setFill", &json!({ "none": true }));
    if let Ok(st) = s.doc() {
        let layer1_id = st
            .doc
            .layers
            .iter()
            .find(|l| l.legend().is_none() && l.display_name() == "Layer 1" && l.children().is_none_or(|c| c.is_empty()))
            .map(|l| l.id);
        if let Some(id) = layer1_id
            && st.doc.layers.len() > 1
        {
            let _ = s.execute("layer.delete", &json!({ "ids": [id.0] }));
        }
    }
    if let Ok(st) = s.doc() {
        let fav_id = st
            .doc
            .layers
            .iter()
            .rev()
            .find(|l| !l.locked && l.visible && l.legend().is_some_and(|leg| leg.favourite))
            .or_else(|| st.doc.layers.iter().rev().find(|l| !l.locked && l.visible))
            .map(|l| l.id);
        if let Some(id) = fav_id {
            let _ = s.execute("legend.pick", &json!({ "id": id.0 }));
        }
    }
    Ok(())
}

/// 1:20 A3 Landscape archaeological plan drawing with plan legend group.
fn new_plan(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").unwrap_or("개별유구 평·단면도");
    let epsg = u32_or(p, "epsg", 5186);

    s.execute(
        "file.new",
        &json!({
            "name": name,
            "units": "Millimeters",
            "width": "420 mm",
            "height": "297 mm",
            "orientation": "landscape",
        }),
    )?;

    s.execute(
        "doc.setScale",
        &json!({
            "scale": 20.0,
            "epsg": epsg,
        }),
    )?;

    s.execute(
        "legend.addGroup",
        &json!({
            "group": "plan",
        }),
    )?;
    finish_template_setup(s)?;

    Ok(json!({ "template": "plan", "name": name, "scale": 20.0, "epsg": epsg }))
}

/// 1:20 A3 Landscape archaeological section drawing with section legend group.
fn new_section(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").unwrap_or("토층 단면도");
    let epsg = u32_or(p, "epsg", 5186);

    s.execute(
        "file.new",
        &json!({
            "name": name,
            "units": "Millimeters",
            "width": "420 mm",
            "height": "297 mm",
            "orientation": "landscape",
        }),
    )?;

    s.execute(
        "doc.setScale",
        &json!({
            "scale": 20.0,
            "epsg": epsg,
        }),
    )?;

    s.execute(
        "legend.addGroup",
        &json!({
            "group": "section",
        }),
    )?;
    finish_template_setup(s)?;

    Ok(json!({ "template": "section", "name": name, "scale": 20.0, "epsg": epsg }))
}

/// 1:1 A4 Portrait artifact drawing with finds legend group.
fn new_finds(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").unwrap_or("유물 실측도");
    let epsg = u32_or(p, "epsg", 5186);

    s.execute(
        "file.new",
        &json!({
            "name": name,
            "units": "Millimeters",
            "width": "210 mm",
            "height": "297 mm",
            "orientation": "portrait",
        }),
    )?;

    s.execute(
        "doc.setScale",
        &json!({
            "scale": 1.0,
            "epsg": epsg,
        }),
    )?;

    s.execute(
        "legend.addGroup",
        &json!({
            "group": "finds",
        }),
    )?;
    finish_template_setup(s)?;

    Ok(json!({ "template": "finds", "name": name, "scale": 1.0, "epsg": epsg }))
}

/// 1:100 A3 Landscape archaeological site layout drawing with plan legend group.
fn new_site_plan(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").unwrap_or("유구배치도");
    let epsg = u32_or(p, "epsg", 5186);

    s.execute(
        "file.new",
        &json!({
            "name": name,
            "units": "Millimeters",
            "width": "420 mm",
            "height": "297 mm",
            "orientation": "landscape",
        }),
    )?;

    s.execute(
        "doc.setScale",
        &json!({
            "scale": 100.0,
            "epsg": epsg,
        }),
    )?;

    s.execute(
        "legend.addGroup",
        &json!({
            "group": "plan",
        }),
    )?;
    finish_template_setup(s)?;

    Ok(json!({ "template": "siteplan", "name": name, "scale": 100.0, "epsg": epsg }))
}

/// A4 Portrait report plate drawing.
fn new_plate(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").unwrap_or("보고서 도판");
    let epsg = u32_or(p, "epsg", 5186);

    s.execute(
        "file.new",
        &json!({
            "name": name,
            "units": "Millimeters",
            "width": "210 mm",
            "height": "297 mm",
            "orientation": "portrait",
        }),
    )?;

    s.execute(
        "doc.setScale",
        &json!({
            "scale": 1.0,
            "epsg": epsg,
        }),
    )?;
    let _ = s.execute("paint.setFill", &json!({ "none": true }));

    Ok(json!({ "template": "plate", "name": name, "scale": 1.0, "epsg": epsg }))
}
