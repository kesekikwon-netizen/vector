//! Integration tests for Strata GeoPackage Survey Reader (Task J019).
#![allow(clippy::all)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::json;
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
