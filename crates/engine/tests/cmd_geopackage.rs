//! Integration tests for Strata GeoPackage Survey Reader (Task J019).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
use vectorcraft_doc::hatch::HatchDirection;
use vectorcraft_engine::Session;

fn test_session() -> Session {
    let mut s = Session::new();
    let _ = s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    s
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_gpkg_survey_inspect_and_import() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_survey_{}.gpkg", now));
    vectorcraft_cad::create_sample_gpkg(&db_path).unwrap();

    let path_str = db_path.to_str().unwrap();
    let mut s = test_session();

    // 1. Test gpkg.inspect query command
    let info = s.execute("gpkg.inspect", &json!({ "path": path_str })).unwrap();
    assert_eq!(info["srsId"], 5186);
    assert_eq!(info["srsName"], "Korea 2000 / Central Belt 2010");
    assert_eq!(info["totalFeatures"], 2);
    let layers = info["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 2);

    // 2. Test gpkg.importLayers command
    let import_res = s
        .execute(
            "gpkg.importLayers",
            &json!({
                "path": path_str,
                "scale": 20.0,
                "setOrigin": true,
                "applyHatchPresets": true
            }),
        )
        .unwrap();

    assert_eq!(import_res["importedLayersCount"], 2);
    assert_eq!(import_res["importedFeaturesCount"], 2);

    let doc = &s.doc().unwrap().doc;
    assert!(doc.layers.iter().any(|l| l.display_name().contains("주거지")));
    assert!(doc.layers.iter().any(|l| l.display_name().contains("표고점")));
    assert!(doc.scale.is_some());
    assert_eq!(doc.scale.as_ref().unwrap().crs.epsg, 5186);

    // 3. Test gpkg.syncSurvey command
    let sync_res = s.execute("gpkg.syncSurvey", &json!({ "path": path_str })).unwrap();
    assert_eq!(sync_res["synced"], true);
    assert_eq!(sync_res["totalFeatures"], 2);

    // 4. Test Undo rolls back import in a single step
    s.execute("edit.undo", &json!({})).unwrap();
    let doc_undone = &s.doc().unwrap().doc;
    assert!(!doc_undone.layers.iter().any(|l| l.display_name().contains("주거지")));

    // Clean up temporary database file
    std::fs::remove_file(&db_path).ok();
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_gpkg_feature_hatch_and_7_strata_layers_survey() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_7layers_{}.gpkg", now));

    // Create a 7-layer survey database via CAD helper
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    let path_str = db_path.to_str().unwrap();
    let mut s = test_session();

    // 1. Test inspect recognizes all 7 Strata layers
    let inspect_info = s.execute("gpkg.inspect", &json!({ "path": path_str })).unwrap();
    let layers = inspect_info["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 7, "All 7 Strata survey layers must be cataloged");

    let layer_names: Vec<&str> = layers.iter().map(|l| l["table_name"].as_str().unwrap()).collect();
    assert!(layer_names.contains(&"tr_investigation_boundary"));
    assert!(layer_names.contains(&"tr_grid"));
    assert!(layer_names.contains(&"tr_remains_polygon"));
    assert!(layer_names.contains(&"tr_remains_center"));
    assert!(layer_names.contains(&"tr_elevation_point"));
    assert!(layer_names.contains(&"tr_strata_section"));
    assert!(layer_names.contains(&"tr_artifact_point"));

    // 2. Import layers with applyHatchPresets: true
    let import_res = s.execute("gpkg.importLayers", &json!({
        "path": path_str,
        "applyHatchPresets": true
    })).unwrap();
    assert_eq!(import_res["importedLayersCount"], 7);

    // 3. Empirically inspect whether hatch presets were attached to imported nodes
    let doc = &s.doc().unwrap().doc;
    let mut node_with_hatch_count = 0;
    let mut pit_hatch_direction = None;
    for layer in &doc.layers {
        if let Some(children) = layer.children() {
            for child in children {
                if let Some(h) = &child.hatch {
                    node_with_hatch_count += 1;
                    pit_hatch_direction = Some(h.direction);
                }
            }
        }
    }

    assert!(node_with_hatch_count >= 1, "At least 1 polygon feature (수혈) must receive a hatch preset");
    assert_eq!(pit_hatch_direction, Some(HatchDirection::Deg45), "수혈 feature must receive Deg45 hatch");

    // Clean up
    std::fs::remove_file(&db_path).ok();
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_gpkg_import_layers_filter() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let db_path = std::env::temp_dir().join(format!("test_filter_{}.gpkg", now));
    vectorcraft_cad::create_7layers_sample_gpkg(&db_path).unwrap();

    let path_str = db_path.to_str().unwrap();
    let mut s = test_session();

    // 1. Import single layer
    let res1 = s.execute("gpkg.importLayers", &json!({
        "path": path_str,
        "layers": ["tr_remains_polygon"]
    })).unwrap();
    assert_eq!(res1["importedLayersCount"], 1);

    // 2. Import two layers
    let mut s2 = test_session();
    let res2 = s2.execute("gpkg.importLayers", &json!({
        "path": path_str,
        "layers": ["tr_investigation_boundary", "tr_grid"]
    })).unwrap();
    assert_eq!(res2["importedLayersCount"], 2);

    // 3. Import with applyHatchPresets: false
    let mut s3 = test_session();
    let res3 = s3.execute("gpkg.importLayers", &json!({
        "path": path_str,
        "applyHatchPresets": false
    })).unwrap();
    assert_eq!(res3["importedLayersCount"], 7);
    let doc3 = &s3.doc().unwrap().doc;
    let mut hatch_count3 = 0;
    for layer in &doc3.layers {
        if let Some(children) = layer.children() {
            for child in children {
                if child.hatch.is_some() {
                    hatch_count3 += 1;
                }
            }
        }
    }
    assert_eq!(hatch_count3, 0, "No nodes should receive hatch when applyHatchPresets is false");

    std::fs::remove_file(&db_path).ok();
}


