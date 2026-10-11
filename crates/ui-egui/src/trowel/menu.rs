//! The menu bar: the app's menus with the Drawing menu (design 2.1 §3) before View. Its items come
//! with later jobs; until then they show dimmed, each with its job.

use super::*;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(MENU).exact_size(MENU_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let mut bar =
            ui.new_child(egui::UiBuilder::new().max_rect(r.shrink2(vec2(6.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
        let body = |ui: &mut Ui| drawing_menu(ui);
        let extra = crate::menus::ExtraMenu { before: "View", title: tw!("Drawing"), body: &body };
        let (_, title) = crate::menus::menu_bar_with(app, &mut bar, 12.0, Some(10.0), Some(&extra));
        // The new menu is underlined, as in the mockup.
        if let Some(tr) = title {
            let under = Rect::from_min_max(pos2(tr.left() + 6.0, r.bottom() - 3.0), pos2(tr.right() - 6.0, r.bottom() - 1.0));
            ui.painter().rect_filled(under, 0.0, t.row_selected);
        }
    });
}

/// The Drawing menu's items, in groups, each with the job that brings it.
fn drawing_menu(ui: &mut Ui) {
    let groups: [&[(&str, &str)]; 5] = [
        &[(tw!("Change Weights at Once…"), "J005"), (tw!("Back to the Legend"), "J005")],
        &[(tw!("Cut Face"), "J014"), (tw!("Make Level Lines"), "J015")],
        &[(tw!("Scale and Coordinate System…"), "J006"), (tw!("Align Underlay…"), "J008")],
        &[(tw!("Insert Legend Table"), "J012"), (tw!("Insert Title Block"), "J012")],
        &[(tw!("Send to Strata…"), "J019")],
    ];
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            ui.separator();
        }
        for (label, job) in group.iter() {
            ui.add_enabled(false, egui::Button::new(*label)).on_disabled_hover_text(soon(job));
        }
    }
}
