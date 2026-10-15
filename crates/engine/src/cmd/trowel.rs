//! Trowel archaeological project drawing commands (design 2.1 §2, §7, §14(1)):
//! - `trowel.newPlan`: 1:20 A3 landscape feature plan drawing with plan legend group.
//! - `trowel.newSection`: 1:20 A3 landscape stratigraphy section drawing with section legend group.
//! - `trowel.newFinds`: 1:1 A4 portrait artifact drawing with finds legend group.
//! - `trowel.newSitePlan`: 1:100 A3 landscape site layout drawing with plan legend group.
//! - `trowel.newPlate`: A4 portrait report plate drawing.

use serde_json::{json, Value};

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

/// Configure the single initial layer of `file.new` as the primary working layer for the template,
/// set default fill to `None`, and pick it so the user starts with 1 clean layer and can add
/// further rules as needed.
fn setup_initial_layer(s: &mut Session, layer_name: &str, weight: f64, sample: &str, group: &str) -> Result<()> {
    let _ = s.execute("paint.setFill", &json!({ "none": true }));
    let _ = s.execute("paint.toggleActive", &json!({ "fill": false }));
    let first_id = s.doc().ok().and_then(|st| st.doc.layers.first().map(|l| l.id));
    if let Some(id) = first_id {
        let _ = s.execute("layer.setProps", &json!({ "ids": [id.0], "name": layer_name, "locked": false, "visible": true }));
        let _ = s.execute(
            "legend.set",
            &json!({
                "id": id.0,
                "weight": weight,
                "line": "solid",
                "favourite": true,
                "sample": sample,
                "group": group,
            }),
        );
        let _ = s.execute("legend.pick", &json!({ "id": id.0 }));
    }
    Ok(())
}

/// 1:20 A3 Landscape archaeological plan drawing starting with 1 working layer.
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

    setup_initial_layer(s, "굴광선 (유구 면)", 0.8, "area", "plan")?;

    Ok(json!({ "template": "plan", "name": name, "scale": 20.0, "epsg": epsg }))
}

/// 1:20 A3 Landscape archaeological section drawing starting with 1 working layer.
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

    setup_initial_layer(s, "단면선 (잘린 땅)", 0.8, "line", "section")?;

    Ok(json!({ "template": "section", "name": name, "scale": 20.0, "epsg": epsg }))
}

/// 1:1 A4 Portrait artifact drawing starting with 1 working layer.
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

    setup_initial_layer(s, "외형선", 0.8, "line", "finds")?;

    Ok(json!({ "template": "finds", "name": name, "scale": 1.0, "epsg": epsg }))
}

/// 1:100 A3 Landscape archaeological site layout drawing starting with 1 working layer.
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

    setup_initial_layer(s, "굴광선 (유구 면)", 0.8, "area", "plan")?;

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
