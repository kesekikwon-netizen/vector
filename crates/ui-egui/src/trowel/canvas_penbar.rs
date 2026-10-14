//! Canvas Pen Bar: floating quick-action toolbar for pen tablet users (Trowel design 2.1 §12, J016 A5).
//!
//! Provides 7 large touch/pen action buttons centered at the bottom of the canvas:
//! - `되돌리기` (Undo)
//! - `다시` (Redo)
//! - `선 끝내기` (FinishPath, primary action in terracotta accent)
//! - `닫기` (ClosePath)
//! - `확대` (ZoomIn)
//! - `축소` (ZoomOut)
//! - `맞춤` (Fit)

use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use super::wid;
use crate::VectorcraftApp;
use crate::theme::Tokens;

pub const BAR_H: f32 = 40.0;
pub const BTN_H: f32 = 32.0;

/// Returns whether the Canvas Pen Bar should be displayed for the active tool.
pub fn is_visible(app: &VectorcraftApp) -> bool {
    let tool = app.session.tool_id();
    matches!(
        tool,
        "pen"
            | "pencil"
            | "paintbrush"
            | "curvature"
            | "lineSegment"
            | "section-line"
            | "spot-height"
            | "find-number"
            | "posthole"
            | "hachure"
            | "stratigraphy"
            | "pottery"
            | "pottery-rim"
    )
}

/// Renders the floating Canvas Pen Bar over the canvas rect.
pub fn show(app: &mut VectorcraftApp, ui: &mut Ui, canvas: Rect, t: &Tokens) {
    if !is_visible(app) {
        return;
    }

    // Set 0.5s double-click window for pen tablets (J016 A4)
    ui.ctx().options_mut(|o| o.input_options.max_double_click_delay = 0.5);

    struct PenBtn {
        key: &'static str,
        label: &'static str,
        cmd: Option<&'static str>,
        is_primary: bool,
    }

    let buttons = [
        PenBtn { key: "undo", label: "되돌리기", cmd: Some("edit.undo"), is_primary: false },
        PenBtn { key: "redo", label: "다시", cmd: Some("edit.redo"), is_primary: false },
        PenBtn { key: "finish", label: "선 끝내기", cmd: None, is_primary: true },
        PenBtn { key: "close", label: "닫기", cmd: None, is_primary: false },
        PenBtn { key: "zoomIn", label: "확대", cmd: Some("view.zoomIn"), is_primary: false },
        PenBtn { key: "zoomOut", label: "축소", cmd: Some("view.zoomOut"), is_primary: false },
        PenBtn { key: "fit", label: "맞춤", cmd: Some("view.fitArtboard"), is_primary: false },
    ];

    let btn_widths = [64.0, 52.0, 78.0, 52.0, 52.0, 52.0, 52.0];
    let total_content_w: f32 = btn_widths.iter().sum::<f32>() + 12.0;
    let bar_w = total_content_w.min(canvas.width() - 32.0);

    let bar_rect = Rect::from_min_size(
        pos2(canvas.center().x - bar_w / 2.0, canvas.bottom() - 14.0 - BAR_H),
        vec2(bar_w, BAR_H),
    );

    // Shield interaction from the underlying canvas
    ui.painter().add(
        egui::epaint::Shadow {
            offset: [0, 3],
            blur: 14,
            spread: 0,
            color: t.text.gamma_multiply(0.12),
        }
        .as_shape(bar_rect, CornerRadius::same(8)),
    );

    let bg_color = t.input;
    ui.painter().rect_filled(bar_rect, CornerRadius::same(8), bg_color);
    ui.painter().rect_stroke(bar_rect, CornerRadius::same(8), Stroke::new(1.0, t.input_border), StrokeKind::Inside);

    let _shield = ui.interact(bar_rect, wid("penbar-shield", ()), Sense::click_and_drag());

    let mut cur_x = bar_rect.left() + 6.0;
    let cy = bar_rect.center().y;

    let mut action_to_run: Option<&'static str> = None;
    let mut finish_clicked = false;
    let mut close_clicked = false;

    for (i, btn) in buttons.iter().enumerate() {
        let bw = btn_widths[i];
        let btn_rect = Rect::from_center_size(pos2(cur_x + bw / 2.0, cy), vec2(bw, BTN_H));
        let resp = ui.interact(btn_rect, wid("penbar", btn.key), Sense::click());

        let is_hovered = resp.hovered();

        let (fill_col, text_col) = if btn.is_primary {
            let base = t.action;
            let col = if is_hovered { base.gamma_multiply(1.15) } else { base };
            (col, t.on_action)
        } else if is_hovered {
            (t.hover, t.text)
        } else {
            (Color32::TRANSPARENT, t.text)
        };

        if fill_col != Color32::TRANSPARENT {
            ui.painter().rect_filled(btn_rect, CornerRadius::same(6), fill_col);
        }

        // Draw icon
        let icon_box = Rect::from_min_size(pos2(btn_rect.left() + 4.0, btn_rect.center().y - 6.0), vec2(12.0, 12.0));
        paint_btn_icon(ui, btn.key, icon_box, text_col);

        // Draw text
        let font_id = egui::FontId::proportional(11.0);
        let text_pos = pos2(btn_rect.left() + 18.0, cy);
        ui.painter().text(text_pos, Align2::LEFT_CENTER, btn.label, font_id, text_col);

        if resp.clicked() {
            if btn.key == "finish" {
                finish_clicked = true;
            } else if btn.key == "close" {
                close_clicked = true;
            } else if let Some(c) = btn.cmd {
                action_to_run = Some(c);
            }
        }

        cur_x += bw;
        if i + 1 < buttons.len() {
            ui.painter().vline(cur_x, (cy - 8.0)..=(cy + 8.0), Stroke::new(1.0, t.border));
        }
    }

    if finish_clicked {
        // Delivering ToolKey::Enter finishes the current drawing path
        let view = app.view_info();
        let r = app.session.tool_key(vectorcraft_tools::ToolKey::Enter, vectorcraft_tools::Mods::default(), view);
        crate::canvas::apply_requests(app, r);
    }

    if close_clicked {
        if let Some(id) = app.session.active().and_then(|d| d.selection.objects.first().copied()) {
            crate::menus::invoke(app, "path.close", json!({ "id": id.0 }));
        }
    }

    if let Some(cmd) = action_to_run {
        crate::menus::invoke(app, cmd, json!({}));
    }
}

fn paint_btn_icon(ui: &mut Ui, key: &str, r: Rect, fg: Color32) {
    let p = ui.painter();
    match key {
        "undo" => {
            // Arc / hook pointing left
            let p0 = pos2(r.left() + 2.0, r.center().y);
            let p1 = pos2(r.left() + 6.0, r.top() + 2.0);
            let p2 = pos2(r.right() - 2.0, r.center().y);
            p.line_segment([p0, p1], Stroke::new(1.4, fg));
            p.line_segment([p0, pos2(r.left() + 6.0, r.bottom() - 2.0)], Stroke::new(1.4, fg));
            p.line_segment([p1, p2], Stroke::new(1.4, fg));
        }
        "redo" => {
            // Arrow pointing right
            let p0 = pos2(r.right() - 2.0, r.center().y);
            let p1 = pos2(r.right() - 6.0, r.top() + 2.0);
            let p2 = pos2(r.left() + 2.0, r.center().y);
            p.line_segment([p0, p1], Stroke::new(1.4, fg));
            p.line_segment([p0, pos2(r.right() - 6.0, r.bottom() - 2.0)], Stroke::new(1.4, fg));
            p.line_segment([p1, p2], Stroke::new(1.4, fg));
        }
        "finish" => {
            // Checkmark
            let p0 = pos2(r.left() + 1.0, r.center().y + 1.0);
            let p1 = pos2(r.left() + 4.5, r.bottom() - 2.0);
            let p2 = pos2(r.right() - 1.0, r.top() + 2.0);
            p.line_segment([p0, p1], Stroke::new(1.8, fg));
            p.line_segment([p1, p2], Stroke::new(1.8, fg));
        }
        "close" => {
            // Closed polygon loop
            let box_r = r.shrink(2.0);
            p.rect_stroke(box_r, CornerRadius::same(2), Stroke::new(1.4, fg), StrokeKind::Inside);
        }
        "zoomIn" => {
            let cx = r.center().x;
            let cy = r.center().y;
            p.line_segment([pos2(cx - 4.5, cy), pos2(cx + 4.5, cy)], Stroke::new(1.4, fg));
            p.line_segment([pos2(cx, cy - 4.5), pos2(cx, cy + 4.5)], Stroke::new(1.4, fg));
        }
        "zoomOut" => {
            let cx = r.center().x;
            let cy = r.center().y;
            p.line_segment([pos2(cx - 4.5, cy), pos2(cx + 4.5, cy)], Stroke::new(1.4, fg));
        }
        "fit" => {
            let box_r = r.shrink(2.5);
            p.rect_stroke(box_r, CornerRadius::same(1), Stroke::new(1.2, fg), StrokeKind::Inside);
            p.line_segment([pos2(r.left() + 1.0, r.top() + 1.0), pos2(r.left() + 4.0, r.top() + 1.0)], Stroke::new(1.5, fg));
            p.line_segment([pos2(r.right() - 4.0, r.top() + 1.0), pos2(r.right() - 1.0, r.top() + 1.0)], Stroke::new(1.5, fg));
            p.line_segment([pos2(r.left() + 1.0, r.bottom() - 1.0), pos2(r.left() + 4.0, r.bottom() - 1.0)], Stroke::new(1.5, fg));
            p.line_segment([pos2(r.right() - 4.0, r.bottom() - 1.0), pos2(r.right() - 1.0, r.bottom() - 1.0)], Stroke::new(1.5, fg));
        }
        _ => {}
    }
}
