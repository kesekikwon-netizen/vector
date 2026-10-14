//! Artifact Pottery drafting UI panel, 2x2 measurement ledger, and control bar (Trowel design 2.1 §4-3, §5, §14(3), J011).
#![allow(dead_code)]

use egui::{Align2, CornerRadius, Rect, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;
use vectorcraft_doc::pottery::PotteryMeasurements;

use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

pub const PANEL_W: f32 = 304.0;

/// Renders the Artifact Pottery drafting panel in the right column.
pub fn show_panel(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);

    let (vessel_id, vessel_name, find_info, measurements, feature, preservation_note) = match app.session.active() {
        Some(st) => {
            if let Some(v) = st.doc.pottery_vessels.first() {
                (Some(v.id.clone()), v.name.clone(), v.find_number.clone(), v.measurements.clone(), v.feature.clone(), v.preservation_note.clone())
            } else {
                (None, "유물 실측".to_string(), "1:1 실측".to_string(), PotteryMeasurements::default(), "1호 주거지".to_string(), "완형".to_string())
            }
        }
        None => return,
    };

    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, &vessel_name, &format!("출토 번호 {}", find_info), 0, &t);

    let mut cy = head.bottom() + 8.0;

    // 2x2 Ledger Readout Card
    let ledger_card_r = Rect::from_min_size(pos2(r.left() + 12.0, cy), vec2(r.width() - 24.0, 76.0));
    ui.painter().rect_filled(ledger_card_r, CornerRadius::same(6), t.panel_darker);
    ui.painter().rect_stroke(ledger_card_r, CornerRadius::same(6), Stroke::new(1.0, t.border), StrokeKind::Inside);

    let (rim_str, height_str, base_str, wall_str) = measurements.format_ledger();

    let half_w = ledger_card_r.width() / 2.0;
    let row1_y = ledger_card_r.top() + 20.0;
    let row2_y = ledger_card_r.top() + 54.0;

    // Cell 1: 구경 (Rim diameter)
    text(ui, pos2(ledger_card_r.left() + 10.0, row1_y), Align2::LEFT_CENTER, &rim_str, bold(13.0), t.text);
    // Cell 2: 기고 (Total height)
    text(ui, pos2(ledger_card_r.left() + half_w + 10.0, row1_y), Align2::LEFT_CENTER, &height_str, bold(13.0), t.text);

    // Inner dividing lines
    ui.painter().hline(ledger_card_r.x_range(), ledger_card_r.top() + 38.0, Stroke::new(1.0, t.border));
    ui.painter().vline(ledger_card_r.left() + half_w, ledger_card_r.y_range(), Stroke::new(1.0, t.border));

    // Cell 3: 저경 (Base diameter)
    text(ui, pos2(ledger_card_r.left() + 10.0, row2_y), Align2::LEFT_CENTER, &base_str, bold(13.0), t.text);
    // Cell 4: 기벽 (Wall thickness)
    text(ui, pos2(ledger_card_r.left() + half_w + 10.0, row2_y), Align2::LEFT_CENTER, &wall_str, bold(13.0), t.text);

    cy += 88.0;

    // Property rows
    let prop_label_w = 60.0;
    let prop_val_w = r.width() - 24.0 - prop_label_w;

    let props = [
        ("기종", "심발"),
        ("문양", "공열문 · 구연부 아래 한 줄"),
        ("출토", feature.as_str()),
        ("남은 정도", preservation_note.as_str()),
    ];

    for (label, val) in props {
        text(ui, pos2(r.left() + 14.0, cy + 10.0), Align2::LEFT_CENTER, label, font(11.0), t.text_dim);
        let val_r = Rect::from_min_size(pos2(r.left() + 14.0 + prop_label_w, cy), vec2(prop_val_w, 22.0));
        ui.painter().rect_filled(val_r, CornerRadius::same(4), t.input);
        ui.painter().rect_stroke(val_r, CornerRadius::same(4), Stroke::new(1.0, t.input_border), StrokeKind::Inside);
        text(ui, pos2(val_r.left() + 6.0, val_r.center().y), Align2::LEFT_CENTER, val, font(11.0), t.text);
        cy += 26.0;
    }

    cy += 8.0;

    // Action buttons row: [반쪽 대칭] [단면 빗금] [계측선]
    let btn_w = (r.width() - 28.0 - 12.0) / 3.0;
    let b1 = Rect::from_min_size(pos2(r.left() + 14.0, cy), vec2(btn_w, 28.0));
    let b2 = Rect::from_min_size(pos2(r.left() + 14.0 + btn_w + 6.0, cy), vec2(btn_w, 28.0));
    let b3 = Rect::from_min_size(pos2(r.left() + 14.0 + (btn_w + 6.0) * 2.0, cy), vec2(btn_w, 28.0));

    if small_button(ui, wid("pottery-reflect-btn", ()), b1.left_center(), 28.0, "반쪽 대칭", true, &t).clicked() {
        if let Some(ref vid) = vessel_id {
            let _ = app.run("pottery.reflectExterior", json!({ "vesselId": vid }));
        }
    }

    if small_button(ui, wid("pottery-hatch-btn", ()), b2.left_center(), 28.0, "단면 빗금", true, &t).clicked() {
        if let Some(ref vid) = vessel_id {
            let _ = app.run("pottery.wallHatch", json!({ "vesselId": vid, "fillMode": "hatch" }));
        }
    }

    if small_button(ui, wid("pottery-annot-btn", ()), b3.left_center(), 28.0, "계측선", true, &t).clicked() {
        if let Some(ref vid) = vessel_id {
            let _ = app.run("pottery.measurements", json!({ "vesselId": vid, "emitAnnotations": true }));
        }
    }
}

/// Renders the artifact pottery control bar.
pub fn show_control_bar(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let cy = r.center().y;
    let mut x = r.left() + 16.0;

    x = text(ui, pos2(x, cy), Align2::LEFT_CENTER, "유물 실측 ›", bold(12.0), t.text).right() + 12.0;

    let look = SegLook {
        h: 24.0,
        pad: 8.0,
        font: font(11.0),
    };

    // Fill mode selector: [없음 | 검정 | 빗금]
    let fill_parts = [
        Seg { key: "none", label: "칠 없음", on: false, soon: None },
        Seg { key: "black", label: "검정", on: false, soon: None },
        Seg { key: "hatch", label: "빗금 (45°)", on: true, soon: None },
    ];
    let (sel_fill, next_x) = segments(ui, "pottery-fill-mode", pos2(x, cy), &look, &fill_parts, &t);
    x = next_x + 16.0;

    if let Some(fill) = sel_fill {
        let _ = app.run("pottery.wallHatch", json!({ "fillMode": fill }));
    }

    // Section side selector: [오른쪽 | 왼쪽] per §14(3) default 오른쪽
    let side_parts = [
        Seg { key: "right", label: "단면: 오른쪽", on: true, soon: None },
        Seg { key: "left", label: "단면: 왼쪽", on: false, soon: None },
    ];
    let (_, next_x) = segments(ui, "pottery-side-mode", pos2(x, cy), &look, &side_parts, &t);
    x = next_x + 16.0;

    // Report plate scale selector: [1:1 | 1:2 | 1:3 | 1:4]
    let scale_parts = [
        Seg { key: "1:1", label: "1:1", on: false, soon: None },
        Seg { key: "1:2", label: "1:2", on: false, soon: None },
        Seg { key: "1:3", label: "1:3", on: true, soon: None },
        Seg { key: "1:4", label: "1:4", on: false, soon: None },
    ];
    let (_, _next_x) = segments(ui, "pottery-scale-mode", pos2(x, cy), &look, &scale_parts, &t);
}
