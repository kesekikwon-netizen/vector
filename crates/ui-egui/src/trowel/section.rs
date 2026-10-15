//! Section and Stratigraphy UI panel, stratum editor cards, and section control bar (Trowel design 2.1 §4-2, J010).
#![allow(dead_code)]

use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;
use vectorcraft_doc::stratigraphy::{NumberingStyle, StratigraphyFillStyle};

use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

pub const PANEL_W: f32 = 304.0;
pub const ROW_H: f32 = 34.0;

fn section_selected_layer_id() -> egui::Id {
    wid("section-selected-layer", ())
}

/// Renders the Stratigraphy dock panel in the right column.
pub fn show_dock_panel(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);

    let (layer_count, title, layers_to_render, numbering_style) = match app.session.active() {
        Some(st) => {
            if let Some(tbl) = st.doc.stratigraphy_tables.first() {
                (tbl.layers.len(), tbl.title.clone(), tbl.layers.clone(), tbl.numbering_style)
            } else {
                (0, "토층".to_string(), Vec::new(), NumberingStyle::Circled)
            }
        }
        None => return,
    };

    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, &title, &format!("A–A′ · {}개", layer_count), 1, &t);

    if head_button(ui, wid("section-add-layer", ()), head, 0, "plus", "새 층위 추가", &t).clicked() {
        let _ = app.run(
            "stratigraphy.addLayer",
            json!({
                "colorName": "암갈색",
                "texture": "사질토",
            }),
        );
    }

    let mut cy = head.bottom() + 4.0;

    // Order caption: "위에서 아래로"
    text(ui, pos2(r.left() + 14.0, cy + 6.0), Align2::LEFT_CENTER, "위에서 아래로", font(11.0), t.text_dim);
    cy += 18.0;

    // Layer list rows
    let selected_layer: Option<u32> = ui.data(|d| d.get_temp(section_selected_layer_id()));

    let list_h = (layers_to_render.len() as f32 * ROW_H).min(180.0);
    let list_rect = Rect::from_min_size(pos2(r.left(), cy), vec2(r.width(), list_h));

    let mut newly_selected = selected_layer;

    egui::ScrollArea::vertical().id_salt(wid("section-scroll", ())).auto_shrink([false, false]).show(
        &mut ui.new_child(egui::UiBuilder::new().max_rect(list_rect)),
        |ui| {
            for layer in &layers_to_render {
                let (row_rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
                let is_sel = selected_layer == Some(layer.index);

                if resp.clicked() {
                    newly_selected = if is_sel { None } else { Some(layer.index) };
                }

                // Row background
                let fill = if is_sel {
                    t.row_selected
                } else if resp.hovered() {
                    t.hover
                } else {
                    Color32::TRANSPARENT
                };

                if fill != Color32::TRANSPARENT {
                    ui.painter().rect_filled(row_rect, CornerRadius::same(4), fill);
                }

                // 1. Badge circle symbol
                let badge_text = if let Some(ref c) = layer.custom_badge {
                    c.clone()
                } else if numbering_style == NumberingStyle::Roman {
                    vectorcraft_doc::stratigraphy::to_ascii_roman(layer.index)
                } else {
                    layer.badge_text(numbering_style)
                };
                let badge_center = pos2(row_rect.left() + 20.0, row_rect.center().y);
                ui.painter().circle_filled(badge_center, 9.0, t.input);
                ui.painter().circle_stroke(badge_center, 9.0, Stroke::new(1.0, t.input_border));
                text(ui, badge_center, Align2::CENTER_CENTER, &badge_text, bold(11.0), t.text);

                // 2. Swatch patch
                let patch_r = Rect::from_center_size(pos2(row_rect.left() + 44.0, row_rect.center().y), vec2(16.0, 16.0));
                let patch_col = match &layer.fill_style {
                    StratigraphyFillStyle::Tint { .. } => t.action,
                    StratigraphyFillStyle::Pattern { .. } => t.input_border,
                    StratigraphyFillStyle::Hatch { .. } => t.border,
                    StratigraphyFillStyle::None => t.input,
                };
                ui.painter().rect_filled(patch_r, CornerRadius::same(3), patch_col);
                ui.painter().rect_stroke(patch_r, CornerRadius::same(3), Stroke::new(1.0, t.border), StrokeKind::Inside);

                // 3. Two-line title
                let col_name = if layer.description.color_name.is_empty() { "토층" } else { &layer.description.color_name };
                let texture = &layer.description.texture;
                let line1 = format!("{} {}", col_name, texture).trim().to_string();

                let munsell = &layer.description.munsell;
                let inc = &layer.description.inclusions;
                let line2 = if munsell.is_empty() && inc.is_empty() {
                    "비고 없음".to_string()
                } else if munsell.is_empty() {
                    inc.clone()
                } else if inc.is_empty() {
                    munsell.clone()
                } else {
                    format!("{} · {}", munsell, inc)
                };

                let text_x = row_rect.left() + 60.0;
                text(ui, pos2(text_x, row_rect.center().y - 6.0), Align2::LEFT_CENTER, &line1, font(11.0), t.text);
                text(ui, pos2(text_x, row_rect.center().y + 6.0), Align2::LEFT_CENTER, &line2, mono(10.0), t.text_dim);
            }
        },
    );

    ui.data_mut(|d| d.insert_temp(section_selected_layer_id(), newly_selected));
    cy += list_h + 10.0;

    // Action buttons
    let btn_w = (r.width() - 28.0 - 12.0) / 3.0;
    let b1 = Rect::from_min_size(pos2(r.left() + 14.0, cy), vec2(btn_w, 28.0));
    let b2 = Rect::from_min_size(pos2(r.left() + 14.0 + btn_w + 6.0, cy), vec2(btn_w, 28.0));
    let b3 = Rect::from_min_size(pos2(r.left() + 14.0 + (btn_w + 6.0) * 2.0, cy), vec2(btn_w, 28.0));

    if small_button(ui, wid("section-gen-tbl", ()), b1.left_center(), 28.0, "표 넣기", true, &t).clicked() {
        let _ = app.run("stratigraphy.generateTable", json!({ "at": [100, 500], "format": "grid" }));
    }

    if small_button(ui, wid("section-gen-inline", ()), b2.left_center(), 28.0, "토층설명(7p)", true, &t)
        .on_hover_text("현장 보고서 양식(I : 흑갈색 사질점토, 7pt)으로 도면에 바로 넣기")
        .clicked()
    {
        let _ = app.run("stratigraphy.generateTable", json!({ "at": [100, 500], "format": "inline", "fontSizePt": 7.0 }));
    }

    if small_button(ui, wid("section-copy-tsv", ()), b3.left_center(), 28.0, "TSV 복사", true, &t).clicked()
        && let Ok(res) = app.run("stratigraphy.exportTsv", json!({}))
        && let Some(tsv) = res.get("tsv").and_then(|v| v.as_str())
    {
        ui.ctx().copy_text(tsv.to_string());
    }

    cy += 36.0;

    // Help callout banner
    let callout_r = Rect::from_min_size(pos2(r.left() + 14.0, cy), vec2(r.width() - 28.0, 48.0));
    ui.painter().rect_filled(callout_r, CornerRadius::same(6), t.panel_darker);
    ui.painter().rect_stroke(callout_r, CornerRadius::same(6), Stroke::new(1.0, t.border), StrokeKind::Inside);

    let help_msg = "토층 칠은 단축키 K로 선 사이를 누르면 됩니다.\n선을 하나로 닫지 않아도 칠해집니다(라이브 페인트).";
    text(ui, pos2(callout_r.left() + 8.0, callout_r.center().y), Align2::LEFT_CENTER, help_msg, font(10.0), t.text_dim);
}

/// Renders the section and stratigraphy control bar when section tools are active.
pub fn show_control_bar(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let cy = r.center().y;
    let mut x = r.left() + 16.0;

    let current_style =
        app.session.active().and_then(|st| st.doc.stratigraphy_tables.first().map(|tbl| tbl.numbering_style)).unwrap_or(NumberingStyle::Circled);

    x = text(ui, pos2(x, cy), Align2::LEFT_CENTER, "토층 번호 ›", bold(12.0), t.text).right() + 12.0;

    // Numbering style selector
    let look = SegLook { h: 24.0, pad: 8.0, font: font(11.0) };

    let style_parts = [
        Seg { key: "circled", label: "① 원문자", on: current_style == NumberingStyle::Circled, soon: None },
        Seg { key: "roman", label: "Ⅰ 로마 숫자(I~XX)", on: current_style == NumberingStyle::Roman, soon: None },
        Seg { key: "arabic", label: "1 숫자", on: current_style == NumberingStyle::Arabic, soon: None },
    ];

    let (picked_style, next_x) = segments(ui, "section-style", pos2(x, cy), &look, &style_parts, &t);
    if let Some(style_key) = picked_style {
        let _ = app.run("stratigraphy.setStyle", json!({ "style": style_key }));
    }
    x = next_x + 12.0;

    if small_button(ui, wid("section-bar-inline", ()), pos2(x, cy), 24.0, "토층설명(7p) 넣기", true, &t).clicked() {
        let _ = app.run("stratigraphy.generateTable", json!({ "at": [100, 460], "format": "inline", "fontSizePt": 7.0 }));
    }
    x += 114.0;

    // Table sync toggle
    let tog_r = Rect::from_min_size(pos2(x, cy - 12.0), vec2(120.0, 24.0));
    let _ = toggle(ui, wid("section-sync-tog", ()), tog_r, "설명표에 줄 만들기", true, &t);
    x = tog_r.right() + 16.0;

    // Plan meeting projection toggle
    let proj_r = Rect::from_min_size(pos2(x, cy - 12.0), vec2(130.0, 24.0));
    let _ = toggle(ui, wid("section-proj-tog", ()), proj_r, "평면과 만나는 자리", true, &t);
}
