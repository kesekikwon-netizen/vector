//! The properties at the bottom of the right column (design 2.1 §3): what is selected and how big
//! it is. The feature's kind and period, which Strata's submission check asks for, come with J009.
//! ⋯ pops the full Properties panel out; the arrow folds this one to its head.

use serde_json::json;

use super::*;

/// The panel's padding at the sides.
const PAD: f32 = 14.0;
const FIELD_H: f32 = 22.0;

/// The properties' height: open, or folded to the head.
pub(super) fn height(ctx: &egui::Context) -> f32 {
    if folded(ctx) { 33.0 } else { 112.0 }
}

fn folded(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(folded_id())).unwrap_or(false)
}

fn folded_id() -> egui::Id {
    wid("props-folded", ())
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let size = app.selection_box().map(|b| (b.rect.width(), b.rect.height()));
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, tw!("Properties"), &summary(app), 2, &t);
    let folded = folded(ui.ctx());
    let (icon, tip) = if folded { ("chevron-right", tw!("Show the properties")) } else { ("chevron-down", tw!("Fold the properties")) };
    if head_button(ui, wid("props-fold", ()), head, 0, icon, tip, &t).clicked() {
        ui.data_mut(|d| d.insert_temp(folded_id(), !folded));
    }
    if more_button(ui, wid("props-more", ()), head, 1, tw!("Properties panel"), &t).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "properties" }));
    }
    if folded {
        return;
    }
    let left = r.left() + PAD;
    let fields = left + 40.0 + 6.0;
    // Kind and period (J009).
    let cy = head.bottom() + 2.0 + FIELD_H / 2.0;
    text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Kind"), font(11.0), t.text_dim);
    let x = field_soon(ui, wid("props-kind", ()), pos2(fields, cy), 112.0, tw!("Kind"), tw!("Choose"), true, &t);
    field_soon(ui, wid("props-number", ()), pos2(x + 6.0, cy), 44.0, tw!("No."), tw!("No."), false, &t);
    let cy = cy + FIELD_H + 3.0;
    text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Period"), font(11.0), t.text_dim);
    field_soon(ui, wid("props-period", ()), pos2(fields, cy), 112.0, tw!("Period"), tw!("Choose"), true, &t);
    // "Required" (what Strata's submission check asks for) shows with J009, which checks it.
    // How big the selection is.
    if let Some((w, h)) = size {
        let cy = cy + FIELD_H / 2.0 + 5.0 + 8.0;
        let u = app.session.general_unit();
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Size"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, format!("{} × {}", u.format(w), u.format(h)), mono(11.0), t.text);
    }
}

/// What is selected: one object's name, how many, or nothing.
fn summary(app: &VectorcraftApp) -> String {
    let Some(st) = app.session.active() else { return String::new() };
    match st.selection.objects.as_slice() {
        [] => tw!("No selection").to_string(),
        [one] => st.doc.node(*one).map(|n| crate::panels::layers::painted_name(n, &n.display_name(), crate::i18n::current())).unwrap_or_default(),
        many => crate::i18n::fmt(tw!("{n} objects"), &[("n", &many.len().to_string())]),
    }
}

/// A field a later job (J009) brings for `what`, `w` wide at `at` (its left centre), showing
/// `placeholder`. Returns where it ends.
#[allow(clippy::too_many_arguments)]
fn field_soon(ui: &Ui, id: egui::Id, at: Pos2, w: f32, what: &str, placeholder: &str, chevron: bool, t: &Tokens) -> f32 {
    let r = Rect::from_min_size(pos2(at.x, at.y - FIELD_H / 2.0), vec2(w, FIELD_H));
    ui.painter().rect(r, CornerRadius::same(6), t.panel_darker, Stroke::new(1.0, t.input_border), StrokeKind::Inside);
    text(ui, pos2(r.left() + 8.0, at.y), Align2::LEFT_CENTER, placeholder, font(11.0), t.text_disabled);
    if chevron {
        triangle(ui, pos2(r.right() - 8.0 - 3.5, at.y), 7.0, true, t.text_disabled);
    }
    soon_widget(ui, id, r, what, "J009");
    r.right()
}
