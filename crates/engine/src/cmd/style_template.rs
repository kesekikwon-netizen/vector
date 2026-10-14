//! Style Template (.trowel-style) and Dynamic Auto Legend engine commands
//! (Trowel design 2.1 §6, §7, §8, §10, §14, Task J018).

use std::sync::Arc;

use kurbo::Point;
use serde_json::{json, Value};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{
    LegendLayoutOptions, LegendSample, LegendTable,
};
use vectorcraft_doc::style_template::{
    MergeAction, StyleCategory, StyleMergeResolution, TemplateLegendRow, TrowelStyleTemplate,
    diff_template, TROWEL_STYLE_FORMAT, TROWEL_STYLE_VERSION,
};
use vectorcraft_doc::{Appearance, CharStyle, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, SubPath};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "style.exportTemplate",
            "Export Style Template",
            ["File", "Export", "Trowel Style Template…"],
            None,
            "{name?: string, path?: string, author?: string, organization?: string} export document styles to a standalone .trowel-style JSON file",
            has_doc,
            export_template
        ),
        cmd!(
            "style.diffTemplate",
            "Diff Style Template",
            ["Trowel", "양식"],
            None,
            "{template: string|object} compare incoming .trowel-style template against document and return diff categorization (Identical, Added, Modified, Missing)",
            has_doc,
            cmd_diff_template
        ),
        cmd!(
            "style.applyTemplate",
            "Apply Style Template",
            ["File", "Import", "Trowel Style Template…"],
            None,
            "{template: string|object, resolutions?: [{key: string, category: string, action: 'keep_existing'|'overwrite'|'import_as_new_copy'}]} apply template styles with resolved merge actions. One undo step.",
            has_doc,
            apply_template
        ),
        cmd!(
            "style.generateLegend",
            "Generate Auto Legend Table",
            ["Drawing", "Insert Legend Table"],
            None,
            "{at?: [x,y], width?: f64, columns?: usize, filterUnused?: bool, replaceExisting?: bool} scan active drawing layers and generate on-canvas vector legend table. One undo step.",
            has_doc,
            generate_legend
        ),
        cmd!(
            "style.updateLegend",
            "Update Legend Table",
            ["Drawing", "Update Legend Table"],
            None,
            "{tableNodeId?: u64} re-scan active layers and update existing canvas legend table in-place. One undo step.",
            has_doc,
            update_legend
        ),
    ]
}

fn parse_template_param(p: &Value) -> Result<TrowelStyleTemplate> {
    let tmpl_val = p.get("template").ok_or_else(|| bad("style", "missing `template` parameter"))?;
    let tmpl: TrowelStyleTemplate = if let Some(s) = tmpl_val.as_str() {
        serde_json::from_str(s).map_err(|e| bad("style", format!("invalid template JSON: {e}")))?
    } else {
        serde_json::from_value(tmpl_val.clone())
            .map_err(|e| bad("style", format!("invalid template object: {e}")))?
    };

    if tmpl.format != TROWEL_STYLE_FORMAT {
        return Err(bad(
            "style",
            format!("unsupported format '{}', expected '{}'", tmpl.format, TROWEL_STYLE_FORMAT),
        ));
    }
    if tmpl.version > TROWEL_STYLE_VERSION {
        return Err(bad(
            "style",
            format!("unsupported template version {}, maximum supported is {}", tmpl.version, TROWEL_STYLE_VERSION),
        ));
    }

    Ok(tmpl)
}

fn export_template(s: &mut Session, p: &Value) -> Result<Value> {
    let doc_st = s.doc()?;
    let doc = &doc_st.doc;

    let mut tmpl = TrowelStyleTemplate::default();
    if let Some(n) = p.get("name").and_then(Value::as_str) {
        tmpl.name = n.to_string();
    }
    if let Some(a) = p.get("author").and_then(Value::as_str) {
        tmpl.author = Some(a.to_string());
    }
    if let Some(o) = p.get("organization").and_then(Value::as_str) {
        tmpl.organization = Some(o.to_string());
    }

    // Extract legend rows from active document layers
    let mut rows = Vec::new();
    for layer in &doc.layers {
        if let Some(leg) = layer.legend() {
            rows.push(TemplateLegendRow {
                name: layer.display_name().trim().to_string(),
                group: leg.group,
                weight_pt: leg.weight,
                line_type: leg.line.clone(),
                sample: leg.sample.clone(),
                favourite: leg.favourite,
                strata_key: leg.strata.clone(),
                description: None,
                hatch_preset_ref: None,
            });
        }
    }
    if !rows.is_empty() {
        tmpl.legend_rows = rows;
    }

    let json_str = serde_json::to_string_pretty(&tmpl)
        .map_err(|e| EngineError::Other(format!("Failed to serialize template: {e}")))?;

    if let Some(path_str) = p.get("path").and_then(Value::as_str) {
        std::fs::write(path_str, &json_str)
            .map_err(|e| EngineError::Other(format!("Failed to write template to {path_str}: {e}")))?;
    }

    Ok(json!({
        "name": tmpl.name,
        "version": tmpl.version,
        "format": tmpl.format,
        "rowCount": tmpl.legend_rows.len(),
        "template": tmpl,
        "path": p.get("path")
    }))
}

fn cmd_diff_template(s: &mut Session, p: &Value) -> Result<Value> {
    let tmpl = parse_template_param(p)?;
    let doc_st = s.doc()?;
    let diff = diff_template(&doc_st.doc, &tmpl);

    Ok(json!({
        "identicalCount": diff.identical_count,
        "addedCount": diff.added_count,
        "modifiedCount": diff.modified_count,
        "missingCount": diff.missing_count,
        "summaryKo": diff.summary_ko(),
        "items": diff.items
    }))
}

fn apply_template(s: &mut Session, p: &Value) -> Result<Value> {
    let tmpl = parse_template_param(p)?;

    // Parse resolutions if provided
    let mut resolutions: Vec<StyleMergeResolution> = Vec::new();
    if let Some(arr) = p.get("resolutions").and_then(Value::as_array) {
        for item in arr {
            let key = item.get("key").and_then(Value::as_str).unwrap_or("").to_string();
            let action_str = item.get("action").and_then(Value::as_str).unwrap_or("overwrite");
            let action = match action_str {
                "keep_existing" => MergeAction::KeepExisting,
                "import_as_new_copy" => MergeAction::ImportAsNewCopy,
                _ => MergeAction::Overwrite,
            };
            resolutions.push(StyleMergeResolution {
                category: StyleCategory::LegendRow,
                key,
                action,
            });
        }
    }

    let mut applied_count = 0;

    s.edit("Apply Style Template", |doc, _| {
        for row in &tmpl.legend_rows {
            let res = resolutions.iter().find(|r| r.key == row.name).map(|r| r.action).unwrap_or(MergeAction::Overwrite);

            match res {
                MergeAction::KeepExisting => {}
                MergeAction::Overwrite => {
                    let existing = doc.layers.iter_mut().find(|l| l.display_name().trim() == row.name.trim());
                    if let Some(layer) = existing {
                        // Update existing layer legend
                        if let NodeKind::Layer { ref mut legend, .. } = Arc::make_mut(layer).kind {
                            *legend = Some(Box::new(vectorcraft_doc::Legend {
                                weight: row.weight_pt,
                                line: row.line_type.clone(),
                                favourite: row.favourite,
                                sample: row.sample.clone(),
                                strata: row.strata_key.clone(),
                                group: row.group,
                            }));
                        }
                    } else {
                        // Add as new layer
                        let new_id = doc.alloc_id();
                        let mut new_layer = Node::layer(new_id, &row.name, vectorcraft_doc::LayerColor::Preset(0));
                        if let NodeKind::Layer { ref mut legend, .. } = new_layer.kind {
                            *legend = Some(Box::new(vectorcraft_doc::Legend {
                                weight: row.weight_pt,
                                line: row.line_type.clone(),
                                favourite: row.favourite,
                                sample: row.sample.clone(),
                                strata: row.strata_key.clone(),
                                group: row.group,
                            }));
                        }
                        doc.layers.push(Arc::new(new_layer));
                    }
                    applied_count += 1;
                }
                MergeAction::ImportAsNewCopy => {
                    let copy_name = format!("{} (양식)", row.name);
                    let new_id = doc.alloc_id();
                    let mut new_layer = Node::layer(new_id, &copy_name, vectorcraft_doc::LayerColor::Preset(1));
                    if let NodeKind::Layer { ref mut legend, .. } = new_layer.kind {
                        *legend = Some(Box::new(vectorcraft_doc::Legend {
                            weight: row.weight_pt,
                            line: row.line_type.clone(),
                            favourite: row.favourite,
                            sample: row.sample.clone(),
                            strata: row.strata_key.clone(),
                            group: row.group,
                        }));
                    }
                    doc.layers.push(Arc::new(new_layer));
                    applied_count += 1;
                }
            }
        }
        Ok(())
    })?;

    Ok(json!({ "appliedCount": applied_count }))
}

const LEGEND_TABLE_TAG: &str = "trowel_legend_table";

fn build_legend_vector_nodes(table: &LegendTable, alloc_id: &mut impl FnMut() -> NodeId) -> Vec<Arc<Node>> {
    let mut nodes = Vec::new();
    let opt = &table.options;
    let b = table.bounds;

    // 1. Top bounding rule (0.7 pt)
    if opt.top_rule {
        let top_path = PathData::single(SubPath::polyline(
            &[Point::new(b.x0, b.y0), Point::new(b.x1, b.y0)],
            false,
        ));
        nodes.push(Arc::new(Node::path(
            alloc_id(),
            top_path,
            Appearance::basic(Paint::None, Paint::solid(Color::rgb(0.08, 0.08, 0.08)), 0.7),
        )));
    }

    // 2. Entries
    for (i, entry) in table.entries.iter().enumerate() {
        let row_y = b.y0 + 6.0 + (i as f64) * opt.row_height_pt;
        let swatch_x0 = b.x0 + 4.0;
        let swatch_x1 = swatch_x0 + opt.swatch_width_pt;
        let swatch_cy = row_y + opt.row_height_pt / 2.0;

        // Sample swatch
        match &entry.sample_kind {
            LegendSample::Line => {
                let weight = entry.weight_pt.unwrap_or(0.5);
                let swatch_line = PathData::single(SubPath::polyline(
                    &[Point::new(swatch_x0, swatch_cy), Point::new(swatch_x1, swatch_cy)],
                    false,
                ));
                let mut app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), weight);
                if let Some(dashes) = entry.line_type.dash_pt() {
                    if let Some(st) = app.stroke_mut() {
                        st.dash = Some(vectorcraft_doc::Dash { pattern: dashes, ..Default::default() });
                    }
                }
                nodes.push(Arc::new(Node::path(alloc_id(), swatch_line, app)));
            }
            LegendSample::Area => {
                let sh = opt.swatch_height_pt;
                let swatch_rect = PathData::single(SubPath::polyline(&[
                    Point::new(swatch_x0, swatch_cy - sh / 2.0),
                    Point::new(swatch_x1, swatch_cy - sh / 2.0),
                    Point::new(swatch_x1, swatch_cy + sh / 2.0),
                    Point::new(swatch_x0, swatch_cy + sh / 2.0),
                ], true));
                let app = Appearance::basic(Paint::solid(Color::rgb(0.92, 0.92, 0.90)), Paint::solid(Color::BLACK), 0.4);
                nodes.push(Arc::new(Node::path(alloc_id(), swatch_rect, app)));
            }
            LegendSample::Mark(mark_str) => {
                let text_node = Node::new(
                    alloc_id(),
                    NodeKind::Text(Box::new(TextObject::point(
                        Point::new((swatch_x0 + swatch_x1) / 2.0 - 4.0, swatch_cy + 3.0),
                        mark_str,
                        CharStyle { size: 8.0, ..Default::default() },
                    ))),
                );
                nodes.push(Arc::new(text_node));
            }
        }

        // Label text (8 pt)
        let label_x = swatch_x1 + opt.swatch_gap_pt;
        let label_node = Node::new(
            alloc_id(),
            NodeKind::Text(Box::new(TextObject::point(
                Point::new(label_x, swatch_cy + 3.0),
                &entry.label,
                CharStyle { size: opt.font_size_pt, ..Default::default() },
            ))),
        );
        nodes.push(Arc::new(label_node));
    }

    // 3. Bottom bounding rule (0.7 pt)
    if opt.bottom_rule {
        let bottom_path = PathData::single(SubPath::polyline(
            &[Point::new(b.x0, b.y1), Point::new(b.x1, b.y1)],
            false,
        ));
        nodes.push(Arc::new(Node::path(
            alloc_id(),
            bottom_path,
            Appearance::basic(Paint::None, Paint::solid(Color::rgb(0.08, 0.08, 0.08)), 0.7),
        )));
    }

    nodes
}

fn generate_legend(s: &mut Session, p: &Value) -> Result<Value> {
    let mut opts = LegendLayoutOptions::default();

    if let Some(arr) = p.get("at").and_then(Value::as_array) {
        if arr.len() >= 2 {
            opts.origin = Point::new(arr[0].as_f64().unwrap_or(0.0), arr[1].as_f64().unwrap_or(0.0));
        }
    }
    if let Some(w) = p.get("width").and_then(Value::as_f64) {
        opts.table_width_pt = w;
    }
    if let Some(c) = p.get("columns").and_then(Value::as_u64) {
        opts.column_count = (c as usize).max(1);
    }
    if let Some(f) = p.get("filterUnused").or_else(|| p.get("filter_unused")).and_then(Value::as_bool) {
        opts.filter_unused = f;
    }

    let replace_existing = p.get("replaceExisting").or_else(|| p.get("replace_existing")).and_then(Value::as_bool).unwrap_or(true);

    let doc_st = s.doc()?;
    let table = LegendTable::build_from_document(&doc_st.doc, &opts);
    let entry_count = table.entries.len();
    let bounds = table.bounds;

    let mut created_node_id = NodeId(0);

    s.edit("Generate Auto Legend Table", |doc, sel| {
        // If replace_existing, remove any existing legend table node
        if replace_existing {
            for layer in &mut doc.layers {
                if let Some(children) = Arc::make_mut(layer).children_mut() {
                    children.retain(|c| c.name.as_deref() != Some(LEGEND_TABLE_TAG));
                }
            }
        }

        let mut alloc = || doc.alloc_id();
        let group_id = alloc();
        created_node_id = group_id;

        let child_nodes = build_legend_vector_nodes(&table, &mut alloc);
        let mut group_node = Node::group(group_id, child_nodes);
        group_node.name = Some(LEGEND_TABLE_TAG.to_string());

        // Place into active layer or topmost layer
        let target_layer = doc.layers.last_mut();
        if let Some(layer) = target_layer {
            if let Some(children) = Arc::make_mut(layer).children_mut() {
                children.push(Arc::new(group_node));
            }
        }

        sel.objects = vec![group_id];
        Ok(())
    })?;

    Ok(json!({
        "nodeId": created_node_id.0,
        "entryCount": entry_count,
        "bounds": [bounds.x0, bounds.y0, bounds.width(), bounds.height()]
    }))
}

fn update_legend(s: &mut Session, p: &Value) -> Result<Value> {
    let doc_st = s.doc()?;
    let mut found_pos = None;

    // Search for existing legend table node
    let target_id = p.get("tableNodeId").or_else(|| p.get("table_node_id")).and_then(Value::as_u64).map(NodeId);

    for layer in &doc_st.doc.layers {
        if let Some(ch) = layer.children() {
            for c in ch {
                if target_id == Some(c.id) || (target_id.is_none() && c.name.as_deref() == Some(LEGEND_TABLE_TAG)) {
                    if let Some(b) = c.geometric_bounds() {
                        found_pos = Some((c.id, Point::new(b.x0, b.y0), b.width()));
                        break;
                    }
                }
            }
        }
    }

    let Some((_node_id, origin, width)) = found_pos else {
        return Err(bad("style.updateLegend", "no existing legend table found on canvas"));
    };

    let opts = LegendLayoutOptions {
        origin,
        table_width_pt: width,
        ..Default::default()
    };

    let table = LegendTable::build_from_document(&doc_st.doc, &opts);
    let entry_count = table.entries.len();
    let bounds = table.bounds;

    s.edit("Update Legend Table", |doc, sel| {
        for layer in &mut doc.layers {
            if let Some(children) = Arc::make_mut(layer).children_mut() {
                children.retain(|c| c.name.as_deref() != Some(LEGEND_TABLE_TAG));
            }
        }

        let mut alloc = || doc.alloc_id();
        let group_id = alloc();
        let child_nodes = build_legend_vector_nodes(&table, &mut alloc);
        let mut group_node = Node::group(group_id, child_nodes);
        group_node.name = Some(LEGEND_TABLE_TAG.to_string());

        if let Some(layer) = doc.layers.last_mut() {
            if let Some(children) = Arc::make_mut(layer).children_mut() {
                children.push(Arc::new(group_node));
            }
        }

        sel.objects = vec![group_id];
        Ok(())
    })?;

    Ok(json!({
        "updated": true,
        "entryCount": entry_count,
        "bounds": [bounds.x0, bounds.y0, bounds.width(), bounds.height()]
    }))
}
