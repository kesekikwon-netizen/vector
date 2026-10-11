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
        crate::dock::set_column_left(ui.ctx(), r.left());
        let top = Rect::from_min_size(r.min, vec2(r.width(), palette::HEIGHT));
        let bottom = Rect::from_min_max(pos2(r.left(), (r.bottom() - props::height(ui.ctx())).max(top.bottom())), r.max);
        palette::show(app, ui, top);
        legend::show(app, ui, Rect::from_min_max(top.left_bottom(), bottom.right_top()));
        props::show(app, ui, bottom);
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
    let x = text(ui, pos2(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, title, bold(13.0), t.text).right() + 6.0;
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

/// A small button a later job brings, labelled `label` at `at` (its left centre), 26 points tall.
/// Returns where it ends.
fn button_soon(ui: &Ui, id: egui::Id, at: Pos2, label: &str, strong: bool, job: &str, t: &Tokens) -> f32 {
    let f = if strong { bold(11.0) } else { font(11.0) };
    let r = Rect::from_min_size(pos2(at.x, at.y - 13.0), vec2(width_of(ui, label, f.clone()) + 16.0, 26.0));
    field_box(ui, r, t.input, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, t.text_disabled);
    soon_widget(ui, id, r, job);
    r.right()
}

/// A caption over a list, from `x` to `right` with a line running from it; `cy` is its middle.
fn list_caption(ui: &Ui, x: f32, right: f32, cy: f32, label: &str, t: &Tokens) {
    let c = text(ui, pos2(x, cy), Align2::LEFT_CENTER, label, font(11.0), t.text_dim);
    if c.right() + 8.0 < right {
        ui.painter().hline((c.right() + 8.0)..=right, cy, Stroke::new(1.0, t.border));
    }
}

/// The hover text of something a later job brings: "Coming soon · J005".
fn soon(job: &str) -> String {
    crate::i18n::fmt(tw!("Coming soon · {job}"), &[("job", job)])
}

/// A dimmed control a later job brings: it shows `job` on hover and does nothing.
fn soon_widget(ui: &Ui, id: egui::Id, r: Rect, job: &str) -> egui::Response {
    ui.interact(r, id, Sense::hover()).on_hover_text(soon(job))
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

/// The active tool's hint at `size`, its keys bold in `key` and the rest in `rest` as on the classic
/// hint bar, its parts apart by "·"; `first` keeps the first part ("Click to add a corner point").
fn hint_job(app: &VectorcraftApp, first: bool, size: f32, key: Color32, rest: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (s, is_key) in crate::chrome::current_hint(app) {
        let (font_id, color) = if is_key { (bold(size), key) } else { (font(size), rest) };
        let format = egui::TextFormat { font_id, color, ..Default::default() };
        match s.split_once('|') {
            Some((part, _)) if first => {
                job.append(part.trim_end(), 0.0, format);
                break;
            }
            _ => job.append(&s.replace('|', "·"), 0.0, format),
        }
    }
    job
}

#[cfg(test)]
mod tests;
