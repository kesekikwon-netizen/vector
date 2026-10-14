//! Report Plate Layout UI panel & dialog (Trowel design 2.1 §6, §9, §14(a, 1), Task J012).
#![allow(dead_code)]

use egui::{Align2, CornerRadius, Rect, Sense, Ui, pos2, vec2};
use serde_json::json;
use vectorcraft_doc::plate::PlateOrientation;

use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

pub const PANEL_W: f32 = 304.0;
pub const ROW_H: f32 = 30.0;

fn plate_selected_sheet_id() -> egui::Id {
    wid("plate-selected-sheet", ())
}

fn plate_dialog_open_id() -> egui::Id {
    wid("plate-dialog-open", ())
}

/// Open or toggle the Report Plate dialog.
pub fn toggle_dialog(ctx: &egui::Context) {
    let cur: bool = ctx.data(|d| d.get_temp(plate_dialog_open_id())).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(plate_dialog_open_id(), !cur));
}

/// Renders the Report Plate panel in the Trowel dock.
pub fn show_dock_panel(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);

    let (sheet_count, active_sheet) = match app.session.active() {
        Some(st) => (st.doc.plate_sheets.len(), st.doc.plate_sheets.first().cloned()),
        None => return,
    };

    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, "도판 배치", &format!("총 {}매 · 보고서 규격", sheet_count), 2, &t);

    // Export PDF icon button
    if head_button(ui, wid("plate-export-pdf-head", ()), head, 0, "file-pdf", "도판 벡터 PDF 내보내기", &t).clicked() {
        let _ = app.run("plate.exportPdf", json!({}));
    }

    // Add Sheet button
    if head_button(ui, wid("plate-add-sheet-head", ()), head, 1, "plus", "새 도판 시트 생성", &t).clicked() {
        let _ = app.run(
            "plate.createSheet",
            json!({
                "size": "4x6",
                "orientation": "portrait",
                "name": format!("도판 {}", sheet_count + 1),
                "caption": "유구 평·단면도",
                "pageNumber": "1",
            }),
        );
    }

    let mut cy = head.bottom() + 6.0;

    let Some(sh) = active_sheet else {
        text(ui, pos2(r.left() + 14.0, cy + 10.0), Align2::LEFT_CENTER, "생성된 도판 시트가 없습니다. ＋를 눌러 추가하세요.", font(11.0), t.text_dim);
        return;
    };

    // Size & Orientation badges
    let (w_mm, h_mm) = sh.size.dimensions_mm(sh.orientation);
    let orient_label = if sh.orientation == PlateOrientation::Landscape { "가로" } else { "세로" };
    let size_str = format!("{} ({}×{}mm, {})", sh.size.label(), w_mm as u32, h_mm as u32, orient_label);

    text(ui, pos2(r.left() + 14.0, cy + 6.0), Align2::LEFT_CENTER, &sh.name, bold(12.0), t.text);
    text(ui, pos2(r.right() - 14.0, cy + 6.0), Align2::RIGHT_CENTER, &size_str, font(10.0), t.text_dim);
    cy += 22.0;

    // Viewport list
    text(ui, pos2(r.left() + 14.0, cy + 6.0), Align2::LEFT_CENTER, format!("뷰포트 ({}개)", sh.viewports.len()), font(11.0), t.text_dim);
    cy += 18.0;

    for vp in &sh.viewports {
        let vp_row = Rect::from_min_size(pos2(r.left() + 10.0, cy), vec2(r.width() - 20.0, 24.0));
        ui.painter().rect_filled(vp_row, CornerRadius::same(4), t.row_selected);
        text(ui, pos2(vp_row.left() + 8.0, vp_row.center().y), Align2::LEFT_CENTER, &vp.name, font(11.0), t.text);
        text(
            ui,
            pos2(vp_row.right() - 8.0, vp_row.center().y),
            Align2::RIGHT_CENTER,
            format!("축척 1:{}", vp.scale.denom as u32),
            mono(10.0),
            t.text,
        );
        cy += 28.0;
    }

    // Action buttons: Add Viewport
    let btn_w = (r.width() - 28.0 - 8.0) / 2.0;
    let add_vp_rect = Rect::from_min_size(pos2(r.left() + 14.0, cy), vec2(btn_w, 26.0));
    let add_vp_resp = ui.interact(add_vp_rect, wid("plate-add-vp-btn", ()), Sense::click());
    field_box(ui, add_vp_rect, if add_vp_resp.hovered() { t.hover } else { t.panel }, t.border);
    text(ui, add_vp_rect.center(), Align2::CENTER_CENTER, "＋ 뷰포트 추가", font(11.0), t.text);
    if add_vp_resp.clicked() {
        let _ = app.run(
            "plate.addViewport",
            json!({
                "name": format!("도면 뷰포트 {}", sh.viewports.len() + 1),
                "scale": 60.0,
                "sourceScale": 20.0,
            }),
        );
    }

    // Title Block settings button
    let tb_btn_rect = Rect::from_min_size(pos2(r.left() + 14.0 + btn_w + 8.0, cy), vec2(btn_w, 26.0));
    let tb_btn_resp = ui.interact(tb_btn_rect, wid("plate-tb-btn", ()), Sense::click());
    field_box(ui, tb_btn_rect, if tb_btn_resp.hovered() { t.hover } else { t.panel }, t.border);
    text(ui, tb_btn_rect.center(), Align2::CENTER_CENTER, "표제란·축척 설정", font(11.0), t.text);
    if tb_btn_resp.clicked() {
        toggle_dialog(ui.ctx());
    }
    cy += 34.0;

    // Export Vector Buttons (PDF / SVG)
    line_below(ui, Rect::from_min_size(pos2(r.left(), cy), vec2(r.width(), 1.0)), t.border);
    cy += 8.0;

    let pdf_rect = Rect::from_min_size(pos2(r.left() + 14.0, cy), vec2(btn_w, 28.0));
    let pdf_resp = ui.interact(pdf_rect, wid("plate-export-pdf", ()), Sense::click());
    field_box(ui, pdf_rect, if pdf_resp.hovered() { t.hover } else { t.row_selected }, t.accent);
    text(ui, pdf_rect.center(), Align2::CENTER_CENTER, "PDF 내보내기", bold(11.0), t.text);
    if pdf_resp.on_hover_text("논문·보고서 인쇄용 벡터 PDF (선 굵기 pt 불변 보존)").clicked() {
        let _ = app.run("plate.exportPdf", json!({}));
    }

    let svg_rect = Rect::from_min_size(pos2(r.left() + 14.0 + btn_w + 8.0, cy), vec2(btn_w, 28.0));
    let svg_resp = ui.interact(svg_rect, wid("plate-export-svg", ()), Sense::click());
    field_box(ui, svg_rect, if svg_resp.hovered() { t.hover } else { t.panel }, t.border);
    text(ui, svg_rect.center(), Align2::CENTER_CENTER, "SVG 내보내기", font(11.0), t.text);
    if svg_resp.on_hover_text("웹·표준 CAD/벡터 그래픽 호환 SVG (선 굵기 pt 불변 보존)").clicked() {
        let _ = app.run("plate.exportSvg", json!({}));
    }
}

/// Renders the Modal/Window Dialog for Plate Layout and Title Block Inspector.
pub fn show_dialog(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let mut is_open: bool = ctx.data(|d| d.get_temp(plate_dialog_open_id())).unwrap_or(false);
    if !is_open {
        return;
    }

    let (has_doc, first_sheet) = match app.session.active() {
        Some(st) => (true, st.doc.plate_sheets.first().cloned()),
        None => (false, None),
    };

    egui::Window::new("보고서 도판 배치 및 표제란 설정 (J012)")
        .id(wid("plate-dialog-window", ()))
        .open(&mut is_open)
        .resizable(true)
        .default_size(vec2(440.0, 520.0))
        .show(ctx, |ui| {
            if !has_doc {
                ui.label("열린 문서가 없습니다.");
                return;
            }

            let Some(sheet) = first_sheet else {
                ui.label("도판 시트가 없습니다.");
                if ui.button("새 4×6배판 도판 시트 생성").clicked() {
                    let _ = app.run(
                        "plate.createSheet",
                        json!({
                            "size": "4x6",
                            "orientation": "portrait",
                            "name": "도면 1",
                            "caption": "1호 주거지 평·단면도",
                        }),
                    );
                }
                return;
            };

            ui.heading(format!("도판 시트: {}", sheet.name));
            ui.label(format!(
                "판형 규격: {} ({} × {} pt)",
                sheet.size.label(),
                sheet.size.dimensions_pt(sheet.orientation).0 as u32,
                sheet.size.dimensions_pt(sheet.orientation).1 as u32
            ));
            ui.separator();

            ui.collapsing("표제란 정보 (Title Block)", |ui| {
                let tb = &sheet.title_block;
                ui.label(format!("도면명: {}", tb.drawing_title));
                ui.label(format!("유적명: {}", tb.project_title));
                ui.label(format!("유적부호: {}", tb.site_code));
                ui.label(format!("조사단: {}", tb.excavation_team));
                ui.label(format!("조사원: {}", tb.drawn_by));
                ui.label(format!("축척표기: {}", tb.scale_text));
                ui.label(format!("좌표계: {}", tb.crs_datum));
            });

            ui.collapsing("축척 막대 및 방위표", |ui| {
                let sb = &sheet.scale_bar;
                ui.label(format!("단위: {}m · 구간: {}개 · 축척 1:{}", sb.segment_unit_m, sb.segment_count, sb.scale.denom as u32));
                let na = &sheet.north_arrow;
                ui.label(format!("방위 회전각: {:.1}°", na.rotation_deg));
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("벡터 PDF 내보내기").clicked() {
                    let _ = app.run("plate.exportPdf", json!({}));
                }
                if ui.button("벡터 SVG 내보내기").clicked() {
                    let _ = app.run("plate.exportSvg", json!({}));
                }
            });
        });

    ctx.data_mut(|d| d.insert_temp(plate_dialog_open_id(), is_open));
}
