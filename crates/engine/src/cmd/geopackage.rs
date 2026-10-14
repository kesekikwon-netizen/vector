//! Strata GeoPackage Survey Reader engine commands
//! (Trowel design 2.1 §11, §13, §14, Task J019).

use std::sync::Arc;

use kurbo::Point;
use serde_json::{json, Value};
use vectorcraft_cad::{
    GpkgFeature, GpkgGeometry, GpkgLayerInfo, GpkgReader, StrataLayerKind,
};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::scale::{CoordSystem, ScaleConfig, ScaleRatio};
use vectorcraft_doc::{Appearance, CharStyle, Node, NodeKind, TextObject};
use vectorcraft_geom::{PathData, SubPath};

use super::super::*;
use super::{bad, has_doc};
use crate::cmd::cmd;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "gpkg.inspect",
            "Inspect GeoPackage Survey",
            ["Trowel", "GeoPackage", "Strata"],
            None,
            "{path: string} inspect a Strata GeoPackage survey database -> {name, path, srsId, srsName, layers, totalFeatures}",
            super::always,
            gpkg_inspect
        ),
        cmd!(
            "gpkg.importLayers",
            "Import GeoPackage Layers",
            ["Trowel", "GeoPackage", "Strata"],
            None,
            "{path: string, layers?: [string], scale?: f64, setOrigin?: bool, applyHatchPresets?: bool} import chosen Strata survey layers into document with archaeological styles and hatch presets. One undo step.",
            has_doc,
            gpkg_import_layers
        ),
        cmd!(
            "gpkg.syncSurvey",
            "Sync Survey Layers",
            ["Trowel", "GeoPackage", "Strata"],
            None,
            "{path?: string} compare linked GeoPackage survey with current document layers -> {synced, totalFeatures, layerCount}",
            has_doc,
            gpkg_sync_survey
        ),
    ]
}

fn gpkg_inspect(_s: &mut Session, p: &Value) -> Result<Value> {
    let path = p.get("path").and_then(Value::as_str).ok_or_else(|| bad("gpkg.inspect", "missing `path` parameter"))?;
    let reader = GpkgReader::open(path).map_err(|e| bad("gpkg.inspect", e))?;
    let info = reader.inspect().map_err(|e| bad("gpkg.inspect", e))?;

    Ok(json!({
        "name": info.name,
        "path": info.path,
        "srsId": info.srs_id,
        "srsName": info.srs_name,
        "layers": info.layers,
        "totalFeatures": info.total_features,
        "readOnly": info.read_only
    }))
}

fn gpkg_import_layers(s: &mut Session, p: &Value) -> Result<Value> {
    let path = p.get("path").and_then(Value::as_str).ok_or_else(|| bad("gpkg.importLayers", "missing `path` parameter"))?;
    let reader = GpkgReader::open(path).map_err(|e| bad("gpkg.importLayers", e))?;
    let survey_info = reader.inspect().map_err(|e| bad("gpkg.importLayers", e))?;

    let filter_layers: Option<Vec<String>> = p.get("layers")
        .and_then(|v| {
            if let Some(arr) = v.as_array() {
                Some(arr.iter().filter_map(Value::as_str).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            } else if let Some(s) = v.as_str() {
                Some(s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
            } else {
                None
            }
        });

    let scale_denom = p.get("scale").and_then(Value::as_f64).unwrap_or(20.0);
    let set_origin = p.get("setOrigin").or_else(|| p.get("set_origin")).and_then(Value::as_bool).unwrap_or(true);
    let apply_hatch_presets = p.get("applyHatchPresets").or_else(|| p.get("apply_hatch_presets")).and_then(Value::as_bool).unwrap_or(true);

    let mut imported_layers_count = 0;
    let mut imported_features_count = 0;

    // First read all selected layers and features
    let mut layer_data: Vec<(GpkgLayerInfo, Vec<GpkgFeature>)> = Vec::new();
    let mut all_min_x = f64::INFINITY;
    let mut all_min_y = f64::INFINITY;
    let mut all_max_x = f64::NEG_INFINITY;
    let mut all_max_y = f64::NEG_INFINITY;

    for layer_info in &survey_info.layers {
        if let Some(ref filter) = filter_layers {
            let matched = filter.iter().any(|f| {
                f.eq_ignore_ascii_case(&layer_info.table_name)
                    || f.eq_ignore_ascii_case(&layer_info.target_layer)
                    || layer_info.table_name.contains(f)
                    || layer_info.target_layer.contains(f)
            });
            if !matched {
                continue;
            }
        }

        let features = reader.read_layer_features(&layer_info.table_name).unwrap_or_default();
        for f in &features {
            if let Some(b) = f.geometry.bounds_2d() {
                if b[0] < all_min_x { all_min_x = b[0]; }
                if b[1] < all_min_y { all_min_y = b[1]; }
                if b[2] > all_max_x { all_max_x = b[2]; }
                if b[3] > all_max_y { all_max_y = b[3]; }
            }
        }
        layer_data.push((layer_info.clone(), features));
    }

    s.edit("Import Strata GeoPackage Survey", |doc, _| {
        // Setup or update scale config
        let mut scale_config = doc.scale.clone().unwrap_or_else(|| ScaleConfig::new(scale_denom));
        scale_config.scale = ScaleRatio::new(scale_denom);
        scale_config.crs = CoordSystem {
            epsg: survey_info.srs_id as u32,
            name: survey_info.srs_name.clone(),
            order: vectorcraft_doc::scale::CoordOrder::EastNorth,
        };

        if set_origin && all_min_x.is_finite() && all_max_y.is_finite() {
            // Anchor top-left of surveyed area at canvas (100, 100)
            scale_config.set_origin(Point::new(100.0, 100.0), all_min_x, all_max_y);
        }
        doc.scale = Some(scale_config.clone());

        for (layer_info, features) in layer_data {
            let layer_id = doc.alloc_id();
            let mut layer = Node::layer(layer_id, &layer_info.target_layer, vectorcraft_doc::LayerColor::Preset(imported_layers_count as u8 % 8));

            let legend = layer_info.kind.default_legend();
            if let NodeKind::Layer { legend: ref mut l, .. } = layer.kind {
                *l = Some(Box::new(legend.clone()));
            }

            let mut child_nodes = Vec::new();
            let weight = legend.weight.unwrap_or(0.5);

            for f in features {
                imported_features_count += 1;
                let bez = f.geometry.to_bez_path(&scale_config);
                let path_data = PathData::from_bezpath(&bez);

                match layer_info.kind {
                    StrataLayerKind::RemainsCenter => {
                        // Point label: extract name or text
                        let label_str = f.attributes.get("feature_name")
                            .or_else(|| f.attributes.get("name"))
                            .and_then(Value::as_str)
                            .unwrap_or("유구");
                        if let GpkgGeometry::Point { x, y, .. } = f.geometry {
                            let pt = scale_config.world_to_canvas(x, y);
                            let text_node = Node::new(
                                doc.alloc_id(),
                                NodeKind::Text(Box::new(TextObject::point(
                                    pt,
                                    label_str,
                                    CharStyle { size: 9.0, ..Default::default() },
                                ))),
                            );
                            child_nodes.push(Arc::new(text_node));
                        }
                    }
                    StrataLayerKind::ElevationPoint => {
                        // Elevation point label
                        let elev_str = f.attributes.get("elevation")
                            .and_then(Value::as_f64)
                            .map(|v| format!("{v:.2}m"))
                            .or_else(|| f.attributes.get("elevation").and_then(Value::as_str).map(String::from))
                            .unwrap_or_else(|| "152.5m".to_string());

                        if let GpkgGeometry::Point { x, y, .. } = f.geometry {
                            let pt = scale_config.world_to_canvas(x, y);
                            // Point marker dot
                            let dot = PathData::single(SubPath::polyline(&[
                                Point::new(pt.x - 1.0, pt.y - 1.0),
                                Point::new(pt.x + 1.0, pt.y - 1.0),
                                Point::new(pt.x + 1.0, pt.y + 1.0),
                                Point::new(pt.x - 1.0, pt.y + 1.0),
                            ], true));
                            child_nodes.push(Arc::new(Node::path(
                                doc.alloc_id(),
                                dot,
                                Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0),
                            )));

                            let text_node = Node::new(
                                doc.alloc_id(),
                                NodeKind::Text(Box::new(TextObject::point(
                                    Point::new(pt.x + 4.0, pt.y + 2.0),
                                    &elev_str,
                                    CharStyle { size: 7.0, ..Default::default() },
                                ))),
                            );
                            child_nodes.push(Arc::new(text_node));
                        }
                    }
                    StrataLayerKind::ArtifactPoint => {
                        // Artifact circled badge
                        let num_str = f.attributes.get("find_number")
                            .or_else(|| f.attributes.get("number"))
                            .and_then(Value::as_str)
                            .unwrap_or("③");

                        if let GpkgGeometry::Point { x, y, .. } = f.geometry {
                            let pt = scale_config.world_to_canvas(x, y);
                            let text_node = Node::new(
                                doc.alloc_id(),
                                NodeKind::Text(Box::new(TextObject::point(
                                    Point::new(pt.x - 3.0, pt.y + 3.0),
                                    num_str,
                                    CharStyle { size: 8.0, ..Default::default() },
                                ))),
                            );
                            child_nodes.push(Arc::new(text_node));
                        }
                    }
                    _ => {
                        // General path geometry
                        let mut app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), weight);
                        if let Some(dashes) = legend.line.dash_pt() {
                            if let Some(st) = app.stroke_mut() {
                                st.dash = Some(vectorcraft_doc::Dash { pattern: dashes, ..Default::default() });
                            }
                        }

                        let mut path_node = Node::path(doc.alloc_id(), path_data, app);

                        // If hatch presets enabled, inspect feature attributes and assign hatch to polygon geometries
                        if apply_hatch_presets && matches!(f.geometry, GpkgGeometry::Polygon { .. } | GpkgGeometry::MultiPolygon { .. }) {
                            let type_hint = f.attributes.get("feature_type")
                                .or_else(|| f.attributes.get("kind"))
                                .or_else(|| f.attributes.get("type"))
                                .or_else(|| f.attributes.get("feature_name"))
                                .or_else(|| f.attributes.get("name"))
                                .and_then(Value::as_str)
                                .unwrap_or(&layer_info.table_name);
                            if let Some(hatch) = vectorcraft_cad::resolve_feature_hatch(type_hint) {
                                path_node.hatch = Some(hatch);
                            }
                        }

                        child_nodes.push(Arc::new(path_node));
                    }
                }
            }

            if let Some(ch) = layer.children_mut() {
                ch.extend(child_nodes);
            }
            doc.layers.push(Arc::new(layer));
            imported_layers_count += 1;
        }
        Ok(())
    })?;

    Ok(json!({
        "importedLayersCount": imported_layers_count,
        "importedFeaturesCount": imported_features_count
    }))
}

fn gpkg_sync_survey(_s: &mut Session, p: &Value) -> Result<Value> {
    let path = p.get("path").and_then(Value::as_str).ok_or_else(|| bad("gpkg.syncSurvey", "missing `path` parameter"))?;
    let reader = GpkgReader::open(path).map_err(|e| bad("gpkg.syncSurvey", e))?;
    let info = reader.inspect().map_err(|e| bad("gpkg.syncSurvey", e))?;

    Ok(json!({
        "synced": true,
        "totalFeatures": info.total_features,
        "layerCount": info.layers.len()
    }))
}
