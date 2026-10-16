//! The pen bar (design 3.0 §9, the mockup's `.penbar`): while a drawing tool is in hand, three
//! buttons centred at the bottom of the canvas for pen tablet users: Undo, Finish Line (Enter, in
//! clay) and Close. The zoom has its place in the zoom buttons at the bottom right (principle 1);
//! the bar keeps clear of them.

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use super::{font, wid, width_of};
use crate::VectorcraftApp;
use crate::theme::Tokens;

pub const BAR_H: f32 = 40.0;
pub const BTN_H: f32 = 30.0;
/// The bar's padding and the gap between its buttons.
const PAD: f32 = 5.0;
const GAP: f32 = 4.0;
/// The room either side of a button's label, and in a canvas too narrow for that.
const BTN_PAD: f32 = 12.0;
const BTN_PAD_NARROW: f32 = 6.0;
/// How far the bar keeps from the canvas's left edge and from the zoom buttons.
const CLEAR_LEFT: f32 = 16.0;
const CLEAR_ZOOM: f32 = 8.0;

/// Returns whether the Canvas Pen Bar should be displayed for the active tool.
pub fn is_visible(app: &VectorcraftApp) -> bool {
    let tool = app.session.tool_id();
    matches!(tool, "pen" | "pencil" | "paintbrush" | "curvature" | "lineSegment" | "sectionCut" | "strataLine" | "pottery" | "pottery-rim")
}

/// The pen bar over `canvas`, while a drawing tool is in hand ([`is_visible`]).
pub fn show(app: &mut VectorcraftApp, ui: &mut Ui, canvas: Rect, t: &Tokens) {
    if !is_visible(app) {
        return;
    }

    // Set 0.5s double-click window for pen tablets (J016 A4)
    ui.ctx().options_mut(|o| o.input_options.max_double_click_delay = 0.5);

    // Each button's key, label and the key that does the same (shown after the label).
    let buttons = [("undo", tw!("Undo"), None), ("finish", tw!("Finish Line"), Some("Enter")), ("close", tw!("Close"), None)];
    let f = font(12.5);
    let label_w = |label: &str, hint: Option<&str>| width_of(ui, label, f.clone()) + hint.map_or(0.0, |k| 5.0 + width_of(ui, k, f.clone()));
    // The buttons' widths with `pad` either side of their labels, with or without the key.
    let widths =
        |pad: f32, keys: bool| -> Vec<f32> { buttons.iter().map(|(_, label, hint)| label_w(label, hint.filter(|_| keys)) + 2.0 * pad).collect() };
    let total = |w: &[f32]| w.iter().sum::<f32>() + GAP * w.len().saturating_sub(1) as f32 + 2.0 * PAD;
    // Beside the zoom buttons as in the mockup; in a canvas too narrow for that, above them. In
    // each place the buttons tighten, then leave out the key, before the bar moves.
    let zoom = super::center::zoom_column(canvas);
    let bottom = canvas.bottom() - 14.0;
    let places =
        [(canvas.left() + CLEAR_LEFT, zoom.left() - CLEAR_ZOOM, bottom), (canvas.left() + CLEAR_LEFT, zoom.right(), zoom.top() - CLEAR_ZOOM)];
    let looks = [(BTN_PAD, true), (BTN_PAD_NARROW, true), (BTN_PAD_NARROW, false)];
    let fits = places
        .iter()
        .flat_map(|&place| looks.iter().map(move |&look| (place, look)))
        .find(|&((left, right, _), (pad, keys))| total(&widths(pad, keys)) <= right - left);
    // Nothing fits: above the zoom buttons, cut at the bar's edge.
    let ((left, right, bottom), (pad, keys)) = fits.unwrap_or((places[1], looks[2]));
    let btn_widths = widths(pad, keys);
    let bar_w = total(&btn_widths).min((right - left).max(0.0));
    // Centred on the canvas, moved left when that would reach the zoom buttons.
    let x = (canvas.center().x - bar_w / 2.0).min(right - bar_w).max(left);
    let bar_rect = Rect::from_min_size(pos2(x, bottom - BAR_H), vec2(bar_w, BAR_H));

    ui.painter().add(
        egui::epaint::Shadow { offset: [0, 2], blur: 8, spread: 0, color: t.text.gamma_multiply(0.12) }.as_shape(bar_rect, CornerRadius::same(10)),
    );
    ui.painter().rect(bar_rect, CornerRadius::same(10), t.panel, Stroke::new(1.0, t.input_border), StrokeKind::Inside);
    // Presses on the bar don't reach the canvas.
    let _shield = ui.interact(bar_rect, wid("penbar-shield", ()), Sense::click_and_drag());

    // A canvas too narrow even for the narrow buttons cuts them at the bar's edge.
    let inside = bar_rect.shrink(1.0);
    let clip = ui.painter().with_clip_rect(inside);
    let mut pressed = None;
    let mut cur_x = bar_rect.left() + PAD;
    let cy = bar_rect.center().y;
    for ((key, label, hint), bw) in buttons.iter().zip(&btn_widths) {
        let hint = hint.filter(|_| keys);
        let btn_rect = Rect::from_min_size(pos2(cur_x, cy - BTN_H / 2.0), vec2(*bw, BTN_H));
        cur_x += bw + GAP;
        let Some(hit) = Some(btn_rect.intersect(inside)).filter(|r| r.is_positive()) else { continue };
        let resp = ui.interact(hit, wid("penbar", key), Sense::click());
        let primary = *key == "finish";
        let (fill, color) = match (primary, resp.hovered()) {
            (true, true) => (t.action.gamma_multiply(1.15), t.on_action),
            (true, false) => (t.action, t.on_action),
            (false, true) => (t.hover, t.text),
            (false, false) => (t.button, t.text),
        };
        clip.rect_filled(btn_rect, CornerRadius::same(7), fill);
        let w = label_w(label, hint);
        let name = clip.text(pos2(btn_rect.center().x - w / 2.0, cy), Align2::LEFT_CENTER, label, f.clone(), color);
        if let Some(k) = hint {
            clip.text(pos2(name.right() + 5.0, cy), Align2::LEFT_CENTER, k, f.clone(), color);
        }
        if resp.clicked() {
            pressed = Some(*key);
        }
    }

    match pressed {
        Some("finish") => {
            // Delivering ToolKey::Enter finishes the current drawing path
            let view = app.view_info();
            let r = app.session.tool_key(vectorcraft_tools::ToolKey::Enter, vectorcraft_tools::Mods::default(), view);
            crate::canvas::apply_requests(app, r);
        }
        Some("close") => {
            if let Some(id) = app.session.active().and_then(|d| d.selection.objects.first().copied()) {
                crate::menus::invoke(app, "path.close", json!({ "id": id.0 }));
            }
        }
        // While a strata line is being drawn its points aren't in the drawing yet: Undo takes the
        // last one back, as Backspace does.
        Some(_) if app.session.tool_claims_key(vectorcraft_tools::ToolKey::Backspace, app.view_info()) => {
            let view = app.view_info();
            let r = app.session.tool_key(vectorcraft_tools::ToolKey::Backspace, vectorcraft_tools::Mods::default(), view);
            crate::canvas::apply_requests(app, r);
        }
        Some(_) => crate::menus::invoke(app, "edit.undo", json!({})),
        None => {}
    }
}
