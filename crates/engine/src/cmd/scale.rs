//! Scale and TM coordinate commands (Trowel Archaeological CAD, design 2.1 §6, §14):
//! - `doc.setScale`: set drawing scale ratio (e.g. 1:20, 1:10, 1:1).
//! - `doc.setOrigin`: bind a canvas point to real-world TM East/North coordinates.
//! - `doc.getScale`: get current scale, CRS, and origin information.
//! - `doc.coordToWorld`: convert canvas points to TM real-world coordinates.
//! - `doc.coordToCanvas`: convert TM real-world coordinates to canvas points.
//! - `doc.graticuleTicks`: compute graticule border ticks at given meter intervals.

use serde_json::{Value, json};
use vectorcraft_doc::scale::{CoordOrder, CoordSystem, ScaleConfig, ScaleRatio, compute_graticule_ticks};
use vectorcraft_geom::{Point, Rect};

use super::*;
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "doc.setScale",
            "Set Drawing Scale",
            [],
            None,
            "{scale: number|\"1:S\", epsg?: u32, order?: \"eastnorth\"|\"northeast\"} set drawing scale and coordinate system",
            has_doc,
            set_scale
        ),
        cmd!(
            "doc.setOrigin",
            "Set Coordinate Origin",
            [],
            None,
            "{canvas: [x,y]|{x,y}, world: [east,north]|{east,north}|{x,y}, scale?: number} bind canvas point to real-world TM coordinates",
            has_doc,
            set_origin
        ),
        cmd!("doc.getScale", "Get Drawing Scale", [], None, "{} get current drawing scale and coordinate system", has_doc, get_scale),
        cmd!(
            "doc.coordToWorld",
            "Canvas To World Coordinates",
            [],
            None,
            "{x: f64, y: f64} convert canvas point to real-world TM coordinates",
            has_doc,
            coord_to_world
        ),
        cmd!(
            "doc.coordToCanvas",
            "World To Canvas Coordinates",
            [],
            None,
            "{east?: f64, north?: f64, x?: f64, y?: f64} convert real-world TM coordinates to canvas point",
            has_doc,
            coord_to_canvas
        ),
        cmd!(
            "doc.graticuleTicks",
            "Compute Graticule Ticks",
            [],
            None,
            "{interval_m?: f64, bounds?: [x0,y0,x1,y1]} compute frame/graticule ticks along borders",
            has_doc,
            graticule_ticks
        ),
    ]
}

fn parse_scale_denom(val: &Value) -> Option<f64> {
    if let Some(n) = val.as_f64() {
        return Some(n);
    }
    if let Some(s) = val.as_str() {
        let trimmed = s.trim();
        if let Some(after) = trimmed.strip_prefix("1:") {
            return after.trim().parse().ok();
        }
        return trimmed.parse().ok();
    }
    None
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

fn parse_world_coords(v: &Value) -> Option<(f64, f64)> {
    if let Some(arr) = v.as_array()
        && arr.len() >= 2
    {
        let e = arr.first()?.as_f64()?;
        let n = arr.get(1)?.as_f64()?;
        return Some((e, n));
    }
    if let Some(obj) = v.as_object() {
        let e = obj.get("east").or_else(|| obj.get("x"))?.as_f64()?;
        let n = obj.get("north").or_else(|| obj.get("y"))?.as_f64()?;
        return Some((e, n));
    }
    None
}

fn set_scale(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "doc.setScale";
    let denom = p.get("scale").and_then(parse_scale_denom).ok_or_else(|| bad(C, "missing or invalid `scale` (expected number or \"1:20\")"))?;

    let epsg = p.get("epsg").and_then(Value::as_u64).unwrap_or(5186) as u32;
    let order_str = p.get("order").and_then(Value::as_str).unwrap_or("eastnorth");
    let order = match order_str.to_lowercase().as_str() {
        "northeast" => CoordOrder::NorthEast,
        _ => CoordOrder::EastNorth,
    };

    let doc = s.doc_mut()?;
    let mut config = doc.doc.scale.clone().unwrap_or_else(|| ScaleConfig::new(denom));
    config.scale = ScaleRatio::new(denom);
    config.crs = CoordSystem { epsg, name: format!("EPSG:{epsg}"), order };
    let label = config.scale.label();
    std::sync::Arc::make_mut(&mut doc.doc).scale = Some(config);

    Ok(json!({
        "scale": label,
        "denom": denom,
        "epsg": epsg,
        "order": if order == CoordOrder::EastNorth { "eastnorth" } else { "northeast" },
    }))
}

fn set_origin(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "doc.setOrigin";
    let canvas_pt = p.get("canvas").and_then(parse_point).ok_or_else(|| bad(C, "missing or invalid `canvas` point ([x, y] or {x, y})"))?;

    let world_coords = p
        .get("world")
        .and_then(parse_world_coords)
        .ok_or_else(|| bad(C, "missing or invalid `world` coordinates ([east, north] or {east, north})"))?;

    let scale_denom = p.get("scale").and_then(parse_scale_denom);

    let doc = s.doc_mut()?;
    let mut config = doc.doc.scale.clone().unwrap_or_default();
    if let Some(denom) = scale_denom {
        config.scale = ScaleRatio::new(denom);
    }
    config.set_origin(canvas_pt, world_coords.0, world_coords.1);
    let scale_label = config.scale.label();
    std::sync::Arc::make_mut(&mut doc.doc).scale = Some(config);

    Ok(json!({
        "canvas": [canvas_pt.x, canvas_pt.y],
        "world": [world_coords.0, world_coords.1],
        "scale": scale_label,
    }))
}

fn get_scale(s: &mut Session, _p: &Value) -> Result<Value> {
    let doc = s.doc()?;
    let Some(config) = &doc.doc.scale else {
        return Ok(json!({
            "is_set": false,
            "scale": "not set",
        }));
    };

    Ok(json!({
        "is_set": true,
        "scale": config.scale.label(),
        "denom": config.scale.denom,
        "epsg": config.crs.epsg,
        "crs_name": config.crs.name,
        "order": if config.crs.order == CoordOrder::EastNorth { "eastnorth" } else { "northeast" },
        "canvas_origin": [config.canvas_origin.x, config.canvas_origin.y],
        "world_origin": [config.world_origin.0, config.world_origin.1],
        "points_per_meter": config.scale.points_per_meter(),
    }))
}

fn coord_to_world(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "doc.coordToWorld";
    let x = p.get("x").and_then(Value::as_f64).ok_or_else(|| bad(C, "missing `x`"))?;
    let y = p.get("y").and_then(Value::as_f64).ok_or_else(|| bad(C, "missing `y`"))?;

    let doc = s.doc()?;
    let config = doc.doc.scale.as_ref().ok_or_else(|| bad(C, "document scale is not set"))?;
    let (east, north) = config.canvas_to_world(Point::new(x, y));
    let (x_readout, y_readout) = config.format_coords(Point::new(x, y));

    Ok(json!({
        "east": east,
        "north": north,
        "x": east,
        "y": north,
        "x_readout": x_readout,
        "y_readout": y_readout,
    }))
}

fn coord_to_canvas(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "doc.coordToCanvas";
    let east = p.get("east").or_else(|| p.get("x")).and_then(Value::as_f64).ok_or_else(|| bad(C, "missing `east` or `x`"))?;
    let north = p.get("north").or_else(|| p.get("y")).and_then(Value::as_f64).ok_or_else(|| bad(C, "missing `north` or `y`"))?;

    let doc = s.doc()?;
    let config = doc.doc.scale.as_ref().ok_or_else(|| bad(C, "document scale is not set"))?;
    let pt = config.world_to_canvas(east, north);

    Ok(json!({
        "canvas_x": pt.x,
        "canvas_y": pt.y,
        "x": pt.x,
        "y": pt.y,
    }))
}

fn graticule_ticks(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "doc.graticuleTicks";
    let interval_m = p.get("interval_m").and_then(Value::as_f64).unwrap_or(5.0);

    let doc = s.doc()?;
    let config = doc.doc.scale.as_ref().ok_or_else(|| bad(C, "document scale is not set"))?;

    let bounds = if let Some(arr) = p.get("bounds").and_then(Value::as_array) {
        if arr.len() >= 4 {
            let x0 = arr.first().and_then(Value::as_f64).unwrap_or(0.0);
            let y0 = arr.get(1).and_then(Value::as_f64).unwrap_or(0.0);
            let x1 = arr.get(2).and_then(Value::as_f64).unwrap_or(800.0);
            let y1 = arr.get(3).and_then(Value::as_f64).unwrap_or(600.0);
            Rect::new(x0, y0, x1, y1)
        } else {
            doc.doc.artboards.first().map(|a| a.rect).unwrap_or(Rect::new(0.0, 0.0, 800.0, 600.0))
        }
    } else {
        doc.doc.artboards.first().map(|a| a.rect).unwrap_or(Rect::new(0.0, 0.0, 800.0, 600.0))
    };

    let ticks = compute_graticule_ticks(bounds, config, interval_m);
    let serialized: Vec<Value> = ticks
        .into_iter()
        .map(|t| {
            json!({
                "pos": [t.pos.x, t.pos.y],
                "vector": [t.tick_vector.0, t.tick_vector.1],
                "world_val": t.world_val,
                "axis": t.axis,
                "label": t.label,
            })
        })
        .collect();

    Ok(json!({
        "interval_m": interval_m,
        "count": serialized.len(),
        "ticks": serialized,
    }))
}
