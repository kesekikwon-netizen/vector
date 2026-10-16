//! The middle of the window (design 3.0 §5 ⑥): the document tabs over the canvas (the current
//! drawing's tab raised, a dot on one with changes not saved, a tab's × on hover, ＋ for a new
//! drawing) and what floats on the canvas: the zoom buttons at the bottom right, and the pen bar
//! while drawing ([`super::canvas_penbar`]). The zoom itself shows in the status bar.

use serde_json::json;

use super::*;

/// The ×'s size.
const CLOSE: f32 = 10.0;
/// A tab's height (the strip is [`TABS_H`]), its padding and the dot's size.
const TAB_H: f32 = 28.0;
const TAB_PAD: f32 = 14.0;
const DOT: f32 = 7.0;
/// The narrowest the current tab gets: a letter or two and its ×.
const MIN_TAB: f32 = TAB_PAD + 16.0 + 8.0 + CLOSE + 8.0;

/// The canvas under the tabs (screen mode 3: the canvas alone) and what floats on it.
pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, mode: u8) {
    let full = ui.max_rect();
    let below = if mode < 3 { Rect::from_min_max(pos2(full.left(), tabs(app, ui, full)), full.max) } else { full };
    let mut canvas = ui.new_child(egui::UiBuilder::new().max_rect(below).layout(*ui.layout()));
    if crate::menus::home_showing(app) {
        home::show(app, &mut canvas, below);
        return;
    }
    crate::canvas::show(app, &mut canvas);
    if mode < 3 {
        overlays(app, &mut canvas);
    }
}

/// How wide a tab named `name` is: its padding, the name, the dot when `dirty`, the ×'s room.
fn tab_width(ui: &Ui, name: &str, current: bool, dirty: bool) -> f32 {
    TAB_PAD + width_of(ui, name, tab_font(current)) + if dirty { 8.0 + DOT } else { 0.0 } + 8.0 + CLOSE + 8.0
}

fn tab_font(current: bool) -> FontId {
    if current { bold(12.5) } else { font(12.5) }
}

/// The tab strip at the top of `full`. Returns its bottom.
fn tabs(app: &mut VectorcraftApp, ui: &mut Ui, full: Rect) -> f32 {
    let t = Tokens::get(ui.ctx());
    let strip = Rect::from_min_size(full.min, vec2(full.width(), TABS_H));
    ui.painter().rect_filled(strip, 0.0, t.tab_strip);
    line_below(ui, strip, t.border);
    let home = crate::menus::home_showing(app);
    let active = app.session.active_index().filter(|_| !home);
    let docs: Vec<(String, bool)> = app.session.documents().iter().map(|d| (d.title(), d.is_dirty())).collect();
    let widths: Vec<f32> = docs.iter().enumerate().map(|(i, (n, dirty))| tab_width(ui, n, Some(i) == active, *dirty)).collect();
    let left = strip.left() + 8.0;
    let plus_w = 14.0 + width_of(ui, "＋", font(12.5)) + 14.0;
    // ＋ keeps its room at the right; the current drawing's tab stays in view, the drawing set
    // lists them all.
    let room = strip.right() - 8.0 - plus_w - 2.0 - left;
    let first = crate::chrome::first_tab_shown(&widths, active, room);
    let top = strip.bottom() - TAB_H;
    let end = left + room;
    let mut activate = None;
    let mut close = None;
    let mut x = left;
    for (i, ((title, dirty), w)) in docs.iter().zip(&widths).enumerate().skip(first) {
        if x + w > end && Some(i) != active {
            break;
        }
        // The current tab takes the room there is, its name cut short.
        let w = if x + w > end { (end - x).max(MIN_TAB) } else { *w };
        let r = Rect::from_min_size(pos2(x, top), vec2(w, TAB_H));
        let current = Some(i) == active;
        let resp = tab(ui, i, r, title, current, *dirty, &t);
        let x_rect = Rect::from_center_size(pos2(r.right() - 8.0 - CLOSE / 2.0, r.center().y), vec2(CLOSE, CLOSE));
        let x_resp = ui.interact(x_rect.expand(3.0), wid("tab-close", i), Sense::click());
        if resp.hovered() || x_resp.hovered() {
            crate::icons::paint(ui, "x", x_rect, if x_resp.hovered() { t.text } else { t.text_dim });
        }
        if x_resp.on_hover_text(tw!("Close")).clicked() {
            close = Some(i);
        } else if resp.clicked() {
            activate = Some(i);
        }
        x = r.right() + 2.0;
    }
    // ＋: a new drawing (File › New).
    let plus = Rect::from_min_size(pos2(x, top), vec2(plus_w, TAB_H));
    let plus_resp = ui.interact(plus, wid("tab-new", ()), Sense::click());
    if plus_resp.hovered() {
        ui.painter().rect_filled(plus.shrink2(vec2(0.0, 2.0)), CornerRadius::same(6), t.hover);
    }
    text(ui, plus.center(), Align2::CENTER_CENTER, "＋", font(12.5), t.text_disabled);
    let new = plus_resp.on_hover_text(tw!("New drawing")).clicked();
    if let Some(i) = close {
        if let Err(e) = crate::unsaved::close(app, i) {
            app.status(e);
        }
    } else if let Some(i) = activate {
        app.ui.home = None;
        if let Err(e) = app.run("document.activate", json!({ "index": i })) {
            app.status(e);
        }
    } else if new {
        crate::menus::invoke(app, "file.newDialog", json!({}));
    }
    strip.bottom()
}

/// Tab `i` in `r` (the mockup's `.ctab`): `label`, cut short with "…" when it doesn't fit before
/// the dot and the ×; the `current` one raised on the window's ground with an edge and its name in
/// semibold, a clay dot after the name when `dirty` (it says so on hover).
fn tab(ui: &Ui, i: usize, r: Rect, label: &str, current: bool, dirty: bool, t: &Tokens) -> egui::Response {
    let resp = ui.interact(r, wid("tab", i), Sense::click());
    if current {
        let corners = CornerRadius { nw: 7, ne: 7, sw: 0, se: 0 };
        // Its edge on three sides; its foot runs into the canvas under it.
        ui.painter().rect(r, corners, t.panel, Stroke::new(1.0, t.border), StrokeKind::Inside);
        ui.painter().hline(r.x_range().shrink(1.0), r.bottom() - 0.5, Stroke::new(1.0, t.panel));
    } else if resp.hovered() {
        ui.painter().rect_filled(r.shrink2(vec2(0.0, 2.0)), CornerRadius::same(6), t.hover);
    }
    let color = if current || resp.hovered() { t.text } else { t.text_dim };
    let x = r.left() + TAB_PAD;
    let room = r.right() - 8.0 - CLOSE - 8.0 - if dirty { 8.0 + DOT } else { 0.0 } - x;
    let f = tab_font(current);
    let mut job = egui::text::LayoutJob::simple_singleline(label.to_string(), f.clone(), color);
    if width_of(ui, label, f) > room {
        job.wrap = egui::text::TextWrapping::truncate_at_width(room.max(0.0));
    }
    let name = ui.painter().layout_job(job);
    let name_w = name.size().x;
    ui.painter().with_clip_rect(r).galley(pos2(x, r.center().y - name.size().y / 2.0), name, color);
    if dirty {
        let c = pos2(x + name_w + 8.0 + DOT / 2.0, r.center().y);
        ui.painter().circle_filled(c, DOT / 2.0, t.action);
        ui.interact(Rect::from_center_size(c, vec2(12.0, 12.0)), wid("tab-unsaved", i), Sense::hover()).on_hover_text(tw!("Not saved"));
    }
    resp
}

/// The zoom buttons and, while drawing, the pen bar, over the canvas.
fn overlays(app: &mut VectorcraftApp, ui: &mut Ui) {
    if crate::menus::home_showing(app) || app.session.active().is_none() {
        return;
    }
    let Some(canvas) = app.canvas_rect else { return };
    let t = Tokens::get(ui.ctx());
    zoom_buttons(app, ui, canvas, &t);
    super::canvas_penbar::show(app, ui, canvas, &t);
}

/// Zoom in, zoom out and fit, one above the other at the bottom right (the mockup's `.zoomst`).
fn zoom_buttons(app: &mut VectorcraftApp, ui: &Ui, canvas: Rect, t: &Tokens) {
    const CELL: egui::Vec2 = vec2(34.0, 30.0);
    let r = Rect::from_min_max(
        pos2(canvas.right() - 14.0 - CELL.x, canvas.bottom() - 14.0 - 3.0 * CELL.y),
        pos2(canvas.right() - 14.0, canvas.bottom() - 14.0),
    );
    field_box(ui, r, t.input, t.input_border);
    let mut run = None;
    let buttons =
        [("in", "view.zoomIn", tw!("Zoom In")), ("out", "view.zoomOut", tw!("Zoom Out")), ("fit", "view.fitArtboard", tw!("Fit Artboard in Window"))];
    for (i, (key, cmd, label)) in buttons.into_iter().enumerate() {
        let cell = Rect::from_min_size(pos2(r.left(), r.top() + i as f32 * CELL.y), CELL);
        // Presses here don't reach the canvas.
        let resp = ui.interact(cell, wid("zoom", key), Sense::click_and_drag());
        if resp.hovered() {
            let radius = match i {
                0 => CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 },
                2 => CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 },
                _ => CornerRadius::ZERO,
            };
            ui.painter().rect_filled(cell.shrink(1.0), radius, t.hover);
        }
        if i > 0 {
            ui.painter().hline(cell.x_range().shrink(1.0), cell.top() + 0.5, Stroke::new(1.0, t.border));
        }
        match key {
            "in" => crate::icons::paint(ui, "plus", sharp_square(ui, cell, 15.0), t.text),
            "out" => crate::icons::paint(ui, "minus", sharp_square(ui, cell, 15.0), t.text),
            _ => {
                text(ui, cell.center(), Align2::CENTER_CENTER, tw!("Fit"), font(11.0), t.text);
            }
        }
        let tip = match crate::menus::shortcut_of(cmd) {
            Some(sc) => format!("{label} ({})", crate::menus::pretty_shortcut(sc)),
            None => label.to_string(),
        };
        if resp.on_hover_text(tip).clicked() {
            run = Some(cmd);
        }
    }
    if let Some(cmd) = run {
        crate::menus::invoke(app, cmd, json!({}));
    }
}

/// A `size` square in the middle of `cell`, on whole pixels so a thin icon stays sharp.
fn sharp_square(ui: &Ui, cell: Rect, size: f32) -> Rect {
    let ppp = ui.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    Rect::from_min_size(pos2(snap(cell.center().x - size / 2.0), snap(cell.center().y - size / 2.0)), vec2(size, size))
}
