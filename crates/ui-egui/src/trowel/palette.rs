//! The drawing palette, always at the top of the right column (design 2.1 §3): the weight in
//! points of the selected lines, or of the next line drawn, the hatch and the line type, and Cut
//! Face. The weights and Solid run now; the hatches, the other line types and the legend buttons
//! come with J014 and J005, and adding a weight with J018.

use serde_json::json;

use super::*;

/// The palette's height: its head, three rows of chips and the buttons under them.
pub(super) const HEIGHT: f32 = 166.0;

/// The weights in points (design 2.1 §7) and how thick their sample line is drawn.
const WEIGHTS: [(&str, f64, f32); 6] =
    [("0.1", 0.1, 0.5), ("0.2", 0.2, 0.8), ("0.3", 0.3, 1.0), ("0.5", 0.5, 1.4), ("0.8", 0.8, 2.0), ("1.0", 1.0, 2.6)];

const ROW_H: f32 = 31.0;
const CHIP_H: f32 = 27.0;
/// Where the chips start, after the row's label.
const CHIPS_X: f32 = 14.0 + 28.0 + 4.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);
    let selected = app.session.active().is_some_and(|d| !d.selection.objects.is_empty());
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, tw!("Drawing Palette"), if selected { tw!("Selected lines · pt") } else { tw!("Next line · pt") }, 1, &t);
    if more_button(ui, wid("palette-more", ()), head, 0, tw!("Stroke options"), &t).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "stroke" }));
    }
    let stroke = crate::panels::current_stroke(app);
    let mixed = crate::panels::stroke_mixed(app, ui.ctx());
    let weight = crate::panels::stroke::shown_weight(app, stroke.as_ref(), &mixed);
    let solid = !mixed.dash && stroke.as_ref().is_none_or(|s| s.dash.is_none());
    let row = |i: usize| head.bottom() + 2.0 + ROW_H * (i as f32 + 0.5);
    let mut set = None;
    // Weight.
    let cy = row(0);
    text(ui, pos2(r.left() + 14.0, cy), Align2::LEFT_CENTER, tw!("Weight"), font(11.0), t.text_dim);
    let mut x = r.left() + CHIPS_X;
    for (label, w, sample) in WEIGHTS {
        let chip_r = Rect::from_min_size(pos2(x, cy - CHIP_H / 2.0), vec2(30.0, CHIP_H));
        let resp = ui.interact(chip_r, wid("weight", label), Sense::click());
        let on = weight.is_some_and(|v| (v - w).abs() < 1e-6);
        chip(ui, chip_r, on, resp.hovered(), &t);
        ui.painter().rect_filled(Rect::from_center_size(pos2(chip_r.center().x, cy - 5.0), vec2(20.0, sample)), 0.0, t.text);
        text(ui, pos2(chip_r.center().x, cy + 5.0), Align2::CENTER_CENTER, label, mono(10.0), t.text);
        if resp.on_hover_text(format!("{label} pt")).clicked() {
            set = Some(json!({ "weight": w }));
        }
        x = chip_r.right() + 4.0;
    }
    let add = Rect::from_min_size(pos2(x, cy - CHIP_H / 2.0), vec2(24.0, CHIP_H));
    chip(ui, add, false, false, &t);
    crate::icons::paint(ui, "plus", Rect::from_center_size(add.center(), vec2(12.0, 12.0)), t.text_disabled);
    soon_widget(ui, wid("weight-add", ()), add, tw!("Add a Weight"), "J018");
    // Hatch: none until J014 brings them.
    let cy = row(1);
    text(ui, pos2(r.left() + 14.0, cy), Align2::LEFT_CENTER, tw!("Hatch"), font(11.0), t.text_dim);
    let mut x = r.left() + CHIPS_X;
    for (key, label) in [("45", "45°"), ("135", "135°"), ("alt", tw!("Alternate")), ("none", tw!("None"))] {
        let sample = key != "none";
        let w = 6.0 + if sample { 16.0 + 4.0 } else { 0.0 } + width_of(ui, label, font(11.0)) + 6.0;
        let chip_r = Rect::from_min_size(pos2(x, cy - CHIP_H / 2.0), vec2(w, CHIP_H));
        let on = key == "none";
        chip(ui, chip_r, on, false, &t);
        let color = if on { t.text } else { t.text_disabled };
        let mut tx = chip_r.left() + 6.0;
        if sample {
            hatch_sample(ui, Rect::from_min_size(pos2(tx, cy - 6.0), vec2(16.0, 12.0)), key, color);
            tx += 16.0 + 4.0;
        }
        text(ui, pos2(tx, cy), Align2::LEFT_CENTER, label, font(11.0), color);
        if on {
            ui.interact(chip_r, wid("hatch", key), Sense::hover());
        } else {
            soon_widget(ui, wid("hatch", key), chip_r, &format!("{} {label}", tw!("Hatch")), "J014");
        }
        x = chip_r.right() + 4.0;
    }
    // Line type: Solid now, the others with J014.
    let cy = row(2);
    text(ui, pos2(r.left() + 14.0, cy), Align2::LEFT_CENTER, tw!("Line"), font(11.0), t.text_dim);
    let kinds = [("solid", tw!("Solid")), ("dashed", tw!("Dashed")), ("dotted", tw!("Dotted")), ("dashdot", tw!("Dash-dot"))];
    let widths: Vec<f32> = kinds.iter().map(|(_, l)| width_of(ui, l, font(12.0)) + 18.0).collect();
    let seg = Rect::from_min_size(pos2(r.left() + CHIPS_X, cy - 13.0), vec2(widths.iter().sum(), 26.0));
    field_box(ui, seg, t.input, t.input_border);
    let mut x = seg.left();
    for (i, ((key, label), w)) in kinds.iter().zip(&widths).enumerate() {
        let part = Rect::from_min_size(pos2(x, seg.top()), vec2(*w, seg.height()));
        if i > 0 {
            ui.painter().vline(part.left() + 0.5, part.y_range().shrink(1.0), Stroke::new(1.0, t.border));
        }
        let live = *key == "solid";
        if live && solid {
            // Solid comes first: its outer corners are the box's.
            ui.painter().rect(part, CornerRadius { nw: 8, sw: 8, ne: 0, se: 0 }, t.row_selected, Stroke::new(1.0, t.text), StrokeKind::Inside);
        }
        text(ui, part.center(), Align2::CENTER_CENTER, *label, font(12.0), if live { t.text } else { t.text_disabled });
        if live {
            let resp = ui.interact(part, wid("dash", key), Sense::click());
            if resp.on_hover_text(tw!("No dashes")).clicked() {
                set = Some(json!({ "dash": null }));
            }
        } else {
            soon_widget(ui, wid("dash", key), part, label, "J014");
        }
        x += w;
    }
    // Cut Face, and the legend's own buttons.
    let cy = head.bottom() + 2.0 + 3.0 * ROW_H + 4.0 + 13.0;
    let x = button_soon(ui, wid("palette-cut", ()), pos2(r.left() + 14.0, cy), tw!("Cut Face"), true, "J014", &t) + 4.0;
    let x = button_soon(ui, wid("palette-to-legend", ()), pos2(x, cy), tw!("Add to Legend"), false, "J005", &t) + 4.0;
    button_soon(ui, wid("palette-from-legend", ()), pos2(x, cy), tw!("Back to the Legend"), false, "J005", &t);
    if let Some(p) = set {
        crate::panels::stroke::set(app, p);
    }
}

/// A chip: on, two points of ink around an oat fill; else a one-point edge.
fn chip(ui: &Ui, r: Rect, on: bool, hovered: bool, t: &Tokens) {
    let (fill, edge) = match (on, hovered) {
        (true, _) => (t.row_selected, Stroke::new(2.0, t.text)),
        (false, true) => (t.hover, Stroke::new(1.0, t.input_border)),
        (false, false) => (t.input, Stroke::new(1.0, t.input_border)),
    };
    ui.painter().rect(r, CornerRadius::same(6), fill, edge, StrokeKind::Inside);
}

/// A hatch's sample in `r`: lines rising to the right every 4 points (45°), falling (135°), or
/// the two halves alternating.
fn hatch_sample(ui: &Ui, r: Rect, key: &str, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    let lines = |area: Rect, rising: bool| {
        let p = ui.painter().with_clip_rect(area);
        let mut c = -area.height();
        while c < area.width() {
            let x = area.left() + c;
            let (a, b) = if rising {
                (pos2(x, area.bottom()), pos2(x + area.height(), area.top()))
            } else {
                (pos2(x, area.top()), pos2(x + area.height(), area.bottom()))
            };
            p.line_segment([a, b], stroke);
            c += 4.0;
        }
    };
    match key {
        "45" => lines(r, true),
        "135" => lines(r, false),
        _ => {
            let (left, right) = r.split_left_right_at_fraction(0.5);
            lines(left, true);
            lines(right, false);
            ui.painter().vline(r.center().x, r.y_range(), stroke);
        }
    }
    ui.painter().rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
}
