//! Trowel's panel columns (design 3.1 §13): the drawer left of the canvas, the right column and
//! the layer strip at the window's right edge, each holding the panels the layout
//! ([`crate::trowel_layout::Layout`]) puts there, top first. A panel is moved by its head to
//! another place in any column (a ghost follows the pointer, a clay line shows where it goes;
//! dropped on the canvas it stays where it was). A column is as wide as its edge by the canvas is
//! dragged, and the boundary between two panels sizes them; a double click on either gives back
//! the first size. A column that holds nothing goes, and the canvas takes its room; a panel
//! dragged to that side's edge brings it back. A folded column shows its rail.

use crate::trowel_layout::{self as model, Column, Panel, Shown};

use super::*;

/// The layer strip's column.
pub(crate) const EDGE: &str = "trowel-edge";

/// The id egui keeps column `c` under.
pub(crate) fn column_id(c: Column) -> &'static str {
    match c {
        Column::Left => SET,
        Column::Right => RIGHT,
        Column::Edge => EDGE,
    }
}

/// A column as drawn this pass, for the drop: where it is, folded or not, and where its panels are.
struct Drawn {
    column: Column,
    rect: Rect,
    folded: bool,
    slots: Vec<(Panel, Rect)>,
}

/// A panel being moved by its head: which, where the pointer holds it from its top left, how
/// large it is, and whether Esc gave up the move.
#[derive(Clone, Copy)]
struct Moving {
    panel: Panel,
    grab: egui::Vec2,
    size: egui::Vec2,
    cancelled: bool,
}

fn moving_id() -> egui::Id {
    wid("panel-moving", ())
}

/// A panel is being moved (its ghost shows).
#[cfg(test)]
pub(super) fn is_moving(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Moving>(moving_id())).is_some()
}

/// Lay out the columns (after the tool column, before the canvas) and move a panel dropped by its
/// head.
pub(super) fn columns(app: &mut VectorcraftApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let screen = ctx.content_rect();
    let tools_w = if app.ui.toolbar { TOOLS_W } else { 0.0 };
    let shown = app.ui.trowel_layout.shown(screen.width(), tools_w);
    app.trowel_window = Some((screen.width(), tools_w));
    ctx.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id()));
    let mut drawn = vec![];
    let mut heads = vec![];
    // The strip's column before the right column, so it is the outer one.
    for c in [Column::Left, Column::Edge, Column::Right] {
        if let Some(s) = shown.get(model::column_index(c)).copied().flatten() {
            drawn.push(column(app, ui, c, s, &mut heads));
        }
    }
    if !drawn.iter().any(|d| !d.folded && d.slots.iter().any(|(p, _)| *p == Panel::Layers)) {
        layers::forget_drag(&ctx);
    }
    // The dock's own panels pop out left of the right-hand columns, level with their top.
    let keep = ctx.data_mut(|d| d.remove_temp::<Vec<Rect>>(keep_id())).unwrap_or_default();
    let inner = drawn.iter().filter(|d| d.column != Column::Left).map(|d| d.rect).min_by(|a, b| a.left().total_cmp(&b.left()));
    let (left, top) = inner.map_or((screen.right(), screen.top() + CAPTION_H + MENU_H + RIBBON_H), |r| (r.left(), r.top()));
    crate::dock::set_flyout(&ctx, crate::dock::Flyout { left, top: top + 6.0, keep });
    moving(app, ui, &drawn, &heads, screen.left() + tools_w);
}

/// Column `c`, shown as `s`.
fn column(app: &mut VectorcraftApp, ui: &mut Ui, c: Column, s: Shown, heads: &mut Vec<(Panel, egui::Response)>) -> Drawn {
    let t = Tokens::get(ui.ctx());
    let panel = match c {
        Column::Left => egui::Panel::left(column_id(c)),
        Column::Right | Column::Edge => egui::Panel::right(column_id(c)),
    };
    panel
        .exact_size(s.width)
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.set_min_width(r.width());
            // The line on the canvas's side (and between the right column and the strip).
            match c {
                Column::Left => line_right(ui, r, t.border),
                Column::Right | Column::Edge => line_left(ui, r, t.border),
            }
            if s.folded {
                rail(app, ui, c, r, &t);
                return Drawn { column: c, rect: r, folded: true, slots: vec![] };
            }
            let slots = stack(app, ui, c, r, heads, &t);
            width_edge(app, ui, c, r);
            Drawn { column: c, rect: r, folded: false, slots }
        })
        .inner
}

/// A panel's own height in `app` now: the pen's and the properties' as tall as what they show; the
/// others fill what is left, from their least.
fn natural(app: &mut VectorcraftApp, ctx: &egui::Context, p: Panel) -> f32 {
    match p {
        Panel::Pen => palette::height(app, ctx),
        Panel::Props => props::height(app, ctx),
        _ => least(p),
    }
}

/// The least height panel `p` shrinks to (shorter, it scrolls): the legend two rows, the pen its
/// weight row, the properties their head (design 3.1 §13).
fn least(p: Panel) -> f32 {
    match p {
        Panel::Set => drawings::MIN_HEIGHT,
        Panel::Legend => legend::MIN_HEIGHT,
        Panel::Pen => palette::MIN_HEIGHT,
        Panel::Props => props::HEAD_H,
        Panel::Layers => layers::MIN_HEIGHT,
    }
}

/// Where panel `p`'s head ends below its top: the part that moves it.
fn head_height(p: Panel) -> f32 {
    match p {
        Panel::Set => drawings::HEAD_H,
        Panel::Legend => legend::HEAD_H,
        Panel::Pen => palette::HEAD_END,
        Panel::Props => props::HEAD_H,
        Panel::Layers => layers::HEAD_H,
    }
}

/// How a column's panels share its height `h`.
pub(super) struct Plan {
    pub(super) heights: Vec<f32>,
    /// Each panel's share is what the others leave (a filling panel without a height of its own).
    pub(super) shares: Vec<bool>,
    /// They fit; else the column scrolls, each panel at its least or own height.
    pub(super) fits: bool,
}

/// Share height `h` between `panels` (their own heights `own`, their least `mins`): a panel the
/// user sized keeps that height, the pen and the properties are as tall as they show, and the
/// filling panels without a height of their own share the rest evenly (at least their least; when
/// every filling panel has a height, the first takes the rest). Too short for that, the column
/// scrolls.
pub(super) fn plan(layout: &model::Layout, panels: &[Panel], own: &[f32], mins: &[f32], h: f32) -> Plan {
    let n = panels.len();
    let mut shares: Vec<bool> = panels.iter().map(|p| p.fills() && layout.height(*p).is_none()).collect();
    if !shares.contains(&true)
        && let Some(i) = panels.iter().position(|p| p.fills())
        && let Some(s) = shares.get_mut(i)
    {
        *s = true;
    }
    let min = |i: usize| mins.get(i).copied().unwrap_or(0.0);
    let mut heights: Vec<f32> = (0..n)
        .map(|i| {
            let p = panels.get(i).copied();
            if shares.get(i).copied().unwrap_or(false) {
                min(i)
            } else {
                p.and_then(|p| layout.height(p)).unwrap_or_else(|| own.get(i).copied().unwrap_or(0.0)).max(min(i))
            }
        })
        .collect();
    let tall: f32 = heights.iter().sum();
    if tall > h {
        return Plan { heights, shares, fits: false };
    }
    // The rest shared: the sharers whose least is more than an even share take their least.
    let fixed: f32 = (0..n).filter(|i| !shares.get(*i).copied().unwrap_or(false)).filter_map(|i| heights.get(i)).sum();
    let mut sharers: Vec<usize> = (0..n).filter(|i| shares.get(*i).copied().unwrap_or(false)).collect();
    sharers.sort_by(|a, b| min(*b).total_cmp(&min(*a)));
    let mut rest = h - fixed;
    let mut left = sharers.len();
    for i in sharers {
        let even = rest / left.max(1) as f32;
        let take = even.max(min(i));
        if let Some(x) = heights.get_mut(i) {
            *x = take;
        }
        rest -= take;
        left = left.saturating_sub(1);
    }
    Plan { heights, shares, fits: true }
}

/// Rects of `heights` stacked down from the top of `r`.
fn place(r: Rect, heights: &[f32]) -> Vec<Rect> {
    heights
        .iter()
        .scan(r.top(), |y, h| {
            let slot = Rect::from_min_size(pos2(r.left(), *y), vec2(r.width(), *h));
            *y += h;
            Some(slot)
        })
        .collect()
}

/// Column `c`'s panels in `r`: fitted, with the boundaries between them that size them, or
/// scrolled when the column is too short. Returns where each went.
fn stack(app: &mut VectorcraftApp, ui: &mut Ui, c: Column, r: Rect, heads: &mut Vec<(Panel, egui::Response)>, t: &Tokens) -> Vec<(Panel, Rect)> {
    let ctx = ui.ctx().clone();
    let panels: Vec<Panel> = app.ui.trowel_layout.panels(c).to_vec();
    let own: Vec<f32> = panels.iter().map(|p| natural(app, &ctx, *p)).collect();
    let mins: Vec<f32> = panels.iter().map(|p| least(*p)).collect();
    let plan = plan(&app.ui.trowel_layout, &panels, &own, &mins, r.height());
    if plan.fits {
        let mut rects = place(r, &plan.heights);
        hold_pen(ui, c, &panels, &plan, &mins, &mut rects, r);
        for (i, (p, slot)) in panels.iter().zip(&rects).enumerate() {
            draw(app, ui, *p, *slot, own.get(i).copied().unwrap_or(0.0), i == 0, heads, t);
        }
        boundaries(app, ui, c, &panels, &plan, &mins, &rects, r);
        panels.into_iter().zip(rects).collect()
    } else {
        if panels.contains(&Panel::Pen) {
            ui.data_mut(|d| d.remove::<(f32, f32, bool)>(pen_top_id()));
        }
        let total: f32 = plan.heights.iter().sum();
        let mut out = vec![];
        let mut column = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::top_down(egui::Align::Min)));
        egui::ScrollArea::vertical().id_salt(wid("column-scroll", c.id())).auto_shrink([false, false]).show(&mut column, |ui| {
            let (_, content) = ui.allocate_space(vec2(ui.available_width(), total));
            let rects = place(content, &plan.heights);
            for (i, (p, slot)) in panels.iter().zip(&rects).enumerate() {
                draw(app, ui, *p, *slot, own.get(i).copied().unwrap_or(0.0), i == 0, heads, t);
            }
            out = panels.iter().copied().zip(rects).collect();
        });
        out
    }
}

/// Panel `p` in `slot`, which is `own` tall at its own height: a pen or properties made shorter
/// than that scroll inside. Its head is registered first, under what the panel draws, so its
/// buttons keep their clicks and a drag from anywhere else on it moves it.
#[allow(clippy::too_many_arguments)]
fn draw(app: &mut VectorcraftApp, ui: &mut Ui, p: Panel, slot: Rect, own: f32, first: bool, heads: &mut Vec<(Panel, egui::Response)>, t: &Tokens) {
    let head = Rect::from_min_size(slot.min, vec2(slot.width(), head_height(p).min(slot.height())));
    // Clicks too, so a press on one of the head's buttons waits to see whether it is a drag:
    // a click on « or ＋ is the button's, never the start of a move.
    let resp = ui.interact(head, wid("panel-head", p.id()), Sense::click_and_drag());
    heads.push((p, resp));
    // The pen and the properties draw their own line over them.
    if !first && p.fills() {
        line_above(ui, slot, t.border);
    }
    if !p.fills() && slot.height() + 0.5 < own {
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(slot).layout(egui::Layout::top_down(egui::Align::Min)));
        egui::ScrollArea::vertical().id_salt(wid("panel-scroll", p.id())).auto_shrink([false, false]).show(&mut child, |ui| {
            let (_, content) = ui.allocate_space(vec2(ui.available_width(), own));
            body(app, ui, p, content);
        });
    } else {
        body(app, ui, p, slot);
    }
}

/// What panel `p` shows, in `r`.
fn body(app: &mut VectorcraftApp, ui: &mut Ui, p: Panel, r: Rect) {
    match p {
        Panel::Set => drawings::show(app, ui, r),
        Panel::Legend => legend::show(app, ui, r),
        Panel::Pen => palette::show(app, ui, r),
        Panel::Props => props::show(app, ui, r),
        Panel::Layers => layers::show(app, ui, r),
    }
}

/// When the pen's three buttons come or go under the pointer, its top stays where it is so the
/// rows under the pointer don't move: what is under it gives way (pushed out of the column, or a
/// gap over it) until the pointer leaves the pen. Only where the pen sits under a filling panel,
/// which gives or takes the difference (the right column as it comes), and not while column `c`'s
/// boundaries are dragged, which size it on purpose.
#[allow(clippy::too_many_arguments)]
fn hold_pen(ui: &Ui, c: Column, panels: &[Panel], plan: &Plan, mins: &[f32], rects: &mut [Rect], r: Rect) {
    // Another column's panels leave the pen's hold alone.
    let Some(i) = panels.iter().position(|p| *p == Panel::Pen) else { return };
    let sizing = ui.ctx().dragged_id().is_some_and(|d| (0..panels.len()).any(|k| d == wid("panel-boundary", (c.id(), k))));
    let Some(above) = i.checked_sub(1).filter(|j| !sizing && plan.shares.get(*j).copied().unwrap_or(false)) else {
        ui.data_mut(|d| d.remove::<(f32, f32, bool)>(pen_top_id()));
        return;
    };
    let (Some(pen), Some(over_it)) = (rects.get(i).copied(), rects.get(above).copied()) else { return };
    let pen_h = pen.height();
    let mut top = pen.top();
    let pointer = ui.ctx().pointer_hover_pos();
    let over = |t: f32, h: f32| pointer.is_some_and(|p| Rect::from_min_size(pos2(r.left(), t), vec2(r.width(), h)).contains(p));
    // Where the pen was, how tall, and whether it is held: from when its height changed under the
    // pointer until the pointer leaves it, as tall as it was at its tallest meanwhile.
    let mut h = pen_h;
    let mut held = false;
    if let Some((was, was_h, was_held)) = ui.data(|d| d.get_temp::<(f32, f32, bool)>(pen_top_id()))
        && over(was, was_h.max(pen_h))
        && (was_held || was_h != pen_h)
    {
        top = was.max(over_it.top() + mins.get(above).copied().unwrap_or(0.0));
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
    if top == pen.top() {
        return;
    }
    if let Some(x) = rects.get_mut(above) {
        x.max.y = top;
    }
    if let Some(x) = rects.get_mut(i) {
        *x = Rect::from_min_size(pos2(x.left(), top), x.size());
    }
    // What is under it stays where it was, or goes down to clear it.
    let mut y = top + pen_h;
    for x in rects.iter_mut().skip(i + 1) {
        *x = Rect::from_min_size(pos2(x.left(), x.top().max(y)), x.size());
        y = x.bottom();
    }
}

/// Where the pen's top is held while the pointer is over it ([`hold_pen`]).
fn pen_top_id() -> egui::Id {
    wid("pen-top-held", ())
}

/// The boundaries between column `c`'s panels: dragged, they size the panel beside them on the
/// side away from the panels sharing the rest (the one below when those are all above it, else
/// the one above), so the boundary follows the pointer; a double click gives that panel its own
/// height back.
#[allow(clippy::too_many_arguments)]
fn boundaries(app: &mut VectorcraftApp, ui: &Ui, c: Column, panels: &[Panel], plan: &Plan, mins: &[f32], rects: &[Rect], r: Rect) {
    let shares = |i: usize| plan.shares.get(i).copied().unwrap_or(false);
    for k in 0..panels.len().saturating_sub(1) {
        let (Some(a), Some(b)) = (rects.get(k), rects.get(k + 1)) else { continue };
        let zone = Rect::from_min_max(pos2(r.left(), a.bottom() - 3.0), pos2(r.right(), a.bottom() + 3.0));
        let resp = ui.interact(zone, wid("panel-boundary", (c.id(), k)), Sense::click_and_drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        let below = (0..=k).any(shares) && !(k + 1..panels.len()).any(shares);
        let target = if below { k + 1 } else { k };
        let Some(p) = panels.get(target).copied() else { continue };
        if resp.double_clicked() {
            app.ui.trowel_layout.set_height(p, None);
        } else if resp.dragged()
            && let Some(at) = resp.interact_pointer_pos()
        {
            // As tall as the others leave it: the sharers at their least, the rest as they are.
            let others: f32 = (0..panels.len())
                .filter(|j| *j != target)
                .map(|j| if shares(j) { mins.get(j).copied().unwrap_or(0.0) } else { plan.heights.get(j).copied().unwrap_or(0.0) })
                .sum();
            let lo = mins.get(target).copied().unwrap_or(0.0);
            let hi = (r.height() - others).max(lo);
            let h = if below { b.bottom() - at.y } else { at.y - a.top() };
            app.ui.trowel_layout.set_height(p, Some(h.clamp(lo, hi)));
        }
    }
}

/// Column `c`'s edge by the canvas (`r` is the column): dragged, its width within its range; a
/// double click, its first width.
fn width_edge(app: &mut VectorcraftApp, ui: &Ui, c: Column, r: Rect) {
    let zone = match c {
        Column::Left => Rect::from_min_max(pos2(r.right() - 5.0, r.top()), r.max),
        Column::Right | Column::Edge => Rect::from_min_max(r.min, pos2(r.left() + 5.0, r.bottom())),
    };
    let resp = ui.interact(zone, wid("column-edge", c.id()), Sense::click_and_drag());
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    if resp.double_clicked() {
        app.ui.trowel_layout.set_width(c, c.default_width());
    } else if resp.dragged()
        && let Some(at) = resp.interact_pointer_pos()
    {
        let w = match c {
            Column::Left => at.x - r.left(),
            Column::Right | Column::Edge => r.right() - at.x,
        };
        app.ui.trowel_layout.set_width(c, w);
    }
}

/// The names of what column `c` holds, as its rail and the ghost say them.
fn names(app: &VectorcraftApp, c: Column) -> String {
    app.ui.trowel_layout.panels(c).iter().map(|p| panel_name(*p)).collect::<Vec<_>>().join("·")
}

/// A panel's name.
pub(super) fn panel_name(p: Panel) -> &'static str {
    match p {
        Panel::Set => tw!("Drawing Set"),
        Panel::Legend => tw!("Legend"),
        Panel::Pen => tw!("Pen"),
        Panel::Props => tw!("Properties"),
        Panel::Layers => tw!("Layers"),
    }
}

/// A folded column's rail (the mockup's `.lyrail`, 28 wide): », its panels' names down it and,
/// holding the layer strip, the colour of the layer drawn on. A press anywhere on it opens it.
fn rail(app: &mut VectorcraftApp, ui: &Ui, c: Column, r: Rect, t: &Tokens) {
    let name = names(app, c);
    let resp = ui.interact(r, wid("column-rail", c.id()), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r.shrink(1.0), 0.0, t.hover);
    }
    let button = Rect::from_center_size(pos2(r.center().x, r.top() + 7.0 + 11.0), vec2(22.0, 22.0));
    crate::icons::paint(ui, "chevrons-right", button.shrink(4.0), t.text_dim);
    // The names down the rail, a letter under another (its `.vt`).
    let mut y = button.bottom() + 8.0;
    for ch in name.chars() {
        let g = text(ui, pos2(r.center().x, y), Align2::CENTER_TOP, ch, bold(12.5), t.text);
        y = g.bottom() + 1.5;
        if y > r.bottom() - 20.0 {
            break;
        }
    }
    if app.ui.trowel_layout.panels(c).contains(&Panel::Layers)
        && let Some(colour) = layers::current_colour(app)
    {
        let bar = Rect::from_min_size(pos2(r.center().x - 7.0, y + 6.0), vec2(14.0, 3.0));
        ui.painter().rect_filled(bar, CornerRadius::same(1), colour);
    }
    let tip = crate::i18n::fmt(tw!("Open {name}"), &[("name", &name)]);
    if resp.on_hover_text(tip).clicked() {
        app.ui.trowel_layout.set_folded(c, Some(false));
    }
}

/// Where a panel dropped with the pointer at `p` goes: a column and a place there, and how to show
/// it (a line across the column at that place, or a bar where a column will be).
fn drop_at(drawn: &[Drawn], body_left: f32, screen: Rect, p: Pos2) -> Option<(Column, usize, Rect)> {
    let present = |c: Column| drawn.iter().any(|d| d.column == c);
    let (top, bottom) = drawn.first().map_or((screen.top(), screen.bottom()), |d| (d.rect.top(), d.rect.bottom()));
    if !(top..=bottom).contains(&p.y) {
        return None;
    }
    let bar = |x: f32| Rect::from_min_max(pos2(x - 2.0, top), pos2(x + 2.0, bottom));
    // The strip's column gone: the window's right edge brings it back.
    if !present(Column::Edge) && p.x >= screen.right() - 16.0 {
        return Some((Column::Edge, 0, bar(screen.right() - 2.0)));
    }
    for d in drawn {
        if !d.rect.x_range().contains(p.x) {
            continue;
        }
        if d.folded {
            return Some((d.column, usize::MAX, d.rect.shrink(1.0)));
        }
        let i = d.slots.iter().filter(|(_, s)| s.center().y < p.y).count();
        let y = match i.checked_sub(1).and_then(|j| d.slots.get(j)) {
            Some((_, s)) => s.bottom(),
            None => d.slots.first().map_or(d.rect.top(), |(_, s)| s.top()),
        };
        let line = Rect::from_min_max(pos2(d.rect.left(), y - 1.5), pos2(d.rect.right(), y + 1.5));
        return Some((d.column, i, line));
    }
    // A side without its column: its edge by the canvas brings it back.
    let canvas_right = drawn.iter().filter(|d| d.column != Column::Left).map(|d| d.rect.left()).fold(screen.right(), f32::min);
    let canvas_left = drawn.iter().filter(|d| d.column == Column::Left).map(|d| d.rect.right()).fold(body_left, f32::max);
    if !present(Column::Left) && p.x <= canvas_left + 24.0 {
        return Some((Column::Left, 0, bar(canvas_left + 2.0)));
    }
    if !present(Column::Right) && p.x >= canvas_right - 24.0 {
        return Some((Column::Right, 0, bar(canvas_right - 2.0)));
    }
    None
}

/// Move a panel dragged by its head: a ghost of it follows the pointer and a clay line shows where
/// it would go; let go there, it goes there. Let go over the canvas, or after Esc, it stays.
fn moving(app: &mut VectorcraftApp, ui: &Ui, drawn: &[Drawn], heads: &[(Panel, egui::Response)], body_left: f32) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    for (p, resp) in heads {
        if resp.drag_started() {
            let slot = drawn.iter().flat_map(|d| &d.slots).find(|(q, _)| q == p).map_or(resp.rect, |(_, s)| *s);
            let at = resp.interact_pointer_pos().unwrap_or(slot.min);
            ctx.data_mut(|d| d.insert_temp(moving_id(), Moving { panel: *p, grab: at - slot.min, size: slot.size(), cancelled: false }));
        }
    }
    let Some(mut m) = ctx.data(|d| d.get_temp::<Moving>(moving_id())) else { return };
    let head = heads.iter().find(|(p, _)| *p == m.panel).map(|(_, r)| r);
    let (dragging, stopped) = head.map_or((false, false), |r| (r.dragged(), r.drag_stopped()));
    if !dragging && !stopped {
        ctx.data_mut(|d| d.remove::<Moving>(moving_id()));
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        m.cancelled = true;
        ctx.data_mut(|d| d.insert_temp(moving_id(), m));
    }
    let screen = ctx.content_rect();
    let pointer = ctx.pointer_latest_pos();
    let target = if m.cancelled { None } else { pointer.and_then(|p| drop_at(drawn, body_left, screen, p)) };
    if stopped {
        ctx.data_mut(|d| d.remove::<Moving>(moving_id()));
        if let Some((c, i, _)) = target {
            app.ui.trowel_layout.move_panel(m.panel, c, i);
            // Dropped on a folded column's rail: it opens to show what came.
            if drawn.iter().any(|d| d.column == c && d.folded) {
                app.ui.trowel_layout.set_folded(c, Some(false));
            }
            ctx.request_repaint();
        }
        return;
    }
    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, wid("panel-ghost", ())));
    if let Some((_, _, mark)) = target {
        painter.rect_filled(mark, CornerRadius::same(1), t.clay);
    }
    if let Some(p) = pointer {
        let ghost = Rect::from_min_size(p - m.grab, vec2(m.size.x, m.size.y.min(120.0)));
        painter.rect(ghost, CornerRadius::same(6), t.panel.gamma_multiply(0.82), Stroke::new(1.0, t.border), StrokeKind::Inside);
        let title = Rect::from_min_size(ghost.min, vec2(ghost.width(), head_height(m.panel).min(ghost.height())));
        painter.text(pos2(title.left() + 10.0, title.center().y), Align2::LEFT_CENTER, panel_name(m.panel), bold(13.5), t.text);
    }
}
