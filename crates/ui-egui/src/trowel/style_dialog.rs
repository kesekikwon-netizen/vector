//! Style Template Diff & Merge Dialog and Dynamic Auto Legend UI (Trowel design 2.1 §6, §7, §8, §10, Task J018).
#![allow(dead_code)]

use egui::{Color32, vec2};
use serde_json::json;
use vectorcraft_doc::style_template::TrowelStyleTemplate;

use super::*;
use crate::VectorcraftApp;

fn style_dialog_open_id() -> egui::Id {
    wid("style-dialog-open", ())
}

fn style_template_json_id() -> egui::Id {
    wid("style-template-json-buffer", ())
}

/// Open or toggle the Style Template & Diff dialog.
pub fn toggle_dialog(ctx: &egui::Context) {
    let cur: bool = ctx.data(|d| d.get_temp(style_dialog_open_id())).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(style_dialog_open_id(), !cur));
}

/// Renders the Style Template & Auto Legend dialog.
pub fn show_dialog(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let mut is_open: bool = ctx.data(|d| d.get_temp(style_dialog_open_id())).unwrap_or(false);
    if !is_open {
        return;
    }

    egui::Window::new("고고학 양식 템플릿 (.trowel-style) 및 범례표 (J018)")
        .id(wid("style-template-window", ()))
        .open(&mut is_open)
        .resizable(true)
        .default_size(vec2(520.0, 560.0))
        .show(ctx, |ui| {
            if app.session.active().is_none() {
                ui.label("열린 문서가 없습니다.");
                return;
            }

            ui.heading("도면 양식 템플릿 차이 분석 및 병합");
            ui.label("기관·연구원별 표준 선 굵기 계층, 토층 해치 프리셋, 유구 레이어 양식을 비교·적용합니다.");
            ui.separator();

            // Export current doc styles button
            ui.horizontal(|ui| {
                if ui.button("현재 문서 양식 내보내기 (.trowel-style)").clicked() {
                    let _ = app.run(
                        "style.exportTemplate",
                        json!({
                            "name": "발굴조사 표준 도면양식",
                            "organization": "매장문화재연구원",
                        }),
                    );
                }

                if ui.button("캔버스 자동 범례표 삽입").on_hover_text("도면에서 실제로 사용한 줄만 추출하여 캔버스에 벡터 범례표를 삽입합니다 (0개 요소 레이어 필터링).").clicked() {
                    let _ = app.run(
                        "style.generateLegend",
                        json!({
                            "at": [50.0, 50.0],
                            "width": 140.0,
                            "columns": 1,
                            "filterUnused": true,
                            "replaceExisting": true,
                        }),
                    );
                }

                if ui.button("범례표 갱신").on_hover_text("도면 요소 추가/삭제를 반영하여 범례표를 제자리에서 갱신합니다.").clicked() {
                    let _ = app.run("style.updateLegend", json!({}));
                }
            });

            ui.separator();

            // Diff & Merge section
            ui.label(egui::RichText::new("템플릿 병합 대화상자 (Diff & Merge)").strong());

            // Summary chips
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("동일: 0").color(Color32::from_rgb(100, 180, 100)));
                ui.label(egui::RichText::new("추가: 4").color(Color32::from_rgb(80, 140, 220)));
                ui.label(egui::RichText::new("변경: 1").color(Color32::from_rgb(220, 160, 60)));
                ui.label(egui::RichText::new("누락: 0").color(Color32::from_rgb(180, 100, 100)));
            });

            ui.collapsing("예시 템플릿 레이어 목록", |ui| {
                let default_layers = [
                    ("주거지 벽선", "0.8 pt", "외곽윤곽 (경계)"),
                    ("주거지 내부", "0.5 pt", "내부선 (주요)"),
                    ("주혈", "0.3 pt", "세부선 (세부)"),
                    ("토층 경계", "0.2 pt", "보조선 (해치)"),
                ];

                for (name, weight, role) in default_layers {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(name).strong());
                        ui.label(weight);
                        ui.label(role);
                    });
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("기본 표준 양식 적용 (덮어쓰기)").clicked() {
                    let tmpl = TrowelStyleTemplate::default();
                    let _ = app.run("style.applyTemplate", json!({ "template": tmpl }));
                }

                if ui.button("닫기").clicked() {
                    let _ = false;
                }
            });
        });

    ctx.data_mut(|d| d.insert_temp(style_dialog_open_id(), is_open));
}
