//! Trowel's main window (design 3.0 §4, the main mockup at 1440 × 880): caption, menu bar with
//! the Drawing menu, ribbon, tool column, drawing set, document tabs, the canvas with the zoom
//! buttons on it, the right column (legend, pen, properties) and the status
//! bar. It draws the same engine state and runs the same commands as the classic window
//! ([`crate::chrome`], [`crate::toolbar`], [`crate::dock`]); each thing it does has one place
//! (design 3.0 §3, principle 1), and what later jobs bring shows dimmed, with their job number on
//! hover ([`soon`]).

use std::hash::Hash;

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::VectorcraftApp;
use crate::theme::{self, Tokens};

pub mod book;
pub mod canvas_penbar;
mod caption;
mod card;
mod center;
pub mod drawings;
pub mod gpkg_browser;
pub mod home;
mod legend;
mod menu;
mod palette;
pub mod plate;
pub mod pottery;
mod props;
mod ribbon;
mod rowedit;
mod scale_dialog;
pub mod section;
mod status;
pub mod style_dialog;
mod tools;
mod weights;

/// The panels' ids (egui keeps where each was laid out under it).
pub(crate) const CAPTION: &str = "trowel-caption";
pub(crate) const MENU: &str = "trowel-menu";
pub(crate) const RIBBON: &str = "trowel-ribbon";
pub(crate) const STATUS: &str = "trowel-status";
pub(crate) const TOOLS: &str = "trowel-tools";
pub(crate) const SET: &str = "trowel-set";
pub(crate) const RIGHT: &str = "trowel-right";

/// The parts' sizes in points, as in the mockup.
const CAPTION_H: f32 = 32.0;
const MENU_H: f32 = 28.0;
const RIBBON_H: f32 = 86.0;
const STATUS_H: f32 = 28.0;
const TOOLS_W: f32 = 56.0;
const SET_W: f32 = 216.0;
const RIGHT_W: f32 = 304.0;
const TABS_H: f32 = 34.0;

/// The id of one of Trowel's widgets: its kind and which one.
pub(crate) fn wid(kind: &str, key: impl Hash + std::fmt::Debug) -> egui::Id {
    egui::Id::new(("trowel", kind, key))
}

/// Whether a press this pass only put away a card beside the right column ([`card::show`]): the
/// canvas leaves it at that, so the tool in hand doesn't get it.
pub(crate) fn card_took_press(ctx: &egui::Context) -> bool {
    card::took_press(ctx)
}

/// The elevation the tool column's Spot Height and the drawing set's section and spot height
/// elements use, in metres: the one last set in the drawing set, 93.0 m until then.
pub(crate) fn datum_elevation_m(ctx: &egui::Context) -> f64 {
    ctx.data(|d| d.get_temp::<f64>(datum_elevation_id())).unwrap_or(93.0)
}

/// Where [`datum_elevation_m`] is kept.
pub(crate) fn datum_elevation_id() -> egui::Id {
    wid("spot-datum-elev-val", ())
}

/// Compute a visible placement position near the active viewport center (or active artboard
/// center), applying a small cascade offset on consecutive insertions so multiple 1-click
/// symbols do not stack on the exact same pixel, and switch to the Selection tool so the user
/// can immediately drag the newly inserted symbol.
pub(crate) fn place_symbol_at(app: &mut VectorcraftApp, ctx: &egui::Context, base_offset: [f64; 2]) -> [f64; 2] {
    let step_id = wid("symbol-cascade-step", ());
    let step = ctx.data(|d| d.get_temp::<u32>(step_id).unwrap_or(0));
    ctx.data_mut(|d| d.insert_temp(step_id, (step + 1) % 6));
    let cascade = f64::from(step) * 14.0;
    let center = app
        .view()
        .map(|v| [v.center.x, v.center.y])
        .or_else(|| {
            let d = app.session.active()?;
            let ab = d.doc.artboards.first()?;
            Some([ab.rect.x0 + ab.rect.width() * 0.5, ab.rect.y0 + ab.rect.height() * 0.5])
        })
        .unwrap_or([240.0, 200.0]);
    app.select_tool("selection");
    [center[0] + base_offset[0] + cascade, center[1] + base_offset[1] + cascade * 0.6]
}

/// Lay out Trowel's window. Screen mode 2 (full screen with the menu bar off) leaves out the bars
/// above the canvas, screen mode 3 (full screen) everything but the canvas. There is no Control
/// bar (design 3.0 §5): what it held has its one place in the status bar, the right column or the
/// Window menu's panels.
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
    plate::show_dialog(app, ui.ctx());
    style_dialog::show_dialog(app, ui.ctx());
    gpkg_browser::show_dialog(app, ui.ctx());
    scale_dialog::show(app, ui.ctx());
}

/// The right column (design 3.0 §5 ⑦–⑨): the legend at the top, taking the height left, the pen
/// under it and the properties at the bottom. A window too short for them all scrolls them, the
/// legend with room for two rows. The dock's own panels pop out to its left.
fn right_column(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::right(RIGHT)
        .resizable(true)
        .default_size(RIGHT_W)
        .size_range(220.0..=560.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.set_min_width(r.width());
            line_left(ui, r, t.border);
            ui.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id()));
            let heights = (palette::height(app, ui.ctx()), props::height(app, ui.ctx()));
            let tall = legend::MIN_HEIGHT + heights.0 + heights.1;
            if r.height() >= tall {
                right_parts(app, ui, r, heights, true);
            } else {
                let mut column = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::top_down(egui::Align::Min)));
                egui::ScrollArea::vertical().id_salt(wid("right-scroll", ())).auto_shrink([false, false]).show(&mut column, |ui| {
                    let (_, content) = ui.allocate_space(vec2(ui.available_width(), tall));
                    right_parts(app, ui, content, heights, false);
                });
            }
            // Its panels pop out to its left, level with its top.
            let keep = ui.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id())).unwrap_or_default();
            crate::dock::set_flyout(ui.ctx(), crate::dock::Flyout { left: r.left(), top: r.top() + 6.0, keep });
        });
}

/// The right column's parts in `r`: the properties at the bottom, the pen over them and the legend
/// in the rest; `heights` are the pen's and the properties'. With `hold`, when the pen's three
/// buttons come or go under the pointer its top stays where it is, so the rows under the pointer
/// don't move: the properties under it give way (pushed out of the column, or a gap over them)
/// until the pointer leaves the pen. (Scrolled, the legend keeps its least height and the pen's top stays
/// anyway.)
fn right_parts(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect, (pen_h, props_h): (f32, f32), hold: bool) {
    let mut top = r.bottom() - props_h - pen_h;
    if hold {
        let pointer = ui.ctx().pointer_hover_pos();
        let over = |t: f32, h: f32| pointer.is_some_and(|p| Rect::from_min_size(pos2(r.left(), t), vec2(r.width(), h)).contains(p));
        // Where the pen was, how tall, and whether it is held: from when its height changed under
        // the pointer until the pointer leaves it, as tall as it was at its tallest meanwhile (the
        // buttons gone from under the pointer leave it over the pen still). The properties opening
        // or folding move it.
        let mut h = pen_h;
        let mut held = false;
        if let Some((was, was_h, was_held)) = ui.data(|d| d.get_temp::<(f32, f32, bool)>(pen_top_id()))
            && over(was, was_h.max(pen_h))
            && (was_held || was_h != pen_h)
        {
            top = was.max(r.top() + legend::MIN_HEIGHT);
            h = was_h.max(pen_h);
            held = true;
        }
        let keep = over(top, h);
        ui.data_mut(|d| {
            if keep {
                d.insert_temp(pen_top_id(), (top, h, held));
            } else {
                d.remove::<(f32, f32, bool)>(pen_top_id());
            }
        });
    } else {
        ui.data_mut(|d| d.remove::<(f32, f32, bool)>(pen_top_id()));
    }
    let pen = Rect::from_min_size(pos2(r.left(), top), vec2(r.width(), pen_h));
    let props = Rect::from_min_size(pos2(r.left(), pen.bottom().max(r.bottom() - props_h)), vec2(r.width(), props_h));
    legend::show(app, ui, Rect::from_min_max(r.min, pen.right_top()));
    palette::show(app, ui, pen);
    props::show(app, ui, props);
}

/// Where the pen's top is held while the pointer is over it ([`right_parts`]).
fn pen_top_id() -> egui::Id {
    wid("pen-top-held", ())
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

/// A panel's head in `r` (32 points tall): its title, what it shows (cut short with "…" when
/// longer than the room) and a line under it. Room is left at the right for `buttons`
/// [`head_button`]s. Returns where what it shows went when it was cut short.
fn panel_head(ui: &Ui, r: Rect, title: &str, sub: &str, buttons: usize, t: &Tokens) -> Option<Rect> {
    line_below(ui, r, t.border);
    let x = text(ui, pos2(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, title, font(13.0), t.text).right() + 6.0;
    if sub.is_empty() {
        return None;
    }
    let right = r.right() - 12.0 - 24.0 * buttons as f32 - 4.0;
    let shown = text_to(ui, pos2(x, r.center().y), right, sub, font(11.0), t.text_dim);
    (width_of(ui, sub, font(11.0)) > right - x).then_some(shown)
}

/// A head of the right column's parts in `r` (the mockup's `.ph`): `title` in semibold 13.5 at
/// the left, then `sub` dimmed, cut short with "…" before room for `buttons` [`head_button`]s.
/// Returns where the title ends.
fn column_head(ui: &Ui, r: Rect, title: &str, sub: &str, buttons: usize, t: &Tokens) -> f32 {
    let end = text(ui, pos2(r.left() + 10.0, r.center().y), Align2::LEFT_CENTER, title, bold(13.5), t.text).right();
    if !sub.is_empty() {
        let right = match buttons.checked_sub(1) {
            Some(more) => column_slot(r, more).left() - 4.0,
            None => r.right() - 10.0,
        };
        text_to(ui, pos2(end + 6.0, r.center().y), right, sub, font(12.0), t.text_dim);
    }
    end
}

/// Where a right-column head's button `slot` places from the right goes (0: the last one): a
/// 24-point square whose mark sits where the mockup's `.ph .r` glyphs do, 16 in from the right.
fn column_slot(head: Rect, slot: usize) -> Rect {
    Rect::from_center_size(pos2(head.right() - 16.0 - 24.0 * slot as f32, head.center().y), vec2(24.0, 24.0))
}

/// A panel head's button `slot` places from the right (0: the last one): `icon` in a 24-point
/// square.
fn head_button(ui: &Ui, id: egui::Id, head: Rect, slot: usize, icon: &str, tip: &str, t: &Tokens) -> egui::Response {
    let r = Rect::from_center_size(pos2(head.right() - 12.0 - 12.0 - 24.0 * slot as f32, head.center().y), vec2(24.0, 24.0));
    icon_button(ui, id, r, tip, t, |ui, r| crate::icons::paint(ui, icon, r.shrink(5.0), t.text))
}

/// A right-column head's button `slot` places from the right ([`column_slot`]), drawn by `mark`.
fn column_button(ui: &Ui, id: egui::Id, head: Rect, slot: usize, tip: &str, t: &Tokens, mark: impl FnOnce(&Ui, Rect)) -> egui::Response {
    icon_button(ui, id, column_slot(head, slot), tip, t, mark)
}

/// A button in `r` drawn by `mark`, the wash under it on hover.
fn icon_button(ui: &Ui, id: egui::Id, r: Rect, tip: &str, t: &Tokens, mark: impl FnOnce(&Ui, Rect)) -> egui::Response {
    let resp = ui.interact(r, id, Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(6), t.hover);
    }
    mark(ui, r);
    resp.on_hover_text(tip)
}

/// The ⋯ button of a right-column head that pops a panel out beside the column, or puts it away
/// ([`column_button`]). A press on it doesn't count as one away from the popped-out panel.
fn more_button(ui: &Ui, id: egui::Id, head: Rect, slot: usize, tip: &str, t: &Tokens) -> egui::Response {
    let resp = column_button(ui, id, head, slot, tip, t, |ui, r| crate::icons::paint(ui, "ellipsis", r.shrink(5.0), t.text));
    ui.data_mut(|d| d.get_temp_mut_or_default::<Vec<Rect>>(keep_id()).push(resp.rect));
    resp
}

/// `n` of something as said: `one` for one, else `other` (each with `{n}` where the count goes).
fn count(n: usize, one: &str, other: &str) -> String {
    crate::i18n::fmt(if n == 1 { one } else { other }, &[("n", &n.to_string())])
}

/// The selected objects `ids`, in their order, each with the top-level layer it is in, found in
/// one walk of the drawing that stops once all are found (`Document::node` and `layer_of` walk it
/// for each id, which a selection of thousands can't afford).
fn selected_in_rows<'a>(
    doc: &'a vectorcraft_doc::Document,
    ids: &[vectorcraft_doc::NodeId],
) -> Vec<(&'a vectorcraft_doc::Node, &'a vectorcraft_doc::Node)> {
    use std::collections::{HashMap, HashSet};
    use vectorcraft_doc::{Node, NodeId};
    type Found<'a> = HashMap<NodeId, (&'a Node, &'a Node)>;
    fn walk<'a>(n: &'a Node, row: &'a Node, wanted: &HashSet<NodeId>, found: &mut Found<'a>) {
        if wanted.contains(&n.id) {
            found.insert(n.id, (n, row));
        }
        for child in n.children().into_iter().flatten() {
            if found.len() == wanted.len() {
                return;
            }
            walk(child, row, wanted, found);
        }
    }
    let wanted: HashSet<NodeId> = ids.iter().copied().collect();
    let mut found = Found::with_capacity(wanted.len());
    for layer in &doc.layers {
        if found.len() == wanted.len() {
            break;
        }
        walk(layer, layer, &wanted, &mut found);
    }
    ids.iter().filter_map(|id| found.get(id).copied()).collect()
}

/// Where the ⋯ buttons of the right column are kept while it is drawn.
fn keep_id() -> egui::Id {
    wid("flyout-keep", ())
}

/// A small button labelled `label` at `at` (its left centre), `h` points tall (the mockup's
/// `.btn`): ink on the surface, or dimmed and not pressable when not `enabled`.
fn small_button(ui: &Ui, id: egui::Id, at: Pos2, h: f32, label: &str, enabled: bool, t: &Tokens) -> egui::Response {
    let f = font(11.0);
    let r = Rect::from_min_size(pos2(at.x, at.y - h / 2.0), vec2(width_of(ui, label, f.clone()) + 16.0, h));
    let resp = ui.interact(r, id, if enabled { Sense::click() } else { Sense::hover() });
    let fill = if !enabled {
        t.panel_darker
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    field_box(ui, r, fill, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, if enabled { t.text } else { t.text_disabled });
    resp
}

/// Paint `s` from `at` (its left centre) up to `right`, cut short with "…" when it is longer.
/// Returns where it went.
fn text_to(ui: &Ui, at: Pos2, right: f32, s: &str, font: FontId, color: Color32) -> Rect {
    let mut job = egui::text::LayoutJob::simple_singleline(s.to_string(), font, color);
    job.wrap = egui::text::TextWrapping { max_width: (right - at.x).max(0.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
    let galley = ui.painter().layout_job(job);
    let r = Rect::from_min_size(pos2(at.x, at.y - galley.size().y / 2.0), galley.size());
    ui.painter().with_clip_rect(Rect::from_min_max(pos2(at.x, r.top()), pos2(right.max(at.x), r.bottom()))).galley(r.min, galley, color);
    r
}

/// A small button a later job brings, labelled `label` at `at` (its left centre), 26 points tall.
/// Returns where it ends.
#[allow(dead_code)]
fn button_soon(ui: &Ui, id: egui::Id, at: Pos2, label: &str, strong: bool, job: &str, t: &Tokens) -> f32 {
    let f = if strong { bold(11.0) } else { font(11.0) };
    let r = Rect::from_min_size(pos2(at.x, at.y - 13.0), vec2(width_of(ui, label, f.clone()) + 16.0, 26.0));
    field_box(ui, r, t.panel_darker, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, t.text_disabled);
    soon_widget(ui, id, r, label, job);
    r.right()
}

/// Where a toggle labelled `label` goes from `at` (its left centre), `h` tall: its text in 12
/// points and 10 either side.
fn toggle_rect(ui: &Ui, at: Pos2, h: f32, label: &str) -> Rect {
    Rect::from_min_size(pos2(at.x, at.y - h / 2.0), vec2(width_of(ui, label, font(12.0)) + 20.0, h))
}

/// A toggle labelled `label` in `r` (the mockup's `.tog`): on, oat inside the ring.
fn toggle(ui: &Ui, id: egui::Id, r: Rect, label: &str, on: bool, t: &Tokens) -> egui::Response {
    let resp = ui.interact(r, id, Sense::click());
    let fill = if on {
        t.row_selected
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    field_box(ui, r, fill, if on { t.ring } else { t.input_border });
    text(ui, r.center(), Align2::CENTER_CENTER, label, font(12.0), t.text);
    resp
}

/// A toggle a later job brings, labelled `label`, in `r`.
fn toggle_soon(ui: &Ui, id: egui::Id, r: Rect, label: &str, job: &str, t: &Tokens) {
    field_box(ui, r, t.panel_darker, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, font(12.0), t.text_disabled);
    soon_widget(ui, id, r, label, job);
}

/// The line types a row and the palette offer: key (`legend.set`'s `line`, the parts' ids), label
/// and type.
fn line_types() -> [(&'static str, &'static str, vectorcraft_doc::LineType); 4] {
    use vectorcraft_doc::LineType;
    [
        ("solid", tw!("Solid"), LineType::Solid),
        ("dashed", tw!("Dashed"), LineType::Dashed),
        ("dotted", tw!("Dotted"), LineType::Dotted),
        ("dashdot", tw!("Dash-dot"), LineType::DashDot),
    ]
}

/// One part of a segmented choice: its key (in its id), its label, whether it is on, and when a
/// later job brings it, what it sets and that job.
struct Seg<'a> {
    key: &'a str,
    label: &'a str,
    on: bool,
    soon: Option<(&'a str, &'a str)>,
}

/// How a segmented choice is drawn: its height, the room either side of a label and the labels'
/// font.
struct SegLook {
    h: f32,
    pad: f32,
    font: FontId,
}

/// A segmented choice of `parts` from `at` (its left centre), each part `wid(kind, key)` (the
/// mockup's `.seg`): the part that is on in oat inside an ink edge, drawn last so all four of its
/// sides show; a part a later job brings on the wash, saying so when pressed. Returns the key of
/// the part pressed and where the choice ends.
fn segments<'a>(ui: &Ui, kind: &str, at: Pos2, look: &SegLook, parts: &[Seg<'a>], t: &Tokens) -> (Option<&'a str>, f32) {
    let widths: Vec<f32> = parts.iter().map(|p| width_of(ui, p.label, look.font.clone()) + 2.0 * look.pad).collect();
    let r = Rect::from_min_size(pos2(at.x, at.y - look.h / 2.0), vec2(widths.iter().sum(), look.h));
    let rects: Vec<Rect> = widths
        .iter()
        .scan(r.left(), |x, w| {
            let part = Rect::from_min_size(pos2(*x, r.top()), vec2(*w, look.h));
            *x += w;
            Some(part)
        })
        .collect();
    // A part at an end has the box's outer corners.
    let last = parts.len().saturating_sub(1);
    let corners = |i: usize, radius: u8| {
        let (start, end) = (if i == 0 { radius } else { 0 }, if i == last { radius } else { 0 });
        CornerRadius { nw: start, sw: start, ne: end, se: end }
    };
    field_box(ui, r, t.input, t.input_border);
    let mut pressed = None;
    for (i, (p, part)) in parts.iter().zip(&rects).enumerate() {
        let id = wid(kind, p.key);
        let inside = part.intersect(r.shrink(1.0));
        match p.soon {
            Some((what, job)) => {
                ui.painter().rect_filled(inside, corners(i, 7), t.panel_darker);
                soon_widget(ui, id, *part, what, job);
            }
            None => {
                let resp = ui.interact(*part, id, Sense::click());
                if resp.hovered() && !p.on {
                    ui.painter().rect_filled(inside, corners(i, 7), t.hover);
                }
                if resp.clicked() {
                    pressed = Some(p.key);
                }
            }
        }
        // A line between two parts, unless one of them is on: its edge stands there.
        let before = i.checked_sub(1).and_then(|j| parts.get(j)).map(|q| q.on);
        if before.is_some_and(|on| !on) && !p.on {
            ui.painter().vline(part.left() + 0.5, part.y_range().shrink(1.0), Stroke::new(1.0, t.border));
        }
    }
    for (i, (p, part)) in parts.iter().zip(&rects).enumerate() {
        if p.on {
            ui.painter().rect(*part, corners(i, 8), t.row_selected, Stroke::new(1.0, t.text), StrokeKind::Inside);
        }
        text(ui, part.center(), Align2::CENTER_CENTER, p.label, look.font.clone(), if p.soon.is_some() { t.text_disabled } else { t.text });
    }
    (pressed, r.right())
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

/// The hover text of `what` a later job brings: "Cut Face · Coming soon · J014".
fn soon(what: &str, job: &str) -> String {
    crate::i18n::fmt(tw!("{what} · Coming soon · {job}"), &[("what", what), ("job", job)])
}

/// How long the status bar shows a note, and a message, before it shows the hint again (seconds).
const NOTE_SECS: f64 = 4.0;
const MESSAGE_SECS: f64 = 6.0;
/// How long it shows an error (design 3.0 §5 ⑩), in red.
const ERROR_SECS: f64 = 5.0;

/// Where the status bar's note ([`note`]) is kept, with when.
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

/// Say in the status bar that `what` comes with `job`; a name that opens a dialog loses its "…"
/// before the colon.
fn note_soon(ctx: &egui::Context, what: &str, job: &str) {
    let what = what.strip_suffix('…').unwrap_or(what);
    note(ctx, &crate::i18n::fmt(tw!("{what}: coming soon ({job})"), &[("what", what), ("job", job)]));
}

/// Say `s` in the status bar for [`NOTE_SECS`].
fn note(ctx: &egui::Context, s: &str) {
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(note_id(), (s.to_string(), now)));
}

/// The note a press on something a later job brings, or a [`note`], left, for [`NOTE_SECS`].
fn soon_note(ctx: &egui::Context) -> Option<String> {
    let (note, at) = ctx.data(|d| d.get_temp::<(String, f64)>(note_id()))?;
    let left = NOTE_SECS - (ctx.input(|i| i.time) - at);
    // Drawn again when it is due to go.
    (left > 0.0).then(|| {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
        note
    })
}

/// Where the next line goes.
struct DrawTarget {
    /// Why that is not the current layer (when the current one is locked or hidden), or that no
    /// layer takes it.
    note: Option<String>,
}

/// A locked or hidden layer takes no new art: the engine puts it in the topmost layer that is shown
/// and unlocked instead ([`vectorcraft_engine::DocState::insertion_parent`]), or refuses it. The
/// status bar and the legend say so.
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
    DrawTarget { note }
}

/// The active tool's hint at `size` in `color`, plainly as in the mockup, its parts apart by "·".
fn hint_job(app: &VectorcraftApp, size: f32, color: Color32) -> egui::text::LayoutJob {
    let all: String = crate::chrome::current_hint(app).into_iter().map(|(s, _)| s).collect();
    let shown = all.split('|').map(str::trim).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" · ");
    egui::text::LayoutJob::simple_singleline(shown, font(size), color)
}

#[cfg(test)]
mod tests;
