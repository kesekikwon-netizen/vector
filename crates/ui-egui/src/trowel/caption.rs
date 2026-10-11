//! The caption: Trowel's mark, "Document * - Trowel", and with the app's own title bar the window
//! buttons at the right; the rest of it moves the window.

use super::*;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let custom = app.custom_titlebar;
    let bar = egui::Panel::top(CAPTION)
        .exact_size(CAPTION_H)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let r = ui.max_rect();
            line_below(ui, r, t.border);
            if custom {
                crate::titlebar::drag_area(ui, r);
            }
            // macOS's window buttons take the inset at the left.
            let left = r.left() + app.titlebar_inset.max(14.0);
            let mark = Rect::from_center_size(pos2(left + 8.0, r.center().y), vec2(16.0, 16.0));
            crate::brand::paint_mark(ui, mark);
            let room = Rect::from_min_max(pos2(mark.right() + 8.0, r.top()), pos2(r.right() - crate::titlebar::WIDTH - 10.0, r.bottom()));
            let title = crate::chrome::window_title(app);
            ui.painter().with_clip_rect(room).text(room.left_center(), Align2::LEFT_CENTER, title, font(13.0), t.text);
            r
        })
        .inner;
    if custom {
        crate::titlebar::caption_buttons(app, ui, bar);
    }
}
