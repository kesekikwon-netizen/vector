//! Stratigraphy drawing and table commands (Trowel design 2.1 §4-2, J010):
//! - `stratigraphy.addLayer`: add or update a layer description row in the stratigraphy table.
//! - `stratigraphy.removeLayer`: remove a layer row and optionally remove its badge from canvas.
//! - `stratigraphy.circledNumber`: place a circled layer number badge on canvas and sync with table.
//! - `stratigraphy.generateTable`: generate or update on-canvas vector stratigraphy table.
//! - `stratigraphy.livePaintFill`: bounded planar face extraction between strata curves.
//! - `stratigraphy.exportTsv`: export stratigraphy table as TSV text for clipboard copy.
//! - `stratigraphy.validate`: pre-flight check for orphan layer numbers or missing table rows.

use kurbo::{BezPath, Point, Rect, Shape as _};
use serde_json::{Value, json};
use std::sync::Arc;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::appearance::{AppearanceItem, FillLayer};
use vectorcraft_doc::stratigraphy::{
    NumberingStyle, SoilDescription, StratigraphyFillStyle, StratigraphyLayer, StratigraphyPatternKind, StratigraphyTable,
    StratigraphyTablePlacement, TableRenderFormat,
};
use vectorcraft_doc::{Appearance, CharStyle, Justify, Node, NodeKind, TextObject};
use vectorcraft_geom::{FillRule, PathData};
use vectorcraft_pathops::Shape as PoShape;

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "stratigraphy.addLayer",
            "Add Stratigraphy Layer",
            ["Trowel", "토층"],
            None,
            "{tableId?: string, index?: u32, colorName: string, munsell?: string, texture?: string, inclusions?: string, notes?: string, fillStyle?: object, customBadge?: string} add or update a layer description row in the stratigraphy table",
            has_doc,
            add_layer
        ),
        cmd!(
            "stratigraphy.removeLayer",
            "Remove Stratigraphy Layer",
            ["Trowel", "토층"],
            None,
            "{tableId?: string, index: u32, deleteBadge?: bool} remove a layer row and optionally remove its badge from canvas",
            has_doc,
            remove_layer
        ),
        cmd!(
            "stratigraphy.circledNumber",
            "Circled Layer Number",
            ["Trowel", "토층"],
            None,
            "{at: [x,y]|{x,y}, number?: u32, style?: 'circled'|'roman'|'arabic', customText?: string, fontSizePt?: f64, radiusPt?: f64, leaderTo?: [x,y], addToTable?: bool, tableId?: string} place a circled layer number badge on canvas and sync with table",
            has_doc,
            circled_number
        ),
        cmd!(
            "stratigraphy.generateTable",
            "Generate Stratigraphy Table",
            ["Trowel", "토층"],
            None,
            "{tableId?: string, at?: [x,y]|{x,y}, width?: f64, format?: 'grid'|'inline', fontSizePt?: f64} generate or update vector table on canvas",
            has_doc,
            generate_table
        ),
        cmd!(
            "stratigraphy.livePaintFill",
            "Live Paint Strata Fill",
            ["Trowel", "토층"],
            None,
            "{point: [x,y]|{x,y}, layerIndex?: u32, fillStyle?: object, tableId?: string} extract bounded region between strata lines and fill with pattern or tint",
            has_doc,
            live_paint_fill
        ),
        cmd!(
            query "stratigraphy.exportTsv",
            "Export Stratigraphy Table TSV",
            ["Trowel", "토층"],
            None,
            "{tableId?: string} export the stratigraphy table as TSV text for clipboard copy",
            has_doc,
            export_tsv
        ),
        cmd!(
            query "stratigraphy.validate",
            "Validate Stratigraphy",
            ["Trowel", "토층"],
            None,
            "{tableId?: string} run pre-flight check for orphan layer numbers, missing table rows, or unclosed strata lines",
            has_doc,
            validate_stratigraphy
        ),
    ]
}

const C: &str = "stratigraphy";

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

fn parse_fill_style(v: Option<&Value>) -> StratigraphyFillStyle {
    let Some(v) = v else {
        return StratigraphyFillStyle::None;
    };
    if let Some(kind_str) = v.get("pattern").and_then(Value::as_str) {
        let kind = match kind_str {
            "sburnt" | "burnt_soil" | "burnt" => StratigraphyPatternKind::BurntSoil,
            "schar" | "charcoal" => StratigraphyPatternKind::Charcoal,
            "sash" | "ash" => StratigraphyPatternKind::Ash,
            "sdist" | "disturbance" => StratigraphyPatternKind::Disturbance,
            _ => StratigraphyPatternKind::BurntSoil,
        };
        return StratigraphyFillStyle::Pattern { kind };
    }
    if let Some(color_hex) = v.get("colorHex").and_then(Value::as_str) {
        let opacity = v.get("opacity").and_then(Value::as_f64).map(|f| f as f32).unwrap_or(0.15);
        return StratigraphyFillStyle::Tint { color_hex: color_hex.to_string(), opacity };
    }
    if let Some(angle_deg) = v.get("angleDeg").and_then(Value::as_f64) {
        let spacing_mm = v.get("spacingMm").and_then(Value::as_f64).unwrap_or(1.0);
        let weight_pt = v.get("weightPt").and_then(Value::as_f64).unwrap_or(0.1);
        return StratigraphyFillStyle::Hatch { angle_deg, spacing_mm, weight_pt };
    }
    StratigraphyFillStyle::None
}

fn get_or_create_table<'a>(tables: &'a mut Vec<StratigraphyTable>, table_id: Option<&str>) -> Result<&'a mut StratigraphyTable> {
    let id_str = table_id.unwrap_or("default");
    let pos = tables.iter().position(|t| t.id == id_str);
    if let Some(idx) = pos {
        return tables.get_mut(idx).ok_or_else(|| bad(C, "table lookup error"));
    }
    tables.push(StratigraphyTable {
        id: id_str.to_string(),
        title: "토층 설명표".to_string(),
        section_line_id: None,
        numbering_style: NumberingStyle::Circled,
        layers: Vec::new(),
        placement: None,
    });
    let last = tables.len().saturating_sub(1);
    tables.get_mut(last).ok_or_else(|| bad(C, "table allocation error"))
}

fn add_layer(s: &mut Session, p: &Value) -> Result<Value> {
    let table_id = p.get("tableId").and_then(Value::as_str);
    let color_name = p.get("colorName").and_then(Value::as_str).ok_or_else(|| bad(C, "colorName is required"))?.to_string();
    let munsell = p.get("munsell").and_then(Value::as_str).unwrap_or("").to_string();
    let texture = p.get("texture").and_then(Value::as_str).unwrap_or("").to_string();
    let inclusions = p.get("inclusions").and_then(Value::as_str).unwrap_or("").to_string();
    let notes = p.get("notes").and_then(Value::as_str).unwrap_or("").to_string();
    let custom_badge = p.get("customBadge").and_then(Value::as_str).map(|s| s.to_string());
    let fill_style = parse_fill_style(p.get("fillStyle"));

    let given_index = p.get("index").and_then(Value::as_u64).map(|u| u as u32);

    s.edit("Add Stratigraphy Layer", |doc, _| {
        let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
        let index = given_index.unwrap_or_else(|| table.layers.iter().map(|l| l.index).max().unwrap_or(0) + 1);

        let desc = SoilDescription { color_name, munsell, texture, inclusions, notes };

        if let Some(existing) = table.layers.iter_mut().find(|l| l.index == index) {
            existing.description = desc;
            existing.fill_style = fill_style;
            if custom_badge.is_some() {
                existing.custom_badge = custom_badge;
            }
        } else {
            table.layers.push(StratigraphyLayer { index, custom_badge, description: desc, fill_style, badge_node_id: None, face_node_id: None });
            table.layers.sort_by_key(|l| l.index);
        }

        Ok(json!({
            "tableId": table.id,
            "index": index,
            "count": table.layers.len(),
        }))
    })
}

fn remove_layer(s: &mut Session, p: &Value) -> Result<Value> {
    let table_id = p.get("tableId").and_then(Value::as_str);
    let index = p.get("index").and_then(Value::as_u64).map(|u| u as u32).ok_or_else(|| bad(C, "index is required"))?;
    let delete_badge = p.get("deleteBadge").and_then(Value::as_bool).unwrap_or(false);

    s.edit("Remove Stratigraphy Layer", |doc, _| {
        let (pos, badge_to_delete) = {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            let pos_opt = table.layers.iter().position(|l| l.index == index);
            let b_opt = if delete_badge { pos_opt.and_then(|i| table.layers.get(i)).and_then(|l| l.badge_node_id) } else { None };
            (pos_opt, b_opt)
        };

        let count = if let Some(idx) = pos {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            table.layers.remove(idx);
            table.layers.len()
        } else {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            table.layers.len()
        };

        if let Some(b_id) = badge_to_delete {
            let _ = doc.remove(b_id);
        }

        Ok(json!({
            "tableId": table_id.unwrap_or("default"),
            "removedIndex": index,
            "count": count,
        }))
    })
}

fn circled_number(s: &mut Session, p: &Value) -> Result<Value> {
    let at_val = p.get("at").ok_or_else(|| bad(C, "at coordinates required"))?;
    let center = parse_point(at_val).ok_or_else(|| bad(C, "invalid at coordinates"))?;

    let style_str = p.get("style").and_then(Value::as_str).unwrap_or("circled");
    let style = match style_str {
        "roman" => NumberingStyle::Roman,
        "arabic" => NumberingStyle::Arabic,
        _ => NumberingStyle::Circled,
    };

    let table_id = p.get("tableId").and_then(Value::as_str);
    let custom_text = p.get("customText").and_then(Value::as_str).map(|s| s.to_string());
    let font_size_pt = p.get("fontSizePt").and_then(Value::as_f64).unwrap_or(8.0);
    let radius_pt = p.get("radiusPt").and_then(Value::as_f64).unwrap_or(7.0);
    let leader_to = p.get("leaderTo").and_then(parse_point);
    let add_to_table = p.get("addToTable").and_then(Value::as_bool).unwrap_or(true);

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Circled Layer Number", |doc, sel| {
        let next_num = {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            table.layers.iter().map(|l| l.index).max().unwrap_or(0) + 1
        };

        let number = p.get("number").and_then(Value::as_u64).map(|u| u as u32).unwrap_or(next_num);

        let badge_text = if let Some(ref ct) = custom_text { ct.clone() } else { style.format(number) };

        let mut children = Vec::with_capacity(3);

        // 1. Leader line if requested
        if let Some(target) = leader_to {
            let mut bez = BezPath::new();
            bez.move_to(target);
            let dir_x = center.x - target.x;
            let dir_y = center.y - target.y;
            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            let end_pt =
                if len > radius_pt { Point::new(center.x - (dir_x / len) * radius_pt, center.y - (dir_y / len) * radius_pt) } else { center };
            bez.line_to(end_pt);

            let leader_id = doc.alloc_id();
            let leader_pd = PathData::from_bezpath(&bez);
            let leader_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
            let mut leader_node = Node::path(leader_id, leader_pd, leader_app);
            leader_node.name = Some("지시선".to_string());
            children.push(Arc::new(leader_node));
        }

        // 2. White circle badge background with 0.5pt black stroke
        let circle_id = doc.alloc_id();
        let circle_rect = Rect::from_center_size(center, kurbo::Size::new(radius_pt * 2.0, radius_pt * 2.0));
        let circle_bez = circle_rect.to_path(0.1);
        let circle_pd = PathData::from_bezpath(&circle_bez);
        let circle_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut circle_node = Node::path(circle_id, circle_pd, circle_app);
        circle_node.name = Some(format!("번호 원 {badge_text}"));
        children.push(Arc::new(circle_node));

        // 3. Text object with white halo
        let text_id = doc.alloc_id();
        let char_style = CharStyle {
            size: font_size_pt,
            fill: Paint::solid(Color::rgb8(20, 20, 19)),
            stroke: Paint::solid(Color::WHITE),
            stroke_width: 2.0,
            ..Default::default()
        };
        let mut text_obj = TextObject::point(Point::new(center.x, center.y + font_size_pt * 0.35), &badge_text, char_style);
        text_obj.para.justify = Justify::Center;
        let mut text_node = Node::new(text_id, NodeKind::Text(Box::new(text_obj)));
        text_node.name = Some(format!("층위 번호 {badge_text}"));
        children.push(Arc::new(text_node));

        // Group container
        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some(format!("토층 번호 {badge_text}"));

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![group_id];

        let mut layer_created = false;
        let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
        if let Some(existing) = table.layers.iter_mut().find(|l| l.index == number) {
            existing.badge_node_id = Some(group_id);
        } else if add_to_table {
            table.layers.push(StratigraphyLayer {
                index: number,
                custom_badge: custom_text.clone(),
                description: SoilDescription::default(),
                fill_style: StratigraphyFillStyle::None,
                badge_node_id: Some(group_id),
                face_node_id: None,
            });
            table.layers.sort_by_key(|l| l.index);
            layer_created = true;
        }

        Ok(json!({
            "id": group_id.0,
            "number": number,
            "badge": badge_text,
            "layerCreated": layer_created,
        }))
    })
}

fn generate_table(s: &mut Session, p: &Value) -> Result<Value> {
    let table_id = p.get("tableId").and_then(Value::as_str);
    let origin = p.get("at").and_then(parse_point).unwrap_or(Point::new(100.0, 400.0));
    let width_pt = p.get("width").and_then(Value::as_f64).unwrap_or(420.0);
    let font_size_pt = p.get("fontSizePt").and_then(Value::as_f64).unwrap_or(8.0);
    let format_str = p.get("format").and_then(Value::as_str).unwrap_or("grid");
    let render_format = if format_str == "inline" { TableRenderFormat::Inline } else { TableRenderFormat::Grid };

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Generate Stratigraphy Table", |doc, sel| {
        let (n_layers, table_title, rows_data) = {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            let numbering_style = table.numbering_style;
            let rows: Vec<_> = table
                .layers
                .iter()
                .map(|l| {
                    (
                        l.badge_text(numbering_style),
                        l.description.color_name.clone(),
                        l.description.munsell.clone(),
                        l.description.texture.clone(),
                        l.description.inclusions.clone(),
                        l.description.notes.clone(),
                    )
                })
                .collect();
            (table.layers.len(), table.title.clone(), rows)
        };

        let header_h = 22.0;
        let row_h = 20.0;
        let total_h = header_h + (n_layers as f64) * row_h;

        let col_widths = [36.0, 64.0, 70.0, 60.0, 90.0, (width_pt - 320.0).max(80.0)];
        let headers = ["층위", "토색", "먼셀", "토성", "포함유물", "비고"];

        let mut children = Vec::new();

        // 1. Table Outer Frame & Grid Lines
        let mut lines_bez = BezPath::new();
        // Outer rect
        let table_rect = Rect::from_origin_size(origin, kurbo::Size::new(width_pt, total_h));
        lines_bez.extend(table_rect.to_path(0.1));

        // Header horizontal separator (0.5 pt)
        lines_bez.move_to(Point::new(origin.x, origin.y + header_h));
        lines_bez.line_to(Point::new(origin.x + width_pt, origin.y + header_h));

        // Row horizontal separators (0.2 pt)
        for i in 1..n_layers {
            let y = origin.y + header_h + (i as f64) * row_h;
            lines_bez.move_to(Point::new(origin.x, y));
            lines_bez.line_to(Point::new(origin.x + width_pt, y));
        }

        // Column vertical separators
        let mut cur_col_x = origin.x;
        for &cw in &col_widths[..col_widths.len() - 1] {
            cur_col_x += cw;
            lines_bez.move_to(Point::new(cur_col_x, origin.y));
            lines_bez.line_to(Point::new(cur_col_x, origin.y + total_h));
        }

        let lines_id = doc.alloc_id();
        let lines_pd = PathData::from_bezpath(&lines_bez);
        let lines_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::rgb8(20, 20, 19)), 0.5);
        let mut lines_node = Node::path(lines_id, lines_pd, lines_app);
        lines_node.name = Some("표 격자".to_string());
        children.push(Arc::new(lines_node));

        // 2. Header Text
        let mut col_left = origin.x;
        for (col_idx, &head_title) in headers.iter().enumerate() {
            let cw = col_widths[col_idx];
            let cell_center = Point::new(col_left + cw * 0.5, origin.y + header_h * 0.5 + font_size_pt * 0.35);

            let txt_id = doc.alloc_id();
            let char_style = CharStyle { size: font_size_pt, fill: Paint::solid(Color::rgb8(20, 20, 19)), ..Default::default() };
            let mut txt_obj = TextObject::point(cell_center, head_title, char_style);
            txt_obj.para.justify = Justify::Center;
            let mut txt_node = Node::new(txt_id, NodeKind::Text(Box::new(txt_obj)));
            txt_node.name = Some(format!("헤더 {head_title}"));
            children.push(Arc::new(txt_node));

            col_left += cw;
        }

        // 3. Row Text
        for (row_idx, row_tuple) in rows_data.iter().enumerate() {
            let row_top = origin.y + header_h + (row_idx as f64) * row_h;
            let cell_cy = row_top + row_h * 0.5 + font_size_pt * 0.35;

            let row_values =
                [row_tuple.0.clone(), row_tuple.1.clone(), row_tuple.2.clone(), row_tuple.3.clone(), row_tuple.4.clone(), row_tuple.5.clone()];

            let mut r_col_left = origin.x;
            for (col_idx, cell_text) in row_values.iter().enumerate() {
                let cw = col_widths[col_idx];
                let is_badge = col_idx == 0;
                let text_pos = if is_badge { Point::new(r_col_left + cw * 0.5, cell_cy) } else { Point::new(r_col_left + 6.0, cell_cy) };

                let txt_id = doc.alloc_id();
                let char_style = CharStyle { size: font_size_pt, fill: Paint::solid(Color::rgb8(20, 20, 19)), ..Default::default() };
                let mut txt_obj = TextObject::point(text_pos, cell_text, char_style);
                if is_badge {
                    txt_obj.para.justify = Justify::Center;
                }
                let mut txt_node = Node::new(txt_id, NodeKind::Text(Box::new(txt_obj)));
                txt_node.name = Some(format!("줄 {} 열 {}", row_idx + 1, col_idx + 1));
                children.push(Arc::new(txt_node));

                r_col_left += cw;
            }
        }

        // Group container
        let group_id = doc.alloc_id();
        let mut group = Node::new(group_id, NodeKind::Group { children, clip: false });
        group.name = Some(format!("토층 설명표 ({table_title})"));

        doc.insert(target_parent, usize::MAX, group).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![group_id];

        let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
        table.placement = Some(StratigraphyTablePlacement { origin, width_pt, font_size_pt, format: render_format, node_id: Some(group_id) });

        Ok(json!({
            "tableId": table.id,
            "nodeId": group_id.0,
            "rowCount": n_layers,
            "bounds": [origin.x, origin.y, width_pt, total_h],
        }))
    })
}

fn live_paint_fill(s: &mut Session, p: &Value) -> Result<Value> {
    let pt_val = p.get("point").ok_or_else(|| bad(C, "point is required"))?;
    let point = parse_point(pt_val).ok_or_else(|| bad(C, "invalid point coordinates"))?;

    let layer_index = p.get("layerIndex").and_then(Value::as_u64).map(|u| u as u32);
    let table_id = p.get("tableId").and_then(Value::as_str);
    let fill_style = parse_fill_style(p.get("fillStyle"));

    let target_parent = s.doc()?.target_parent()?;

    s.edit("Live Paint Strata Fill", |doc, sel| {
        let mut shapes = Vec::new();
        doc.walk(|node| {
            if let NodeKind::Path { path, .. } = &node.kind
                && !path.subpaths.is_empty()
            {
                shapes.push(PoShape::new(path.clone(), FillRule::NonZero, node.id.0));
            }
        });

        if shapes.is_empty() {
            return Err(bad(C, "no vector curves available for live paint face extraction"));
        }

        let (regions, _) = vectorcraft_pathops::live_paint(&shapes);

        let face =
            regions.into_iter().find(|r| r.contains(point)).ok_or_else(|| bad(C, "click point does not lie inside any bounded planar face"))?;

        let area = face.path.to_bezpath().area().abs();

        let appearance = match &fill_style {
            StratigraphyFillStyle::Tint { color_hex, opacity } => {
                let col = Color::from_hex(color_hex).unwrap_or(Color::rgb8(180, 140, 100));
                let mut fill = FillLayer::new(Paint::solid(col));
                fill.opacity = *opacity;
                Appearance { items: vec![AppearanceItem::Fill(fill)], ..Default::default() }
            }
            StratigraphyFillStyle::Pattern { kind } => {
                let (col, op) = match kind {
                    StratigraphyPatternKind::BurntSoil => (Color::rgb8(180, 80, 40), 0.25),
                    StratigraphyPatternKind::Charcoal => (Color::rgb8(40, 40, 40), 0.35),
                    StratigraphyPatternKind::Ash => (Color::rgb8(160, 160, 160), 0.20),
                    StratigraphyPatternKind::Disturbance => (Color::rgb8(120, 100, 80), 0.25),
                };
                let mut fill = FillLayer::new(Paint::solid(col));
                fill.opacity = op;
                Appearance { items: vec![AppearanceItem::Fill(fill)], ..Default::default() }
            }
            StratigraphyFillStyle::Hatch { angle_deg, spacing_mm, weight_pt } => {
                let hatch_lines = vectorcraft_pathops::generate_section_hatch(&face.path, *spacing_mm, *weight_pt, *angle_deg);
                let hatch_pd = PathData::from_bezpath(&hatch_lines);
                let hatch_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(80, 60, 40)), *weight_pt);

                let face_id = doc.alloc_id();
                let mut face_node = Node::path(face_id, hatch_pd, hatch_app);
                face_node.name = Some("토층 빗금 칠".to_string());

                doc.insert(target_parent, usize::MAX, face_node).map_err(|e| bad(C, e.to_string()))?;
                sel.objects = vec![face_id];

                if let Some(idx) = layer_index {
                    let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
                    if let Some(layer) = table.layers.iter_mut().find(|l| l.index == idx) {
                        layer.face_node_id = Some(face_id);
                        layer.fill_style = fill_style.clone();
                    }
                }

                return Ok(json!({
                    "faceId": face_id.0,
                    "area": area,
                    "isHatch": true,
                }));
            }
            StratigraphyFillStyle::None => {
                let mut fill = FillLayer::new(Paint::solid(Color::rgb8(210, 180, 140)));
                fill.opacity = 0.2;
                Appearance { items: vec![AppearanceItem::Fill(fill)], ..Default::default() }
            }
        };

        let face_id = doc.alloc_id();
        let mut face_node = Node::path(face_id, face.path.clone(), appearance);
        face_node.name = Some("토층 칠".to_string());

        doc.insert(target_parent, usize::MAX, face_node).map_err(|e| bad(C, e.to_string()))?;
        sel.objects = vec![face_id];

        if let Some(idx) = layer_index {
            let table = get_or_create_table(&mut doc.stratigraphy_tables, table_id)?;
            if let Some(layer) = table.layers.iter_mut().find(|l| l.index == idx) {
                layer.face_node_id = Some(face_id);
                layer.fill_style = fill_style;
            }
        }

        Ok(json!({
            "faceId": face_id.0,
            "area": area,
        }))
    })
}

fn export_tsv(s: &mut Session, p: &Value) -> Result<Value> {
    let table_id = p.get("tableId").and_then(Value::as_str);
    let doc = &s.doc()?.doc;

    let table = if let Some(id) = table_id { doc.stratigraphy_tables.iter().find(|t| t.id == id) } else { doc.stratigraphy_tables.first() };

    let Some(table) = table else {
        return Ok(json!({
            "tsv": "층위\t토색\t먼셀\t토성\t포함유물\t비고\n",
            "rowCount": 0,
        }));
    };

    let tsv = table.to_tsv();
    Ok(json!({
        "tsv": tsv,
        "rowCount": table.layers.len(),
    }))
}

fn validate_stratigraphy(s: &mut Session, p: &Value) -> Result<Value> {
    let table_id = p.get("tableId").and_then(Value::as_str);
    let doc = &s.doc()?.doc;

    let table = if let Some(id) = table_id { doc.stratigraphy_tables.iter().find(|t| t.id == id) } else { doc.stratigraphy_tables.first() };

    let mut warnings = Vec::new();
    let mut orphan_badges = Vec::new();
    let mut missing_table_rows = Vec::new();

    if let Some(t) = table {
        let table_indices: std::collections::HashSet<u32> = t.layers.iter().map(|l| l.index).collect();

        // Check if rows have descriptions
        for layer in &t.layers {
            if layer.description.color_name.is_empty() && layer.description.texture.is_empty() {
                missing_table_rows.push(layer.index);
                warnings.push(format!("토층 {}번 설명표 줄이 비어 있습니다", layer.badge_text(t.numbering_style)));
            }
            if let Some(b_id) = layer.badge_node_id
                && doc.node(b_id).is_none()
            {
                warnings.push(format!("토층 {}번 기호 원이 도면에서 누락되었습니다", layer.index));
            }
        }

        // Check if any badge nodes on canvas reference numbers outside table
        doc.walk(|node| {
            if let Some(name) = &node.name
                && name.starts_with("토층 번호")
            {
                for part in name.split_whitespace() {
                    if let Ok(num) = part.parse::<u32>()
                        && !table_indices.contains(&num)
                    {
                        orphan_badges.push(num);
                        warnings.push(format!("도면에 토층 {}번 기호가 있으나 설명표에 없습니다", num));
                    }
                }
            }
        });
    } else {
        warnings.push("문서에 토층 설명표가 정의되어 있지 않습니다".to_string());
    }

    let is_valid = warnings.is_empty();
    Ok(json!({
        "valid": is_valid,
        "warnings": warnings,
        "orphanBadges": orphan_badges,
        "missingTableRows": missing_table_rows,
    }))
}
