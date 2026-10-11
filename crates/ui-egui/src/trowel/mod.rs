//! Trowel's main window (design 2.1 §3, the `win-plan` mockup at 1600 × 988): caption, menu bar
//! with the Drawing menu, ribbon, Control bar, tool column, drawing set, document tabs, the canvas
//! with what floats on it, the right column (drawing palette, legend and layers, properties) and
//! the status bar. It draws the same engine state and runs the same commands as the classic window
//! ([`crate::chrome`], [`crate::toolbar`], [`crate::dock`]); what later jobs bring shows dimmed,
//! with their job number on hover ([`soon`]).

use std::hash::Hash;

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::VectorcraftApp;
use crate::theme::{self, Tokens};

mod caption;
mod center;
mod control;
mod drawings;
mod legend;
mod menu;
mod palette;
mod props;
mod ribbon;
mod status;
mod tools;

/// The panels' ids (egui keeps where each was laid out under it).
pub(crate) const CAPTION: &str = "trowel-caption";
pub(crate) const MENU: &str = "trowel-menu";
pub(crate) const RIBBON: &str = "trowel-ribbon";
pub(crate) const CONTROL: &str = "trowel-control";
pub(crate) const STATUS: &str = "trowel-status";
pub(crate) const TOOLS: &str = "trowel-tools";
pub(crate) const SET: &str = "trowel-set";
pub(crate) const RIGHT: &str = "trowel-right";

/// The parts' sizes in points, as in the mockup.
const CAPTION_H: f32 = 32.0;
const MENU_H: f32 = 28.0;
const RIBBON_H: f32 = 96.0;
const CONTROL_H: f32 = 40.0;
const STATUS_H: f32 = 32.0;
const TOOLS_W: f32 = 56.0;
const SET_W: f32 = 216.0;
const RIGHT_W: f32 = 304.0;
const TABS_H: f32 = 32.0;

/// The id of one of Trowel's widgets: its kind and which one.
pub(crate) fn wid(kind: &str, key: impl Hash + std::fmt::Debug) -> egui::Id {
    egui::Id::new(("trowel", kind, key))
}

/// Lay out Trowel's window. Screen mode 2 (full screen with the menu bar off) leaves out the bars
/// above the canvas, screen mode 3 (full screen) everything but the canvas.
pub(crate) fn frame(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let mode = app.ui.screen_mode;
    if mode < 2 {
        // With the system's title bar it shows the title; macOS's transparent one leaves us its
        // inset for the window buttons.
        if app.custom_titlebar || app.titlebar_inset > 0.0 {
            caption::show(app, ui);
        }
        // macOS shows the menus in its own menu bar.
        if app.services.native_menu.is_none() {
            menu::show(app, ui);
        }
        ribbon::show(app, ui);
        if app.ui.control_bar {
            control::show(app, ui);
        }
    }
    if mode < 3 {
        if app.ui.status_bar {
            status::show(app, ui);
        }
        if app.ui.toolbar {
            tools::show(app, ui);
        }
        if app.ui.dock {
            drawings::show(app, ui);
            right_column(app, ui);
        }
    }
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.pasteboard)).show(ui, |ui| center::show(app, ui, mode));
}

/// The right column: the drawing palette on top, the properties at the bottom, the legend and
/// layers between them. The dock's own panels pop out to its left.
fn right_column(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::right(RIGHT).resizable(false).exact_size(RIGHT_W).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_left(ui, r, t.border);
        ui.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id()));
        let top = Rect::from_min_size(r.min, vec2(r.width(), palette::HEIGHT));
        let bottom = Rect::from_min_max(pos2(r.left(), (r.bottom() - props::height(ui.ctx())).max(top.bottom())), r.max);
        palette::show(app, ui, top);
        legend::show(app, ui, Rect::from_min_max(top.left_bottom(), bottom.right_top()));
        props::show(app, ui, bottom);
        // Its panels pop out to its left, level with its top.
        let keep = ui.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id())).unwrap_or_default();
        crate::dock::set_flyout(ui.ctx(), crate::dock::Flyout { left: r.left(), top: r.top() + 6.0, keep });
    });
}

fn font(size: f32) -> FontId {
    FontId::proportional(size)
}

fn bold(size: f32) -> FontId {
    theme::semibold(size)
}

fn mono(size: f32) -> FontId {
    theme::mono(size)
}

/// Paint `s` anchored at `at`; returns where it went.
fn text(ui: &Ui, at: Pos2, align: Align2, s: impl ToString, font: FontId, color: Color32) -> Rect {
    ui.painter().text(at, align, s.to_string(), font, color)
}

/// How wide `s` is in `font`.
fn width_of(ui: &Ui, s: &str, font: FontId) -> f32 {
    ui.painter().layout_no_wrap(s.to_string(), font, Color32::TRANSPARENT).size().x
}

/// One-point lines along the edges of `r`.
fn line_below(ui: &Ui, r: Rect, color: Color32) {
    ui.painter().hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, color));
}

fn line_above(ui: &Ui, r: Rect, color: Color32) {
    ui.painter().hline(r.x_range(), r.top() + 0.5, Stroke::new(1.0, color));
}

fn line_left(ui: &Ui, r: Rect, color: Color32) {
    ui.painter().vline(r.left() + 0.5, r.y_range(), Stroke::new(1.0, color));
}

fn line_right(ui: &Ui, r: Rect, color: Color32) {
    ui.painter().vline(r.right() - 0.5, r.y_range(), Stroke::new(1.0, color));
}

/// A field or toggle's box, 8-point corners: `fill` inside a one-point `edge`.
fn field_box(ui: &Ui, r: Rect, fill: Color32, edge: Color32) {
    ui.painter().rect(r, CornerRadius::same(8), fill, Stroke::new(1.0, edge), StrokeKind::Inside);
}

/// A panel's head in `r` (32 points tall): its title, what it shows and a line under it. Room is
/// left at the right for `buttons` [`head_button`]s.
fn panel_head(ui: &Ui, r: Rect, title: &str, sub: &str, buttons: usize, t: &Tokens) {
    line_below(ui, r, t.border);
    let x = text(ui, pos2(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, title, font(13.0), t.text).right() + 6.0;
    let room = Rect::from_min_max(r.min, pos2(r.right() - 12.0 - 24.0 * buttons as f32 - 4.0, r.bottom()));
    ui.painter().with_clip_rect(room).text(pos2(x, r.center().y), Align2::LEFT_CENTER, sub, font(11.0), t.text_dim);
}

/// A panel head's button `slot` places from the right (0: the last one): `icon` in a 24-point
/// square.
fn head_button(ui: &Ui, id: egui::Id, head: Rect, slot: usize, icon: &str, tip: &str, t: &Tokens) -> egui::Response {
    let r = Rect::from_center_size(pos2(head.right() - 12.0 - 12.0 - 24.0 * slot as f32, head.center().y), vec2(24.0, 24.0));
    let resp = ui.interact(r, id, Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(6), t.hover);
    }
    crate::icons::paint(ui, icon, r.shrink(5.0), t.text);
    resp.on_hover_text(tip)
}

/// The ⋯ button of a head that pops a panel out beside the right column, or puts it away
/// ([`head_button`]). A press on it doesn't count as one away from the popped-out panel.
fn more_button(ui: &Ui, id: egui::Id, head: Rect, slot: usize, tip: &str, t: &Tokens) -> egui::Response {
    let resp = head_button(ui, id, head, slot, "ellipsis", tip, t);
    ui.data_mut(|d| d.get_temp_mut_or_default::<Vec<Rect>>(keep_id()).push(resp.rect));
    resp
}

/// Where the ⋯ buttons of the right column are kept while it is drawn.
fn keep_id() -> egui::Id {
    wid("flyout-keep", ())
}

/// A small button a later job brings, labelled `label` at `at` (its left centre), 26 points tall.
/// Returns where it ends.
fn button_soon(ui: &Ui, id: egui::Id, at: Pos2, label: &str, strong: bool, job: &str, t: &Tokens) -> f32 {
    let f = if strong { bold(11.0) } else { font(11.0) };
    let r = Rect::from_min_size(pos2(at.x, at.y - 13.0), vec2(width_of(ui, label, f.clone()) + 16.0, 26.0));
    field_box(ui, r, t.panel_darker, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, t.text_disabled);
    soon_widget(ui, id, r, label, job);
    r.right()
}

/// A small filled triangle around `c`, `size` wide, pointing down (`down`) or right: the mockup's
/// ▾ and ▸.
fn triangle(ui: &Ui, c: Pos2, size: f32, down: bool, color: Color32) {
    let (h, w) = (size / 2.0, size * 0.3);
    let points = if down {
        vec![c + vec2(-h, -w), c + vec2(h, -w), c + vec2(0.0, w + 1.0)]
    } else {
        vec![c + vec2(-w, -h), c + vec2(w + 1.0, 0.0), c + vec2(-w, h)]
    };
    ui.painter().add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

/// A caption over a list, from `x` to `right` with a line running from it; `cy` is its middle.
fn list_caption(ui: &Ui, x: f32, right: f32, cy: f32, label: &str, t: &Tokens) {
    let c = text(ui, pos2(x, cy), Align2::LEFT_CENTER, label, font(11.0), t.text_dim);
    if c.right() + 8.0 < right {
        ui.painter().hline((c.right() + 8.0)..=right, cy, Stroke::new(1.0, t.border));
    }
}

/// The hover text of `what` a later job brings: "Change All Weights… · Coming soon · J005".
fn soon(what: &str, job: &str) -> String {
    crate::i18n::fmt(tw!("{what} · Coming soon · {job}"), &[("what", what), ("job", job)])
}

/// How long the status bar shows a note, and a message, before it shows the hint again (seconds).
const NOTE_SECS: f64 = 4.0;
const MESSAGE_SECS: f64 = 6.0;

/// Where the note a press on something a later job brings left is kept, with when.
fn note_id() -> egui::Id {
    wid("soon-note", ())
}

/// A dimmed control a later job brings, `what` it is: on hover it says when (its `job`); a press
/// says so in the status bar ([`soon_note`]) and does nothing else.
fn soon_widget(ui: &Ui, id: egui::Id, r: Rect, what: &str, job: &str) -> egui::Response {
    let resp = ui.interact(r, id, Sense::click());
    if resp.clicked() {
        note_soon(ui.ctx(), what, job);
    }
    resp.on_hover_text(soon(what, job))
}

/// Say in the status bar that `what` comes with `job`.
fn note_soon(ctx: &egui::Context, what: &str, job: &str) {
    let note = crate::i18n::fmt(tw!("{what}: coming soon ({job})"), &[("what", what), ("job", job)]);
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(note_id(), (note, now)));
}

/// The note a press on something a later job brings left, for [`NOTE_SECS`].
fn soon_note(ctx: &egui::Context) -> Option<String> {
    let (note, at) = ctx.data(|d| d.get_temp::<(String, f64)>(note_id()))?;
    let left = NOTE_SECS - (ctx.input(|i| i.time) - at);
    // Drawn again when it is due to go.
    (left > 0.0).then(|| {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
        note
    })
}

/// The name of tool `id` as Trowel shows it: its label without "Tool".
fn tool_name(id: &str) -> String {
    vectorcraft_tools::tool_info(id)
        .map(|info| {
            let label = tl!(info.label);
            label.strip_suffix(tw!("Tool")).unwrap_or(label).trim().to_string()
        })
        .unwrap_or_default()
}

/// Where the next line goes.
struct DrawTarget {
    /// The layer (or isolated group) new art lands in; none when every layer is locked or hidden.
    into: Option<vectorcraft_doc::NodeId>,
    /// Why that is not the current layer, when the current one is locked or hidden.
    note: Option<String>,
}

/// A locked or hidden layer takes no new art: the engine puts it in the topmost layer that is shown
/// and unlocked instead ([`vectorcraft_engine::DocState::insertion_parent`]), or refuses it. The
/// band, the Control bar and the legend show that layer and say why.
fn draw_target(st: &vectorcraft_engine::DocState) -> DrawTarget {
    let into = st.insertion_parent().filter(|p| st.doc.is_editable(*p));
    let name = |id| st.doc.node(id).map(|n| n.display_name()).unwrap_or_default();
    let note = match st.current_layer() {
        Some(c) if !st.doc.is_editable(c) && into != Some(c) => {
            let template = match (st.doc.is_visible(c), into.is_some()) {
                (false, true) => tw!("{layer} is hidden: drawing on {target}"),
                (true, true) => tw!("{layer} is locked: drawing on {target}"),
                (false, false) => tw!("{layer} is hidden: no layer to draw on"),
                (true, false) => tw!("{layer} is locked: no layer to draw on"),
            };
            Some(crate::i18n::fmt(template, &[("layer", &name(c)), ("target", &into.map(name).unwrap_or_default())]))
        }
        _ if into.is_none() => Some(tw!("No layer to draw on").to_string()),
        _ => None,
    };
    DrawTarget { into, note }
}

/// The active tool's hint at `size` in `color`, plainly as in the mockup, its parts apart by "·";
/// `first` keeps the first part ("Click to add a corner point").
fn hint_job(app: &VectorcraftApp, first: bool, size: f32, color: Color32) -> egui::text::LayoutJob {
    let all: String = crate::chrome::current_hint(app).into_iter().map(|(s, _)| s).collect();
    let mut parts = all.split('|').map(str::trim).filter(|p| !p.is_empty());
    let shown = if first { parts.next().unwrap_or_default().to_string() } else { parts.collect::<Vec<_>>().join(" · ") };
    egui::text::LayoutJob::simple_singleline(shown, font(size), color)
}

#[cfg(test)]
mod tests;
