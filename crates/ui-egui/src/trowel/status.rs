//! The status bar (design 2.1 §3): at the left what is going on (a save running, the last message,
//! else how to use the tool in hand, its keys in bold); at the right the tool, Snap, how many drawings have changes
//! not saved, where the pointer is in the drawing, and the drawing's scale and coordinate system
//! (J006).

use super::*;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom(STATUS).exact_size(STATUS_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_above(ui, r, t.border);
        // The cells, right to left.
        let scale = cell(ui, r, r.right(), false, &[(tw!("Scale and coordinate system not set").into(), font(12.0), t.text_dim)], &t);
        soon_widget(ui, wid("status-scale", ()), scale, "J006");
        let u = app.session.general_unit();
        let (x, y) = app.hover_doc.map_or(("–".to_string(), "–".to_string()), |p| (u.readout(p.x), u.readout(p.y)));
        let at = cell(
            ui,
            r,
            scale.left(),
            false,
            &[("X".into(), font(12.0), t.text_dim), (x, mono(12.0), t.text), ("Y".into(), font(12.0), t.text_dim), (y, mono(12.0), t.text)],
            &t,
        );
        let unsaved = app.session.documents().iter().filter(|d| d.is_dirty()).count();
        let saved = if unsaved > 0 {
            cell(ui, r, at.left(), true, &[(crate::i18n::fmt(tw!("{n} not saved"), &[("n", &unsaved.to_string())]), font(12.0), t.text)], &t)
        } else {
            cell(ui, r, at.left(), false, &[(tw!("All saved").into(), font(12.0), t.text_dim)], &t)
        };
        let snap = if app.ui.view.snap_to_point { tw!("Snap on") } else { tw!("Snap off") };
        let snap = cell(ui, r, saved.left(), false, &[(snap.into(), font(12.0), t.text)], &t);
        let tool = cell(ui, r, snap.left(), false, &[(tool_name(app.session.tool_id()), font(12.0), t.text)], &t);
        // What is going on, in the room left: a message, else how to use the tool in hand.
        let room = Rect::from_min_max(pos2(r.left() + 16.0, r.top()), pos2(tool.left() - 12.0, r.bottom()));
        let p = ui.painter().with_clip_rect(room);
        match message(app) {
            Some(m) => {
                p.text(pos2(room.left(), r.center().y), Align2::LEFT_CENTER, m, font(12.0), t.text_dim);
            }
            None => {
                let hint = p.layout_job(hint_job(app, false, 12.0, t.text, t.text_dim));
                p.galley(pos2(room.left(), r.center().y - hint.size().y / 2.0), hint, t.text_dim);
            }
        }
    });
}

/// A save or export running, else the last message.
fn message(app: &VectorcraftApp) -> Option<String> {
    if let Some(job) = app.background.jobs.first() {
        let more = app.background.jobs.len() - 1;
        let label = crate::i18n::msg(&job.label);
        return Some(if more > 0 { format!("{label}… (+{more})") } else { format!("{label}…") });
    }
    // The message stays English in `ui.status` (agents and tests read it).
    (!app.ui.status.is_empty()).then(|| crate::i18n::msg(&app.ui.status))
}

/// A cell of the status bar `r` ending at `right`: its parts left to right (text, font, colour),
/// after a clay dot when `dot`, and a line at its left. Returns its rect.
fn cell(ui: &Ui, r: Rect, right: f32, dot: bool, parts: &[(String, FontId, Color32)], t: &Tokens) -> Rect {
    let widths: Vec<f32> = parts.iter().map(|(s, f, _)| width_of(ui, s, f.clone())).collect();
    let gaps = 6.0 * parts.len().saturating_sub(1) as f32;
    let inner = widths.iter().sum::<f32>() + gaps + if dot { 12.0 } else { 0.0 };
    let c = Rect::from_min_max(pos2(right - 12.0 - inner - 12.0, r.top()), pos2(right, r.bottom()));
    let cy = r.center().y;
    ui.painter().vline(c.left() + 0.5, (cy - 9.0)..=(cy + 9.0), Stroke::new(1.0, t.border));
    let mut x = c.left() + 12.0;
    if dot {
        ui.painter().circle_filled(pos2(x + 3.0, cy), 3.0, t.clay);
        x += 12.0;
    }
    for ((s, f, color), w) in parts.iter().zip(&widths) {
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, s, f.clone(), *color);
        x += w + 6.0;
    }
    c
}
