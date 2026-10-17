//! The layer strip (design 3.1 §4 ⑪, §13): the drawing's layers, top first, with their sublayers
//! and named objects (the elements placed from the element box, …) under them, each 14 further in.
//! A click on a layer draws on it with its legend row's look (as a click on the row in the legend:
//! the selection doesn't move), a click on an object selects it (Shift: adds it). The eye and the
//! padlock are the legend row's own switches, so the two show the same; Alt+eye shows a layer
//! alone. Dragging a row restacks it, onto a layer into it; a double click renames it. The layers
//! with nothing drawn on them wait in one line at the foot. ＋ makes a sublayer in the current
//! layer and « folds the strip to its rail ([`super::layout`]).

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::json;
use vectorcraft_doc::{Document, Node, NodeId, NodeKind};

use super::*;

pub(super) const HEAD_H: f32 = 36.0;
const ROW_H: f32 = 24.0;
const FOOT_H: f32 = 30.0;
const LIST_PAD: f32 = 4.0;
/// The spacer a row has per level in (the mockup's 10-point `<i>`; with the gap after it, 14).
const INDENT: f32 = 10.0;
/// The mockup's `.lr`: 6 from the edges, the eye 14, then 4 between the parts.
const PAD: f32 = 6.0;
const EYE: f32 = 14.0;
const GAP: f32 = 4.0;
const ARROW_W: f32 = 10.0;
const MARK_W: f32 = 12.0;
/// The least it shrinks to: its head, two rows and the foot.
pub(super) const MIN_HEIGHT: f32 = HEAD_H + LIST_PAD + 2.0 * ROW_H + LIST_PAD + FOOT_H;

/// One row of the strip.
#[derive(Clone)]
struct Row {
    id: NodeId,
    depth: usize,
    name: String,
    /// A layer or sublayer (a colour bar), else an object (a small box).
    layer: bool,
    /// The top-level layer it is in (itself for one).
    top: NodeId,
    /// It holds rows, and shows them.
    opens: bool,
    open: bool,
    visible: bool,
    locked: bool,
    /// Drawn pale: hidden, in a hidden layer, or an underlay.
    dim: bool,
    colour: Option<Color32>,
    current: bool,
}

/// Which rows the user opened or closed in a drawing (`uid`), and whether the empty layers show.
/// The current layer and those it is in are open until closed.
#[derive(Clone, Default)]
struct Open {
    uid: u64,
    opened: HashSet<NodeId>,
    closed: HashSet<NodeId>,
    empties: bool,
    /// Counts the changes, so the rows worked out from them are kept until the next.
    generation: u64,
}

impl Open {
    fn is_open(&self, id: NodeId, path: &HashSet<NodeId>) -> bool {
        self.opened.contains(&id) || (path.contains(&id) && !self.closed.contains(&id))
    }

    fn toggle(&mut self, id: NodeId, path: &HashSet<NodeId>) {
        if self.is_open(id, path) {
            self.opened.remove(&id);
            self.closed.insert(id);
        } else {
            self.closed.remove(&id);
            self.opened.insert(id);
        }
        self.generation += 1;
    }
}

fn open_id() -> egui::Id {
    wid("strip-open", ())
}

/// The strip's open rows for drawing `uid` (none opened by hand in another).
fn open_of(ctx: &egui::Context, uid: u64) -> Open {
    ctx.data(|d| d.get_temp::<Open>(open_id())).filter(|o| o.uid == uid).unwrap_or(Open { uid, ..Open::default() })
}

/// The rows worked out last, with what they came from.
type Rows = Arc<(Vec<Row>, Vec<Row>)>;

/// What the rows were worked out from: the drawing, the document itself (a new one on every edit,
/// kept here so its address can't come back for another), the open rows and the current layer.
/// Not the revision, which moves with the selection and the view too.
type RowsKey = (u64, Arc<Document>, u64, Option<NodeId>);

fn rows_id() -> egui::Id {
    wid("strip-rows", ())
}

/// The strip's rows of `st`, top first, then the empty layers waiting at its foot: from the last
/// pass while the drawing and the open rows are the same.
fn rows(ctx: &egui::Context, st: &vectorcraft_engine::DocState, open: &Open) -> Rows {
    let current = st.current_layer();
    if let Some(((uid, doc, generation, was), rows)) = ctx.data(|d| d.get_temp::<(RowsKey, Rows)>(rows_id()))
        && uid == st.uid
        && Arc::ptr_eq(&doc, &st.doc)
        && generation == open.generation
        && was == current
    {
        return rows;
    }
    let rows = Arc::new(build(st, open));
    ctx.data_mut(|d| d.insert_temp(rows_id(), ((st.uid, st.doc.clone(), open.generation, current), rows.clone())));
    rows
}

/// Whether `n` has a row of its own under what holds it: a sublayer, or an object with a name.
fn has_row(n: &Node) -> bool {
    n.is_layer() || n.name.is_some()
}

/// The layers of `st` that hold something (and the current one) with what shows inside the open
/// ones, top first; then the others, empty. One walk over the rows: each takes the colour and the
/// visibility of what holds it from the walk, not from a search of the drawing.
fn build(st: &vectorcraft_engine::DocState, open: &Open) -> (Vec<Row>, Vec<Row>) {
    let doc = &st.doc;
    let current = st.current_layer();
    let path: HashSet<NodeId> = current.and_then(|c| doc.ancestry(c)).unwrap_or_default().into_iter().collect();
    let cx = Walk { open, path: &path, current };
    let (mut rows, mut empties) = (vec![], vec![]);
    for l in doc.layers.iter().rev() {
        let empty = l.children().is_none_or(|c| c.is_empty());
        let into = if empty && Some(l.id) != current { &mut empties } else { &mut rows };
        push(l, 0, l.id, (true, None), &cx, into);
    }
    (rows, empties)
}

/// What every row of one walk needs.
struct Walk<'a> {
    open: &'a Open,
    path: &'a HashSet<NodeId>,
    current: Option<NodeId>,
}

/// The colour bar of a layer.
fn colour_of_layer(n: &Node) -> Option<Color32> {
    match &n.kind {
        NodeKind::Layer { color, .. } => Some(crate::panels::layers::rgb_colour(color.rgb())),
        _ => None,
    }
}

/// Add the row of `n` (`depth` in, in layer `top`) and those of what shows inside it when it is
/// open. `held`: whether what holds it is visible, and the colour of the innermost layer it is in.
fn push(n: &Arc<Node>, depth: usize, top: NodeId, held: (bool, Option<Color32>), cx: &Walk, out: &mut Vec<Row>) {
    let (shown, inherited) = held;
    let kids: Vec<&Arc<Node>> = n.children().map(|c| c.iter().rev().filter(|c| has_row(c)).collect()).unwrap_or_default();
    let is_open = !kids.is_empty() && cx.open.is_open(n.id, cx.path);
    let colour = if n.is_layer() { colour_of_layer(n) } else { inherited };
    out.push(Row {
        id: n.id,
        depth,
        name: n.display_name(),
        layer: n.is_layer(),
        top,
        opens: !kids.is_empty(),
        open: is_open,
        visible: n.visible,
        locked: n.locked,
        dim: !n.visible || !shown || n.is_template(),
        colour,
        current: n.is_layer() && cx.current == Some(n.id),
    });
    if is_open {
        for k in kids {
            push(k, depth + 1, top, (shown && n.visible, colour), cx, out);
        }
    }
}

/// What the user did in the strip.
enum Act {
    Pick(NodeId, NodeId, bool),
    Select(NodeId, bool),
    Toggle(NodeId),
    Show(NodeId, bool),
    ShowAlone(NodeId),
    Lock(NodeId, bool),
    Rename(NodeId, String),
    Move(NodeId, NodeId, &'static str),
    Empties,
    New,
    Fold,
}

/// A row being renamed, and what has been typed.
fn rename_id() -> egui::Id {
    wid("strip-rename", ())
}

/// The row being dragged.
fn drag_id() -> egui::Id {
    wid("strip-drag", ())
}

/// Forget the row being dragged: the strip isn't there to drop it on (folded, hidden or gone).
pub(super) fn forget_drag(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<NodeId>(drag_id()));
}

/// The layer strip in `r`.
pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
    line_below(ui, head, t.border);
    text(ui, pos2(r.left() + 10.0, head.center().y), Align2::LEFT_CENTER, tw!("Layers"), bold(13.5), t.text);
    // ＋ and «, 22 square, 2 apart and 6 in from the right (the mockup's `.a-layers .ph .r`).
    let fold_r = Rect::from_center_size(pos2(head.right() - PAD - 11.0, head.center().y), vec2(22.0, 22.0));
    let plus_r = fold_r.translate(vec2(-24.0, 0.0));
    let mut act = None;
    if icon_button(ui, wid("strip-new", ()), plus_r, tw!("New sublayer in the current layer"), &t, |ui, r| {
        crate::icons::paint(ui, "plus", r.shrink(4.0), t.text)
    })
    .clicked()
    {
        act = Some(Act::New);
    }
    if icon_button(ui, wid("strip-fold", ()), fold_r, tw!("Fold the layer strip"), &t, |ui, r| {
        crate::icons::paint(ui, "chevrons-left", r.shrink(4.0), t.text_dim)
    })
    .clicked()
    {
        act = Some(Act::Fold);
    }
    let Some(st) = app.session.active() else {
        run(app, &ctx, act);
        return;
    };
    let open = open_of(&ctx, st.uid);
    let rows = rows(&ctx, st, &open);
    let (shown, empties) = (&rows.0, &rows.1);
    let foot = (!empties.is_empty()).then(|| Rect::from_min_max(pos2(r.left(), r.bottom() - FOOT_H), r.max));
    let list_r = Rect::from_min_max(pos2(r.left(), head.bottom() + LIST_PAD), pos2(r.right(), foot.map_or(r.bottom(), |f| f.top()) - LIST_PAD));
    let all: Vec<&Row> = shown.iter().chain(empties.iter().filter(|_| open.empties)).collect();
    if let Some(a) = list(ui, list_r, &all, &t) {
        act = Some(a);
    }
    if let Some(f) = foot {
        line_above(ui, f, t.border);
        let resp = ui.interact(f, wid("strip-empties", ()), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(f.shrink(1.0), 0.0, t.hover);
        }
        let label = count(empties.len(), tw!("{n} empty layer"), tw!("{n} empty layers"));
        let said = text(ui, pos2(f.left() + 10.0, f.center().y), Align2::LEFT_CENTER, &label, font(11.5), t.text_dim);
        triangle(ui, pos2(said.right() + 8.0, f.center().y), 7.0, open.empties, t.text_dim);
        let tip = if open.empties { tw!("Fold the layers with nothing drawn on them") } else { tw!("Show the layers with nothing drawn on them") };
        if resp.on_hover_text(tip).clicked() {
            act = Some(Act::Empties);
        }
    }
    run(app, &ctx, act);
}

/// The rows `all` in `area`, those in view laid out; returns what was done to one.
fn list(ui: &mut Ui, area: Rect, all: &[&Row], t: &Tokens) -> Option<Act> {
    let mut act = None;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    child.spacing_mut().scroll.fade.strength = 0.0;
    let scroll = egui::ScrollArea::vertical().id_salt(wid("strip-scroll", ())).auto_shrink([false, false]).min_scrolled_height(0.0);
    let mut dragging: Option<NodeId> = ui.data(|d| d.get_temp(drag_id()));
    // Esc gives the move up, as it does a panel's.
    if dragging.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        forget_drag(ui.ctx());
        dragging = None;
    }
    let mut target = None;
    scroll.show_viewport(&mut child, |ui, view| {
        ui.set_height(all.len() as f32 * ROW_H);
        let origin = ui.max_rect().min;
        let w = ui.available_width();
        let first = (view.top() / ROW_H).floor().max(0.0) as usize;
        for (i, row) in all.iter().enumerate().skip(first) {
            let top = i as f32 * ROW_H;
            if top >= view.bottom() {
                break;
            }
            let rr = Rect::from_min_size(origin + vec2(0.0, top), vec2(w, ROW_H));
            if let Some(a) = row_ui(ui, row, rr, t) {
                act = Some(a);
            }
            // Where a dragged row would go: over the top quarter above it, the bottom quarter
            // below it, between them into it when it is a layer.
            if let (Some(id), Some(p)) = (dragging, ui.ctx().pointer_latest_pos())
                && rr.contains(p)
                && id != row.id
            {
                let y = (p.y - rr.top()) / ROW_H;
                let place = if row.layer && (0.25..0.75).contains(&y) {
                    "inside"
                } else if y < 0.5 {
                    "above"
                } else {
                    "below"
                };
                target = Some((row.id, place, rr));
            }
        }
    });
    if let Some((_, place, rr)) = target {
        let p = ui.painter();
        match place {
            "inside" => {
                p.rect_stroke(rr.shrink(1.0), CornerRadius::same(3), Stroke::new(1.5, t.clay), StrokeKind::Inside);
            }
            "above" => {
                p.hline(rr.x_range(), rr.top(), Stroke::new(2.0, t.clay));
            }
            _ => {
                p.hline(rr.x_range(), rr.bottom(), Stroke::new(2.0, t.clay));
            }
        }
    }
    // A drag that ended drops the row where the pointer was.
    if let Some(id) = dragging
        && !ui.ctx().input(|i| i.pointer.any_down())
    {
        ui.data_mut(|d| d.remove::<NodeId>(drag_id()));
        if let Some((onto, place, _)) = target {
            act = Some(Act::Move(id, onto, place));
        }
    }
    act
}

/// One row in `rr` (the mockup's `.lr`): eye, ▸/▾, colour bar or object box, name, then the pen
/// mark on the current layer and the padlock when locked (or under the pointer).
fn row_ui(ui: &mut Ui, row: &Row, rr: Rect, t: &Tokens) -> Option<Act> {
    let cy = rr.center().y;
    let x_eye = rr.left() + PAD;
    // Each level in: the mockup's 10-point spacer and the gap after it.
    let x_arrow = x_eye + EYE + GAP + (INDENT + GAP) * row.depth as f32;
    let x_mark = x_arrow + ARROW_W + GAP;
    let lock_r = Rect::from_center_size(pos2(rr.right() - PAD - MARK_W / 2.0, cy), vec2(MARK_W + 4.0, ROW_H));
    let resp = ui.interact(rr, wid("strip-row", row.id.0), Sense::click_and_drag());
    let eye = ui.interact(Rect::from_center_size(pos2(x_eye + EYE / 2.0, cy), vec2(EYE + 4.0, ROW_H)), wid("strip-eye", row.id.0), Sense::click());
    let arrow = row.opens.then(|| {
        ui.interact(
            Rect::from_center_size(pos2(x_arrow + ARROW_W / 2.0, cy), vec2(ARROW_W + 4.0, ROW_H)),
            wid("strip-arrow", row.id.0),
            Sense::click(),
        )
    });
    let lock = ui.interact(lock_r, wid("strip-lock", row.id.0), Sense::click());
    let over = resp.hovered() || eye.hovered() || lock.hovered() || arrow.as_ref().is_some_and(egui::Response::hovered);
    if row.current {
        ui.painter().rect_filled(rr, 0.0, t.row_selected);
    } else if over {
        ui.painter().rect_filled(rr, 0.0, t.hover);
    }
    let icon = |name: &str, c: Pos2, size: f32, color: Color32| crate::icons::paint(ui, name, Rect::from_center_size(c, vec2(size, size)), color);
    icon(if row.visible { "eye" } else { "eye-off" }, pos2(x_eye + EYE / 2.0, cy), EYE, if row.visible { t.icon } else { t.ring });
    if row.opens {
        triangle(ui, pos2(x_arrow + ARROW_W / 2.0, cy), 7.0, row.open, t.text_dim);
    }
    let name_x = if row.layer {
        ui.painter().rect_filled(
            Rect::from_min_size(pos2(x_mark, cy - 7.0), vec2(3.0, 14.0)),
            CornerRadius::same(1),
            row.colour.unwrap_or(t.text_disabled),
        );
        x_mark + 3.0 + GAP + 1.0
    } else {
        let b = Rect::from_min_size(pos2(x_mark, cy - 3.5), vec2(7.0, 7.0));
        ui.painter().rect_stroke(b, CornerRadius::same(1), Stroke::new(1.0, t.text_disabled), StrokeKind::Inside);
        x_mark + 7.0 + GAP + 1.0
    };
    // The name takes the row up to the padlock (when locked, or under the pointer to lock it)
    // and the pen nib (on the current layer), as the mockup's flexible `.nm`.
    let mut name_end = rr.right() - PAD;
    let lock_icon = Rect::from_center_size(lock_r.center(), vec2(MARK_W, MARK_W));
    if row.locked {
        super::legend::padlock(ui, lock_icon, t.text_dim);
    } else if over {
        icon("lock-open", lock_icon.center(), MARK_W, t.ring);
    }
    if row.locked || over {
        name_end = lock_icon.left() - GAP;
    }
    if row.current {
        let nib = Rect::from_min_max(pos2(name_end - MARK_W, cy - MARK_W / 2.0), pos2(name_end, cy + MARK_W / 2.0));
        icon("tw-pen", nib.center(), MARK_W, t.clay);
        name_end = nib.left() - GAP;
    }
    let renaming: Option<(NodeId, String)> = ui.data(|d| d.get_temp(rename_id()));
    if let Some((_, typed)) = renaming.filter(|(id, _)| *id == row.id) {
        return rename_field(ui, row, Rect::from_min_max(pos2(name_x - 2.0, rr.top() + 2.0), pos2(name_end, rr.bottom() - 2.0)), typed);
    }
    let (f, color) = match (row.current, row.dim) {
        (true, _) => (bold(12.0), t.text),
        (false, true) => (font(12.0), t.text_disabled),
        (false, false) => (font(12.0), t.text),
    };
    text_to(ui, pos2(name_x, cy), name_end, &row.name, f.clone(), color);
    let eye = eye.on_hover_text(if row.visible {
        tw!("Hide · Alt+click: show this layer alone")
    } else {
        tw!("Show · Alt+click: show this layer alone")
    });
    let lock = lock.on_hover_text(if row.locked { tw!("Unlock") } else { tw!("Lock") });
    let tip =
        if row.layer { tw!("Draw on this layer · double-click to rename") } else { tw!("Select · Shift+click: add · double-click to rename") };
    let cut = width_of(ui, &row.name, f) > name_end - name_x;
    let resp = resp.on_hover_text(if cut { format!("{}\n{tip}", row.name) } else { tip.to_string() });
    if resp.drag_started() {
        ui.data_mut(|d| d.insert_temp(drag_id(), row.id));
    }
    if resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }
    let shift = ui.input(|i| i.modifiers.shift);
    if eye.clicked() {
        if ui.input(|i| i.modifiers.alt) && row.depth == 0 { Some(Act::ShowAlone(row.id)) } else { Some(Act::Show(row.id, !row.visible)) }
    } else if lock.clicked() {
        Some(Act::Lock(row.id, !row.locked))
    } else if arrow.is_some_and(|a| a.clicked()) {
        Some(Act::Toggle(row.id))
    } else if resp.double_clicked() {
        ui.data_mut(|d| d.insert_temp(rename_id(), (row.id, row.name.clone())));
        None
    } else if resp.clicked() {
        if row.layer { Some(Act::Pick(row.id, row.top, row.depth == 0)) } else { Some(Act::Select(row.id, shift)) }
    } else {
        None
    }
}

/// The field a row is renamed in: Enter (or a press elsewhere) renames it, Esc leaves it as it was.
fn rename_field(ui: &mut Ui, row: &Row, r: Rect, mut typed: String) -> Option<Act> {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let id = wid("strip-rename-field", row.id.0);
    let resp = child.add(egui::TextEdit::singleline(&mut typed).id(id).font(font(12.0)).desired_width(r.width()).margin(vec2(2.0, 1.0)));
    if !resp.has_focus() && !resp.lost_focus() {
        resp.request_focus();
    }
    let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if resp.lost_focus() || esc {
        ui.data_mut(|d| d.remove::<(NodeId, String)>(rename_id()));
        let name = typed.trim().to_string();
        return (!esc && !name.is_empty() && name != row.name).then_some(Act::Rename(row.id, name));
    }
    ui.data_mut(|d| d.insert_temp(rename_id(), (row.id, typed)));
    None
}

/// Do what was done in the strip.
fn run(app: &mut VectorcraftApp, ctx: &egui::Context, act: Option<Act>) {
    let Some(act) = act else { return };
    let uid = app.session.active().map(|st| st.uid);
    let path = || -> HashSet<NodeId> {
        app.session.active().and_then(|st| st.current_layer().and_then(|c| st.doc.ancestry(c))).unwrap_or_default().into_iter().collect()
    };
    let result = match act {
        // As the legend's row: the current layer, the pen with its row's look; a sublayer takes
        // its layer's look and is drawn on itself. A hidden or locked layer (the legend won't
        // draw on it) just becomes the current one, as in the Layers panel.
        Act::Pick(id, top, top_level) => {
            let editable = app.session.doc().is_ok_and(|st| st.doc.is_editable(id));
            let picked = if editable { app.run("legend.pick", json!({ "id": top.0 })).map(drop) } else { Ok(()) };
            if top_level && editable { picked } else { picked.and_then(|()| app.run("layer.setCurrent", json!({ "id": id.0 })).map(drop)) }
        }
        Act::Select(id, add) => app.run("layer.selectAll", json!({ "id": id.0, "add": add })).map(drop),
        Act::Toggle(id) => {
            if let Some(uid) = uid {
                let p = path();
                let mut open = open_of(ctx, uid);
                open.toggle(id, &p);
                ctx.data_mut(|d| d.insert_temp(open_id(), open));
            }
            Ok(())
        }
        Act::Show(id, on) => app.run("layer.setProps", json!({ "ids": [id.0], "visible": on })).map(drop),
        Act::ShowAlone(id) => super::legend::show_alone(app, id),
        Act::Lock(id, on) => app.run("layer.setProps", json!({ "ids": [id.0], "locked": on })).map(drop),
        Act::Rename(id, name) => app.run("layer.setProps", json!({ "ids": [id.0], "name": name })).map(drop),
        Act::Move(id, onto, place) => app.run("layer.move", json!({ "ids": [id.0], "target": onto.0, "place": place })).map(drop),
        Act::Empties => {
            if let Some(uid) = uid {
                let mut open = open_of(ctx, uid);
                open.empties = !open.empties;
                open.generation += 1;
                ctx.data_mut(|d| d.insert_temp(open_id(), open));
            }
            Ok(())
        }
        Act::New => new_sublayer(app, ctx),
        Act::Fold => {
            if let Some((column, _)) = app.ui.trowel_layout.place_of(crate::trowel_layout::Panel::Layers) {
                app.ui.trowel_layout.set_folded(column, Some(true));
            }
            Ok(())
        }
    };
    if let Err(e) = result {
        app.status_error(e);
    }
}

/// A sublayer on top of the current layer's contents, drawn on next, its layer open to show it.
fn new_sublayer(app: &mut VectorcraftApp, ctx: &egui::Context) -> Result<(), String> {
    let Some(st) = app.session.active() else { return Ok(()) };
    let Some(parent) = st.current_layer() else { return Err(tw!("No layer to draw on").to_string()) };
    let uid = st.uid;
    let done = app.run("layer.newSublayer", json!({ "parent": parent.0 }))?;
    if let Some(id) = done["id"].as_u64() {
        app.run("layer.setCurrent", json!({ "id": id }))?;
    }
    let mut open = open_of(ctx, uid);
    open.closed.remove(&parent);
    open.opened.insert(parent);
    open.generation += 1;
    ctx.data_mut(|d| d.insert_temp(open_id(), open));
    Ok(())
}

/// The colour of the layer drawn on, for the folded strip's rail.
pub(super) fn current_colour(app: &VectorcraftApp) -> Option<Color32> {
    let st = app.session.active()?;
    let c = st.current_layer()?;
    Some(crate::panels::layers::layer_colour(&st.doc, c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_engine::Session;

    /// The rows are worked out again when the drawing or the current layer changes, not when only
    /// the selection or the view does, so a drawing of thousands of named objects is no slower to
    /// select in; and an object's row takes the colour of the layer it is in from the walk.
    #[test]
    fn the_rows_are_kept_until_the_drawing_changes() {
        let mut s = Session::new();
        s.execute("file.new", &json!({ "width": 400, "height": 300 })).unwrap();
        for i in 0..3 {
            let r = s.execute("shape.rectangle", &json!({ "x": 10 * i, "y": 10, "width": 5, "height": 5 })).unwrap();
            s.execute("layer.setProps", &json!({ "ids": [r["id"]], "name": format!("돌 {i}") })).unwrap();
        }
        let ctx = egui::Context::default();
        let open = |s: &Session| open_of(&ctx, s.doc().unwrap().uid);
        let first = rows(&ctx, s.doc().unwrap(), &open(&s));
        let layer = first.0.first().cloned().unwrap();
        assert!(layer.layer && layer.current && layer.open);
        assert_eq!(first.0.len(), 4, "the layer and its three named objects");
        assert!(first.0.iter().skip(1).all(|r| !r.layer && r.depth == 1 && r.colour == layer.colour && r.colour.is_some()));
        // The selection moves the revision, not the rows.
        let revision = s.doc().unwrap().revision;
        s.execute("select.set", &json!({ "ids": [first.0.get(1).unwrap().id.0] })).unwrap();
        assert!(s.doc().unwrap().revision > revision);
        let again = rows(&ctx, s.doc().unwrap(), &open(&s));
        assert!(Arc::ptr_eq(&first, &again), "kept");
        // A rename is a new drawing: new rows.
        s.execute("layer.setProps", &json!({ "ids": [layer.id.0], "name": "평면" })).unwrap();
        let renamed = rows(&ctx, s.doc().unwrap(), &open(&s));
        assert!(!Arc::ptr_eq(&first, &renamed));
        assert_eq!(renamed.0.first().map(|r| r.name.as_str()), Some("평면"));
    }
}
