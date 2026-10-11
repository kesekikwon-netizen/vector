//! The legend and layers, the biggest part of the right column (design 2.1 §3): one row per layer
//! of the drawing, the top one first. A row's eye and padlock show, hide, lock and unlock the
//! layer, a click makes it the layer drawn on, and its arrow lists the objects on it (a click
//! selects one). The legend's entries on the rows (weight, line type, favourites) and Weights at
//! Once come with J005.

use serde_json::json;
use vectorcraft_doc::NodeId;

use super::*;

const ROW_H: f32 = 24.0;
const OBJECT_H: f32 = 20.0;
/// The panel's padding at the sides.
const PAD: f32 = 14.0;

/// A layer's row as shown.
struct Row {
    id: NodeId,
    name: String,
    visible: bool,
    locked: bool,
    colour: Color32,
    count: usize,
    current: bool,
    open: bool,
    /// Its objects, top first, with whether each is selected (an open row's only).
    objects: Vec<(NodeId, String, bool)>,
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
    let rows = rows(app, ui.ctx());
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    let sub = if app.session.active().is_some() { crate::i18n::fmt(tw!("Layers: {n}"), &[("n", &rows.len().to_string())]) } else { String::new() };
    panel_head(ui, head, tw!("Legend · Layers"), &sub, 2, &t);
    let new_layer = app.session.active().is_some() && head_button(ui, wid("legend-new", ()), head, 1, "plus", tw!("New layer"), &t).clicked();
    if head_button(ui, wid("legend-more", ()), head, 0, "ellipsis", tw!("Layers panel"), &t).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "layers" }));
    }
    let cap = head.bottom() + 10.0 + 7.0;
    list_caption(ui, r.left() + PAD, r.right() - PAD, cap, tw!("This drawing's layers"), &t);
    let foot = footer(ui, r, &t);
    let area = Rect::from_min_max(pos2(r.left() + PAD, cap + 7.0 + 4.0), pos2(r.right() - PAD, foot.top() - 6.0));
    let mut act = None;
    let mut list = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    egui::ScrollArea::vertical().id_salt(wid("legend-scroll", ())).auto_shrink([false, false]).show(&mut list, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in &rows {
            if let Some(a) = layer_row(ui, row, &t) {
                act = Some(a);
            }
            for (id, name, selected) in &row.objects {
                let (_, rr) = ui.allocate_space(vec2(ui.available_width(), OBJECT_H));
                let resp = ui.interact(rr, wid("legend-obj", id.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(rr, CornerRadius::same(4), t.hover);
                }
                let f = if *selected { bold(11.0) } else { font(11.0) };
                ui.painter().with_clip_rect(rr).text(pos2(rr.left() + 36.0, rr.center().y), Align2::LEFT_CENTER, name, f, t.text);
                if resp.clicked() {
                    act = Some(Act::Select(*id));
                }
            }
        }
    });
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

/// The layers of the active drawing, top first.
fn rows(app: &VectorcraftApp, ctx: &egui::Context) -> Vec<Row> {
    let Some(st) = app.session.active() else { return vec![] };
    let current = st.current_layer();
    let lang = crate::i18n::current();
    st.doc
        .layers
        .iter()
        .rev()
        .map(|l| {
            let open = ctx.data(|d| d.get_temp::<bool>(opened(l.id))).unwrap_or(false);
            let children = l.children().map_or(&[][..], |c| c.as_slice());
            let objects = if open {
                children
                    .iter()
                    .rev()
                    .map(|c| (c.id, crate::panels::layers::painted_name(c, &c.display_name(), lang), st.selection.objects.contains(&c.id)))
                    .collect()
            } else {
                vec![]
            };
            Row {
                id: l.id,
                name: l.display_name(),
                visible: l.visible,
                locked: l.locked,
                colour: crate::panels::layers::layer_colour(&st.doc, l.id),
                count: children.len(),
                current: Some(l.id) == current,
                open,
                objects,
            }
        })
        .collect()
}

/// Where whether a layer's row is open is kept.
fn opened(id: NodeId) -> egui::Id {
    wid("legend-opened", id.0)
}

/// One layer's row: its arrow, eye, padlock, colour, name and number of objects.
fn layer_row(ui: &mut Ui, row: &Row, t: &Tokens) -> Option<Act> {
    let (_, rr) = ui.allocate_space(vec2(ui.available_width(), ROW_H));
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
    let icon =
        |name: &str, cx: f32, size: f32, color: Color32| crate::icons::paint(ui, name, Rect::from_center_size(pos2(cx, cy), vec2(size, size)), color);
    icon(if row.open { "chevron-down" } else { "chevron-right" }, x0 + 4.5, 10.0, t.text_dim);
    icon(if row.visible { "eye" } else { "eye-off" }, x0 + 23.5, 15.0, if row.visible { t.text } else { t.text_disabled });
    icon(if row.locked { "lock" } else { "lock-open" }, x0 + 45.5, 15.0, if row.locked { t.text } else { t.text_disabled });
    let patch = Rect::from_min_size(pos2(x0 + 60.0, cy - 6.0), vec2(20.0, 12.0));
    ui.painter().rect(patch, 0.0, row.colour.gamma_multiply(0.25), Stroke::new(1.5, row.colour), StrokeKind::Inside);
    // The name after the favourite's star (J005), the number of objects at the right.
    let count = text(ui, pos2(rr.right() - 4.0, cy), Align2::RIGHT_CENTER, row.count, mono(10.0), t.text_dim);
    let name = Rect::from_min_max(pos2(x0 + 104.0, rr.top()), pos2(count.left() - 7.0, rr.bottom()));
    let color = if row.visible { t.text } else { t.text_dim };
    ui.painter().with_clip_rect(name).text(
        pos2(name.left(), cy),
        Align2::LEFT_CENTER,
        &row.name,
        if row.current { bold(12.0) } else { font(12.0) },
        color,
    );
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
    } else if resp.on_hover_text(tw!("Draw on this layer")).clicked() {
        Some(Act::Current(row.id))
    } else {
        None
    }
}

/// Weights at Once under the list (J005). Returns its rect.
fn footer(ui: &Ui, r: Rect, t: &Tokens) -> Rect {
    let foot = Rect::from_min_max(pos2(r.left() + PAD, r.bottom() - 10.0 - 26.0 - 6.0), pos2(r.right() - PAD, r.bottom()));
    line_above(ui, foot, t.border);
    let cy = foot.top() + 6.0 + 13.0;
    let x = text(ui, pos2(foot.left(), cy), Align2::LEFT_CENTER, tw!("Weights at once"), font(11.0), t.text_dim).right() + 6.0;
    let x = button_soon(ui, wid("legend-weights", ()), pos2(x, cy), tw!("Change All Weights…"), false, "J005", t) + 4.0;
    button_soon(ui, wid("legend-overrides", ()), pos2(x, cy), tw!("Overrides to Legend"), false, "J005", t);
    foot
}
