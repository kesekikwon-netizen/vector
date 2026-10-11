//! A card beside the right column (design 2.1 §7, figures ② and ③): the legend row's editor
//! ([`super::rowedit`]) and Weights at Once ([`super::weights`]). It floats over the canvas level
//! with what opened it, within the column's height, and scrolls when that is too short for it; ✕,
//! Esc and a press outside it put it away. Its rows are the mockup's `.ed`: a label column, then
//! the row's controls.

use super::*;

/// A card's width.
pub(super) const W: f32 = 304.0;
/// The width of its labels' column, and the gap after it.
const LABEL_W: f32 = 52.0;
const LABEL_GAP: f32 = 8.0;
/// The room between its one-point edge and what it shows, and under the last row.
const INSET: i8 = 13;
const BOTTOM: i8 = 9;
const HEAD_H: f32 = 32.0;
/// The gap between rows, and between controls side by side.
const GAP_Y: f32 = 5.0;
pub(super) const GAP_X: f32 = 6.0;

/// Where the pass a press outside a card put it away is kept ([`took_press`]).
fn took_id() -> egui::Id {
    wid("card-took-press", ())
}

/// Whether a press this pass put a card away: it goes no further.
pub(super) fn took_press(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<u64>(took_id())) == Some(ctx.cumulative_pass_nr())
}

/// Where a card shows: from `at` (its top left) within `bounds`. `fresh`: it opened this pass, so
/// this pass's press (which opened it, or came before it) doesn't put it away.
#[derive(Clone, Copy)]
pub(super) struct Place {
    pub(super) at: Pos2,
    pub(super) bounds: Rect,
    pub(super) fresh: bool,
}

/// Show card `id` at `place`: a head with `title`, `sub` (in full on hover when cut short) and ✕,
/// then `body`. Returns whether it stays open and what `body` returned. ✕, Esc (when no field had
/// the keyboard: then Esc puts back what was typed) and a press outside put it away; that press
/// goes no further ([`took_press`]), and a field being typed in when it comes keeps what was typed.
pub(super) fn show<R>(
    ctx: &egui::Context,
    id: egui::Id,
    Place { at, bounds, fresh }: Place,
    (title, sub): (&str, &str),
    body: impl FnOnce(&mut Ui, &Tokens) -> R,
) -> (bool, R) {
    let t = Tokens::get(ctx);
    let was = ctx.memory(|m| m.area_rect(id));
    // Where a button went down this pass. From the events: a tap's release in the same pass (a
    // touchpad's) has already cleared the pointer's press origin.
    let press = |e: &egui::Event| match e {
        egui::Event::PointerButton { pos, pressed: true, .. } => Some(*pos),
        _ => None,
    };
    let (esc, press) = ctx.input(|i| (i.key_pressed(egui::Key::Escape), i.events.iter().find_map(press)));
    // Whether a field had the keyboard at the end of the pass before: egui takes it from the field
    // before this pass when Esc comes, and that Esc only puts back what was typed.
    let (typing_id, pass) = (id.with("typing"), ctx.cumulative_pass_nr());
    let typing = ctx.data(|d| d.get_temp::<(u64, bool)>(typing_id)).is_some_and(|(at, on)| on && at + 1 == pass);
    let away = !fresh && matches!((was, press), (Some(card), Some(p)) if !card.contains(p));
    if away {
        ctx.data_mut(|d| d.insert_temp(took_id(), pass));
    }
    if away
        && let (Some(card), Some(field)) = (was, ctx.memory(|m| m.focused()))
        && ctx.read_response(field).is_some_and(|f| card.contains_rect(f.rect))
    {
        // It loses the keyboard while the card is still drawn this time, which keeps what was typed.
        ctx.memory_mut(|m| m.surrender_focus(field));
    }
    let inner_w = W - 2.0 * (f32::from(INSET) + 1.0);
    let shown = egui::Area::new(id).order(egui::Order::Middle).fixed_pos(at).constrain_to(bounds).fade_in(false).show(ctx, |ui| {
        egui::Frame::NONE
            .fill(t.panel)
            .stroke(Stroke::new(1.0, t.input_border))
            .corner_radius(12)
            .shadow(ui.visuals().popup_shadow)
            .inner_margin(egui::Margin { left: INSET, right: INSET, top: 0, bottom: BOTTOM })
            .show(ui, |ui| {
                // What the edges, the head and the gap under it leave of `bounds` for the body.
                let room = (bounds.height() - 2.0 - HEAD_H - GAP_Y - f32::from(BOTTOM)).max(48.0);
                ui.set_width(inner_w);
                // The area offers the size the card had the last time; the body may grow up to `room`.
                ui.set_max_height(HEAD_H + GAP_Y + room);
                ui.spacing_mut().item_spacing = vec2(GAP_X, GAP_Y);
                // No fading edges, as in the legend's list.
                ui.spacing_mut().scroll.fade.strength = 0.0;
                let (head, _) = ui.allocate_exact_size(vec2(inner_w, HEAD_H), Sense::hover());
                let head = head.expand2(vec2(f32::from(INSET), 0.0));
                if let Some(cut) = panel_head(ui, head, title, sub, 1, &t) {
                    ui.interact(cut, id.with("sub"), Sense::hover()).on_hover_text(sub);
                }
                let close = head_button(ui, wid("card-close", ()), head, 0, "x", tw!("Close"), &t).clicked();
                let scroll = egui::ScrollArea::vertical().id_salt(id.with("scroll")).max_height(room).auto_shrink([false, true]);
                (close, scroll.show(ui, |ui| body(ui, &t)).inner)
            })
            .inner
    });
    let (close, inner) = shown.inner;
    // Read before `data_mut`: the context can't be read while it is being written.
    let keyboard = ctx.egui_wants_keyboard_input();
    ctx.data_mut(|d| d.insert_temp(typing_id, (pass, keyboard)));
    (!(close || away || (esc && !typing)), inner)
}

/// A row of a card, `h` tall: `label` in the labels' column (the mockup's `.ed .l`). Returns the
/// room after it for the row's controls.
pub(super) fn row(ui: &mut Ui, label: &str, h: f32, t: &Tokens) -> Rect {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    if !label.is_empty() {
        text(ui, pos2(r.left(), r.center().y), Align2::LEFT_CENTER, label, font(11.0), t.text_dim);
    }
    Rect::from_min_max(pos2(r.left() + LABEL_W + LABEL_GAP, r.top()), r.max)
}

/// A caption across the card with a line after it (the mockup's `.cap2`).
pub(super) fn caption(ui: &mut Ui, label: &str, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 10.0 + 14.0), Sense::hover());
    list_caption(ui, r.left(), r.right(), r.bottom() - 7.0, label, t);
}

/// A quiet note across the card, wrapped (the mockup's `.note`), in `color`.
pub(super) fn note(ui: &mut Ui, s: &str, color: Color32) {
    ui.add(egui::Label::new(egui::RichText::new(s).size(12.0).color(color)).wrap());
}

/// Lay out boxes of `widths` from the left, `h` tall, as many on a row as fit, `GAP_X` apart.
/// Returns where each goes.
pub(super) fn flow(ui: &mut Ui, h: f32, widths: &[f32]) -> Vec<Rect> {
    let room = ui.available_width();
    // Each box's place from the top left, by row.
    let mut at = vec![];
    let (mut x, mut y) = (0.0, 0.0);
    for w in widths {
        if x > 0.0 && x + w > room {
            (x, y) = (0.0, y + h + GAP_Y);
        }
        at.push(vec2(x, y));
        x += w + GAP_X;
    }
    let (r, _) = ui.allocate_exact_size(vec2(room, y + h), Sense::hover());
    at.iter().zip(widths).map(|(p, w)| Rect::from_min_size(r.min + *p, vec2(*w, h))).collect()
}

/// How wide a button labelled `label` is (the mockup's `.btn`: 12 points either side).
pub(super) fn button_w(ui: &Ui, label: &str) -> f32 {
    width_of(ui, label, font(12.0)) + 24.0
}

/// A button labelled `label` in `r`: ink on the surface, or dimmed and not pressable when not
/// `enabled`.
pub(super) fn button(ui: &Ui, id: egui::Id, r: Rect, label: &str, enabled: bool, t: &Tokens) -> egui::Response {
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let fill = if !enabled {
        t.panel_darker
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    field_box(ui, r, fill, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, font(12.0), if enabled { t.text } else { t.text_disabled });
    resp
}

/// The card's one button that does the job (the mockup's `.btn.pri`): bold on the action colour,
/// or dimmed when not `enabled`.
pub(super) fn primary(ui: &Ui, id: egui::Id, r: Rect, label: &str, enabled: bool, t: &Tokens) -> egui::Response {
    if !enabled {
        return button(ui, id, r, label, false, t);
    }
    let resp = ui.interact(r, id, Sense::click());
    field_box(ui, r, t.action, t.action);
    text(ui, r.center(), Align2::CENTER_CENTER, label, bold(12.0), t.on_action);
    resp
}

/// A field a later job brings in `r`, showing `value` (with ▾ when it will be a list), dimmed;
/// a press says when it comes (`what`, `job`).
pub(super) fn field_soon(ui: &Ui, id: egui::Id, r: Rect, value: &str, list: bool, (what, job): (&str, &str), t: &Tokens) {
    field_box(ui, r, t.panel_darker, t.input_border);
    let right = if list { r.right() - 8.0 - 8.0 - 4.0 } else { r.right() - 8.0 };
    text_to(ui, pos2(r.left() + 8.0, r.center().y), right, value, font(11.0), t.text_disabled);
    if list {
        triangle(ui, pos2(r.right() - 8.0 - 3.5, r.center().y), 7.0, true, t.text_disabled);
    }
    soon_widget(ui, id, r, what, job);
}

/// A text field in `r` (the mockup's `.field`) showing `value` in `f`, after `prefix` (in the same
/// type and ink, as in "×0.8") and before a faint `unit` right after the text ("0.8 pt"). What is
/// typed stays while it has the keyboard; when the typing ends (Enter, Tab, a press elsewhere) it
/// comes back trimmed if it changed anything. Esc drops it.
pub(super) fn text_field(ui: &mut Ui, id: egui::Id, r: Rect, value: &str, f: FontId, (prefix, unit): (&str, &str), t: &Tokens) -> Option<String> {
    let draft_id = id.with("draft");
    let draft: Option<String> = ui.data(|d| d.get_temp(draft_id));
    let mut s = draft.clone().unwrap_or_else(|| value.to_string());
    let typing = ui.memory(|m| m.has_focus(id));
    field_box(ui, r, t.input, if typing { t.text } else { t.input_border });
    let cy = r.center().y;
    // Margins are whole points.
    let margin = |m: f32| m.round().clamp(0.0, 100.0) as i8;
    let mut left = 8.0;
    if !prefix.is_empty() {
        left += text(ui, pos2(r.left() + 8.0, cy), Align2::LEFT_CENTER, prefix, f.clone(), t.text).width() + 2.0;
    }
    // The unit's width; the text keeps clear of it.
    let unit_w = if unit.is_empty() { 0.0 } else { width_of(ui, unit, font(f.size)) };
    let (left, right) = (margin(left), margin(if unit.is_empty() { 8.0 } else { 8.0 + unit_w + 4.0 }));
    let edit = egui::TextEdit::singleline(&mut s)
        .id(id)
        .font(f.clone())
        .text_color(t.text)
        .frame(egui::Frame::NONE.inner_margin(egui::Margin { left, right, top: 0, bottom: 0 }))
        .vertical_align(egui::Align::Center)
        .min_size(r.size())
        .desired_width(r.width());
    let mut box_ui = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let resp = box_ui.add(edit);
    if !unit.is_empty() {
        // Right after the text as it now is; at the right end when the text is long.
        let x = (r.left() + f32::from(left) + width_of(ui, &s, f.clone()) + 4.0).min(r.right() - 8.0 - unit_w);
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, unit, font(f.size), t.text_disabled);
    }
    crate::widgets::select_all_on_focus(ui, &resp, &s);
    if ui.memory(|m| m.has_focus(id)) {
        ui.data_mut(|d| d.insert_temp(draft_id, s));
        return None;
    }
    ui.data_mut(|d| d.remove::<String>(draft_id));
    let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
    let typed = s.trim();
    (draft.is_some() && !esc && typed != value.trim()).then(|| typed.to_string())
}

/// What is being typed in the [`text_field`] `id`, while it has the keyboard.
pub(super) fn draft(ctx: &egui::Context, id: egui::Id) -> Option<String> {
    ctx.data(|d| d.get_temp(id.with("draft")))
}

/// A number of points typed in a field ("0.5", "0.5 pt"), from 0 to 1000.
pub(super) fn points(s: &str) -> Option<f64> {
    let s = s.trim();
    let s = s.strip_suffix("pt").unwrap_or(s).trim();
    s.parse::<f64>().ok().filter(|w| w.is_finite() && (0.0..=1000.0).contains(w))
}
