//! Strata GeoPackage Survey Browser UI panel & dialog (Trowel design 2.1 §11, §13, §14, Task J019).
#![allow(dead_code)]

use std::collections::BTreeSet;

use egui::vec2;
use serde_json::{Value, json};

use super::*;
use crate::VectorcraftApp;

pub(crate) fn gpkg_dialog_open_id() -> egui::Id {
    wid("gpkg-dialog-open", ())
}

pub(crate) fn gpkg_path_input_id() -> egui::Id {
    wid("gpkg-path-input", ())
}

pub(crate) fn gpkg_selected_layers_id() -> egui::Id {
    wid("gpkg-selected-layers", ())
}

pub(crate) fn gpkg_inspected_layers_id() -> egui::Id {
    wid("gpkg-inspected-layers", ())
}

/// Standard 7 Strata layers in Gwangnyeong-ri 1 survey.
pub(crate) const DEFAULT_STRATA_LAYERS: &[(&str, &str)] = &[
    ("remains_pit", "수혈유구 (실선 0.5 pt, 갈색 계열)"),
    ("dwelling_perimeter", "주거지 외곽선 (외곽윤곽 0.8 pt)"),
    ("stone_wall", "석축/배수로 (0.5 pt, 점선)"),
    ("contours", "등고선/지형 (보조선 0.2 pt)"),
    ("remains_center", "유구 중심점 (문자 명칭 라벨 9 pt)"),
    ("elevation_points", "표고점 (표고 수치 라벨 7 pt, 점 마커)"),
    ("artifact_finds", "유물출토지 (원형 기호 번호 8 pt)"),
];

/// Open the Strata GeoPackage Survey Browser dialog (an item ending in "…" opens, it doesn't
/// toggle).
pub fn open_dialog(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(gpkg_dialog_open_id(), true));
}

/// Open or toggle the Strata GeoPackage Survey Browser dialog.
pub fn toggle_dialog(ctx: &egui::Context) {
    let cur: bool = ctx.data(|d| d.get_temp(gpkg_dialog_open_id())).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(gpkg_dialog_open_id(), !cur));
}

/// Renders the GeoPackage Survey Browser Dialog.
pub fn show_dialog(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let is_open: bool = ctx.data(|d| d.get_temp(gpkg_dialog_open_id())).unwrap_or(false);
    if !is_open {
        return;
    }

    let mut open_flag = is_open;
    let mut close_requested = false;

    egui::Window::new("Strata GeoPackage 발굴조사 데이터 연동 (J019)")
        .id(wid("gpkg-browser-window", ()))
        .open(&mut open_flag)
        .resizable(true)
        .default_size(vec2(480.0, 520.0))
        .show(ctx, |ui| {
            if app.session.active().is_none() {
                ui.label("열린 문서가 없습니다.");
                return;
            }

            ui.heading("Strata OGC GeoPackage 현장 측량 데이터 연동");
            ui.label("광령리 1호 유적 등 현장 RTK-GNSS/광파기 지리공간 조사 데이터를 안전하게 가져옵니다.");
            ui.separator();

            let mut gpkg_path: String = ui.data(|d| d.get_temp(gpkg_path_input_id())).unwrap_or_else(|| "strata_survey.gpkg".to_string());
            ui.horizontal(|ui| {
                ui.label("GeoPackage 파일:");
                ui.text_edit_singleline(&mut gpkg_path);
            });
            ui.data_mut(|d| d.insert_temp(gpkg_path_input_id(), gpkg_path.clone()));

            ui.horizontal(|ui| {
                if ui.button("조사 데이터 분석 (Inspect)").clicked()
                    && let Ok(info) = app.run("gpkg.inspect", json!({ "path": gpkg_path }))
                    && let Some(arr) = info.get("layers").and_then(Value::as_array)
                {
                    let mut detected: Vec<(String, String)> = Vec::new();
                    for item in arr {
                        let tbl = item.get("table_name").and_then(Value::as_str).unwrap_or("");
                        if tbl.is_empty() {
                            continue;
                        }
                        let desc = item.get("target_layer").and_then(Value::as_str).unwrap_or(tbl);
                        let count = item.get("feature_count").and_then(Value::as_u64).unwrap_or(0);
                        detected.push((tbl.to_string(), format!("{desc} ({count}개 객체)")));
                    }
                    if !detected.is_empty() {
                        let all_names: BTreeSet<String> = detected.iter().map(|(tbl, _)| tbl.clone()).collect();
                        ui.data_mut(|d| {
                            d.insert_temp(gpkg_inspected_layers_id(), detected);
                            d.insert_temp(gpkg_selected_layers_id(), all_names);
                        });
                    }
                }

                if ui.button("조사 데이터 동기화 (Sync)").on_hover_text("외부 현장 조사 파일의 최신 수정사항을 검사합니다.").clicked()
                {
                    let _ = app.run("gpkg.syncSurvey", json!({ "path": gpkg_path }));
                }
            });

            ui.separator();

            let inspected: Option<Vec<(String, String)>> = ui.data(|d| d.get_temp(gpkg_inspected_layers_id()));
            let layers_to_show: Vec<(String, String)> =
                inspected.unwrap_or_else(|| DEFAULT_STRATA_LAYERS.iter().map(|(tbl, desc)| ((*tbl).to_string(), (*desc).to_string())).collect());

            let mut selected_layers: BTreeSet<String> =
                ui.data(|d| d.get_temp(gpkg_selected_layers_id())).unwrap_or_else(|| layers_to_show.iter().map(|(tbl, _)| tbl.clone()).collect());

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("광령리 1호 유적 표준 레이어 (선택 가능)").strong());
                if ui.small_button("전체 선택").clicked() {
                    for (tbl, _) in &layers_to_show {
                        selected_layers.insert(tbl.clone());
                    }
                }
                if ui.small_button("전체 해제").clicked() {
                    selected_layers.clear();
                }
                ui.label(format!("({}/{} 선택)", selected_layers.len(), layers_to_show.len()));
            });

            for (table_name, desc) in &layers_to_show {
                let mut checked = selected_layers.contains(table_name);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut checked, table_name).changed() {
                        if checked {
                            selected_layers.insert(table_name.clone());
                        } else {
                            selected_layers.remove(table_name);
                        }
                    }
                    ui.label(desc);
                });
            }

            ui.data_mut(|d| d.insert_temp(gpkg_selected_layers_id(), selected_layers.clone()));

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("기본 도면 축척:");
                ui.label("1:20 (상세도)");
            });

            ui.horizontal(|ui| {
                let selected_vec: Vec<String> = selected_layers.iter().cloned().collect();
                let import_enabled = !selected_vec.is_empty();
                if ui.add_enabled(import_enabled, egui::Button::new("선택한 레이어 도면으로 가져오기 (Import)")).clicked() {
                    let _ = app.run(
                        "gpkg.importLayers",
                        json!({
                            "path": gpkg_path,
                            "layers": selected_vec,
                            "scale": 20.0,
                            "setOrigin": true,
                            "applyHatchPresets": true,
                        }),
                    );
                }

                if ui.button("닫기").clicked() {
                    close_requested = true;
                }
            });
        });

    let final_open = open_flag && !close_requested;
    ctx.data_mut(|d| d.insert_temp(gpkg_dialog_open_id(), final_open));
}
