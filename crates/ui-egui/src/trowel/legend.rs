//! The legend and layers, the biggest part of the right column (design 2.1 §3): one row per layer
//! of the drawing, the top one first. A row's eye and padlock show, hide, lock and unlock the
//! layer, a click makes it the layer drawn on, and its arrow lists the objects on it (a click
//! selects one). The legend's entries on the rows (weight, line type, favourites) and Weights at
//! Once come with J005.

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::json;
use vectorcraft_doc::{Node, NodeId};

use super::*;

const ROW_H: f32 = 24.0;
const OBJECT_H: f32 = 20.0;
/// The panel's padding at the sides.
const PAD: f32 = 14.0;
/// Its head and caption over the list, and Weights at Once under it ([`footer`]).
const ABOVE: f32 = 32.0 + 10.0 + 7.0 + 7.0 + 4.0;
const BELOW: f32 = 6.0 + FOOT_H;
const FOOT_H: f32 = 6.0 + 26.0 + 10.0;
/// How tall the legend has to be to list two layers.
pub(super) const MIN_HEIGHT: f32 = ABOVE + 2.0 * ROW_H + BELOW;

/// A layer's row as shown.
struct Row<'a> {
    id: NodeId,
    name: String,
    visible: bool,
    locked: bool,
    /// Hidden, or an underlay (a template layer): its name shows faint.
    dim: bool,
    colour: Color32,
    current: bool,
    open: bool,
    /// The current layer's when it is locked or hidden: where lines go instead ([`draw_target`]).
    note: Option<String>,
    /// Its objects, bottom first; an open row lists them under it, top first.
    objects: &'a [Arc<Node>],
}

impl Row<'_> {
    /// How tall the row is with the objects it lists.
    fn height(&self) -> f32 {
        ROW_H + if self.open { self.objects.len() as f32 * OBJECT_H } else { 0.0 }
    }
}

/// What a click on the list asks for.
enum Act {
    Current(NodeId),
    Show(NodeId, bool),
    Lock(NodeId, bool),
    Select(NodeId),
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);
    let layers = app.session.active().map(|st| st.doc.layers.len());
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    let sub = layers.map(|n| crate::i18n::fmt(tw!("Layers: {n}"), &[("n", &n.to_string())])).unwrap_or_default();
    panel_head(ui, head, tw!("Legend · Layers"), &sub, 2, &t);
    let new_layer = layers.is_some() && head_button(ui, wid("legend-new", ()), head, 1, "plus", tw!("New layer"), &t).clicked();
    let more = more_button(ui, wid("legend-more", ()), head, 0, tw!("Layers panel"), &t).clicked();
    let cap = head.bottom() + 10.0 + 7.0;
    list_caption(ui, r.left() + PAD, r.right() - PAD, cap, tw!("This drawing's layers"), &t);
    footer(ui, r, &t);
    let area = Rect::from_min_max(pos2(r.left() + PAD, r.top() + ABOVE), pos2(r.right() - PAD, r.bottom() - BELOW));
    let act = list(app, ui, area, &t);
    if more {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "layers" }));
    }
    let result = match act {
        Some(Act::Current(id)) => app.run("layer.setCurrent", json!({ "id": id.0 })),
        Some(Act::Show(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "visible": on })),
        Some(Act::Lock(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "locked": on })),
        Some(Act::Select(id)) => app.run("select.set", json!({ "ids": [id.0] })),
        None if new_layer => app.run("layer.new", json!({})),
        None => Ok(serde_json::Value::Null),
    };
    if let Err(e) = result {
        app.status(e);
    }
}

/// The layers, top first, each followed by its objects when open. Only the rows in view are laid
/// out: one layer can hold thousands of objects.
fn list(app: &VectorcraftApp, ui: &mut Ui, area: Rect, t: &Tokens) -> Option<Act> {
    let rows = rows(app, ui.ctx());
    // Where each row starts, from the top of the list.
    let tops: Vec<f32> = rows
        .iter()
        .scan(0.0, |y, row| {
            let top = *y;
            *y += row.height();
            Some(top)
        })
        .collect();
    let total: f32 = rows.iter().map(Row::height).sum();
    let selected: HashSet<NodeId> = match app.session.active() {
        Some(st) if rows.iter().any(|row| row.open) => st.selection.objects.iter().copied().collect(),
        _ => HashSet::new(),
    };
    let lang = crate::i18n::current();
    let mut act = None;
    let mut list = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    // Exactly `area`, however short: egui would make it at least 64 tall, over the footer.
    let scroll = egui::ScrollArea::vertical().id_salt(wid("legend-scroll", ())).auto_shrink([false, false]).min_scrolled_height(0.0);
    scroll.show_viewport(&mut list, |ui, view| {
        ui.set_height(total);
        let origin = ui.max_rect().min;
        let w = ui.available_width();
        // The first row in view is the last one starting at or above the view's top.
        let first = tops.partition_point(|top| *top <= view.top()).saturating_sub(1);
        for (row, top) in rows.iter().zip(&tops).skip(first) {
            if *top >= view.bottom() {
                break;
            }
            let rr = Rect::from_min_size(origin + vec2(0.0, *top), vec2(w, ROW_H));
            if let Some(a) = layer_row(ui, row, rr, t) {
                act = Some(a);
            }
            if !row.open {
                continue;
            }
            // Its objects in view, top first.
            let below = top + ROW_H;
            let from = ((view.top() - below) / OBJECT_H).floor().max(0.0) as usize;
            let to = ((view.bottom() - below) / OBJECT_H).ceil().max(0.0) as usize;
            for (k, node) in row.objects.iter().rev().enumerate().take(to).skip(from) {
                let rr = Rect::from_min_size(origin + vec2(0.0, below + k as f32 * OBJECT_H), vec2(w, OBJECT_H));
                if object_row(ui, node, selected.contains(&node.id), lang, rr, t) {
                    act = Some(Act::Select(node.id));
                }
            }
        }
    });
    act
}

/// The layers of the active drawing, top first.
fn rows<'a>(app: &'a VectorcraftApp, ctx: &egui::Context) -> Vec<Row<'a>> {
    let Some(st) = app.session.active() else { return vec![] };
    let current = st.current_layer();
    let note = draw_target(st).note;
    st.doc
        .layers
        .iter()
        .rev()
        .map(|l| Row {
            id: l.id,
            name: l.display_name(),
            visible: l.visible,
            locked: l.locked,
            dim: !l.visible || l.is_template(),
            colour: crate::panels::layers::layer_colour(&st.doc, l.id),
            current: Some(l.id) == current,
            open: ctx.data(|d| d.get_temp::<bool>(opened(l.id))).unwrap_or(false),
            note: note.clone().filter(|_| Some(l.id) == current),
            objects: l.children().map_or(&[][..], |c| c.as_slice()),
        })
        .collect()
}

/// Where whether a layer's row is open is kept.
fn opened(id: NodeId) -> egui::Id {
    wid("legend-opened", id.0)
}

/// One layer's row in `rr`: its arrow, eye, padlock, colour, name and number of objects.
fn layer_row(ui: &Ui, row: &Row, rr: Rect, t: &Tokens) -> Option<Act> {
    let x0 = rr.left() + 4.0;
    let column = |a: f32, b: f32| Rect::from_min_max(pos2(a, rr.top()), pos2(b, rr.bottom()));
    let resp = ui.interact(rr, wid("legend-row", row.id.0), Sense::click());
    let open = ui.interact(column(rr.left(), x0 + 12.5), wid("legend-open", row.id.0), Sense::click());
    let eye = ui.interact(column(x0 + 12.5, x0 + 34.5), wid("legend-eye", row.id.0), Sense::click());
    let lock = ui.interact(column(x0 + 34.5, x0 + 56.5), wid("legend-lock", row.id.0), Sense::click());
    if row.current {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.row_selected);
    } else if resp.hovered() || open.hovered() || eye.hovered() || lock.hovered() {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.hover);
    }
    let cy = rr.center().y;
    // As in the mockup: a small arrow; the eye dark when shown, pale when hidden; the padlock
    // black when locked, pale when not.
    triangle(ui, pos2(x0 + 4.5, cy), 7.0, row.open, t.text_dim);
    let icon = |name: &str, cx: f32, color: Color32| crate::icons::paint(ui, name, Rect::from_center_size(pos2(cx, cy), vec2(15.0, 15.0)), color);
    icon(if row.visible { "eye" } else { "eye-off" }, x0 + 23.5, if row.visible { t.icon } else { t.ring });
    icon(if row.locked { "lock" } else { "lock-open" }, x0 + 45.5, if row.locked { t.text } else { t.ring });
    let patch = Rect::from_min_size(pos2(x0 + 60.0, cy - 6.0), vec2(20.0, 12.0));
    ui.painter().rect(patch, 0.0, row.colour.gamma_multiply(0.25), Stroke::new(1.5, row.colour), StrokeKind::Inside);
    // The name after the favourite's star (J005), the number of objects at the right.
    let count = text(ui, pos2(rr.right() - 4.0, cy), Align2::RIGHT_CENTER, row.objects.len(), mono(10.0), t.text_dim);
    let name = Rect::from_min_max(pos2(x0 + 104.0, rr.top()), pos2(count.left() - 7.0, rr.bottom()));
    let color = if row.note.is_some() {
        t.clay_text
    } else if row.dim {
        t.text_disabled
    } else {
        t.text
    };
    // Regular weight, the current row's too; favourites are bold (J005).
    ui.painter().with_clip_rect(name).text(pos2(name.left(), cy), Align2::LEFT_CENTER, &row.name, font(12.0), color);
    let open = open.on_hover_text(if row.open { tw!("Hide the objects") } else { tw!("Show the objects") });
    let eye = eye.on_hover_text(if row.visible { tw!("Hide") } else { tw!("Show") });
    let lock = lock.on_hover_text(if row.locked { tw!("Unlock") } else { tw!("Lock") });
    if open.clicked() {
        ui.data_mut(|d| d.insert_temp(opened(row.id), !row.open));
        None
    } else if eye.clicked() {
        Some(Act::Show(row.id, !row.visible))
    } else if lock.clicked() {
        Some(Act::Lock(row.id, !row.locked))
    } else if resp.on_hover_text(row.note.as_deref().unwrap_or(tw!("Draw on this layer"))).clicked() {
        Some(Act::Current(row.id))
    } else {
        None
    }
}

/// One object under an open layer's row, in `rr`. True when it is clicked.
fn object_row(ui: &Ui, node: &Node, selected: bool, lang: crate::i18n::Lang, rr: Rect, t: &Tokens) -> bool {
    let resp = ui.interact(rr, wid("legend-obj", node.id.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.hover);
    }
    let name = crate::panels::layers::painted_name(node, &node.display_name(), lang);
    // The selected one black and bold; the others in the mockup's ink2, of which `icon` is the
    // nearest token.
    let (f, color) = if selected { (bold(11.0), t.text) } else { (font(11.0), t.icon) };
    ui.painter().with_clip_rect(rr).text(pos2(rr.left() + 36.0, rr.center().y), Align2::LEFT_CENTER, name, f, color);
    resp.clicked()
}

/// Weights at Once under the list (J005), [`FOOT_H`] tall.
fn footer(ui: &Ui, r: Rect, t: &Tokens) {
    let foot = Rect::from_min_max(pos2(r.left() + PAD, r.bottom() - FOOT_H), pos2(r.right() - PAD, r.bottom()));
    line_above(ui, foot, t.border);
    let cy = foot.top() + 6.0 + 13.0;
    let x = text(ui, pos2(foot.left(), cy), Align2::LEFT_CENTER, tw!("Weights at once"), font(11.0), t.text_dim).right() + 6.0;
    let x = button_soon(ui, wid("legend-weights", ()), pos2(x, cy), tw!("Change All Weights…"), false, "J005", t) + 4.0;
    button_soon(ui, wid("legend-overrides", ()), pos2(x, cy), tw!("Overrides to Legend"), false, "J005", t);
}
