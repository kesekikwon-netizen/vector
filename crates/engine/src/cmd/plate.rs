//! Report Plate layout, viewports, title block, scale bar, and vector export engine commands
//! (Trowel design 2.1 §6, §9, §14(a, 1), Task J012).

use std::sync::Arc;

use kurbo::{Point, Rect};
use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::MM;
use vectorcraft_doc::plate::{BorderFrame, NorthArrowItem, PlateOrientation, PlateSheet, PlateSize, ScaleBarItem, TitleBlockData, ViewportItem};
use vectorcraft_doc::scale::ScaleRatio;
use vectorcraft_doc::{Appearance, Artboard, CharStyle, Document, Node, NodeKind, TextObject};
use vectorcraft_geom::{PathData, SubPath, Vec2};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "plate.createSheet",
            "Create Report Plate Sheet",
            ["Trowel", "도판"],
            None,
            "{id?: string, name?: string, size?: 'A4'|'4x6'|'A3'|{widthMm: f64, heightMm: f64}, orientation?: 'portrait'|'landscape', caption?: string, pageNumber?: string} create a new report publication sheet and adjust document artboard",
            has_doc,
            create_sheet
        ),
        cmd!(
            "plate.addViewport",
            "Add Viewport to Sheet",
            ["Trowel", "도판"],
            None,
            "{sheetId?: string, name?: string, bounds?: [x,y,w,h], scale?: string|f64, sourceScale?: string|f64, rotation?: f64, panOffset?: [dx,dy]} add drawing viewport with independent scale to report plate",
            has_doc,
            add_viewport
        ),
        cmd!(
            "plate.updateTitleBlock",
            "Update Plate Title Block",
            ["Trowel", "도판"],
            None,
            "{sheetId?: string, drawingTitle?: string, projectTitle?: string, siteCode?: string, excavationTeam?: string, director?: string, drawnBy?: string, checkedBy?: string, date?: string, scaleText?: string, viewDirection?: string, crsDatum?: string, sheetNumber?: string, visible?: bool} update title block fields",
            has_doc,
            update_title_block
        ),
        cmd!(
            "plate.updateScaleBar",
            "Update Plate Scale Bar",
            ["Trowel", "도판"],
            None,
            "{sheetId?: string, unit?: f64, count?: usize, scale?: string|f64, linkedViewportId?: string, visible?: bool} configure segmented metric scale bar",
            has_doc,
            update_scale_bar
        ),
        cmd!(
            "plate.updateNorthArrow",
            "Update Plate North Arrow",
            ["Trowel", "도판"],
            None,
            "{sheetId?: string, rotation?: f64, linkedViewportId?: string, visible?: bool} configure dynamic north arrow",
            has_doc,
            update_north_arrow
        ),
        cmd!(
            "plate.exportPdf",
            "Export Plate to PDF",
            ["File", "Export", "Plate PDF…"],
            None,
            "{sheetId?: string, path?: string} export plate sheet to vector PDF with paper-fixed pt line weights",
            has_doc,
            export_pdf
        ),
        cmd!(
            "plate.exportSvg",
            "Export Plate to SVG",
            ["File", "Export", "Plate SVG…"],
            None,
            "{sheetId?: string, path?: string} export plate sheet to vector SVG with paper-fixed pt line weights",
            has_doc,
            export_svg
        ),
    ]
}

fn parse_plate_size(v: Option<&Value>) -> PlateSize {
    let Some(v) = v else { return PlateSize::A4 };
    if let Some(s) = v.as_str() {
        let lower = s.trim().to_ascii_lowercase();
        match lower.as_str() {
            "a4" => PlateSize::A4,
            "4x6" | "4×6" | "4x6배판" | "4×6배판" | "crownquarto" => PlateSize::CrownQuarto4x6,
            "a3" => PlateSize::A3,
            _ => PlateSize::A4,
        }
    } else if let Some(obj) = v.as_object() {
        let w = obj.get("widthMm").or_else(|| obj.get("width_mm")).and_then(Value::as_f64).unwrap_or(210.0);
        let h = obj.get("heightMm").or_else(|| obj.get("height_mm")).and_then(Value::as_f64).unwrap_or(297.0);
        PlateSize::Custom { width_mm: w, height_mm: h }
    } else {
        PlateSize::A4
    }
}

fn parse_orientation(v: Option<&Value>) -> PlateOrientation {
    match v.and_then(Value::as_str).map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("landscape") | Some("가로") => PlateOrientation::Landscape,
        _ => PlateOrientation::Portrait,
    }
}

fn parse_scale_ratio(v: Option<&Value>, default_denom: f64) -> ScaleRatio {
    let Some(v) = v else { return ScaleRatio::new(default_denom) };
    if let Some(s) = v.as_str() {
        let s = s.trim().strip_prefix("1:").unwrap_or(s.trim());
        if let Ok(denom) = s.parse::<f64>() {
            return ScaleRatio::new(denom);
        }
    } else if let Some(n) = v.as_f64() {
        return ScaleRatio::new(n);
    }
    ScaleRatio::new(default_denom)
}

fn create_sheet(s: &mut Session, p: &Value) -> Result<Value> {
    let size = parse_plate_size(p.get("size"));
    let orientation = parse_orientation(p.get("orientation"));
    let name = p.get("name").and_then(Value::as_str).unwrap_or("도면 12").to_string();
    let caption = p.get("caption").and_then(Value::as_str).unwrap_or("1호 주거지 평·단면도").to_string();
    let page_number = p.get("pageNumber").or_else(|| p.get("page_number")).and_then(Value::as_str).unwrap_or("57").to_string();

    let (w_pt, h_pt) = size.dimensions_pt(orientation);
    let sheet_id = p
        .get("id")
        .and_then(Value::as_str)
        .map(String::from)
        .unwrap_or_else(|| format!("plate-{}", s.doc().map(|d| d.doc.plate_sheets.len() + 1).unwrap_or(1)));

    let sheet = PlateSheet {
        id: sheet_id.clone(),
        name: name.clone(),
        caption,
        page_number,
        size,
        orientation,
        border_frame: BorderFrame::default(),
        viewports: Vec::new(),
        title_block: TitleBlockData::default(),
        scale_bar: ScaleBarItem::default(),
        north_arrow: NorthArrowItem::default(),
        show_legend_table: true,
        show_stratigraphy_table: true,
    };

    s.edit("Create Report Plate Sheet", |doc, _| {
        doc.plate_sheets.push(sheet);
        if let Some(ab) = doc.artboards.first_mut() {
            ab.rect = Rect::new(0.0, 0.0, w_pt, h_pt);
        }
        Ok(())
    })?;

    Ok(json!({
        "sheetId": sheet_id,
        "name": name,
        "widthPt": w_pt,
        "heightPt": h_pt,
        "size": size.label(),
        "orientation": if orientation == PlateOrientation::Landscape { "landscape" } else { "portrait" }
    }))
}

fn add_viewport(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);
    let vp_name = p.get("name").and_then(Value::as_str).unwrap_or("도면 평면").to_string();

    let bounds = if let Some(arr) = p.get("bounds").and_then(Value::as_array) {
        if arr.len() >= 4 {
            let x = arr[0].as_f64().unwrap_or(50.0);
            let y = arr[1].as_f64().unwrap_or(50.0);
            let w = arr[2].as_f64().unwrap_or(200.0);
            let h = arr[3].as_f64().unwrap_or(200.0);
            Rect::new(x, y, x + w, y + h)
        } else {
            Rect::new(50.0, 50.0, 250.0, 250.0)
        }
    } else {
        Rect::new(50.0, 50.0, 250.0, 250.0)
    };

    let scale = parse_scale_ratio(p.get("scale"), 60.0);
    let source_scale = parse_scale_ratio(p.get("sourceScale").or_else(|| p.get("source_scale")), 20.0);
    let rotation = p.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);

    let pan_offset = if let Some(arr) = p.get("panOffset").or_else(|| p.get("pan_offset")).and_then(Value::as_array) {
        if arr.len() >= 2 { Vec2::new(arr[0].as_f64().unwrap_or(0.0), arr[1].as_f64().unwrap_or(0.0)) } else { Vec2::ZERO }
    } else {
        Vec2::ZERO
    };

    let vp_id = format!("vp-{}", (s.doc()?.doc.peek_next_id()));

    let mut vp = ViewportItem::new(&vp_id, vp_name, bounds, scale, source_scale);
    vp.rotation_deg = rotation;
    vp.pan_offset = pan_offset;

    s.edit("Add Viewport to Plate", |doc, _| {
        let sheet = if let Some(sid) = sheet_id { doc.plate_sheets.iter_mut().find(|sh| sh.id == sid) } else { doc.plate_sheets.first_mut() };

        if let Some(sh) = sheet {
            sh.viewports.push(vp);
            sh.sync_dependencies();
        } else {
            let mut sh = PlateSheet::default();
            sh.viewports.push(vp);
            sh.sync_dependencies();
            doc.plate_sheets.push(sh);
        }
        Ok(())
    })?;

    Ok(json!({
        "viewportId": vp_id,
        "scale": scale.label(),
        "sourceScale": source_scale.label(),
        "bounds": [bounds.x0, bounds.y0, bounds.width(), bounds.height()]
    }))
}

fn update_title_block(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);

    s.edit("Update Title Block", |doc, _| {
        let sheet = if let Some(sid) = sheet_id { doc.plate_sheets.iter_mut().find(|sh| sh.id == sid) } else { doc.plate_sheets.first_mut() };

        let Some(sheet) = sheet else { return Ok(()) };
        let tb = &mut sheet.title_block;

        if let Some(v) = p.get("drawingTitle").or_else(|| p.get("drawing_title")).and_then(Value::as_str) {
            tb.drawing_title = v.to_string();
        }
        if let Some(v) = p.get("projectTitle").or_else(|| p.get("project_title")).and_then(Value::as_str) {
            tb.project_title = v.to_string();
        }
        if let Some(v) = p.get("siteCode").or_else(|| p.get("site_code")).and_then(Value::as_str) {
            tb.site_code = v.to_string();
        }
        if let Some(v) = p.get("excavationTeam").or_else(|| p.get("excavation_team")).and_then(Value::as_str) {
            tb.excavation_team = v.to_string();
        }
        if let Some(v) = p.get("director").and_then(Value::as_str) {
            tb.director = v.to_string();
        }
        if let Some(v) = p.get("drawnBy").or_else(|| p.get("drawn_by")).and_then(Value::as_str) {
            tb.drawn_by = v.to_string();
        }
        if let Some(v) = p.get("checkedBy").or_else(|| p.get("checked_by")).and_then(Value::as_str) {
            tb.checked_by = v.to_string();
        }
        if let Some(v) = p.get("date").and_then(Value::as_str) {
            tb.date = v.to_string();
        }
        if let Some(v) = p.get("scaleText").or_else(|| p.get("scale_text")).and_then(Value::as_str) {
            tb.scale_text = v.to_string();
        }
        if let Some(v) = p.get("viewDirection").or_else(|| p.get("view_direction")).and_then(Value::as_str) {
            tb.view_direction = v.to_string();
        }
        if let Some(v) = p.get("crsDatum").or_else(|| p.get("crs_datum")).and_then(Value::as_str) {
            tb.crs_datum = v.to_string();
        }
        if let Some(v) = p.get("sheetNumber").or_else(|| p.get("sheet_number")).and_then(Value::as_str) {
            tb.sheet_number = v.to_string();
        }
        if let Some(v) = p.get("visible").and_then(Value::as_bool) {
            tb.visible = v;
        }
        Ok(())
    })?;

    Ok(json!({ "updated": true }))
}

fn update_scale_bar(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);

    let mut total_w = 0.0;
    s.edit("Update Scale Bar", |doc, _| {
        let sheet = if let Some(sid) = sheet_id { doc.plate_sheets.iter_mut().find(|sh| sh.id == sid) } else { doc.plate_sheets.first_mut() };

        let Some(sheet) = sheet else { return Ok(()) };
        let sb = &mut sheet.scale_bar;

        if let Some(u) = p.get("unit").and_then(Value::as_f64) {
            sb.segment_unit_m = u;
        }
        if let Some(c) = p.get("count").and_then(Value::as_u64) {
            sb.segment_count = c as usize;
        }
        if let Some(sc) = p.get("scale") {
            sb.scale = parse_scale_ratio(Some(sc), sb.scale.denom);
        }
        if let Some(lid) = p.get("linkedViewportId").or_else(|| p.get("linked_viewport_id")).and_then(Value::as_str) {
            sb.linked_viewport_id = Some(lid.to_string());
        }
        if let Some(vis) = p.get("visible").and_then(Value::as_bool) {
            sb.visible = vis;
        }

        sheet.sync_dependencies();
        total_w = sheet.scale_bar.total_width_pt();
        Ok(())
    })?;

    Ok(json!({ "updated": true, "totalWidthPt": total_w }))
}

fn update_north_arrow(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);

    s.edit("Update North Arrow", |doc, _| {
        let sheet = if let Some(sid) = sheet_id { doc.plate_sheets.iter_mut().find(|sh| sh.id == sid) } else { doc.plate_sheets.first_mut() };

        let Some(sheet) = sheet else { return Ok(()) };
        let na = &mut sheet.north_arrow;

        if let Some(rot) = p.get("rotation").and_then(Value::as_f64) {
            na.rotation_deg = rot;
        }
        if let Some(lid) = p.get("linkedViewportId").or_else(|| p.get("linked_viewport_id")).and_then(Value::as_str) {
            na.linked_viewport_id = Some(lid.to_string());
        }
        if let Some(vis) = p.get("visible").and_then(Value::as_bool) {
            na.visible = vis;
        }

        sheet.sync_dependencies();
        Ok(())
    })?;

    Ok(json!({ "updated": true }))
}

/// Synthesize a clean `Document` instance corresponding to the plate sheet layout,
/// strictly preserving stroke weights in paper points (`pt`) without scaling distortion.
pub fn synthesize_plate_document(source_doc: &Document, sheet: &PlateSheet) -> Document {
    let (w_pt, h_pt) = sheet.size.dimensions_pt(sheet.orientation);
    let mut syn_doc = Document::new(w_pt, h_pt);

    // Replace artboard with exact sheet dimensions
    syn_doc.artboards =
        vec![Artboard { id: 1, name: sheet.name.clone(), rect: Rect::new(0.0, 0.0, w_pt, h_pt), show_center_mark: false, show_cross_hairs: false }];
    syn_doc.layers.clear();

    // 1. Layer: Border Frame & Graticule Ticks
    if sheet.border_frame.visible {
        let mut border_nodes = Vec::new();
        let m = sheet.border_frame.margin_mm * MM;
        let outer_rect = Rect::new(m, m, (w_pt - m).max(m), (h_pt - m).max(m));

        // Outer rule (1.2 pt)
        let outer_path = PathData::single(SubPath::polyline(
            &[
                Point::new(outer_rect.x0, outer_rect.y0),
                Point::new(outer_rect.x1, outer_rect.y0),
                Point::new(outer_rect.x1, outer_rect.y1),
                Point::new(outer_rect.x0, outer_rect.y1),
            ],
            true,
        ));
        let mut outer_node = Node::path(
            syn_doc.alloc_id(),
            outer_path,
            Appearance::basic(Paint::None, Paint::solid(Color::BLACK), sheet.border_frame.outer_weight_pt),
        );
        outer_node.name = Some("Outer Border".to_string());
        border_nodes.push(Arc::new(outer_node));

        // Inner rule (0.5 pt, inset by inner_offset_mm)
        if sheet.border_frame.double_border {
            let inset = sheet.border_frame.inner_offset_mm * MM;
            let inner_rect = Rect::new(
                outer_rect.x0 + inset,
                outer_rect.y0 + inset,
                (outer_rect.x1 - inset).max(outer_rect.x0 + inset),
                (outer_rect.y1 - inset).max(outer_rect.y0 + inset),
            );
            let inner_path = PathData::single(SubPath::polyline(
                &[
                    Point::new(inner_rect.x0, inner_rect.y0),
                    Point::new(inner_rect.x1, inner_rect.y0),
                    Point::new(inner_rect.x1, inner_rect.y1),
                    Point::new(inner_rect.x0, inner_rect.y1),
                ],
                true,
            ));
            let mut inner_node = Node::path(
                syn_doc.alloc_id(),
                inner_path,
                Appearance::basic(Paint::None, Paint::solid(Color::BLACK), sheet.border_frame.inner_weight_pt),
            );
            inner_node.name = Some("Inner Border".to_string());
            border_nodes.push(Arc::new(inner_node));
        }

        let mut border_layer = Node::layer(syn_doc.alloc_id(), "도판 테두리", vectorcraft_doc::LayerColor::Preset(0));
        if let Some(ch) = border_layer.children_mut() {
            ch.extend(border_nodes);
        }
        syn_doc.layers.push(Arc::new(border_layer));
    }

    // 2. Viewports: Transformed geometry preserving paper pt stroke weight
    for vp in &sheet.viewports {
        if !vp.visible {
            continue;
        }

        let mut vp_nodes = Vec::new();
        let xf = vp.transform();

        // Recursively walk visible layers in source document
        for layer in &source_doc.layers {
            if !layer.visible || layer.is_template() {
                continue;
            }

            for line_node in vectorcraft_doc::legend::lines(layer) {
                // Clone node and transform geometry
                let mut transformed = line_node.clone();
                transformed.id = syn_doc.alloc_id();

                if let NodeKind::Path { ref mut path, .. } = transformed.kind {
                    let mut bez = path.to_bezpath();
                    bez.apply_affine(xf);
                    *path = PathData::from_bezpath(&bez);
                }

                // ARCHAEOLOGICAL INVARIANT: Do NOT scale stroke weights!
                // Line weight remains exact paper pt (0.8 pt, 0.5 pt, 0.2 pt, 0.1 pt).
                vp_nodes.push(Arc::new(transformed));
            }
        }

        let layer_title = format!("도면 - {}", vp.name);
        let mut vp_layer = Node::layer(syn_doc.alloc_id(), &layer_title, vectorcraft_doc::LayerColor::Preset(1));
        if let Some(ch) = vp_layer.children_mut() {
            ch.extend(vp_nodes);
        }
        syn_doc.layers.push(Arc::new(vp_layer));
    }

    // 3. Title Block (표제란)
    if sheet.title_block.visible {
        let mut tb_nodes = Vec::new();
        let tb_w = sheet.title_block.width_mm * MM;
        let tb_h = sheet.title_block.height_mm * MM;
        let m = sheet.border_frame.margin_mm * MM;

        // Placed at bottom-right of printable area
        let tb_x = w_pt - m - tb_w;
        let tb_y = h_pt - m - tb_h;
        let tb_rect = Rect::new(tb_x, tb_y, tb_x + tb_w, tb_y + tb_h);

        // Outer frame 1.0 pt
        let tb_border = PathData::single(SubPath::polyline(
            &[
                Point::new(tb_rect.x0, tb_rect.y0),
                Point::new(tb_rect.x1, tb_rect.y0),
                Point::new(tb_rect.x1, tb_rect.y1),
                Point::new(tb_rect.x0, tb_rect.y1),
            ],
            true,
        ));
        tb_nodes.push(Arc::new(Node::path(
            syn_doc.alloc_id(),
            tb_border,
            Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 1.0),
        )));

        // Text title
        let mut title_text = Node::new(
            syn_doc.alloc_id(),
            NodeKind::Text(Box::new(TextObject::point(
                Point::new(tb_x + 10.0, tb_y + 20.0),
                &sheet.title_block.drawing_title,
                CharStyle { size: 9.0, ..Default::default() },
            ))),
        );
        title_text.name = Some("Title Text".to_string());
        tb_nodes.push(Arc::new(title_text));

        let mut tb_layer = Node::layer(syn_doc.alloc_id(), "표제란", vectorcraft_doc::LayerColor::Preset(2));
        if let Some(ch) = tb_layer.children_mut() {
            ch.extend(tb_nodes);
        }
        syn_doc.layers.push(Arc::new(tb_layer));
    }

    // 4. Graphical Scale Bar (막대 축척)
    if sheet.scale_bar.visible {
        let mut sb_nodes = Vec::new();
        let seg_w = sheet.scale_bar.segment_width_pt();
        let seg_h = sheet.scale_bar.height_pt;
        let start_x = sheet.scale_bar.pos.x;
        let start_y = sheet.scale_bar.pos.y;

        for i in 0..sheet.scale_bar.segment_count {
            let bx0 = start_x + (i as f64) * seg_w;
            let bx1 = bx0 + seg_w;
            let fill_color = if i % 2 == 0 { Color::BLACK } else { Color::WHITE };

            let block_path = PathData::single(SubPath::polyline(
                &[Point::new(bx0, start_y), Point::new(bx1, start_y), Point::new(bx1, start_y + seg_h), Point::new(bx0, start_y + seg_h)],
                true,
            ));
            sb_nodes.push(Arc::new(Node::path(
                syn_doc.alloc_id(),
                block_path,
                Appearance::basic(Paint::solid(fill_color), Paint::solid(Color::BLACK), 0.5),
            )));
        }

        let mut sb_layer = Node::layer(syn_doc.alloc_id(), "축척 막대", vectorcraft_doc::LayerColor::Preset(3));
        if let Some(ch) = sb_layer.children_mut() {
            ch.extend(sb_nodes);
        }
        syn_doc.layers.push(Arc::new(sb_layer));
    }

    // 5. North Arrow (방위표)
    if sheet.north_arrow.visible {
        let mut na_nodes = Vec::new();
        let center = sheet.north_arrow.pos;
        let sz = sheet.north_arrow.size_pt;
        let rad = sheet.north_arrow.rotation_deg.to_radians();

        let tip = center + Vec2::new(-sz * rad.sin(), -sz * rad.cos());
        let _tail = center + Vec2::new(sz * 0.3 * rad.sin(), sz * 0.3 * rad.cos());
        let left = center + Vec2::new(-sz * 0.25 * rad.cos(), sz * 0.25 * rad.sin());
        let right = center + Vec2::new(sz * 0.25 * rad.cos(), -sz * 0.25 * rad.sin());

        // Left half (Black)
        let left_needle = PathData::single(SubPath::polyline(&[center, tip, left], true));
        na_nodes.push(Arc::new(Node::path(
            syn_doc.alloc_id(),
            left_needle,
            Appearance::basic(Paint::solid(Color::BLACK), Paint::solid(Color::BLACK), 0.4),
        )));

        // Right half (White)
        let right_needle = PathData::single(SubPath::polyline(&[center, tip, right], true));
        na_nodes.push(Arc::new(Node::path(
            syn_doc.alloc_id(),
            right_needle,
            Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 0.4),
        )));

        let mut na_layer = Node::layer(syn_doc.alloc_id(), "방위표", vectorcraft_doc::LayerColor::Preset(4));
        if let Some(ch) = na_layer.children_mut() {
            ch.extend(na_nodes);
        }
        syn_doc.layers.push(Arc::new(na_layer));
    }

    syn_doc
}

fn export_pdf(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);
    let doc_st = s.doc()?;
    let sheet = if let Some(sid) = sheet_id { doc_st.doc.plate_sheets.iter().find(|sh| sh.id == sid) } else { doc_st.doc.plate_sheets.first() };

    let Some(sheet) = sheet else {
        return Err(bad("plate.exportPdf", "no plate sheet found in document"));
    };

    let syn_doc = synthesize_plate_document(&doc_st.doc, sheet);
    let pdf_opts = vectorcraft_pdf::PdfOptions::default();
    let bytes = vectorcraft_pdf::export(&syn_doc, &pdf_opts).map_err(|e| EngineError::Other(format!("PDF export failed: {e}")))?;

    if let Some(path_str) = p.get("path").and_then(Value::as_str) {
        std::fs::write(path_str, &bytes).map_err(|e| EngineError::Other(format!("Failed to write PDF to {path_str}: {e}")))?;
    }

    Ok(json!({
        "sheetId": sheet.id,
        "bytesLen": bytes.len(),
        "path": p.get("path")
    }))
}

fn export_svg(s: &mut Session, p: &Value) -> Result<Value> {
    let sheet_id = p.get("sheetId").or_else(|| p.get("sheet_id")).and_then(Value::as_str);
    let doc_st = s.doc()?;
    let sheet = if let Some(sid) = sheet_id { doc_st.doc.plate_sheets.iter().find(|sh| sh.id == sid) } else { doc_st.doc.plate_sheets.first() };

    let Some(sheet) = sheet else {
        return Err(bad("plate.exportSvg", "no plate sheet found in document"));
    };

    let syn_doc = synthesize_plate_document(&doc_st.doc, sheet);
    let svg_opts = vectorcraft_svg::ExportOptions::default();
    let svg_str = vectorcraft_svg::export(&syn_doc, &svg_opts);

    if let Some(path_str) = p.get("path").and_then(Value::as_str) {
        std::fs::write(path_str, &svg_str).map_err(|e| EngineError::Other(format!("Failed to write SVG to {path_str}: {e}")))?;
    }

    Ok(json!({
        "sheetId": sheet.id,
        "svgLen": svg_str.len(),
        "path": p.get("path")
    }))
}
