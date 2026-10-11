//! Trowel legend rows (J005) in `.vectorcraft` files: a layer's legend saves and opens again,
//! layers without one save as before, and files from before legends open with none.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::{Value, json};
use vectorcraft_doc::{Document, Legend, LegendGroup, LegendSample, LineType, NodeKind};
use vectorcraft_format::{VERSION, load, save};

/// The saved layer's kind object (where its layer fields are).
fn layer_json(v: &Value) -> &Value {
    &v["document"]["layers"][0]["kind"]
}

fn set_legend(d: &mut Document, legend: Legend) {
    let id = d.layers[0].id;
    if let Some(NodeKind::Layer { legend: l, .. }) = d.node_mut(id).map(|n| &mut n.kind) {
        *l = Some(Box::new(legend));
    }
}

#[test]
fn a_legend_survives_save_and_open() {
    for legend in [
        Legend {
            weight: Some(0.8),
            line: LineType::DashDot,
            favourite: true,
            sample: LegendSample::Area,
            strata: Some("feature_poly".into()),
            group: Some(LegendGroup::Plan),
        },
        Legend { weight: Some(0.35), line: LineType::Custom(vec![2.5, 1.25]), sample: LegendSample::Mark("③".into()), ..Legend::default() },
        Legend::default(),
    ] {
        let mut d = Document::new(200.0, 100.0);
        set_legend(&mut d, legend.clone());
        let back = load(&save(&d, false)).unwrap();
        assert_eq!(back.layers[0].legend(), Some(&legend));
        assert_eq!(back, d);
    }
}

#[test]
fn a_layer_without_a_legend_saves_as_before() {
    let d = Document::new(200.0, 100.0);
    let v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    assert_eq!(v["version"], json!(4), "the file format version stays 4");
    assert_eq!(VERSION, 4);
    assert_eq!(layer_json(&v)["type"], json!("layer"));
    assert!(layer_json(&v).get("legend").is_none(), "no legend key: {}", layer_json(&v));
    let mut with = d.clone();
    set_legend(&mut with, Legend { weight: Some(0.5), ..Legend::default() });
    let v: Value = serde_json::from_slice(&save(&with, false)).unwrap();
    assert_eq!(layer_json(&v)["legend"], json!({"weight": 0.5}), "{}", layer_json(&v));
}

#[test]
fn older_files_open_without_a_legend() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(10.0, 10.0), false)).unwrap();
    let layer = &mut v["document"]["layers"][0]["kind"];
    assert_eq!(layer["type"], json!("layer"));
    assert!(layer.get("legend").is_none());
    let d = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(d.layers[0].legend(), None);
}

/// Older builds read layers with serde's default rule (unknown keys are skipped), which is what
/// lets them open files with legends: a key this build doesn't know opens the same way.
#[test]
fn unknown_layer_fields_are_ignored() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(10.0, 10.0), false)).unwrap();
    v["document"]["layers"][0]["kind"]["somethingNewer"] = json!({"weight": 3});
    v["document"]["layers"][0]["somethingNewer"] = json!(true);
    let d = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(d.layers.len(), 1);
    assert!(d.layers[0].is_layer());
}
