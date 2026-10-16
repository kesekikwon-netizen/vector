//! The pen, under the legend in the right column (design 3.0 §5 ⑧, §7): the look of the next line
//! drawn, or of the selected lines, in four rows (weight from the app's one list, line type,
//! colour, hatch), and whether it is its legend row's. When it isn't, three buttons: Just This
//! Once (draw on with it, the lines set apart), Update the Legend (the row takes it) and Back to
//! the Legend (the row's look again).

use serde_json::{Value, json};
use vectorcraft_doc::legend::{SAME, is_line, lines, stroke_of};
use vectorcraft_doc::{LineType, Node, NodeId};
use vectorcraft_engine::cmd::legend::LADDER;

use super::*;

/// Above the head, and the head (the mockup's `.pen .ph`).
const TOP: f32 = 2.0;
const HEAD_H: f32 = 34.0;
/// The weight row (chips 30 tall and 3 either side) and the other rows.
const WEIGHT_ROW: f32 = 36.0;
const ROW: f32 = 32.0;
/// The same-or-not line, the three buttons under it when they show, and the room at the foot.
const STATE_H: f32 = 22.0;
const BUTTONS_H: f32 = 6.0 + 24.0 + 10.0;
const BOTTOM: f32 = 8.0;
/// The rows' labels at the left, 36 wide, then 6 to the controls (the mockup's `.pr2`).
const PAD: f32 = 10.0;
const LABEL_W: f32 = 36.0;
/// A weight chip (the mockup's `.wch`) and the room between two.
const CHIP_W: f32 = 24.0;
const CHIP_H: f32 = 30.0;
const CHIP_GAP: f32 = 2.0;
/// How thick a weight's sample is drawn (points → pixels), as the mockup's chips.
const SAMPLE_PX: [(f64, f32); 9] = [(0.1, 0.5), (0.15, 0.7), (0.2, 0.9), (0.3, 1.2), (0.4, 1.5), (0.5, 1.8), (0.6, 2.2), (0.8, 2.8), (1.0, 3.4)];

/// The pen's height: taller by the three buttons while they show.
pub(super) fn height(app: &VectorcraftApp, ctx: &egui::Context) -> f32 {
    let base = TOP + HEAD_H + WEIGHT_ROW + 3.0 * ROW + STATE_H + BOTTOM;
    if buttons_show(app, ctx, &against(app)) { base + BUTTONS_H } else { base }
}

/// The colours the pen offers (design 3.0 §5 ⑧): key, name and colour. The colours are what the
/// drawing gets, as the palette has always put down.
fn swatches() -> [(&'static str, &'static str, [u8; 3]); 8] {
    [
        ("black", tw!("Black"), [20, 20, 20]),
        ("red", tw!("Red"), [210, 35, 35]),
        ("blue", tw!("Blue"), [30, 95, 195]),
        ("darkbrown", tw!("Dark brown"), [75, 52, 38]),
        ("brown", tw!("Brown"), [130, 84, 48]),
        ("ochre", tw!("Yellow brown"), [190, 142, 74]),
        ("gray", tw!("Grey"), [140, 140, 140]),
        ("white", tw!("White"), [250, 248, 242]),
    ]
}

/// A legend row: its id, name and look.
type RowLook = (NodeId, String, f64, LineType);
/// A look: weight and line type.
type Look = (f64, LineType);

/// The pen against its legend row.
#[derive(Clone, Default, PartialEq)]
struct Against {
    /// The row the next line goes on, or the row of the first selected line set apart (else of the
    /// first selected line); none when it has no look.
    row: Option<RowLook>,
    /// The look shown: the pen's, or that line's.
    shown: Option<Look>,
    /// The name of the layer the next line goes on, or of that line's layer, with a look or without.
    name: Option<String>,
    /// How many lines are selected.
    selected: usize,
    /// Whether the look shown, or any selected line, is set apart from its row.
    differs: bool,
}

impl Against {
    /// What differs, as the mockup says it: "Weight 0.6 → 0.8", "Line Solid → Dashed".
    fn what(&self) -> String {
        let (Some((_, _, rw, rt)), Some((w, line))) = (&self.row, &self.shown) else { return String::new() };
        let mut parts = vec![];
        if (rw - w).abs() > SAME {
            parts.push(crate::i18n::fmt(tw!("Weight {from} → {to}"), &[("from", &super::legend::pt(*rw)), ("to", &super::legend::pt(*w))]));
        }
        if !rt.same(line) {
            parts.push(crate::i18n::fmt(tw!("Line {from} → {to}"), &[("from", line_name(rt)), ("to", line_name(line))]));
        }
        parts.join(" · ")
    }

    /// Key of the look that differs against its row, for Just This Once: the buttons stay folded
    /// until it changes. The same for the pen and for lines drawn with it.
    fn key(&self) -> String {
        let row = self.row.as_ref().map(|(id, _, w, t)| format!("{}:{w}:{t:?}", id.0)).unwrap_or_default();
        let shown = self.shown.as_ref().map(|(w, t)| format!("{w}:{t:?}")).unwrap_or_default();
        format!("{row}|{shown}")
    }
}

/// The name of a line type, as the line row shows it.
fn line_name(t: &LineType) -> &'static str {
    line_types().iter().find(|(_, _, kind)| kind.same(t)).map_or(tw!("Custom"), |(_, label, _)| label)
}

/// The lines of a selected object: itself, or the lines in a group.
fn lines_of(n: &Node) -> Vec<&Node> {
    if is_line(n) { vec![n] } else { lines(n) }
}

fn against(app: &VectorcraftApp) -> Against {
    let Some(st) = app.session.active() else { return Against::default() };
    let doc = &st.doc;
    let row_of = |id: NodeId| {
        let layer = doc.node(doc.layer_of(id)?)?;
        let (w, t) = layer.legend()?.look()?;
        Some((layer.id, layer.display_name(), w, t.clone()))
    };
    let selected: Vec<&Node> = st.selection.objects.iter().filter_map(|id| doc.node(*id)).flat_map(lines_of).collect();
    if selected.is_empty() {
        let row = st.current_layer().and_then(row_of);
        let name = st.current_layer().and_then(|id| doc.node(id)).map(|l| l.display_name());
        let pen = app.session.new_art_stroke();
        let shown = (pen.width, LineType::of(&pen));
        let differs = row.as_ref().is_some_and(|(_, _, w, t)| (w - shown.0).abs() > SAME || !t.same(&shown.1));
        return Against { row, shown: Some(shown), name, selected: 0, differs };
    }
    // The first selected line set apart from its row, else the first one.
    let looks: Vec<(Option<RowLook>, Option<Look>)> =
        selected.iter().map(|l| (row_of(l.id), stroke_of(l).map(|s| (s.width, LineType::of(s))))).collect();
    let apart = |(row, shown): &(Option<RowLook>, Option<Look>)| match (row, shown) {
        (Some((_, _, rw, rt)), Some((w, t))) => (rw - w).abs() > SAME || !rt.same(t),
        _ => false,
    };
    let at = looks.iter().position(apart).unwrap_or(0);
    let first = looks.get(at).cloned().unwrap_or_default();
    let name = selected.get(at).and_then(|l| doc.node(doc.layer_of(l.id)?)).map(|l| l.display_name());
    Against { row: first.0, shown: first.1, name, selected: selected.len(), differs: looks.iter().any(apart) }
}

/// Where Just This Once keeps the look it was pressed for, by drawing.
fn once_id() -> egui::Id {
    wid("pen-once", ())
}

/// Where the pen's look against its row is kept while nothing is selected and it differs.
fn pen_key_id() -> egui::Id {
    wid("pen-differs", ())
}

/// Whether the tool in hand draws: not one of the selection tools.
fn drawing_tool(app: &VectorcraftApp) -> bool {
    let tool = app.session.tool_id();
    !(vectorcraft_tools::catalog::is_selection_tool(tool) || matches!(tool, "lasso" | "magicWand"))
}

/// Just This Once without the button (design 3.0 §7: drawing on without pressing anything counts
/// as it): with a drawing tool in hand, lines selected in the pen's look against its row were
/// drawn with it, and the buttons fold. The pen as its row again forgets it.
fn drawn_once(app: &VectorcraftApp, ctx: &egui::Context, a: &Against) {
    if a.selected == 0 {
        ctx.data_mut(|d| {
            if a.differs {
                d.insert_temp(pen_key_id(), a.key());
            } else {
                d.remove::<String>(pen_key_id());
                d.remove::<String>(once_id());
            }
        });
        return;
    }
    let pen = ctx.data(|d| d.get_temp::<String>(pen_key_id()));
    if drawing_tool(app) && buttons_show(app, ctx, a) && pen.is_some_and(|k| k == a.key()) {
        ctx.data_mut(|d| d.insert_temp(once_id(), a.key()));
        // The pen was laid out with its buttons: once more without them.
        ctx.request_repaint();
    }
}

/// Whether the three buttons show: the look differs, and Just This Once wasn't pressed for it (or
/// the lines in it were picked out with a selection tool: they may be for the legend now).
fn buttons_show(app: &VectorcraftApp, ctx: &egui::Context, a: &Against) -> bool {
    let once = ctx.data(|d| d.get_temp::<String>(once_id())).is_some_and(|k| k == a.key());
    a.differs && a.row.is_some() && !(once && (a.selected == 0 || drawing_tool(app)))
}

/// Where whether the weight is being typed is kept.
fn typing_id() -> egui::Id {
    wid("pen-weight-typing", ())
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_above(ui, r, t.border);
    let ctx = ui.ctx().clone();
    let a = against(app);
    drawn_once(app, &ctx, &a);
    let open = app.session.active().is_some();
    // The head: "Pen", the row and what it draws.
    let head = Rect::from_min_size(r.min + vec2(0.0, TOP), vec2(r.width(), HEAD_H));
    let what =
        if a.selected > 0 { crate::i18n::fmt(tw!("{n} selected lines"), &[("n", &a.selected.to_string())]) } else { tw!("Next line").to_string() };
    let sub = match &a.name {
        Some(name) => format!("{name} · {what}"),
        None => what,
    };
    column_head(ui, head, tw!("Pen"), &sub, 1, &t);
    if more_button(ui, wid("palette-more", ()), head, 0, tw!("Stroke options"), &t).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "stroke" }));
    }
    let label = |ui: &Ui, cy: f32, s: &str| {
        text(ui, pos2(r.left() + PAD, cy), Align2::LEFT_CENTER, s, font(12.0), t.text_dim);
    };
    let x0 = r.left() + PAD + LABEL_W + 6.0;
    let room = (r.right() - PAD - x0).max(0.0);
    let mut set: Option<Value> = None;

    // Weight: the one list's chips, or points typed after a double click.
    let cy = head.bottom() + WEIGHT_ROW / 2.0;
    label(ui, cy, tw!("Weight"));
    let stroke = crate::panels::current_stroke(app);
    let mixed = crate::panels::stroke_mixed(app, &ctx);
    let weight = crate::panels::stroke::shown_weight(app, stroke.as_ref(), &mixed);
    let typing = ctx.data(|d| d.get_temp::<bool>(typing_id())).unwrap_or(false);
    if typing {
        let field = Rect::from_min_size(pos2(x0, cy - 13.0), vec2(room.min(120.0), 26.0));
        let id = wid("pen-weight-field", ());
        let shown = weight.map(super::legend::pt).unwrap_or_default();
        let typed = super::card::text_field(ui, id, field, &shown, mono(11.5), ("", "pt"), &t);
        if !ui.memory(|m| m.has_focus(id)) {
            ctx.data_mut(|d| d.insert_temp(typing_id(), false));
        }
        match typed.as_deref().map(super::card::points) {
            Some(Some(w)) => set = Some(json!({ "weight": w })),
            Some(None) => app.status_error(tw!("Type a weight in points, such as 0.5")),
            None => {}
        }
    } else {
        let chip_w = ((room - CHIP_GAP * (LADDER.len() as f32 - 1.0)) / LADDER.len() as f32).clamp(14.0, CHIP_W);
        let mut x = x0;
        for w in LADDER {
            let chip = Rect::from_min_size(pos2(x, cy - CHIP_H / 2.0), vec2(chip_w, CHIP_H));
            let resp = ui.interact(chip, wid("weight", super::legend::pt(w)), Sense::click());
            let on = weight.is_some_and(|v| (v - w).abs() < 1e-6);
            weight_chip(ui, chip, w, on, resp.hovered(), &t);
            let resp = resp.on_hover_text(crate::i18n::fmt(tw!("{w} pt · double-click to type a weight"), &[("w", &super::legend::pt(w))]));
            if resp.double_clicked() {
                ctx.data_mut(|d| d.insert_temp(typing_id(), true));
                ui.memory_mut(|m| m.request_focus(wid("pen-weight-field", ())));
            } else if resp.clicked() {
                set = Some(json!({ "weight": w }));
            }
            x = chip.right() + CHIP_GAP;
        }
    }

    // Line type: the first selected line's, or the next line's; none when the selected lines'
    // dashes differ.
    let cy = head.bottom() + WEIGHT_ROW + ROW / 2.0;
    label(ui, cy, tw!("Line"));
    let shown_dash = (!mixed.dash).then(|| stroke.as_ref().map_or(LineType::Solid, LineType::of));
    let kinds = line_types();
    let parts: Vec<(&str, &str, bool)> =
        kinds.iter().map(|(key, label, kind)| (*key, *label, shown_dash.as_ref().is_some_and(|s| s.same(kind)))).collect();
    if let Some(key) = even_segments(ui, "dash", Rect::from_min_size(pos2(x0, cy - 13.0), vec2(room, 26.0)), &parts, &t)
        && let Some((_, _, kind)) = kinds.iter().find(|(k, _, _)| *k == key)
    {
        set = Some(json!({ "dash": kind.dash_pt() }));
    }

    // Colour: a click the line's, Alt and a click the fill's.
    let cy = head.bottom() + WEIGHT_ROW + ROW + ROW / 2.0;
    label(ui, cy, tw!("Colour"));
    let line_colour = stroke.as_ref().and_then(|s| match &s.paint {
        vectorcraft_color::Paint::Solid { color, .. } => Some(crate::panels::c32(color)),
        _ => None,
    });
    let alt = ui.input(|i| i.modifiers.alt);
    let mut x = x0;
    for (key, name, rgb) in swatches() {
        let sw = Rect::from_min_size(pos2(x, cy - 10.0), vec2(20.0, 20.0));
        let resp = ui.interact(sw, wid("pal-swatch", key), Sense::click());
        let colour = crate::panels::c32(&vectorcraft_color::Color::rgb8(rgb[0], rgb[1], rgb[2]));
        if line_colour == Some(colour) {
            // The mockup's ring: the panel, then ink around it.
            ui.painter().rect_stroke(sw.expand(2.75), CornerRadius::same(6), Stroke::new(1.5, t.text), StrokeKind::Middle);
        }
        let edge = if resp.hovered() { t.text } else { t.input_border };
        ui.painter().rect(sw, CornerRadius::same(4), colour, Stroke::new(1.0, edge), StrokeKind::Inside);
        let resp = resp.on_hover_text(crate::i18n::fmt(tw!("{name} · click: the line's colour · Alt+click: the fill's"), &[("name", name)]));
        if resp.clicked() && open {
            let cmd = if alt { "paint.setFill" } else { "paint.setStroke" };
            if let Err(e) = app.run(cmd, json!({ "color": rgb })) {
                app.status_error(e);
            }
        }
        x = sw.right() + 4.0;
    }
    let more = Rect::from_min_size(pos2(x, cy - 10.0), vec2(20.0, 20.0));
    let more_resp = ui.interact(more, wid("pal-color-more", ()), Sense::click());
    if more_resp.hovered() {
        ui.painter().rect_filled(more, CornerRadius::same(4), t.hover);
    }
    text(ui, more.center(), Align2::CENTER_CENTER, "⋯", font(12.0), t.text_dim);
    if more_resp.on_hover_text(tw!("Swatches")).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "swatches" }));
    }

    // Hatch: on the selected closed shapes, or the next ones; Alternate again flips it.
    let cy = head.bottom() + WEIGHT_ROW + 2.0 * ROW + ROW / 2.0;
    label(ui, cy, tw!("Hatch"));
    let hatch = if open { app.session.execute("hatch.get", &json!({})).ok() } else { None };
    let dir = hatch.as_ref().and_then(|v| v["direction"].as_str()).unwrap_or("none").to_string();
    let conflict = hatch.as_ref().and_then(|v| v["has_conflict"].as_bool()).unwrap_or(false);
    let alternate = if conflict { format!("{} !", tw!("Alternate")) } else { tw!("Alternate").to_string() };
    let parts =
        [("none", tw!("None"), dir == "none"), ("45", "45°", dir == "45"), ("135", "135°", dir == "135"), ("alt", alternate.as_str(), dir == "alt")];
    let hatch_r = Rect::from_min_size(pos2(x0, cy - 13.0), vec2(room, 26.0));
    if let Some(key) = even_segments(ui, "hatch", hatch_r, &parts, &t) {
        let done = if key == "alt" && dir == "alt" { app.run("hatch.flip", json!({})) } else { app.run("hatch.set", json!({ "direction": key })) };
        if let Err(e) = done {
            app.status_error(e);
        }
    }
    if conflict {
        ui.interact(hatch_r, wid("hatch-conflict", ()), Sense::hover())
            .on_hover_text(tw!("Two neighbours hatch the same way: press Alternate to flip it"));
    }

    // Same as the legend, or not, and the three buttons.
    let cy = head.bottom() + WEIGHT_ROW + 3.0 * ROW + 6.0 + 8.0;
    let x = r.left() + PAD;
    match (&a.row, a.differs) {
        (None, _) => {
            text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("The layer has no legend look"), font(11.5), t.text_dim);
        }
        (Some(_), false) => {
            // The mockup's mark: a check in a filled circle, 12 across.
            let mark = Rect::from_center_size(pos2(x + 6.0, cy), vec2(12.0, 12.0));
            ui.painter().circle_filled(mark.center(), 5.0, t.ok);
            let p = |px: f32, py: f32| mark.min + vec2(px, py);
            ui.painter().add(egui::Shape::line(vec![p(3.5, 6.2), p(5.3, 7.9), p(8.6, 4.3)], Stroke::new(1.4, t.on_action)));
            text(ui, pos2(mark.right() + 6.0, cy), Align2::LEFT_CENTER, tw!("Same as the legend"), font(11.5), t.text_dim);
        }
        (Some(_), true) => {
            ui.painter().circle_filled(pos2(x + 4.0, cy), 3.5, t.action);
            let said = format!("{} · {}", tw!("Not as the legend"), a.what());
            text_to(ui, pos2(x + 14.0, cy), r.right() - PAD, &said, font(11.5), t.text_dim);
        }
    }
    let mut asked = None;
    if buttons_show(app, &ctx, &a) {
        let top = cy + 8.0 + 6.0;
        let w = ((r.width() - 2.0 * PAD - 2.0 * 4.0) / 3.0).max(0.0);
        for (i, (key, label)) in
            [("once", tw!("Just This Once")), ("update", tw!("Update the Legend")), ("back", tw!("Back to the Legend"))].into_iter().enumerate()
        {
            let b = Rect::from_min_size(pos2(r.left() + PAD + i as f32 * (w + 4.0), top), vec2(w, 24.0));
            let resp = ui.interact(b, wid("pen-state", key), Sense::click());
            field_box(ui, b, if resp.hovered() { t.hover } else { t.button }, t.input_border);
            text_to(
                ui,
                pos2(b.center().x - width_of(ui, label, font(11.5)).min(b.width() - 8.0) / 2.0, b.center().y),
                b.right() - 4.0,
                label,
                font(11.5),
                t.text,
            );
            let tip = match key {
                "once" => tw!("Draw on with this look: the lines keep it, set apart from the row"),
                "update" => tw!("The row takes this look, and its lines that follow it"),
                _ => tw!("The pen and the selected lines take the row's look again"),
            };
            if resp.on_hover_text(tip).clicked() {
                asked = Some(key);
            }
        }
    }
    if let Some(p) = set {
        crate::panels::stroke::set(app, p);
    }
    let done = match asked {
        Some("once") => {
            ctx.data_mut(|d| d.insert_temp(once_id(), a.key()));
            Ok(())
        }
        Some("update") => update_legend(app, &a),
        Some(_) => back_to_legend(app),
        None => Ok(()),
    };
    if let Err(e) = done {
        app.status_error(e);
    }
}

/// The row takes the look shown (`legend.set`: its lines that followed the old look follow).
fn update_legend(app: &mut VectorcraftApp, a: &Against) -> Result<(), String> {
    let (Some((id, name, _, _)), Some((w, line))) = (&a.row, &a.shown) else { return Ok(()) };
    let done = app.run("legend.set", json!({ "id": id.0, "weight": w, "line": line_param(line) }))?;
    let n = done["lines"].as_u64().unwrap_or(0);
    app.status(crate::i18n::fmt(tw!("{row} takes the pen's look · lines: {n}"), &[("row", name), ("n", &n.to_string())]));
    Ok(())
}

/// A line type as `legend.set` takes it.
fn line_param(t: &LineType) -> Value {
    match t {
        LineType::Solid => json!("solid"),
        LineType::Dashed => json!("dashed"),
        LineType::Dotted => json!("dotted"),
        LineType::DashDot => json!("dashdot"),
        LineType::Custom(d) => json!(d),
    }
}

/// Put the selected lines back to their row's look; with nothing selected the next line takes the
/// current row's look.
fn back_to_legend(app: &mut VectorcraftApp) -> Result<(), String> {
    if app.session.active().is_some_and(|st| !st.selection.objects.is_empty()) {
        return super::legend::revert(app, json!({}));
    }
    app.run("legend.revert", json!({}))?;
    let row = app.session.active().and_then(|st| st.doc.node(st.current_layer().and_then(|l| st.doc.layer_of(l))?));
    if let Some(row) = row.filter(|n| n.legend().is_some_and(|l| l.look().is_some())) {
        let note = crate::i18n::fmt(tw!("The next line takes {row}'s look"), &[("row", &row.display_name())]);
        app.status(note);
    }
    Ok(())
}

/// A weight chip (the mockup's `.wch`): its sample over the weight, ".6" for 0.6; on, two points
/// of ink around oat.
fn weight_chip(ui: &Ui, r: Rect, w: f64, on: bool, hovered: bool, t: &Tokens) {
    let (fill, edge) = match (on, hovered) {
        (true, _) => (t.row_selected, Stroke::new(2.0, t.text)),
        (false, true) => (t.hover, Stroke::new(1.0, t.input_border)),
        (false, false) => (t.input, Stroke::new(1.0, t.input_border)),
    };
    ui.painter().rect(r, CornerRadius::same(5), fill, edge, StrokeKind::Inside);
    let label = short(w);
    let f = mono(9.5);
    let text_r = text(ui, pos2(r.center().x, r.bottom() - 2.0), Align2::CENTER_BOTTOM, &label, f, t.text);
    let px = sample_px(w);
    let sample = Rect::from_center_size(pos2(r.center().x, text_r.top() - 5.0 - px / 2.0), vec2((r.width() - 8.0).min(16.0), px));
    ui.painter().rect_filled(sample, 0.0, t.text);
}

/// A weight as a chip shows it: ".6" for 0.6, "1.0" for 1.
pub(super) fn short(w: f64) -> String {
    let label = super::legend::pt(w);
    label.strip_prefix('0').filter(|s| s.starts_with('.')).unwrap_or(&label).to_string()
}

/// How thick weight `w` (points) is drawn in a chip's sample: the mockup's table, straight between
/// its rows.
pub(super) fn sample_px(w: f64) -> f32 {
    if !w.is_finite() {
        return 1.0;
    }
    let mut seg = None;
    for pair in SAMPLE_PX.windows(2) {
        if let [a, b] = pair {
            seg = Some((*a, *b));
            if w <= b.0 {
                break;
            }
        }
    }
    let Some(((x0, y0), (x1, y1))) = seg else { return 1.0 };
    (y0 + (y1 - y0) * ((w - x0) / (x1 - x0)) as f32).clamp(0.3, 5.0)
}

/// A segmented choice filling `r` in equal parts (the mockup's `.seg`): the part that is on in oat
/// and semibold, lines between the others. Parts are (key, label, on), each `wid(kind, key)`.
/// Returns the key pressed.
fn even_segments<'a>(ui: &Ui, kind: &str, r: Rect, parts: &[(&'a str, &str, bool)], t: &Tokens) -> Option<&'a str> {
    let n = parts.len().max(1) as f32;
    let w = r.width() / n;
    field_box(ui, r, t.input, t.input_border);
    let last = parts.len().saturating_sub(1);
    let mut pressed = None;
    for (i, (key, label, on)) in parts.iter().enumerate() {
        let part = Rect::from_min_size(pos2(r.left() + i as f32 * w, r.top()), vec2(w, r.height()));
        let inside = part.intersect(r.shrink(1.0));
        let corners = CornerRadius {
            nw: if i == 0 { 5 } else { 0 },
            sw: if i == 0 { 5 } else { 0 },
            ne: if i == last { 5 } else { 0 },
            se: if i == last { 5 } else { 0 },
        };
        let resp = ui.interact(part, wid(kind, *key), Sense::click());
        if *on {
            ui.painter().rect_filled(inside, corners, t.row_selected);
        } else if resp.hovered() {
            ui.painter().rect_filled(inside, corners, t.hover);
        }
        if i > 0 {
            ui.painter().vline(part.left() + 0.5, part.y_range().shrink(1.0), Stroke::new(1.0, t.border));
        }
        let f = if *on { bold(11.5) } else { font(11.5) };
        text_to(
            ui,
            pos2(part.center().x - width_of(ui, label, f.clone()).min(part.width() - 4.0) / 2.0, part.center().y),
            part.right() - 2.0,
            label,
            f,
            t.text,
        );
        if resp.clicked() {
            pressed = Some(*key);
        }
    }
    pressed
}
