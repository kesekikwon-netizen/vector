//! The drawing palette, always at the top of the right column (design 2.1 §3): the weight in
//! points of the selected lines, or of the next line drawn, the hatch and the line type, Cut Face
//! and the legend's two buttons: Add to Legend makes a row of the look and opens its editor, Back
//! to the Legend puts the selected lines (or the next line) back to their row's look. The hatches
//! and Cut Face come with J014, adding a weight with J018.

use std::collections::HashSet;

use serde_json::json;
use vectorcraft_doc::{LineType, NodeId};

use super::legend::{Card, open_card};
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
    let add_resp = ui.interact(add, wid("weight-add", ()), Sense::click());
    chip(ui, add, false, add_resp.hovered(), &t);
    crate::icons::paint(ui, "plus", Rect::from_center_size(add.center(), vec2(12.0, 12.0)), t.text);
    let add_resp = add_resp.on_hover_text("클릭: 양식 템플릿 관리 · 우클릭: 현장 세부 굵기(0.15 / 0.4 / 0.6 pt) 선택");
    let mut picked_preset: Option<(f64, Option<LineType>)> = None;
    egui::Popup::context_menu(&add_resp).show(|ui| {
        ui.label("현장 실무 세부 선 굵기 (6p·7p 규격)");
        ui.separator();
        for (w, desc, dash_opt) in [
            (0.15, "0.15 pt · 석곽-유물 깨진면", Some(LineType::Solid)),
            (0.2, "0.2 pt · 석곽·유물 내선 / 단면빗금", Some(LineType::Solid)),
            (0.3, "0.3 pt · 내선(바닥) / 유물외곽 / 해발선", Some(LineType::Solid)),
            (0.4, "0.4 pt · 이중굴광시-가운데선 (파선)", Some(LineType::Dashed)),
            (0.5, "0.5 pt · 석곽-평면외곽선 / 해발 0.5단위", Some(LineType::Solid)),
            (0.6, "0.6 pt · 평면외곽선", Some(LineType::Solid)),
            (0.8, "0.8 pt · 단면선", Some(LineType::Solid)),
            (1.0, "1.0 pt · 조사구역 경계", None),
        ] {
            if ui.button(desc).clicked() {
                picked_preset = Some((w, dash_opt));
                ui.close();
            }
        }
        ui.separator();
        if ui.button("도면 양식 템플릿 관리 (J018)…").clicked() {
            super::style_dialog::toggle_dialog(ui.ctx());
            ui.close();
        }
    });
    if let Some((w, dash_opt)) = picked_preset {
        set = Some(match dash_opt {
            Some(kind) => json!({ "weight": w, "dash": kind.dash_pt() }),
            None => json!({ "weight": w }),
        });
    } else if add_resp.clicked() {
        super::style_dialog::toggle_dialog(ui.ctx());
    }
    // Hatch: 45°, 135°, Alternate, None.
    let cy = row(1);
    text(ui, pos2(r.left() + 14.0, cy), Align2::LEFT_CENTER, tw!("Hatch"), font(11.0), t.text_dim);
    let mut x = r.left() + CHIPS_X;

    let hatch_info = app.session.execute("hatch.get", &json!({})).ok();
    let current_dir = hatch_info.as_ref().and_then(|v| v["direction"].as_str()).unwrap_or("none");
    let has_conflict = hatch_info.as_ref().and_then(|v| v["has_conflict"].as_bool()).unwrap_or(false);

    for (key, label) in [("45", "45°"), ("135", "135°"), ("alt", tw!("Alternate")), ("none", tw!("None"))] {
        let sample = key != "none";
        let is_alt = key == "alt";
        let conflict_extra = if is_alt && has_conflict { 14.0 } else { 0.0 };
        let w = 6.0 + if sample { 16.0 + 4.0 } else { 0.0 } + width_of(ui, label, font(11.0)) + conflict_extra + 6.0;
        let chip_r = Rect::from_min_size(pos2(x, cy - CHIP_H / 2.0), vec2(w, CHIP_H));
        let on = current_dir == key;
        let resp = ui.interact(chip_r, wid("hatch", key), Sense::click());
        chip(ui, chip_r, on, resp.hovered(), &t);
        let color = if on { t.text } else { t.text_dim };
        let mut tx = chip_r.left() + 6.0;
        if sample {
            hatch_sample(ui, Rect::from_min_size(pos2(tx, cy - 6.0), vec2(16.0, 12.0)), key, color);
            tx += 16.0 + 4.0;
        }
        text(ui, pos2(tx, cy), Align2::LEFT_CENTER, label, font(11.0), color);
        if is_alt && has_conflict {
            let badge_r = Rect::from_center_size(pos2(chip_r.right() - 10.0, cy), vec2(14.0, 14.0));
            ui.painter().rect(badge_r, CornerRadius::same(4), t.caution_wash, Stroke::new(1.0, t.caution_edge), StrokeKind::Inside);
            text(ui, badge_r.center(), Align2::CENTER_CENTER, "!", font(10.0), t.caution);
        }

        let hover_text = if is_alt {
            if has_conflict {
                tw!("Odd-cycle conflict (3+ mutually adjacent faces)! Click to flip direction")
            } else {
                tw!("Alternating Hatch (45°/135°). Click to flip direction")
            }
        } else {
            label
        };

        if resp.on_hover_text(hover_text).clicked() {
            if is_alt && on {
                app.run("hatch.flip", json!({})).ok();
            } else {
                app.run("hatch.set", json!({ "direction": key })).ok();
            }
        }
        x = chip_r.right() + 4.0;
    }
    // Line type: the first selected line's, or the next line's; none when the selected lines'
    // dashes differ.
    let cy = row(2);
    text(ui, pos2(r.left() + 14.0, cy), Align2::LEFT_CENTER, tw!("Line"), font(11.0), t.text_dim);
    let shown = (!mixed.dash).then(|| stroke.as_ref().map_or(LineType::Solid, LineType::of));
    let kinds = line_types();
    let parts: Vec<Seg> =
        kinds.iter().map(|(key, label, kind)| Seg { key, label, on: shown.as_ref().is_some_and(|s| s.same(kind)), soon: None }).collect();
    let look = SegLook { h: 26.0, pad: 9.0, font: font(12.0) };
    if let (Some(key), _) = segments(ui, "dash", pos2(r.left() + CHIPS_X, cy), &look, &parts, &t)
        && let Some((_, _, kind)) = kinds.iter().find(|(k, _, _)| *k == key)
    {
        set = Some(json!({ "dash": kind.dash_pt() }));
    }
    // Cut Face, and the legend's own buttons.
    let cy = head.bottom() + 2.0 + 3.0 * ROW_H + 4.0 + 13.0;
    let open = app.session.active().is_some();
    let cut_btn = small_button(ui, wid("palette-cut", ()), pos2(r.left() + 14.0, cy), 26.0, tw!("Cut Face"), open, &t)
        .on_hover_text(tw!("Cut border (0.8 pt) and alternating hatch"));
    if cut_btn.clicked() {
        app.run("hatch.cutFace", json!({})).ok();
    }
    let x = cut_btn.rect.right() + 4.0;
    let add = small_button(ui, wid("palette-to-legend", ()), pos2(x, cy), 26.0, tw!("Add to Legend"), open, &t)
        .on_hover_text(tw!("Make a legend row of this look and put the selected lines on it"));
    let back = small_button(ui, wid("palette-from-legend", ()), pos2(add.rect.right() + 4.0, cy), 26.0, tw!("Back to the Legend"), open, &t)
        .on_hover_text(tw!("Give the selected lines, or the next line, their row's look"));
    if let Some(p) = set {
        crate::panels::stroke::set(app, p);
    }
    let done = if add.clicked() {
        add_row(app, ui.ctx())
    } else if back.clicked() {
        back_to_legend(app)
    } else {
        Ok(())
    };
    if let Err(e) = done {
        app.status(e);
    }
}

/// Make a legend row of the selected line's look (or the next line's) named "Legend Row n", the
/// selection moving onto it, and open the row's editor.
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

/// A chip: on, two points of ink around an oat fill; else a one-point edge.
fn chip(ui: &Ui, r: Rect, on: bool, hovered: bool, t: &Tokens) {
    let (fill, edge) = match (on, hovered) {
        (true, _) => (t.row_selected, Stroke::new(2.0, t.text)),
        (false, true) => (t.hover, Stroke::new(1.0, t.input_border)),
        (false, false) => (t.input, Stroke::new(1.0, t.input_border)),
    };
    ui.painter().rect(r, CornerRadius::same(6), fill, edge, StrokeKind::Inside);
}

/// The chip of something a later job brings: on the wash, like its other controls.
#[allow(dead_code)]
fn chip_soon(ui: &Ui, r: Rect, t: &Tokens) {
    ui.painter().rect(r, CornerRadius::same(6), t.panel_darker, Stroke::new(1.0, t.input_border), StrokeKind::Inside);
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
