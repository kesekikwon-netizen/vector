//! Strata GeoPackage Survey Browser UI panel & dialog (Trowel design 2.1 §11, §13, §14, Task J019).
#![allow(dead_code)]

use egui::vec2;
use serde_json::json;

use super::*;
use crate::VectorcraftApp;

fn gpkg_dialog_open_id() -> egui::Id {
    wid("gpkg-dialog-open", ())
}

fn gpkg_path_input_id() -> egui::Id {
    wid("gpkg-path-input", ())
}

/// Open or toggle the Strata GeoPackage Survey Browser dialog.
pub fn toggle_dialog(ctx: &egui::Context) {
    let cur: bool = ctx.data(|d| d.get_temp(gpkg_dialog_open_id())).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(gpkg_dialog_open_id(), !cur));
}

/// Renders the GeoPackage Survey Browser Dialog.
pub fn show_dialog(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let mut is_open: bool = ctx.data(|d| d.get_temp(gpkg_dialog_open_id())).unwrap_or(false);
    if !is_open {
        return;
    }

    egui::Window::new("Strata GeoPackage 발굴조사 데이터 연동 (J019)")
        .id(wid("gpkg-browser-window", ()))
        .open(&mut is_open)
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
                if ui.button("조사 데이터 분석 (Inspect)").clicked() {
                    let _ = app.run("gpkg.inspect", json!({ "path": gpkg_path }));
                }

                if ui.button("조사 데이터 동기화 (Sync)").on_hover_text("외부 현장 조사 파일의 최신 수정사항을 검사합니다.").clicked() {
                    let _ = app.run("gpkg.syncSurvey", json!({ "path": gpkg_path }));
                }
            });

            ui.separator();

            ui.label(egui::RichText::new("광령리 1호 유적 표준 레이어 (7종)").strong());
            let strata_layers = [
                ("remains_pit", "수혈유구 (실선 0.5 pt, 갈색 계열)"),
                ("dwelling_perimeter", "주거지 외곽선 (외곽윤곽 0.8 pt)"),
                ("stone_wall", "석축/배수로 (0.5 pt, 점선)"),
                ("contours", "등고선/지형 (보조선 0.2 pt)"),
                ("remains_center", "유구 중심점 (문자 명칭 라벨 9 pt)"),
                ("elevation_points", "표고점 (표고 수치 라벨 7 pt, 점 마커)"),
                ("artifact_finds", "유물출토지 (원형 기호 번호 8 pt)"),
            ];

            for (table_name, desc) in strata_layers {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut true, table_name);
                    ui.label(desc);
                });
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("기본 도면 축척:");
                ui.label("1:20 (상세도)");
            });

            ui.horizontal(|ui| {
                if ui.button("선택한 레이어 도면으로 가져오기 (Import)").clicked() {
                    let _ = app.run(
                        "gpkg.importLayers",
                        json!({
                            "path": gpkg_path,
                            "scale": 20.0,
                            "setOrigin": true,
                            "applyHatchPresets": true,
                        }),
                    );
                }

                if ui.button("닫기").clicked() {
                    let _ = false;
                }
            });
        });

    ctx.data_mut(|d| d.insert_temp(gpkg_dialog_open_id(), is_open));
}
