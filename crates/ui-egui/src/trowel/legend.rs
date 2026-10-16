//! The legend, the top of the right column taking the height left (design 3.0 §5 ⑦, §7): one row
//! per top-level layer, in sections: ★ Used Often, the legend groups (Plan, Section, Finds; the one
//! chosen in the head first), Underlays (template layers) and the other rows. A legend row (J005,
//! `vectorcraft_doc::Legend`) is the look of its layer's lines: it shows its eye and padlock, a
//! sample drawn at its weight, its name (a favourite's in clay after ★), 「따로 N」 when N of its
//! lines are set apart from its look, its weight and how many objects it holds.
//!
//! A click picks a row for the pen (`legend.pick`), and with a selection tool in hand moves the
//! selection into it; a double click opens its editor and leaves the selection where it was. A
//! locked or hidden row isn't picked: the status bar says why. The eye and padlock show, hide, lock
//! and unlock the layer (Alt and the eye: this row alone), and the count selects the row's objects.
//! A right click edits, selects, stars, moves up or down, or deletes the row. The head's ＋ makes a
//! new row and opens its editor, and ⋯ lists what is done now and then (Weights at Once among
//! them).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::json;
use vectorcraft_doc::legend::{MM, lines, stroke_of};
use vectorcraft_doc::{Document, Legend, LegendGroup, LegendSample, Node, NodeId};

use super::*;

/// The head (the mockup's `.ph`), and the room above and under the list (`.lg`).
const HEAD_H: f32 = 36.0;
const LIST_PAD: f32 = 4.0;
/// A row (the mockup's `.row`).
const ROW_H: f32 = 27.0;
/// A section's caption: 6 above its text, 14 for it, 3 under it (the mockup's `.lgcap`).
const CAPTION_H: f32 = 6.0 + 14.0 + 3.0;
/// How tall the legend has to be to list two rows under a caption.
pub(super) const MIN_HEIGHT: f32 = HEAD_H + LIST_PAD + CAPTION_H + 2.0 * ROW_H + LIST_PAD;
/// A row's columns (the mockup's `.row`): 12 in from the left, eye 16, padlock 16, sample 34, the
/// name, the weight, the count 26, 6 between them and 10 at the right.
const ROW_IN: f32 = 12.0;
const ROW_OUT: f32 = 10.0;
const GAP: f32 = 6.0;
const ICON_W: f32 = 16.0;
const COUNT_W: f32 = 26.0;
const SAMPLE: egui::Vec2 = vec2(34.0, 10.0);
/// How long a paper millimetre of a line type's dashes is drawn in a sample: long enough to tell
/// the types apart at a glance.
const SAMPLE_MM: f64 = 1.5;
/// The ratio Weights at Once starts with.
pub(super) const RATIO: f64 = 0.8;
/// How soon a second click on a row opens its editor (seconds): a system's usual double click.
const DOUBLE_CLICK: f64 = 0.5;

/// A layer's row as shown.
struct Row<'a> {
    id: NodeId,
    name: String,
    visible: bool,
    locked: bool,
    /// Hidden, or an underlay (a template layer): its name shows faint.
    dim: bool,
    underlay: bool,
    colour: Color32,
    current: bool,
    /// The current layer's when it is locked or hidden: where lines go instead ([`draw_target`]).
    note: Option<String>,
    /// How many objects it holds.
    count: usize,
    legend: Option<&'a Legend>,
    /// How many of its lines are set apart from its look.
    apart: usize,
    /// The rows over and under it in its section, for Up and Down.
    over: Option<NodeId>,
    under: Option<NodeId>,
}

impl Row<'_> {
    fn favourite(&self) -> bool {
        self.legend.is_some_and(|l| l.favourite)
    }
}

/// What the list shows, top to bottom.
enum Item<'a> {
    /// A section's caption.
    Caption(&'static str),
    Layer(Row<'a>),
}

impl Item<'_> {
    fn height(&self) -> f32 {
        match self {
            Item::Caption(_) => CAPTION_H,
            Item::Layer(_) => ROW_H,
        }
    }
}

/// What a click on the list asks for.
enum Act {
    Pick(NodeId),
    /// Open a row's editor level with its row's top.
    Edit(NodeId, f32),
    Show(NodeId, bool),
    /// Alt and the eye: this row alone, or every row again when it already is.
    ShowAlone(NodeId),
    Lock(NodeId, bool),
    /// Select the row's objects.
    SelectAll(NodeId),
    Favourite(NodeId, bool),
    /// Move a row over (`"above"`) or under (`"below"`) another.
    Move(NodeId, NodeId, &'static str),
    Delete(NodeId),
}

/// What the head's ⋯ lists.
#[derive(Clone, Copy)]
enum More {
    EditRow,
    FieldStandard,
    Prune,
    Weights,
    Styles,
    Layers,
}

/// A card beside the right column: a row's editor or Weights at Once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Card {
    Row(NodeId),
    Weights,
}

/// The card that is open, for which drawing (it closes when another one comes to the front), how
/// far down (a row's editor opens level with its row, when not given as it is laid out; Weights at
/// Once level with the legend's head) and since which pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Open {
    pub(super) card: Card,
    pub(super) doc: u64,
    pub(super) top: Option<f32>,
    pub(super) since: u64,
}

/// Where the open card is kept.
fn card_id() -> egui::Id {
    wid("legend-card", ())
}

/// Open `card` for the active drawing, from `top` (see [`Open`]); Weights at Once opens as it
/// starts.
pub(super) fn open_card(app: &VectorcraftApp, ctx: &egui::Context, card: Card, top: Option<f32>) {
    if let Some(st) = app.session.active() {
        if card == Card::Weights {
            super::weights::forget(ctx);
        }
        let since = ctx.cumulative_pass_nr();
        ctx.data_mut(|d| d.insert_temp(card_id(), Open { card, doc: st.uid, top, since }));
    }
}

/// Show the open card beside the column whose visible part `ui` clips to; `r` is the legend's.
fn show_card(app: &mut VectorcraftApp, ui: &Ui, r: Rect) {
    let Some(open) = ui.data(|d| d.get_temp::<Open>(card_id())) else { return };
    let column = ui.clip_rect();
    let ctx = ui.ctx();
    // Over the canvas at the column's left, within its height and under the tab row.
    let top = app.canvas_rect.map_or(column.top(), |c| c.top().max(column.top()));
    let bounds = Rect::from_min_max(pos2(ctx.content_rect().left(), top), pos2(column.left(), column.bottom()));
    let top = match open.card {
        Card::Row(id) => open.top.or_else(|| ctx.read_response(wid("legend-row", id.0)).map(|row| row.rect.top())),
        Card::Weights => open.top,
    };
    let at = pos2(column.left() - 8.0 - super::card::W, top.unwrap_or(r.top()));
    // A press in the pass it opened in opened it, or came before it: it doesn't put it away.
    let fresh = open.since == ctx.cumulative_pass_nr();
    let place = super::card::Place { at, bounds, fresh };
    let keep = app.session.active().is_some_and(|st| st.uid == open.doc)
        && match open.card {
            Card::Row(id) => super::rowedit::show(app, ctx, place, id),
            Card::Weights => super::weights::show(app, ctx, place),
        };
    if !keep {
        ctx.data_mut(|d| d.remove::<Open>(card_id()));
    }
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
    line_below(ui, head, t.border);
    let open = app.session.active().is_some();
    let title_end = column_head(ui, head, tw!("Legend"), "", 2, &t);
    let first = group_button(app, ui, head, title_end, &t);
    let new_row = head_button(ui, wid("legend-new", ()), head, 1, "plus", tw!("New row with the pen's look: the selected lines move into it"), &t)
        .clicked()
        && open;
    let more = more_menu(ui, head, open, &t);
    let apart = app.session.active().map(|st| apart(ui, st)).unwrap_or_default();
    let area = Rect::from_min_max(pos2(r.left(), head.bottom() + LIST_PAD), pos2(r.right(), (r.bottom() - LIST_PAD).max(head.bottom())));
    let act = list(app, ui, area, &apart, &t);
    let ctx = ui.ctx().clone();
    let result = match act {
        Some(Act::Pick(id)) => pick(app, &ctx, id),
        Some(Act::Edit(id, top)) => {
            let undone = unmove(app, &ctx, id);
            open_card(app, &ctx, Card::Row(id), Some(top));
            undone.map(drop)
        }
        Some(Act::Show(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "visible": on })).map(drop),
        Some(Act::ShowAlone(id)) => show_alone(app, id),
        Some(Act::Lock(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "locked": on })).map(drop),
        Some(Act::SelectAll(id)) => select_all(app, id),
        Some(Act::Favourite(id, on)) => app.run("legend.set", json!({ "id": id.0, "favourite": on })).map(drop),
        Some(Act::Move(id, target, place)) => app.run("layer.move", json!({ "ids": [id.0], "target": target.0, "place": place })).map(drop),
        Some(Act::Delete(id)) => delete_layer_row(app, id),
        None if new_row => add_row(app, &ctx),
        None => match (more, first) {
            (Some(m), _) => run_more(app, &ctx, m),
            (None, Some(g)) => {
                if let Some(st) = app.session.active() {
                    ctx.data_mut(|d| d.insert_temp(first_id(st.uid), g));
                }
                Ok(())
            }
            (None, None) => Ok(()),
        },
    };
    if let Err(e) = result {
        app.status_error(e);
    }
    show_card(app, ui, r);
}

/// Why row `id` isn't drawn with: its layer is hidden or locked.
pub(super) fn refused(st: &vectorcraft_engine::DocState, id: NodeId) -> Option<String> {
    let layer = st.doc.node(id)?;
    let why = if !layer.visible {
        tw!("{row} is hidden: show it to draw with it")
    } else if layer.locked {
        tw!("{row} is locked: unlock it to draw with it")
    } else {
        return None;
    };
    Some(crate::i18n::fmt(why, &[("row", &layer.display_name())]))
}

/// Where the last pick that moved the selection is kept: the row, the drawing and the drawing's
/// revision after the move ([`unmove`]).
fn moved_id() -> egui::Id {
    wid("legend-moved", ())
}

/// Pick row `id` for the pen, unless its layer is hidden or locked ([`refused`]). With a selection
/// tool in hand the selection moves into it; a drawing tool's line just drawn stays selected, and
/// stays where it was drawn when the next row is picked.
fn pick(app: &mut VectorcraftApp, ctx: &egui::Context, id: NodeId) -> Result<(), String> {
    ctx.data_mut(|d| d.remove::<(NodeId, u64, u64)>(moved_id()));
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("trowel-last-clicked-layer"), id));
    if let Some(why) = app.session.active().and_then(|st| refused(st, id)) {
        return Err(why);
    }
    let tool = app.session.tool_id();
    let carry = (vectorcraft_tools::catalog::is_selection_tool(tool) || matches!(tool, "lasso" | "magicWand"))
        && app.session.active().is_some_and(|st| !st.selection.objects.is_empty());
    let moved = app.run("legend.pick", json!({ "id": id.0, "move": carry }))?["moved"].as_u64().unwrap_or(0);
    if moved > 0
        && let Some(st) = app.session.active()
    {
        ctx.data_mut(|d| d.insert_temp(moved_id(), (id, st.uid, st.revision)));
        let row = st.doc.node(id).map(|n| n.display_name()).unwrap_or_default();
        app.status(crate::i18n::fmt(tw!("Moved {n} to {row}"), &[("n", &moved.to_string()), ("row", &row)]));
    }
    Ok(())
}

/// A double click's first click on row `id` moved the selection into it: take that move back, the
/// row staying the one drawn with, so a double click only opens the row's editor. Nothing is taken
/// back when the drawing changed in between. Returns whether lines went back.
fn unmove(app: &mut VectorcraftApp, ctx: &egui::Context, id: NodeId) -> Result<bool, String> {
    let Some((row, doc, revision)) = ctx.data_mut(|d| d.remove_temp::<(NodeId, u64, u64)>(moved_id())) else { return Ok(false) };
    if row != id || !app.session.active().is_some_and(|st| st.uid == doc && st.revision == revision) {
        return Ok(false);
    }
    app.run("edit.undo", json!({}))?;
    app.run("legend.pick", json!({ "id": id.0 }))?;
    app.status(tw!("The selected lines stay where they were"));
    Ok(true)
}

/// Put lines set apart back to their row's look (`legend.revert` with `params`) and say how many.
pub(super) fn revert(app: &mut VectorcraftApp, params: serde_json::Value) -> Result<(), String> {
    let done = app.run("legend.revert", params)?;
    let (n, skipped) = (done["lines"].as_u64().unwrap_or(0), done["skipped"].as_u64().unwrap_or(0));
    let template =
        if skipped > 0 { tw!("Lines back to the legend: {n} · locked ones left as they were: {m}") } else { tw!("Lines back to the legend: {n}") };
    app.status(crate::i18n::fmt(template, &[("n", &n.to_string()), ("m", &skipped.to_string())]));
    Ok(())
}

/// Show row `id` alone (the others hidden), or every row again when it already is.
fn show_alone(app: &mut VectorcraftApp, id: NodeId) -> Result<(), String> {
    let Some(st) = app.session.active() else { return Ok(()) };
    let alone = st.doc.layers.iter().all(|l| (l.id == id) == l.visible);
    if alone {
        return app.run("layer.showAll", json!({})).map(drop);
    }
    if st.doc.node(id).is_some_and(|l| !l.visible) {
        app.run("layer.setProps", json!({ "ids": [id.0], "visible": true }))?;
    }
    app.run("layer.hideOthers", json!({ "ids": [id.0] })).map(drop)
}

/// Select row `id`'s objects and say how many.
fn select_all(app: &mut VectorcraftApp, id: NodeId) -> Result<(), String> {
    let done = app.run("layer.selectAll", json!({ "id": id.0 }))?;
    let n = done["selected"].as_array().map_or(0, Vec::len);
    app.status(crate::i18n::fmt(tw!("Selected: {n}"), &[("n", &n.to_string())]));
    Ok(())
}

/// A new row (a new layer) and its editor beside it.
fn add_row(app: &mut VectorcraftApp, ctx: &egui::Context) -> Result<(), String> {
    let Some(st) = app.session.active() else { return Ok(()) };
    let taken: HashSet<String> = st.doc.layers.iter().map(|l| l.display_name()).collect();
    // One of the first taken + 1 numbers is free.
    let name = (1..=taken.len() + 1)
        .map(|n| crate::i18n::fmt(tw!("Legend Row {n}"), &[("n", &n.to_string())]))
        .find(|s| !taken.contains(s))
        .unwrap_or_default();
    let done = app.run("legend.add", json!({ "name": name }))?;
    let Some(id) = done["id"].as_u64() else { return Ok(()) };
    let moved = done["moved"].as_u64().unwrap_or(0);
    if moved > 0 {
        app.status(crate::i18n::fmt(tw!("Moved {n} to {row}"), &[("n", &moved.to_string()), ("row", &name)]));
    }
    open_card(app, ctx, Card::Row(NodeId(id)), None);
    Ok(())
}

/// Delete a layer row (unlocking it first if locked so template layers can also be deleted).
pub(super) fn delete_layer_row(app: &mut VectorcraftApp, id: NodeId) -> Result<(), String> {
    let (is_locked, total, has_legend, has_children) = app
        .session
        .active()
        .and_then(|st| {
            let n = st.doc.node(id)?;
            Some((n.locked, st.doc.layers.len(), n.legend().is_some(), n.children().is_some_and(|c| !c.is_empty())))
        })
        .unwrap_or((false, 0, false, false));
    if is_locked {
        let _ = app.run("layer.setProps", json!({ "ids": [id.0], "locked": false }));
    }
    if total > 1 {
        app.run("layer.delete", json!({ "ids": [id.0] })).map(drop)
    } else if has_legend || has_children {
        let _ = app.run("layer.new", json!({}));
        app.run("layer.delete", json!({ "ids": [id.0] })).map(drop)
    } else {
        Ok(())
    }
}

/// Remove the rows holding nothing but the current one, and say how many went.
fn prune_empty_layers(app: &mut VectorcraftApp) -> Result<(), String> {
    let Some((current, empty_ids)) = app.session.active().map(|st| {
        let cur = st.current_layer().and_then(|l| st.doc.layer_of(l));
        let ids: Vec<(u64, bool)> =
            st.doc.layers.iter().filter(|l| Some(l.id) != cur && l.children().is_none_or(|c| c.is_empty())).map(|l| (l.id.0, l.locked)).collect();
        (cur, ids)
    }) else {
        return Ok(());
    };
    let mut removed = 0usize;
    for (id, locked) in empty_ids {
        if app.session.active().is_some_and(|st| st.doc.layers.len() <= 1) {
            break;
        }
        if locked {
            let _ = app.run("layer.setProps", json!({ "ids": [id], "locked": false }));
        }
        if app.run("layer.delete", json!({ "ids": [id] })).is_ok() {
            removed += 1;
        }
    }
    if let Some(cur) = current {
        let _ = app.run("layer.setCurrent", json!({ "id": cur.0 }));
    }
    app.status(crate::i18n::fmt(tw!("Unused rows removed: {n}"), &[("n", &removed.to_string())]));
    Ok(())
}

/// The head's ⋯: a list of what is done now and then. Returns the one picked.
fn more_menu(ui: &Ui, head: Rect, open: bool, t: &Tokens) -> Option<More> {
    let resp = more_button(ui, wid("legend-more", ()), head, 0, tw!("More legend actions"), t);
    let mut pick = None;
    egui::Popup::menu(&resp).id(wid("legend-more-list", ())).show(|ui| {
        let items = [
            (More::EditRow, tw!("Edit Row…")),
            (More::FieldStandard, tw!("Put the 12 Field Standard Rows Back")),
            (More::Prune, tw!("Remove Unused Rows")),
            (More::Weights, tw!("Change Weights at Once…")),
            (More::Styles, tw!("Save or Load a Style Template…")),
            (More::Layers, tw!("Layers Panel")),
        ];
        for (i, (item, label)) in items.into_iter().enumerate() {
            if i == 3 || i == 5 {
                ui.separator();
            }
            // Without a drawing only the Layers panel opens.
            let enabled = open || matches!(item, More::Layers);
            if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                pick = Some(item);
                ui.close();
            }
        }
    });
    pick
}

fn run_more(app: &mut VectorcraftApp, ctx: &egui::Context, more: More) -> Result<(), String> {
    match more {
        More::EditRow => {
            let row = app.session.active().and_then(|st| st.current_layer().and_then(|l| st.doc.layer_of(l)));
            if let Some(id) = row {
                open_card(app, ctx, Card::Row(id), None);
            }
            Ok(())
        }
        More::FieldStandard => {
            let done = app.run("legend.addFieldPresets", json!({}))?;
            let n = done["added"].as_array().map_or(0, Vec::len);
            let note = if n > 0 {
                crate::i18n::fmt(tw!("Legend rows added: {n}"), &[("n", &n.to_string())])
            } else {
                tw!("The 12 field standard rows are all there").to_string()
            };
            app.status(note);
            Ok(())
        }
        More::Prune => prune_empty_layers(app),
        More::Weights => {
            open_card(app, ctx, Card::Weights, None);
            Ok(())
        }
        More::Styles => {
            super::style_dialog::toggle_dialog(ctx);
            Ok(())
        }
        More::Layers => {
            crate::menus::invoke(app, "window.panel", json!({ "panel": "layers" }));
            Ok(())
        }
    }
}

/// The lines set apart from their row's look, counted again when the drawing changes.
#[derive(Default)]
pub(super) struct Apart {
    /// How many, by row.
    rows: HashMap<NodeId, usize>,
    pub(super) lines: HashSet<NodeId>,
}

pub(super) fn apart(ui: &Ui, st: &vectorcraft_engine::DocState) -> Arc<Apart> {
    let now = (st.uid, st.revision);
    let id = wid("legend-apart", ());
    if let Some((was, apart)) = ui.data(|d| d.get_temp::<((u64, u64), Arc<Apart>)>(id))
        && was == now
    {
        return apart;
    }
    let mut apart = Apart::default();
    for layer in &st.doc.layers {
        let Some(legend) = layer.legend() else { continue };
        for line in lines(layer) {
            if stroke_of(line).is_some_and(|s| legend.differs(s)) {
                apart.lines.insert(line.id);
                *apart.rows.entry(layer.id).or_default() += 1;
            }
        }
    }
    let apart = Arc::new(apart);
    ui.data_mut(|d| d.insert_temp(id, (now, apart.clone())));
    apart
}

/// The legend group most of the drawing's rows come from, the first in the groups' order (plan,
/// section, finds) when two have as many.
pub(super) fn main_group(doc: &Document) -> Option<LegendGroup> {
    let mut counts = [0usize; 3];
    for g in doc.layers.iter().filter_map(|l| l.legend().and_then(|l| l.group)) {
        if let Some(n) = LegendGroup::ALL.iter().position(|x| *x == g).and_then(|i| counts.get_mut(i)) {
            *n += 1;
        }
    }
    // The last of the most in reverse: the first in the groups' order.
    LegendGroup::ALL.iter().zip(counts).rev().filter(|(_, n)| *n > 0).max_by_key(|(_, n)| *n).map(|(g, _)| *g)
}

/// Where the group a drawing's legend shows first is kept.
fn first_id(doc: u64) -> egui::Id {
    wid("legend-first", doc)
}

/// The group shown first, after ★: the one chosen in the head, else the one most rows come from,
/// else plans.
fn first_group(app: &VectorcraftApp, ctx: &egui::Context) -> LegendGroup {
    let Some(st) = app.session.active() else { return LegendGroup::Plan };
    ctx.data(|d| d.get_temp::<LegendGroup>(first_id(st.uid))).or_else(|| main_group(&st.doc)).unwrap_or(LegendGroup::Plan)
}

/// A legend group's name, as its section's caption and in the head.
fn group_name(g: LegendGroup) -> &'static str {
    match g {
        LegendGroup::Plan => tw!("Plan"),
        LegendGroup::Section => tw!("Section"),
        LegendGroup::Finds => tw!("Finds"),
    }
}

/// The head's group, after its title (the mockup's 「평면 ▾」): the group shown first, and the
/// list of the three. Returns the one picked.
fn group_button(app: &VectorcraftApp, ui: &Ui, head: Rect, title_end: f32, t: &Tokens) -> Option<LegendGroup> {
    let shown = first_group(app, ui.ctx());
    let label = group_name(shown);
    let cy = head.center().y;
    let x = title_end + 6.0;
    let f = font(12.0);
    let r = Rect::from_min_max(pos2(x - 4.0, cy - 11.0), pos2(x + width_of(ui, label, f.clone()) + 4.0 + 7.0 + 4.0, cy + 11.0));
    let open = app.session.active().is_some();
    let resp = ui.interact(r, wid("legend-group", ()), if open { Sense::click() } else { Sense::hover() });
    if resp.hovered() && open {
        ui.painter().rect_filled(r, CornerRadius::same(4), t.hover);
    }
    let color = if open { t.text_dim } else { t.text_disabled };
    let said = text(ui, pos2(x, cy), Align2::LEFT_CENTER, label, f, color);
    triangle(ui, pos2(said.right() + 4.0 + 3.5, cy), 7.0, true, color);
    let resp = resp.on_hover_text(tw!("Which legend group comes first"));
    let mut pick = None;
    egui::Popup::menu(&resp).id(wid("legend-group-list", ())).show(|ui| {
        for g in LegendGroup::ALL {
            if ui.selectable_label(g == shown, group_name(g)).clicked() {
                pick = Some(g);
                ui.close();
            }
        }
    });
    pick
}

/// The rows, top first, in their sections. Only the rows in view are laid out.
fn list(app: &VectorcraftApp, ui: &mut Ui, area: Rect, apart: &Apart, t: &Tokens) -> Option<Act> {
    let items = items(app, ui.ctx(), apart);
    // Where each item starts, from the top of the list.
    let tops: Vec<f32> = items
        .iter()
        .scan(0.0, |y, item| {
            let top = *y;
            *y += item.height();
            Some(top)
        })
        .collect();
    let total: f32 = items.iter().map(Item::height).sum();
    let click = clicked(ui, area);
    let mut act = None;
    let mut list = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    // No fading edges: a faded row reads as a hidden layer.
    list.spacing_mut().scroll.fade.strength = 0.0;
    // Exactly `area`, however short: egui would make it at least 64 tall, over the pen.
    let scroll = egui::ScrollArea::vertical().id_salt(wid("legend-scroll", ())).auto_shrink([false, false]).min_scrolled_height(0.0);
    scroll.show_viewport(&mut list, |ui, view| {
        ui.set_height(total);
        let origin = ui.max_rect().min;
        let w = ui.available_width();
        // The first item in view is the last one starting at or above the view's top.
        let first = tops.partition_point(|top| *top <= view.top()).saturating_sub(1);
        for (item, top) in items.iter().zip(&tops).skip(first) {
            if *top >= view.bottom() {
                break;
            }
            match item {
                Item::Caption(label) => {
                    let cy = origin.y + top + 6.0 + 7.0;
                    text_to(ui, pos2(origin.x + ROW_IN, cy), origin.x + w - ROW_OUT, label, font(11.0), t.text_dim);
                }
                Item::Layer(row) => {
                    let rr = Rect::from_min_size(origin + vec2(0.0, *top), vec2(w, ROW_H));
                    if let Some(a) = layer_row(ui, row, rr, t) {
                        act = Some(a);
                    }
                }
            }
        }
        // A second click opens the row first clicked, whatever is under it now.
        if let Some(id) = click.and_then(|c| c.second)
            && let Some(top) = items.iter().zip(&tops).find_map(|(item, top)| matches!(item, Item::Layer(row) if row.id == id).then_some(*top))
        {
            act = Some(Act::Edit(id, origin.y + top));
        }
    });
    if let Some(c) = click {
        // The click a second one may follow: one picking a row.
        let first = match &act {
            Some(Act::Pick(id)) if c.second.is_none() => Some((*id, c.time, c.at)),
            _ => None,
        };
        ui.data_mut(|d| match first {
            Some(first) => {
                d.insert_temp(last_click(), first);
            }
            None => d.remove::<(NodeId, f64, Pos2)>(last_click()),
        });
    }
    act
}

/// A click that ended in this pass: when and where, and the row it opens ([`Act::Edit`]) when it
/// is a second click on the list soon after one picking that row, as near to it as egui's own
/// clicks go. That row, whatever is under the pointer now: the first click's pick may have moved
/// lines out of a row above. Counted here, not by egui, which takes the second click for a third
/// one when another click (✕ on the card) came just before.
#[derive(Clone, Copy)]
struct Click {
    time: f64,
    at: Pos2,
    second: Option<NodeId>,
}

/// Where the click a second one may follow is kept: the row it picked, when and where.
fn last_click() -> egui::Id {
    wid("legend-last-click", ())
}

/// This pass's click, if one ended in it ([`Click`]); `area` is the list's.
fn clicked(ui: &Ui, area: Rect) -> Option<Click> {
    // Where the button came up: a touch or a pen leaves no pointer behind it.
    let (time, at) = ui.input(|i| {
        let up = |e: &egui::Event| match e {
            egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, .. } => Some(*pos),
            _ => None,
        };
        (i.time, if i.pointer.primary_clicked() { i.events.iter().rev().find_map(up) } else { None })
    });
    let at = at?;
    let near = ui.ctx().options(|o| o.input_options.max_click_dist);
    let on_list = ui.clip_rect().intersect(area).contains(at) && ui.ctx().layer_id_at(at) == Some(ui.layer_id());
    let before: Option<(NodeId, f64, Pos2)> = ui.data(|d| d.get_temp(last_click()));
    let second = before.filter(|(_, when, was)| on_list && time - when <= DOUBLE_CLICK && at.distance(*was) <= near).map(|(id, _, _)| id);
    Some(Click { time, at, second })
}

/// The drawing's layers in the legend's sections, top first: ★ Used Often, the legend groups with
/// `first` first, Underlays (template layers) and the other rows, each in the layers' order. An
/// underlay goes under Underlays, starred or not; a section without rows is left out.
pub(super) fn sections(doc: &Document, first: LegendGroup) -> Vec<(&'static str, Vec<&Arc<Node>>)> {
    let groups: Vec<LegendGroup> = std::iter::once(first).chain(LegendGroup::ALL.into_iter().filter(|g| *g != first)).collect();
    let mut favourites = vec![];
    let mut by_group: Vec<Vec<&Arc<Node>>> = vec![vec![]; groups.len()];
    let mut underlays = vec![];
    let mut others = vec![];
    for l in doc.layers.iter().rev() {
        let legend = l.legend();
        if l.is_template() {
            underlays.push(l);
        } else if legend.is_some_and(|g| g.favourite) {
            favourites.push(l);
        } else if let Some(slot) = legend.and_then(|g| g.group).and_then(|g| groups.iter().position(|x| *x == g)).and_then(|i| by_group.get_mut(i)) {
            slot.push(l);
        } else {
            others.push(l);
        }
    }
    let mut out = vec![(tw!("★ Used Often"), favourites)];
    out.extend(groups.into_iter().zip(by_group).map(|(g, rows)| (group_name(g), rows)));
    out.push((tw!("Underlays"), underlays));
    out.push((tw!("Other rows"), others));
    out.retain(|(_, rows)| !rows.is_empty());
    out
}

/// The rows of the active drawing in their [`sections`], each under its caption.
fn items<'a>(app: &'a VectorcraftApp, ctx: &egui::Context, apart: &Apart) -> Vec<Item<'a>> {
    let Some(st) = app.session.active() else { return vec![] };
    let current = st.current_layer();
    let note = draw_target(st).note;
    let mut out = vec![];
    for (caption, layers) in sections(&st.doc, first_group(app, ctx)) {
        out.push(Item::Caption(caption));
        for (i, l) in layers.iter().enumerate() {
            let over = i.checked_sub(1).and_then(|j| layers.get(j)).map(|n| n.id);
            let under = layers.get(i + 1).map(|n| n.id);
            out.push(Item::Layer(Row {
                id: l.id,
                name: l.display_name(),
                visible: l.visible,
                locked: l.locked,
                dim: !l.visible || l.is_template(),
                underlay: l.is_template(),
                colour: crate::panels::layers::layer_colour(&st.doc, l.id),
                current: Some(l.id) == current,
                note: note.clone().filter(|_| Some(l.id) == current),
                count: l.children().map_or(0, Vec::len),
                legend: l.legend(),
                apart: apart.rows.get(&l.id).copied().unwrap_or(0),
                over,
                under,
            }));
        }
    }
    out
}

/// One row in `rr` (the mockup's `.row`): eye, padlock, sample, name (★ and 「따로 N」), weight
/// and count.
fn layer_row(ui: &Ui, row: &Row, rr: Rect, t: &Tokens) -> Option<Act> {
    let x0 = rr.left() + ROW_IN;
    let column = |a: f32, w: f32| Rect::from_min_max(pos2(a, rr.top()), pos2(a + w, rr.bottom()));
    let eye_col = column(x0 - 3.0, ICON_W + 6.0);
    let lock_col = column(x0 + ICON_W + GAP - 3.0, ICON_W + 6.0);
    let right = rr.right() - ROW_OUT;
    let count_col = column(right - COUNT_W, COUNT_W);
    let resp = ui.interact(rr, wid("legend-row", row.id.0), Sense::click());
    let eye = ui.interact(eye_col, wid("legend-eye", row.id.0), Sense::click());
    let lock = ui.interact(lock_col, wid("legend-lock", row.id.0), Sense::click());
    let count = ui.interact(count_col, wid("legend-count", row.id.0), Sense::click());
    if row.current {
        ui.painter().rect_filled(rr, 0.0, t.row_selected);
    } else if resp.hovered() || eye.hovered() || lock.hovered() || count.hovered() {
        ui.painter().rect_filled(rr, 0.0, t.hover);
    }
    let cy = rr.center().y;
    // The eye dark when shown, pale when hidden; the padlock in ink when locked, pale when not.
    let icon =
        |name: &str, cx: f32, size: f32, color: Color32| crate::icons::paint(ui, name, Rect::from_center_size(pos2(cx, cy), vec2(size, size)), color);
    icon(if row.visible { "eye" } else { "eye-off" }, x0 + ICON_W / 2.0, 14.0, if row.visible { t.icon } else { t.ring });
    icon(if row.locked { "lock" } else { "lock-open" }, x0 + ICON_W + GAP + ICON_W / 2.0, 13.0, if row.locked { t.icon } else { t.ring });
    let slot = Rect::from_min_size(pos2(x0 + 2.0 * (ICON_W + GAP), cy - SAMPLE.y / 2.0), SAMPLE);
    match row.legend {
        _ if row.underlay => {
            // The mockup's piece of a photo: a pale box.
            ui.painter().rect_filled(slot.shrink2(vec2(4.0, 0.0)), 0.0, t.border);
        }
        Some(legend) => sample(ui, slot, legend, t),
        None => {
            ui.painter().rect(slot.shrink2(vec2(4.0, 0.0)), 0.0, row.colour.gamma_multiply(0.25), Stroke::new(1.5, row.colour), StrokeKind::Inside);
        }
    }
    // At the right: the count, then the weight ("-" without one).
    text(ui, pos2(right, cy), Align2::RIGHT_CENTER, row.count.to_string(), mono(10.5), t.text_disabled);
    let weight = row.legend.and_then(|l| l.weight).map_or_else(|| "-".to_string(), pt);
    let name_end = text(ui, pos2(count_col.left() - GAP, cy), Align2::RIGHT_CENTER, &weight, mono(11.0), t.text_dim).left() - GAP;
    // The name, with 「따로 N」 after it, cut short to leave the badge room.
    let name_x = slot.right() + GAP;
    let badge_label = (row.apart > 0).then(|| crate::i18n::fmt(tw!("Apart {n}"), &[("n", &row.apart.to_string())]));
    let badge_w = badge_label.as_ref().map_or(0.0, |l| width_of(ui, l, font(10.0)) + 10.0 + 4.0);
    let (name, f, color) = if row.favourite() {
        (format!("★ {}", row.name), font(12.5), t.clay_text)
    } else if row.note.is_some() {
        (row.name.clone(), font(12.5), t.clay_text)
    } else if row.dim {
        (row.name.clone(), font(12.5), t.text_disabled)
    } else {
        (row.name.clone(), font(12.5), t.text)
    };
    let name_room = (name_end - name_x - badge_w).max(0.0);
    let said = text_to(ui, pos2(name_x, cy), name_x + name_room, &name, f.clone(), color);
    let mut badge_r = None;
    if let Some(label) = &badge_label {
        badge_r = Some(badge(ui, pos2(said.right().min(name_x + name_room) + 4.0, cy), label, t));
    }
    let eye =
        eye.on_hover_text(if row.visible { tw!("Hide · Alt+click: show this row alone") } else { tw!("Show · Alt+click: show this row alone") });
    let lock = lock.on_hover_text(if row.locked { tw!("Unlock") } else { tw!("Lock") });
    let count = count.on_hover_text(tw!("Select this row's objects"));
    // On hover, over the badge what it counts; else the note or what a click does, after the
    // whole name when it is cut short.
    let tip = if badge_r.zip(resp.hover_pos()).is_some_and(|(b, p)| b.contains(p)) {
        crate::i18n::fmt(tw!("Lines of this row with a weight or line type of their own: {n}"), &[("n", &row.apart.to_string())])
    } else {
        let tip = row.note.as_deref().unwrap_or(tw!("Draw with this row · double-click to change it"));
        if width_of(ui, &name, f) > name_room { format!("{}\n{tip}", row.name) } else { tip.to_string() }
    };
    let resp = resp.on_hover_text(tip);
    let mut menu_act = None;
    egui::Popup::context_menu(&resp).id(wid("legend-row-menu", row.id.0)).show(|ui| {
        if ui.button(tw!("Edit Row…")).clicked() {
            menu_act = Some(Act::Edit(row.id, rr.top()));
        }
        if ui.button(tw!("Select All on This Row")).clicked() {
            menu_act = Some(Act::SelectAll(row.id));
        }
        if ui.selectable_label(row.favourite(), tw!("★ Used Often")).clicked() {
            menu_act = Some(Act::Favourite(row.id, !row.favourite()));
        }
        ui.separator();
        if ui.add_enabled(row.over.is_some(), egui::Button::new(tw!("Move Up"))).clicked()
            && let Some(over) = row.over
        {
            menu_act = Some(Act::Move(row.id, over, "above"));
        }
        if ui.add_enabled(row.under.is_some(), egui::Button::new(tw!("Move Down"))).clicked()
            && let Some(under) = row.under
        {
            menu_act = Some(Act::Move(row.id, under, "below"));
        }
        ui.separator();
        if ui.button(tw!("Delete Row")).clicked() {
            menu_act = Some(Act::Delete(row.id));
        }
        if menu_act.is_some() {
            ui.close();
        }
    });
    if let Some(a) = menu_act {
        Some(a)
    } else if eye.clicked() {
        if ui.input(|i| i.modifiers.alt) { Some(Act::ShowAlone(row.id)) } else { Some(Act::Show(row.id, !row.visible)) }
    } else if lock.clicked() {
        Some(Act::Lock(row.id, !row.locked))
    } else if count.clicked() {
        Some(Act::SelectAll(row.id))
    } else if resp.clicked() {
        // A second click soon after opens the row's editor instead ([`Click`]).
        Some(Act::Pick(row.id))
    } else {
        None
    }
}

/// The mockup's `.fx` badge: `label` in white on clay, from `at` (its left centre). Returns where
/// it is.
pub(super) fn badge(ui: &Ui, at: Pos2, label: &str, t: &Tokens) -> Rect {
    let f = font(10.0);
    let r = Rect::from_min_size(pos2(at.x, at.y - 8.0), vec2(width_of(ui, label, f.clone()) + 10.0, 16.0));
    ui.painter().rect_filled(r, CornerRadius::same(4), t.action);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, t.on_action);
    r
}

/// A weight in points as the legend shows it: one decimal at least, three at most ("1.0", "0.64").
pub(super) fn pt(w: f64) -> String {
    let s = format!("{w:.3}");
    let s = s.trim_end_matches('0');
    if s.ends_with('.') { format!("{s}0") } else { s.to_string() }
}

/// A dash pattern (points) as drawn in a sample: its dashes and its gaps ([`SAMPLE_MM`] a paper
/// millimetre); `None` when too fine to show.
fn dash_px(pattern: &[f64]) -> Option<(Vec<f32>, Vec<f32>)> {
    let mut px: Vec<f32> = pattern.iter().map(|p| ((p / MM) * SAMPLE_MM).max(0.5) as f32).collect();
    // An odd pattern repeats to make dash and gap pairs, as in PDF and SVG.
    if px.len() % 2 == 1 {
        px.extend(px.clone());
    }
    if px.iter().sum::<f32>() < 2.0 {
        return None;
    }
    Some((px.iter().step_by(2).copied().collect(), px.iter().skip(1).step_by(2).copied().collect()))
}

/// A legend row's sample in `r` (design 3.0 §7): its line drawn at its weight (as the pen's chips,
/// [`super::palette::sample_px`]) in its dashes, an area row's too, or its mark; without a look, a
/// thin faint line.
pub(super) fn sample(ui: &Ui, r: Rect, legend: &Legend, t: &Tokens) {
    let look = legend.look();
    let stroke = match look {
        Some((w, _)) => Stroke::new(super::palette::sample_px(w), t.text),
        None => Stroke::new(1.0, t.text_disabled),
    };
    let dashes = look.and_then(|(_, line)| line.dash_pt()).and_then(|p| dash_px(&p));
    let path = |points: &[Pos2]| match &dashes {
        Some((dash, gap)) => ui.painter().extend(egui::Shape::dashed_line_with_offset(points, stroke, dash, gap, 0.0)),
        None => {
            ui.painter().add(egui::Shape::line(points.to_vec(), stroke));
        }
    };
    // The line from 2 to 32 of the 34 (the mockup's `M2 5h30`).
    let line = r.shrink2(vec2(2.0, 0.0));
    match &legend.sample {
        // The datum's △ drawn as the mockup's: 10 × 9 in one point of ink.
        LegendSample::Mark(mark) if mark == "△" => {
            let b = Rect::from_center_size(r.center(), vec2(12.0, 11.0));
            let p = |x: f32, y: f32| b.min + vec2(x, y);
            ui.painter().add(egui::Shape::closed_line(vec![p(6.0, 1.0), p(11.0, 10.0), p(1.0, 10.0)], Stroke::new(1.0, t.text)));
        }
        LegendSample::Mark(mark) => {
            // 12 points, shrunk to fit down to 7.
            let w = width_of(ui, mark, font(12.0));
            let size = if w > 30.0 { (12.0 * 30.0 / w).max(7.0) } else { 12.0 };
            ui.painter().with_clip_rect(r.expand2(vec2(3.0, 2.0))).text(r.center(), Align2::CENTER_CENTER, mark, font(size), t.text);
        }
        // An area row's outline is a line as well: the mockup draws its rows as lines.
        LegendSample::Line | LegendSample::Area => path(&[pos2(line.left(), line.center().y), pos2(line.right(), line.center().y)]),
    }
}
