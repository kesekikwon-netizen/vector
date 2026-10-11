//! The Control bar. With a drawing tool: the tool, the legend entry it draws with (for now the
//! current layer and the next line's weight) and the options later jobs bring; with any other tool,
//! the classic Control bar's controls. Snap and its tolerance stay at the right.

use serde_json::json;

use super::*;

/// The tools the Control bar shows the drawing layout for.
const DRAWING_TOOLS: &[&str] = &[
    "pen",
    "curvature",
    "addAnchor",
    "deleteAnchor",
    "anchorPoint",
    "pencil",
    "smooth",
    "lineSegment",
    "arc",
    "rectangle",
    "roundedRectangle",
    "ellipse",
    "polygon",
    "star",
    "paintbrush",
    "blobBrush",
];

const H: f32 = 28.0;
const GAP: f32 = 10.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(CONTROL).exact_size(CONTROL_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let left = snap_controls(app, ui, r, &t);
        let room = Rect::from_min_max(pos2(r.left() + 12.0, r.top()), pos2(left - GAP, r.bottom()));
        if DRAWING_TOOLS.contains(&app.session.tool_id()) && app.session.active().is_some() {
            drawing(app, ui, room, &t);
        } else {
            let mut row = ui.new_child(egui::UiBuilder::new().max_rect(room).layout(egui::Layout::left_to_right(egui::Align::Center)));
            row.set_clip_rect(room);
            crate::chrome::control_bar_contents(app, &mut row);
        }
    });
}

/// Snap to Point and its tolerance, at the right end of `r`. Returns where they begin.
fn snap_controls(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect, t: &Tokens) -> f32 {
    let cy = r.center().y;
    let field = Rect::from_min_size(pos2(r.right() - 12.0 - 56.0, cy - H / 2.0), vec2(56.0, H));
    let mut tol = ui.new_child(egui::UiBuilder::new().max_rect(field).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let v = app.session.prefs.snap_to_point_tolerance;
    if let Some(nv) = crate::widgets::range_field(&mut tol, wid("ctl-tolerance", ()), v, 1.0..=8.0, " px", 0, field.width()) {
        app.run("prefs.set", json!({"key": "snapToPointTolerance", "value": nv})).ok();
    }
    let label = text(ui, pos2(field.left() - GAP, cy), Align2::RIGHT_CENTER, tw!("Tolerance"), font(12.0), t.text_dim);
    let on = app.ui.view.snap_to_point;
    let w = width_of(ui, tw!("Snap"), font(12.0)) + 20.0;
    let toggle = Rect::from_min_size(pos2(label.left() - GAP - w, cy - H / 2.0), vec2(w, H));
    let resp = ui.interact(toggle, wid("ctl-snap", ()), Sense::click());
    let fill = if on {
        t.row_selected
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    field_box(ui, toggle, fill, if on { t.ring } else { t.input_border });
    text(ui, toggle.center(), Align2::CENTER_CENTER, tw!("Snap"), if on { bold(12.0) } else { font(12.0) }, t.text);
    if resp.on_hover_text(tl!("Snap to Point")).clicked() {
        crate::menus::invoke(app, "view.snapToPoint", json!({}));
    }
    toggle.left()
}

/// The drawing tools' layout in `room`.
fn drawing(app: &mut VectorcraftApp, ui: &mut Ui, room: Rect, t: &Tokens) {
    let cy = room.center().y;
    let p = ui.painter().with_clip_rect(room);
    let mut x = room.left();
    let label = |s: &str, color: Color32, x: &mut f32| {
        let r = p.text(pos2(*x, cy), Align2::LEFT_CENTER, s, font(12.0), color);
        *x = r.right() + GAP;
    };
    label(&format!("{} ›", tool_name(app.session.tool_id())), t.text_dim, &mut x);
    label(tw!("Legend"), t.text_dim, &mut x);
    x = layer_field(app, ui, pos2(x, cy), t) + GAP;
    // What later jobs bring: estimated lines, closing, pen pressure.
    x = toggle_soon(ui, wid("ctl-estimate", ()), pos2(x, cy), tw!("Estimated (Dashed)"), "J014", t) + GAP;
    x = divider(ui, x, cy, t);
    let mut x2 = x;
    label(tw!("When Closed"), t.text_dim, &mut x2);
    x = dropdown_soon(ui, wid("ctl-closed", ()), pos2(x2, cy), tw!("Area and Perimeter"), "J009", t) + GAP;
    x = divider(ui, x, cy, t);
    let mut x2 = x;
    label(tw!("Pen Pressure"), t.text_dim, &mut x2);
    x = segments_soon(ui, pos2(x2, cy), &[tw!("Same Weight"), tw!("Follow Pressure")], "J016", t) + GAP;
    x = divider(ui, x, cy, t);
    p.text(pos2(x, cy), Align2::LEFT_CENTER, tw!("Weight, hatch and line type: in the Drawing Palette at the right"), font(12.0), t.text_disabled);
}

/// A one-point upright line between groups; returns where the next one starts.
fn divider(ui: &Ui, x: f32, cy: f32, t: &Tokens) -> f32 {
    ui.painter().vline(x + 0.5, (cy - 10.0)..=(cy + 10.0), Stroke::new(1.0, t.border));
    x + 1.0 + GAP
}

/// The legend field: the current layer's colour, its name and the next line's weight; a click
/// lists the layers to draw on. Returns where it ends.
fn layer_field(app: &mut VectorcraftApp, ui: &mut Ui, at: Pos2, t: &Tokens) -> f32 {
    let Some(st) = app.session.active() else { return at.x };
    let current = st.current_layer();
    let name = current.and_then(|id| st.doc.node(id)).map(|n| n.display_name()).unwrap_or_default();
    let colour = current.map(|id| crate::panels::layers::layer_colour(&st.doc, id));
    let stroke = crate::panels::current_stroke(app);
    let mixed = crate::panels::stroke_mixed(app, ui.ctx());
    let weight = crate::panels::stroke::shown_weight(app, stroke.as_ref(), &mixed).map(|w| app.session.stroke_unit().format(w)).unwrap_or_default();
    let name_w = width_of(ui, &name, font(12.0)).min(220.0);
    let weight_w = width_of(ui, &weight, mono(12.0));
    let w = 8.0 + 20.0 + 6.0 + name_w + 8.0 + weight_w + 8.0 + 10.0 + 8.0;
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(w, H));
    let resp = ui.interact(r, wid("ctl-layer", ()), Sense::click());
    field_box(ui, r, if resp.hovered() { t.panel } else { t.input }, t.input_border);
    let patch = Rect::from_min_size(pos2(r.left() + 8.0, at.y - 6.0), vec2(20.0, 12.0));
    if let Some(c) = colour {
        ui.painter().rect(patch, 0.0, c.gamma_multiply(0.25), Stroke::new(1.5, c), StrokeKind::Inside);
    }
    let p = ui.painter().with_clip_rect(r);
    let n = p.text(pos2(patch.right() + 6.0, at.y), Align2::LEFT_CENTER, &name, font(12.0), t.text);
    p.text(pos2(n.right().min(patch.right() + 6.0 + name_w) + 8.0, at.y), Align2::LEFT_CENTER, &weight, mono(12.0), t.text_dim);
    crate::icons::paint(ui, "chevron-down", Rect::from_center_size(pos2(r.right() - 13.0, at.y), vec2(10.0, 10.0)), t.text_dim);
    let resp = resp.on_hover_text(tw!("The layer and weight the next line is drawn with"));
    let mut pick = None;
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        if let Some(st) = app.session.active() {
            for layer in st.doc.layers.iter().rev() {
                let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
                let rr = ui.interact(row, wid("ctl-layer-row", layer.id.0), Sense::click());
                if rr.hovered() {
                    ui.painter().rect_filled(row, CornerRadius::same(4), t.hover);
                }
                let c = crate::panels::layers::layer_colour(&st.doc, layer.id);
                let patch = Rect::from_min_size(pos2(row.left() + 6.0, row.center().y - 6.0), vec2(20.0, 12.0));
                ui.painter().rect(patch, 0.0, c.gamma_multiply(0.25), Stroke::new(1.5, c), StrokeKind::Inside);
                let f = if Some(layer.id) == current { bold(12.0) } else { font(12.0) };
                ui.painter().text(pos2(patch.right() + 8.0, row.center().y), Align2::LEFT_CENTER, layer.display_name(), f, t.text);
                if rr.clicked() {
                    pick = Some(layer.id);
                    ui.close();
                }
            }
        }
    });
    if let Some(id) = pick
        && let Err(e) = app.run("layer.setCurrent", json!({ "id": id.0 }))
    {
        app.status(e);
    }
    r.right()
}

/// A toggle a later job brings, labelled `label`, at `at` (its left centre). Returns where it ends.
fn toggle_soon(ui: &Ui, id: egui::Id, at: Pos2, label: &str, job: &str, t: &Tokens) -> f32 {
    let w = width_of(ui, label, font(12.0)) + 20.0;
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(w, H));
    field_box(ui, r, t.input, t.input_border);
    text(ui, r.center(), Align2::CENTER_CENTER, label, font(12.0), t.text_disabled);
    soon_widget(ui, id, r, job);
    r.right()
}

/// A drop-down a later job brings, showing `value`. Returns where it ends.
fn dropdown_soon(ui: &Ui, id: egui::Id, at: Pos2, value: &str, job: &str, t: &Tokens) -> f32 {
    let w = width_of(ui, value, font(12.0)) + 8.0 + 6.0 + 10.0 + 8.0;
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(w, H));
    field_box(ui, r, t.input, t.input_border);
    text(ui, pos2(r.left() + 8.0, at.y), Align2::LEFT_CENTER, value, font(12.0), t.text_disabled);
    crate::icons::paint(ui, "chevron-down", Rect::from_center_size(pos2(r.right() - 13.0, at.y), vec2(10.0, 10.0)), t.text_disabled);
    soon_widget(ui, id, r, job);
    r.right()
}

/// A segmented choice a later job brings, its first segment on. Returns where it ends.
fn segments_soon(ui: &Ui, at: Pos2, labels: &[&str], job: &str, t: &Tokens) -> f32 {
    let widths: Vec<f32> = labels.iter().map(|l| width_of(ui, l, font(12.0)) + 18.0).collect();
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(widths.iter().sum(), H));
    field_box(ui, r, t.input, t.input_border);
    let mut x = r.left();
    for (i, (label, w)) in labels.iter().zip(&widths).enumerate() {
        let seg = Rect::from_min_size(pos2(x, r.top()), vec2(*w, H));
        if i == 0 {
            ui.painter().rect(seg.shrink(1.0), CornerRadius::same(7), t.row_selected, Stroke::new(1.0, t.text_disabled), StrokeKind::Inside);
        } else {
            ui.painter().vline(seg.left() + 0.5, seg.y_range(), Stroke::new(1.0, t.border));
        }
        text(ui, seg.center(), Align2::CENTER_CENTER, *label, font(12.0), t.text_disabled);
        soon_widget(ui, wid("ctl-pressure", i), seg, job);
        x += w;
    }
    r.right()
}
