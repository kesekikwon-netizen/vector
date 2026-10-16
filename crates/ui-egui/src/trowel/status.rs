//! The status bar (design 3.0 §5 ⑩), 28 points: at the left what is going on (a save running, a
//! note, the last message for a while, else where the next line goes when that isn't the current
//! layer and how to use the tool in hand); at the right the grid and Snap, each a click to turn on
//! or off with its ▾ for the settings, where the pointer is (a click copies it), the drawing's scale
//! and coordinate system, and the zoom. It is the one place for those (principle 5).

use serde_json::json;

use super::*;

/// The grid spacings the grid's list offers, in millimetres on paper.
const GRID_MM: [f64; 3] = [10.0, 20.0, 50.0];
/// The grid colours its list offers: label and colour.
fn grid_colours() -> [(&'static str, &'static str); 5] {
    [
        (tw!("Light grey"), "#CCCCCC"),
        (tw!("Ochre"), "#C8BFA8"),
        (tw!("Sky"), "#6BAED6"),
        (tw!("Graph green"), "#74C476"),
        (tw!("Dark grey"), "#777777"),
    ]
}
/// The scales the scale's list offers (design 3.0 §5 ⑩), as 1:N.
const SCALES: [f64; 9] = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 200.0];
/// The zooms the zoom's list offers, in percent.
const ZOOMS: [f64; 4] = [50.0, 100.0, 200.0, 400.0];
/// The width of a cell's ▾: a click there opens its list.
const ARROW_W: f32 = 16.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom(STATUS).exact_size(STATUS_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_above(ui, r, t.border);
        // The start screen: what to do there, and the version.
        if crate::menus::home_showing(app) {
            let ver = cell(ui, r, r.right(), Mark::None, &[("Trowel 0.2".into(), font(12.0), t.text_dim)], false, &t);
            let room = Rect::from_min_max(pos2(r.left() + 16.0, r.top()), pos2(ver.left() - 12.0, r.bottom()));
            let msg = message(app, ui.ctx()).unwrap_or_else(|| "「이어서 그리기」를 누르면 하던 도면이 그대로 열립니다.".to_string());
            ui.painter().with_clip_rect(room).text(pos2(room.left(), r.center().y), Align2::LEFT_CENTER, msg, font(12.0), t.text_dim);
            return;
        }
        let has_doc = app.session.active().is_some();
        let mut left = r.right();
        if has_doc {
            left = zoom_cell(app, ui, r, left, &t).left();
            left = scale_cell(app, ui, r, left, &t).left();
            left = position_cell(app, ui, r, left, &t).left();
            left = snap_cell(app, ui, r, left, &t).left();
            left = grid_cell(app, ui, r, left, &t).left();
        }
        // What is going on, in the room left: a message, else where lines go and how to use the
        // tool in hand.
        let room = Rect::from_min_max(pos2(r.left() + 12.0, r.top()), pos2(left - 12.0, r.bottom()));
        let p = ui.painter().with_clip_rect(room);
        let cy = r.center().y;
        match message(app, ui.ctx()) {
            Some(m) => {
                p.text(pos2(room.left(), cy), Align2::LEFT_CENTER, m, font(12.0), t.text);
            }
            None if has_doc => {
                let mut x = room.left();
                if let Some(note) = app.session.active().and_then(|st| draw_target(st).note) {
                    x = p.text(pos2(x, cy), Align2::LEFT_CENTER, note, font(12.0), t.clay_text).right() + 12.0;
                }
                let hint = p.layout_job(hint_job(app, 12.0, t.text));
                p.galley(pos2(x, cy - hint.size().y / 2.0), hint, t.text);
            }
            None => {}
        }
    });
}

/// A save or export running; else a note ([`soon_note`]); else the last message, for
/// [`MESSAGE_SECS`] after it is put there (the same one again too), so the hint comes back.
fn message(app: &VectorcraftApp, ctx: &egui::Context) -> Option<String> {
    let status = crate::chrome::status_message(app);
    if let Some((running, true)) = status {
        return Some(running);
    }
    if let Some(note) = soon_note(ctx) {
        return Some(note);
    }
    let (message, _) = status?;
    let now = ctx.input(|i| i.time);
    let seen = wid("status-seen", ());
    let since = match ctx.data(|d| d.get_temp::<(u64, String, f64)>(seen)) {
        Some((seq, s, at)) if seq == app.ui.status_seq && s == app.ui.status => at,
        _ => {
            ctx.data_mut(|d| d.insert_temp(seen, (app.ui.status_seq, app.ui.status.clone(), now)));
            now
        }
    };
    let left = MESSAGE_SECS - (now - since);
    // Drawn again when it is due to go.
    (left > 0.0).then(|| {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
        message
    })
}

/// What a cell shows before its text: an on/off square (the mockup's `.tog b`).
#[derive(Clone, Copy)]
enum Mark {
    None,
    On,
    Off,
}

/// A cell of the status bar `r` ending at `right` with a line at its left: `mark`, its parts left
/// to right (text, font, colour) and a ▾ when `arrow`. Returns its rect.
fn cell(ui: &Ui, r: Rect, right: f32, mark: Mark, parts: &[(String, FontId, Color32)], arrow: bool, t: &Tokens) -> Rect {
    let widths: Vec<f32> = parts.iter().map(|(s, f, _)| width_of(ui, s, f.clone())).collect();
    let gaps = 6.0 * parts.len().saturating_sub(1) as f32;
    let mark_w = if matches!(mark, Mark::None) { 0.0 } else { 8.0 + 6.0 };
    let arrow_w = if arrow { 4.0 + 7.0 } else { 0.0 };
    let inner = mark_w + widths.iter().sum::<f32>() + gaps + arrow_w;
    let c = Rect::from_min_max(pos2(right - 12.0 - inner - 12.0, r.top() + 1.0), pos2(right, r.bottom()));
    let cy = r.center().y;
    ui.painter().vline(c.left() + 0.5, (r.top() + 1.0)..=r.bottom(), Stroke::new(1.0, t.border));
    let mut x = c.left() + 12.0;
    match mark {
        Mark::None => {}
        Mark::On => {
            ui.painter().rect_filled(Rect::from_center_size(pos2(x + 4.0, cy), vec2(8.0, 8.0)), CornerRadius::same(2), t.text);
            x += mark_w;
        }
        Mark::Off => {
            let sq = Rect::from_center_size(pos2(x + 4.0, cy), vec2(8.0, 8.0));
            ui.painter().rect_stroke(sq, CornerRadius::same(2), Stroke::new(1.0, t.text_disabled), StrokeKind::Inside);
            x += mark_w;
        }
    }
    for ((s, f, color), w) in parts.iter().zip(&widths) {
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, s, f.clone(), *color);
        x += w + 6.0;
    }
    if arrow {
        triangle(ui, pos2(x - 6.0 + 4.0 + 3.5, cy + 1.0), 6.0, true, t.text_dim);
    }
    c
}

/// A list's heading: small and dimmed, not a thing to click or select.
fn heading(ui: &mut Ui, s: &str, t: &Tokens) {
    ui.add(egui::Label::new(egui::RichText::new(s).font(font(11.0)).color(t.text_dim)).selectable(false));
}

/// The part of cell `c` a click opens its list from: its ▾ at the right.
fn arrow_part(c: Rect) -> Rect {
    Rect::from_min_max(pos2(c.right() - 12.0 - ARROW_W + 4.0, c.top()), c.max)
}

/// A cell that turns something on or off with a click and lists its settings from its ▾: the
/// response of the whole cell and of its ▾.
fn toggle_cell(ui: &Ui, key: &str, c: Rect) -> (egui::Response, egui::Response) {
    let body = ui.interact(Rect::from_min_max(c.min, pos2(arrow_part(c).left(), c.bottom())), wid("status", key), Sense::click());
    let more = ui.interact(arrow_part(c), wid("status-more", key), Sense::click());
    (body, more)
}

/// The zoom, "91% ▾": a click lists 50 to 400 % and Fit.
fn zoom_cell(app: &mut VectorcraftApp, ui: &Ui, r: Rect, right: f32, t: &Tokens) -> Rect {
    let zoom = app.view().map_or(1.0, |v| v.zoom);
    let c = cell(ui, r, right, Mark::None, &[(percent(zoom), mono(11.5), t.text)], true, t);
    let resp = ui.interact(c, wid("status", "zoom"), Sense::click()).on_hover_text(tw!("Zoom"));
    let mut run = None;
    egui::Popup::menu(&resp).id(wid("status-list", "zoom")).show(|ui| {
        for z in ZOOMS {
            if ui.selectable_label((z / 100.0 - zoom).abs() < 1e-4, format!("{z:.0}%")).clicked() {
                run = Some(("view.setZoom", json!({ "zoom": z })));
            }
        }
        ui.separator();
        if ui.selectable_label(false, tw!("Fit")).clicked() {
            run = Some(("view.fitArtboard", json!({})));
        }
    });
    if let Some((id, p)) = run
        && let Err(e) = app.run(id, p)
    {
        app.status(e);
    }
    c
}

/// The zoom as the status bar shows it: a whole percentage, with a decimal below 10 %.
pub(super) fn percent(zoom: f64) -> String {
    let p = zoom * 100.0;
    if p < 9.95 { format!("{p:.1}%") } else { format!("{p:.0}%") }
}

/// The scale and coordinate system, "1:30 · EPSG:5186 ▾": a click lists the scales and the dialog.
fn scale_cell(app: &mut VectorcraftApp, ui: &Ui, r: Rect, right: f32, t: &Tokens) -> Rect {
    let doc_scale = app.session.active().and_then(|d| d.doc.scale.clone());
    let (label, f, color) = match &doc_scale {
        Some(cfg) => (format!("{} · EPSG:{}", cfg.scale.label(), cfg.crs.epsg), mono(11.5), t.text),
        None => (tw!("Scale not set").to_string(), font(12.0), t.text_dim),
    };
    let c = cell(ui, r, right, Mark::None, &[(label, f, color)], true, t);
    let resp = ui.interact(c, wid("status", "scale"), Sense::click()).on_hover_text(tw!("Scale and coordinate system"));
    let mut set = None;
    let mut dialog = false;
    egui::Popup::menu(&resp).id(wid("status-list", "scale")).show(|ui| {
        for denom in SCALES {
            let on = doc_scale.as_ref().is_some_and(|cfg| (cfg.scale.denom - denom).abs() < 1e-4);
            if ui.selectable_label(on, format!("1:{denom:.0}")).clicked() {
                set = Some(denom);
            }
        }
        ui.separator();
        if ui.selectable_label(false, tw!("Scale and Coordinate System…")).clicked() {
            dialog = true;
        }
    });
    if let Some(denom) = set {
        // The coordinate system it has stays; a drawing without one gets the central belt's.
        let epsg = doc_scale.as_ref().map_or(5186, |cfg| cfg.crs.epsg);
        if let Err(e) = app.run("doc.setScale", json!({ "scale": denom, "epsg": epsg })) {
            app.status(e);
        }
    }
    if dialog {
        super::ribbon::scale_dialog(app);
    }
    c
}

/// Where the pointer is in the drawing (where it last was on this drawing's canvas, once it has left
/// it): TM metres with a coordinate system, else the document's unit. A click copies it.
fn position_cell(app: &mut VectorcraftApp, ui: &Ui, r: Rect, right: f32, t: &Tokens) -> Rect {
    let doc_scale = app.session.active().and_then(|d| d.doc.scale.clone());
    // The pointer leaves the canvas on its way here, so the cell keeps the last place it was.
    let last_id = wid("status-last-position", ());
    let doc = app.session.active_index();
    let at = match app.hover_doc {
        Some(p) => {
            ui.ctx().data_mut(|d| d.insert_temp(last_id, (doc, p)));
            Some(p)
        }
        None => ui.ctx().data(|d| d.get_temp::<(Option<usize>, vectorcraft_geom::Point)>(last_id)).filter(|(i, _)| *i == doc).map(|(_, p)| p),
    };
    let (x, y) = match at {
        Some(p) => match &doc_scale {
            Some(cfg) => cfg.format_coords(p),
            None => {
                let u = app.session.general_unit();
                (u.readout(p.x), u.readout(p.y))
            }
        },
        None => ("–".to_string(), "–".to_string()),
    };
    let parts = [
        ("X".to_string(), font(12.0), t.text_dim),
        (x.clone(), mono(11.5), t.text),
        ("· Y".to_string(), font(12.0), t.text_dim),
        (y.clone(), mono(11.5), t.text),
    ];
    let c = cell(ui, r, right, Mark::None, &parts, false, t);
    let resp = ui.interact(c, wid("status", "position"), Sense::click()).on_hover_text(tw!("Click to copy the coordinates"));
    if resp.clicked() && at.is_some() {
        ui.ctx().copy_text(format!("{x}, {y}"));
        note(ui.ctx(), tw!("Coordinates copied"));
    }
    c
}

/// Snap to Point, "■ Snap 2px ▾": a click turns it on or off, its ▾ lists the tolerance (1 to 8 px).
fn snap_cell(app: &mut VectorcraftApp, ui: &Ui, r: Rect, right: f32, t: &Tokens) -> Rect {
    let on = app.ui.view.snap_to_point;
    let value = app.session.prefs.snap_to_point_tolerance.round().clamp(1.0, 8.0);
    let label = format!("{} {value:.0}px", tw!("Snap"));
    let c = cell(ui, r, right, if on { Mark::On } else { Mark::Off }, &[(label, font(12.0), t.text)], true, t);
    let (body, more) = toggle_cell(ui, "snap", c);
    let tip = match crate::menus::shortcut_of("view.snapToPoint") {
        Some(sc) => format!("{} ({})", tl!("Snap to Point"), crate::menus::pretty_shortcut(sc)),
        None => tl!("Snap to Point").to_string(),
    };
    if body.on_hover_text(tip).clicked() {
        crate::menus::invoke(app, "view.snapToPoint", json!({}));
    }
    let mut set = None;
    egui::Popup::menu(&more).id(wid("status-list", "snap")).show(|ui| {
        heading(ui, tw!("How near the pointer snaps to a point"), t);
        for v in 1..=8 {
            let v = f64::from(v);
            if ui.selectable_label(v == value, format!("{v:.0} px")).clicked() {
                set = Some(v);
            }
        }
    });
    if let Some(v) = set
        && let Err(e) = app.run("prefs.set", json!({ "key": "snapToPointTolerance", "value": v }))
    {
        app.status(e);
    }
    c
}

/// The grid, "■ Grid ▾": a click shows or hides it, its ▾ lists the spacing (10, 20, 50 mm) and
/// colour.
fn grid_cell(app: &mut VectorcraftApp, ui: &Ui, r: Rect, right: f32, t: &Tokens) -> Rect {
    let on = app.ui.view.grid;
    let c = cell(ui, r, right, if on { Mark::On } else { Mark::Off }, &[(tw!("Grid").to_string(), font(12.0), t.text)], true, t);
    let (body, more) = toggle_cell(ui, "grid", c);
    let tip = match crate::menus::shortcut_of("view.grid") {
        Some(sc) => format!("{} ({})", tl!("Show Grid"), crate::menus::pretty_shortcut(sc)),
        None => tl!("Show Grid").to_string(),
    };
    if body.on_hover_text(tip).clicked() {
        crate::menus::invoke(app, "view.grid", json!({}));
    }
    let every = app.session.prefs.gridline_every;
    let mut set: Option<(&str, serde_json::Value)> = None;
    egui::Popup::menu(&more).id(wid("status-list", "grid")).show(|ui| {
        heading(ui, tw!("Spacing"), t);
        for mm in GRID_MM {
            let pt = mm * 72.0 / 25.4;
            if ui.selectable_label((every - pt).abs() < 0.01, format!("{mm:.0} mm")).clicked() {
                set = Some(("gridlineEvery", json!(format!("{mm} mm"))));
            }
        }
        ui.separator();
        heading(ui, tw!("Colour"), t);
        let current = app.session.prefs.grid_color.to_uppercase();
        for (label, hex) in grid_colours() {
            if ui.selectable_label(current == hex, label).clicked() {
                set = Some(("gridColor", json!(hex)));
            }
        }
    });
    if let Some((key, value)) = set {
        match app.run("prefs.set", json!({ "key": key, "value": value })) {
            // A grid set up is a grid to see.
            Ok(_) => app.ui.view.grid = true,
            Err(e) => app.status(e),
        }
    }
    c
}
