//! The properties at the foot of the right column (design 3.0 §5 ⑨): folded to its head while
//! nothing is selected, open by itself when something is. The head says what is selected and on
//! which row; under it the selection's size in metres at the drawing's scale (paper millimetres
//! without one), a closed shape's area and perimeter (`trowel.polygonStats`, with a scale), where it
//! is (TM metres in the drawing's axis order with a coordinate system, paper millimetres without)
//! and, with two or more selected, six align buttons (`object.align`). ▸ and ▾ fold and open it.

use serde_json::json;
use vectorcraft_doc::scale::CoordOrder;

use super::*;

/// The head (the mockup's `.props .ph`), the rows under it and the room above and under them.
const HEAD_H: f32 = 34.0;
const ROW: f32 = 26.0;
const TOP: f32 = 2.0;
const BOTTOM: f32 = 8.0;
/// The rows' labels at the left, then 6 to their values (as the pen's rows).
const PAD: f32 = 10.0;
const LABEL_W: f32 = 36.0;
/// An align button and the room between two.
const BUTTON: f32 = 24.0;
const BUTTON_GAP: f32 = 2.0;

/// What the properties show.
#[derive(Default)]
struct Facts {
    selected: usize,
    /// "3 lines · 평면외곽선".
    head: String,
    size: Option<String>,
    area: Option<String>,
    position: Option<String>,
}

impl Facts {
    /// The rows under the head, when open: one saying there is nothing to show when nothing is.
    fn rows(&self) -> usize {
        let n = [&self.size, &self.area, &self.position].iter().filter(|r| r.is_some()).count() + usize::from(self.selected > 1);
        n.max(1)
    }
}

/// The properties' height: open, or folded to the head.
pub(super) fn height(app: &mut VectorcraftApp, ctx: &egui::Context) -> f32 {
    let facts = facts(app, ctx);
    if open(ctx, &facts) { HEAD_H + TOP + facts.rows() as f32 * ROW + BOTTOM } else { HEAD_H }
}

/// Where a fold or open pressed by hand is kept, with whether anything was selected then: it holds
/// until that changes, when the properties follow the selection again.
fn open_id() -> egui::Id {
    wid("props-open", ())
}

fn open(ctx: &egui::Context, facts: &Facts) -> bool {
    let any = facts.selected > 0;
    match ctx.data(|d| d.get_temp::<(bool, bool)>(open_id())) {
        Some((then, open)) if then == any => open,
        Some(_) => {
            // The selection came or went since: the properties follow it again.
            ctx.data_mut(|d| d.remove::<(bool, bool)>(open_id()));
            any
        }
        None => any,
    }
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let facts = facts(app, &ctx);
    let is_open = open(&ctx, &facts);
    line_above(ui, r, t.border);
    let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
    column_head(ui, head, tw!("Properties"), &facts.head, 1, &t);
    let (icon, tip) = if is_open { ("chevron-down", tw!("Fold the properties")) } else { ("chevron-right", tw!("Show the properties")) };
    if head_button(ui, wid("props-fold", ()), head, 0, icon, tip, &t).clicked() {
        ctx.data_mut(|d| d.insert_temp(open_id(), (facts.selected > 0, !is_open)));
    }
    if !is_open {
        return;
    }
    let x0 = r.left() + PAD + LABEL_W + 6.0;
    let right = r.right() - PAD;
    let mut cy = head.bottom() + TOP + ROW / 2.0;
    let mut row = |ui: &Ui, label: &str, value: &str| {
        text(ui, pos2(r.left() + PAD, cy), Align2::LEFT_CENTER, label, font(12.0), t.text_dim);
        text_to(ui, pos2(x0, cy), right, value, mono(11.5), t.text);
        cy += ROW;
    };
    if facts.selected == 0 {
        text_to(ui, pos2(r.left() + PAD, cy), right, tw!("Select something to see its size and where it is"), font(12.0), t.text_dim);
        return;
    }
    if let Some(s) = &facts.size {
        row(ui, tw!("Size"), s);
    }
    if let Some(s) = &facts.area {
        row(ui, tw!("Area"), s);
    }
    if let Some(s) = &facts.position {
        row(ui, tw!("Position"), s);
    }
    if facts.selected < 2 {
        return;
    }
    text(ui, pos2(r.left() + PAD, cy), Align2::LEFT_CENTER, tw!("Alignment"), font(12.0), t.text_dim);
    let buttons = [
        ("left", "align-start-vertical", tw!("Align Left"), json!({ "horizontal": "left" })),
        ("center", "align-center-vertical", tw!("Align Centres"), json!({ "horizontal": "center" })),
        ("right", "align-end-vertical", tw!("Align Right"), json!({ "horizontal": "right" })),
        ("top", "align-start-horizontal", tw!("Align Tops"), json!({ "vertical": "top" })),
        ("middle", "align-center-horizontal", tw!("Align Middles"), json!({ "vertical": "center" })),
        ("bottom", "align-end-horizontal", tw!("Align Bottoms"), json!({ "vertical": "bottom" })),
    ];
    let mut asked = None;
    for (i, (key, icon, tip, params)) in buttons.into_iter().enumerate() {
        let b = Rect::from_min_size(pos2(x0 + i as f32 * (BUTTON + BUTTON_GAP), cy - BUTTON / 2.0), vec2(BUTTON, BUTTON));
        let resp = ui.interact(b, wid("props-align", key), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(b, CornerRadius::same(5), t.hover);
        }
        crate::icons::paint(ui, icon, b.shrink(4.0), t.icon);
        if resp.on_hover_text(tip).clicked() {
            asked = Some(params);
        }
    }
    if let Some(params) = asked
        && let Err(e) = app.run("object.align", params)
    {
        app.status_error(e);
    }
}

/// What is selected, on which rows, how big and where.
fn facts(app: &mut VectorcraftApp, ctx: &egui::Context) -> Facts {
    let size_box = app.selection_box();
    let bounds = app.selection_bounds();
    let area = area(app, ctx);
    let Some(st) = app.session.active() else { return Facts::default() };
    let ids = &st.selection.objects;
    if ids.is_empty() {
        return Facts { head: tw!("No selection").to_string(), ..Facts::default() };
    }
    let nodes: Vec<&vectorcraft_doc::Node> = ids.iter().filter_map(|id| st.doc.node(*id)).collect();
    let n = ids.len().to_string();
    let what = if !nodes.is_empty() && nodes.iter().all(|n| vectorcraft_doc::legend::is_line(n)) {
        crate::i18n::fmt(tw!("{n} lines"), &[("n", &n)])
    } else {
        crate::i18n::fmt(tw!("{n} objects"), &[("n", &n)])
    };
    let mut rows: Vec<vectorcraft_doc::NodeId> = ids.iter().filter_map(|id| st.doc.layer_of(*id)).collect();
    rows.sort_unstable_by_key(|id| id.0);
    rows.dedup();
    let on = match rows.as_slice() {
        [one] => st.doc.node(*one).map(|l| l.display_name()).unwrap_or_default(),
        [] => String::new(),
        many => crate::i18n::fmt(tw!("{n} rows"), &[("n", &many.len().to_string())]),
    };
    let head = if on.is_empty() { what } else { format!("{what} · {on}") };
    let scale = st.doc.scale.as_ref();
    let mm = vectorcraft_doc::Unit::Millimeters;
    let size = size_box.map(|b| match scale {
        Some(cfg) => {
            let m = cfg.scale.meters_per_point();
            format!("{:.2} × {:.2} m", b.rect.width() * m, b.rect.height() * m)
        }
        None => format!("{} × {} mm", trim(mm.from_pt(b.rect.width())), trim(mm.from_pt(b.rect.height()))),
    });
    let position = bounds.map(|b| {
        let c = b.center();
        match scale {
            Some(cfg) => {
                let (east, north) = cfg.canvas_to_world(c);
                let (x, y) = match cfg.crs.order {
                    CoordOrder::EastNorth => (east, north),
                    CoordOrder::NorthEast => (north, east),
                };
                format!("X {} · Y {}", super::status::grouped(x, 2), super::status::grouped(y, 2))
            }
            None => format!("X {} · Y {} mm", super::status::grouped(mm.from_pt(c.x), 1), super::status::grouped(mm.from_pt(c.y), 1)),
        }
    });
    Facts { selected: ids.len(), head, size, area, position }
}

/// Paper millimetres to a tenth, without ".0": "10", "12.5".
fn trim(v: f64) -> String {
    let s = format!("{v:.1}");
    s.strip_suffix(".0").unwrap_or(&s).to_string()
}

/// A single closed shape's area and perimeter at the drawing's scale, worked out once per revision.
fn area(app: &mut VectorcraftApp, ctx: &egui::Context) -> Option<String> {
    let st = app.session.active()?;
    st.doc.scale.as_ref()?;
    let [id] = st.selection.objects.as_slice() else { return None };
    let id = *id;
    st.doc.node(id)?.path_data().filter(|p| p.is_closed())?;
    let key = (st.uid, st.revision, id.0);
    let cache = wid("props-area", ());
    if let Some((was, said)) = ctx.data(|d| d.get_temp::<((u64, u64, u64), Option<String>)>(cache))
        && was == key
    {
        return said;
    }
    let said = app.session.execute("trowel.polygonStats", &json!({ "id": id.0 })).ok().and_then(|v| {
        let area = v["formatted"]["area"].as_str()?.to_string();
        let perimeter = v["formatted"]["perimeter"].as_str()?.to_string();
        Some(crate::i18n::fmt(tw!("{area} · perimeter {perimeter}"), &[("area", &area), ("perimeter", &perimeter)]))
    });
    ctx.data_mut(|d| d.insert_temp(cache, (key, said.clone())));
    said
}
