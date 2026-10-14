//! Style Template Diff & Merge Dialog and Dynamic Auto Legend UI (Trowel design 2.1 §6, §7, §8, §10, Task J018).
#![allow(dead_code)]

use std::collections::HashMap;

use egui::vec2;
use serde_json::json;
use vectorcraft_doc::style_template::{
    DiffStatus, ItemDiff, MergeAction, StyleCategory, TrowelStyleTemplate, diff_template,
};

use super::*;
use crate::VectorcraftApp;

fn style_dialog_open_id() -> egui::Id {
    wid("style-dialog-open", ())
}

fn style_template_path_id() -> egui::Id {
    wid("style-template-path-input", ())
}

fn style_resolutions_id() -> egui::Id {
    wid("style-merge-resolutions-map", ())
}

fn style_category_filter_id() -> egui::Id {
    wid("style-category-filter", ())
}

/// Open or toggle the Style Template & Diff dialog.
pub fn toggle_dialog(ctx: &egui::Context) {
    let cur: bool = ctx.data(|d| d.get_temp(style_dialog_open_id())).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(style_dialog_open_id(), !cur));
}

/// Renders the Style Template & Auto Legend dialog.
pub fn show_dialog(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let is_open: bool = ctx.data(|d| d.get_temp(style_dialog_open_id())).unwrap_or(false);
    if !is_open {
        return;
    }

    let mut resolutions: HashMap<String, MergeAction> =
        ctx.data(|d| d.get_temp(style_resolutions_id())).unwrap_or_default();
    let template_path: String =
        ctx.data(|d| d.get_temp(style_template_path_id())).unwrap_or_default();
    let mut category_filter: Option<StyleCategory> =
        ctx.data(|d| d.get_temp(style_category_filter_id())).unwrap_or(None);

    let template = if !template_path.is_empty() {
        match std::fs::read_to_string(&template_path) {
            Ok(content) => serde_json::from_str::<TrowelStyleTemplate>(&content)
                .unwrap_or_else(|_| TrowelStyleTemplate::default()),
            Err(_) => TrowelStyleTemplate::default(),
        }
    } else {
        TrowelStyleTemplate::default()
    };

    let diff = match app.session.active() {
        Some(st) => diff_template(&st.doc, &template),
        None => {
            let mut open_flag = is_open;
            egui::Window::new("고고학 양식 템플릿 (.trowel-style) 및 범례표 (J018)")
                .id(wid("style-template-window", ()))
                .open(&mut open_flag)
                .show(ctx, |ui| {
                    ui.label("열린 문서가 없습니다.");
                });
            let final_open = open_flag;
            ctx.data_mut(|d| d.insert_temp(style_dialog_open_id(), final_open));
            return;
        }
    };

    let mut open_flag = is_open;
    let mut close_requested = false;

    egui::Window::new("고고학 양식 템플릿 (.trowel-style) 및 범례표 (J018)")
        .id(wid("style-template-window", ()))
        .open(&mut open_flag)
        .resizable(true)
        .default_size(vec2(640.0, 680.0))
        .show(ctx, |ui| {
            ui.heading("도면 양식 템플릿 차이 분석 및 병합");
            ui.label("기관·연구원별 표준 선 굵기 계층, 토층 해치 프리셋, 유구 레이어 양식을 비교·적용합니다.");
            ui.separator();

            // Section 1: Template and Canvas Legend Toolbar
            ui.horizontal(|ui| {
                if ui.button("현재 문서 양식 내보내기 (.trowel-style)").on_hover_text("현재 문서의 레이어와 범례 속성을 .trowel-style 파일로 내보냅니다.").clicked() {
                    let _ = app.run(
                        "style.exportTemplate",
                        json!({
                            "name": "발굴조사 표준 도면양식",
                            "organization": "매장문화재연구원",
                        }),
                    );
                }

                if ui.button("캔버스 자동 범례표 삽입").on_hover_text("도면에서 실제로 사용한 레이어만 추출하여 캔버스에 벡터 범례표를 삽입합니다 (0개 요소 레이어 필터링).").clicked() {
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

            // Section 2: Template Source
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("비교 대상 템플릿:").strong());
                if template_path.is_empty() {
                    ui.label("기본 고고학 표준 양식 (Trowel v1)");
                } else {
                    ui.label(format!("외부 파일: {template_path}"));
                }
            });

            let t = Tokens::get(ui.ctx());

            // Section 3: Genuine Dynamic Diff Summary
            ui.label(egui::RichText::new("템플릿 차이 분석 결과 (Diff)").strong());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("동일: {}", diff.identical_count))
                        .color(t.text)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("추가: {}", diff.added_count))
                        .color(t.accent)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("변경: {}", diff.modified_count))
                        .color(t.warning)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("누락: {}", diff.missing_count))
                        .color(t.error)
                        .strong(),
                );
                ui.separator();
                ui.label(egui::RichText::new(diff.summary_ko()).color(t.text_dim));
            });

            ui.separator();

            // Section 4: Global Merge Action Batch Buttons & Category Filter
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("일괄 조치:").strong());
                if ui.button("모두 덮어쓰기 (Overwrite)").on_hover_text("모든 항목에 대해 템플릿 정의로 덮어씁니다.").clicked() {
                    for item in &diff.items {
                        resolutions.insert(item.key.clone(), MergeAction::Overwrite);
                    }
                }
                if ui.button("모두 기존 유지 (Keep)").on_hover_text("현재 도면의 기존 설정을 그대로 유지합니다.").clicked() {
                    for item in &diff.items {
                        resolutions.insert(item.key.clone(), MergeAction::KeepExisting);
                    }
                }
                if ui.button("모두 새 사본 (New Copy)").on_hover_text("새로운 이름으로 복제하여 추가합니다.").clicked() {
                    for item in &diff.items {
                        resolutions.insert(item.key.clone(), MergeAction::ImportAsNewCopy);
                    }
                }
                if ui.button("권장값 복원").on_hover_text("차이 분석 엔진이 추천하는 기본 조치로 복원합니다.").clicked() {
                    for item in &diff.items {
                        resolutions.insert(item.key.clone(), item.recommended_action);
                    }
                }
            });

            // Category Filter Bar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("분류 필터:").strong());
                if ui.selectable_label(category_filter.is_none(), "전체").clicked() {
                    category_filter = None;
                }
                if ui.selectable_label(category_filter == Some(StyleCategory::LegendRow), "레이어 규칙").clicked() {
                    category_filter = Some(StyleCategory::LegendRow);
                }
                if ui.selectable_label(category_filter == Some(StyleCategory::StrokeWeight), "선 굵기").clicked() {
                    category_filter = Some(StyleCategory::StrokeWeight);
                }
                if ui.selectable_label(category_filter == Some(StyleCategory::HatchPreset), "단면 해치").clicked() {
                    category_filter = Some(StyleCategory::HatchPreset);
                }
                if ui.selectable_label(category_filter == Some(StyleCategory::SoilPalette), "토층 팔레트").clicked() {
                    category_filter = Some(StyleCategory::SoilPalette);
                }
            });

            ui.separator();

            // Section 5: Dynamic Item Table
            ui.label(egui::RichText::new("항목별 세부 차이 및 병합 조치 선택:").strong());

            let filtered_items: Vec<&ItemDiff> = diff
                .items
                .iter()
                .filter(|item| category_filter.is_none() || category_filter == Some(item.category))
                .collect();

            if filtered_items.is_empty() {
                ui.label("표시할 항목이 없습니다.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new(wid("style-diff-grid", ()))
                            .striped(true)
                            .min_col_width(70.0)
                            .spacing(vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                // Header row
                                ui.label(egui::RichText::new("분류").strong());
                                ui.label(egui::RichText::new("항목명").strong());
                                ui.label(egui::RichText::new("상태").strong());
                                ui.label(egui::RichText::new("차이 상세").strong());
                                ui.label(egui::RichText::new("병합 조치").strong());
                                ui.end_row();

                                for item in filtered_items {
                                    // 1. Category Badge
                                    let (cat_label, cat_color) = match item.category {
                                        StyleCategory::LegendRow => ("레이어 규칙", t.accent),
                                        StyleCategory::StrokeWeight => ("선 굵기", t.text),
                                        StyleCategory::HatchPreset => ("단면 해치", t.warning),
                                        StyleCategory::SoilPalette => ("토층 팔레트", t.clay),
                                        StyleCategory::Typography => ("도면 서체", t.highlight),
                                        StyleCategory::ElevationRules => ("표고선", t.accent_strong),
                                        StyleCategory::TitleBlock => ("표제란", t.text_dim),
                                    };
                                    ui.label(egui::RichText::new(cat_label).color(cat_color));

                                    // 2. Name / Label
                                    ui.label(egui::RichText::new(&item.label).strong());

                                    // 3. Status Badge
                                    let (status_label, status_color) = match item.status {
                                        DiffStatus::Identical => ("동일", t.text),
                                        DiffStatus::Added => ("추가", t.accent),
                                        DiffStatus::Modified => ("변경", t.warning),
                                        DiffStatus::Missing => ("누락", t.error),
                                    };
                                    ui.label(egui::RichText::new(status_label).color(status_color));

                                    // 4. Difference details
                                    if item.property_diffs.is_empty() {
                                        let detail = match item.status {
                                            DiffStatus::Identical => "현재 문서와 일치",
                                            DiffStatus::Missing => "도면 레이어 (템플릿 미포함)",
                                            _ => "-",
                                        };
                                        ui.label(egui::RichText::new(detail).color(t.text_dim));
                                    } else {
                                        let diff_texts: Vec<String> = item
                                            .property_diffs
                                            .iter()
                                            .map(|p| format!("{}: {} → {}", p.property_name, p.current_value, p.incoming_value))
                                            .collect();
                                        ui.label(diff_texts.join(", "));
                                    }

                                    // 5. Merge action resolution selector
                                    let current_action = resolutions
                                        .get(&item.key)
                                        .copied()
                                        .unwrap_or(item.recommended_action);
                                    let mut selected_action = current_action;

                                    egui::ComboBox::from_id_salt(wid("res-combo", &item.key))
                                        .width(110.0)
                                        .selected_text(match selected_action {
                                            MergeAction::Overwrite => "덮어쓰기",
                                            MergeAction::KeepExisting => "기존 유지",
                                            MergeAction::ImportAsNewCopy => "새 사본",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut selected_action, MergeAction::Overwrite, "덮어쓰기 (Overwrite)");
                                            ui.selectable_value(&mut selected_action, MergeAction::KeepExisting, "기존 유지 (Keep)");
                                            ui.selectable_value(&mut selected_action, MergeAction::ImportAsNewCopy, "새 사본 (Copy)");
                                        });

                                    if selected_action != current_action {
                                        resolutions.insert(item.key.clone(), selected_action);
                                    }

                                    ui.end_row();
                                }
                            });
                    });
            }

            ui.separator();

            // Section 6: Action Footer
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("선택한 조치로 양식 적용 (Apply)").strong()).clicked() {
                    let resolutions_param: Vec<serde_json::Value> = diff
                        .items
                        .iter()
                        .map(|item| {
                            let action = resolutions
                                .get(&item.key)
                                .copied()
                                .unwrap_or(item.recommended_action);
                            let action_str = match action {
                                MergeAction::KeepExisting => "keep_existing",
                                MergeAction::ImportAsNewCopy => "import_as_new_copy",
                                MergeAction::Overwrite => "overwrite",
                            };
                            let cat_str = match item.category {
                                StyleCategory::LegendRow => "legend_row",
                                StyleCategory::StrokeWeight => "stroke_weight",
                                StyleCategory::HatchPreset => "hatch_preset",
                                StyleCategory::SoilPalette => "soil_palette",
                                StyleCategory::Typography => "typography",
                                StyleCategory::ElevationRules => "elevation_rules",
                                StyleCategory::TitleBlock => "title_block",
                            };
                            json!({
                                "key": item.key,
                                "category": cat_str,
                                "action": action_str,
                            })
                        })
                        .collect();

                    let _ = app.run(
                        "style.applyTemplate",
                        json!({
                            "template": template,
                            "resolutions": resolutions_param,
                        }),
                    );
                    close_requested = true;
                }

                if ui.button("닫기").clicked() {
                    close_requested = true;
                }
            });
        });

    let final_open = open_flag && !close_requested;
    ctx.data_mut(|d| {
        d.insert_temp(style_dialog_open_id(), final_open);
        d.insert_temp(style_resolutions_id(), resolutions);
        d.insert_temp(style_template_path_id(), template_path);
        d.insert_temp(style_category_filter_id(), category_filter);
    });
}
