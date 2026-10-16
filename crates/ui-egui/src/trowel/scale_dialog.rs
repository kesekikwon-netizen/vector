//! The drawing's scale and coordinate system (design 3.0 §5 ③ 축척·좌표, with §9's chips): the
//! scales of the status bar's list or one's own, and the coordinate system, Korea's TM belts. OK
//! sets them with `doc.setScale`, keeping the drawing's axis order.

use serde_json::json;
use vectorcraft_doc::scale::CoordOrder;

use super::*;

/// The scales offered (design 3.0 §5 ⑩ and §9), as 1:N.
pub(super) const SCALES: [f64; 9] = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 200.0];
/// The largest scale of one's own, 1:N.
const MAX_DENOM: f64 = 5000.0;
/// A drawing without a coordinate system gets the central belt's.
const DEFAULT_EPSG: u32 = 5186;
/// A drawing without a scale starts the dialog at 1:20.
const DEFAULT_DENOM: f64 = 20.0;

/// The coordinate systems offered (design 3.0 §9): EPSG code and how its chip reads.
fn systems() -> [(u32, String); 3] {
    [(5186, format!("EPSG:5186 {}", tw!("Central Belt"))), (5185, format!("5185 {}", tw!("West"))), (5187, format!("5187 {}", tw!("East")))]
}

/// The width of the rows' labels (the mockup's `.frow`), at least.
const LABEL_W: f32 = 70.0;

/// What the dialog holds while it is open: for which drawing (its `uid`), and what it will set.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Draft {
    doc: u64,
    denom: f64,
    epsg: u32,
}

fn draft_id() -> egui::Id {
    wid("scale-dialog", ())
}

/// Open the dialog on the drawing's scale and coordinate system, or 1:20 in the central belt.
pub(super) fn open(app: &VectorcraftApp, ctx: &egui::Context) {
    let Some(st) = app.session.active() else { return };
    let (denom, epsg) = st.doc.scale.as_ref().map_or((DEFAULT_DENOM, DEFAULT_EPSG), |c| (c.scale.denom, c.crs.epsg));
    ctx.data_mut(|d| d.insert_temp(draft_id(), Some(Draft { doc: st.uid, denom, epsg })));
}

/// Set the drawing's scale to 1:`denom`, and its coordinate system to `epsg` (else the one it has,
/// else the central belt's), keeping its axis order: `doc.setScale` resets what it isn't given.
pub(super) fn set_scale(app: &mut VectorcraftApp, denom: f64, epsg: Option<u32>) -> Result<(), String> {
    let cfg = app.session.active().and_then(|d| d.doc.scale.as_ref());
    let epsg = epsg.or(cfg.map(|c| c.crs.epsg)).unwrap_or(DEFAULT_EPSG);
    let order = match cfg.map(|c| c.crs.order) {
        Some(CoordOrder::NorthEast) => "northeast",
        _ => "eastnorth",
    };
    app.run("doc.setScale", json!({ "scale": denom, "epsg": epsg, "order": order })).map(|_| ())
}

/// The dialog, while it is open and its drawing is the one in front (another tab put in front
/// closes it: OK would set that drawing's).
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let Some(mut draft) = ctx.data(|d| d.get_temp::<Option<Draft>>(draft_id())).flatten() else { return };
    let Some(now) = app.session.active().filter(|st| st.uid == draft.doc).map(|st| st.doc.scale.as_ref().map(|c| (c.scale.denom, c.crs.epsg))) else {
        ctx.data_mut(|d| d.insert_temp(draft_id(), None::<Draft>));
        return;
    };
    let t = Tokens::get(ctx);
    let mut open = true;
    let mut done = None;
    egui::Window::new(tw!("Scale and Coordinate System"))
        .id(wid("scale-dialog-window", ()))
        .frame(egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::symmetric(16, 14)))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(520.0)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 10.0);
            let labels = [tw!("Scale"), tw!("Other"), tw!("Coordinate system")];
            let label_w = labels.iter().map(|s| width_of(ui, s, font(12.5))).fold(LABEL_W, f32::max);
            row(ui, tw!("Scale"), label_w, |ui| {
                for denom in SCALES {
                    if chip(ui, wid("scale-chip", denom as u32), &format!("1:{denom:.0}"), (draft.denom - denom).abs() < 1e-4, &t).clicked() {
                        draft.denom = denom;
                    }
                }
            });
            row(ui, tw!("Other"), label_w, |ui| {
                // "1:" and the field as tall as the chips, centred on their line.
                let f = font(12.5);
                let (r, _) = ui.allocate_exact_size(vec2(width_of(ui, "1:", f.clone()), 26.0), Sense::hover());
                text(ui, r.left_center(), Align2::LEFT_CENTER, "1:", f, t.text_dim);
                let mut denom = draft.denom;
                // Whole numbers: a drawing's scale is 1:25, never 1:25.37.
                // A drawing outside 1 to 5000 (2:1 finds, 1:10 000 maps) keeps its scale until
                // one is typed.
                let field = egui::DragValue::new(&mut denom).range(1.0..=MAX_DENOM).clamp_existing_to_range(false).speed(1.0).max_decimals(2);
                if ui.add_sized(vec2(64.0, 26.0), field).changed() && denom.is_finite() {
                    draft.denom = denom.round().clamp(1.0, MAX_DENOM);
                }
            });
            row(ui, tw!("Coordinate system"), label_w, |ui| {
                let mut offered = systems().to_vec();
                // A drawing in another one keeps it on offer.
                if !offered.iter().any(|(code, _)| *code == draft.epsg) {
                    offered.push((draft.epsg, format!("EPSG:{}", draft.epsg)));
                }
                for (code, label) in offered {
                    if chip(ui, wid("crs-chip", code), &label, draft.epsg == code, &t).clicked() {
                        draft.epsg = code;
                    }
                }
            });
            // The buttons at the right, as tall as they are.
            let layout = egui::Layout::right_to_left(egui::Align::Center);
            ui.allocate_ui_with_layout(vec2(ui.available_width(), 30.0), layout, |ui| {
                if dialog_button(ui, wid("scale-dialog", "ok"), tw!("OK"), true, &t).clicked() {
                    done = Some(true);
                }
                if dialog_button(ui, wid("scale-dialog", "cancel"), tw!("Cancel"), false, &t).clicked() {
                    done = Some(false);
                }
            });
        });
    // OK with nothing changed leaves the drawing as it is (no undo step, nothing to save).
    let same = now.is_some_and(|(denom, epsg)| (denom - draft.denom).abs() < 1e-9 && epsg == draft.epsg);
    if done == Some(true)
        && !same
        && let Err(e) = set_scale(app, draft.denom, Some(draft.epsg))
    {
        app.status_error(e);
    }
    let keep = open && done.is_none();
    ctx.data_mut(|d| d.insert_temp(draft_id(), keep.then_some(draft)));
}

/// A row of the dialog (the mockup's `.frow`): its label in a `label_w` column, then `body`,
/// wrapping under itself.
fn row(ui: &mut Ui, label: &str, label_w: f32, body: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(label_w, 26.0), Sense::hover());
        text(ui, r.left_center(), Align2::LEFT_CENTER, label, font(12.5), t.text_dim);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            body(ui);
        });
    });
}

/// A chip (the mockup's `.chip`), 26 points tall: ink with a label in the panel's colour when
/// `on` (ivory on Plate; it reads in every theme).
fn chip(ui: &mut Ui, id: egui::Id, label: &str, on: bool, t: &Tokens) -> egui::Response {
    let f = font(11.5);
    let (r, _) = ui.allocate_exact_size(vec2(width_of(ui, label, f.clone()) + 18.0, 26.0), Sense::hover());
    let resp = ui.interact(r, id, Sense::click());
    let (fill, edge, color) = if on {
        (t.text, t.text, t.panel)
    } else if resp.hovered() {
        (t.hover, t.input_border, t.text)
    } else {
        (t.input, t.input_border, t.text_dim)
    };
    ui.painter().rect(r, CornerRadius::same(13), fill, Stroke::new(1.0, edge), StrokeKind::Inside);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, color);
    resp
}

/// A dialog button (the mockup's `.btn`), 28 points tall: the one that does it in ink.
fn dialog_button(ui: &mut Ui, id: egui::Id, label: &str, strong: bool, t: &Tokens) -> egui::Response {
    let f = font(12.0);
    let (r, _) = ui.allocate_exact_size(vec2(width_of(ui, label, f.clone()) + 28.0, 28.0), Sense::hover());
    let resp = ui.interact(r, id, Sense::click());
    let (fill, edge, color) = match (strong, resp.hovered()) {
        (true, _) => (t.text, t.text, t.panel),
        (false, true) => (t.hover, t.input_border, t.text),
        (false, false) => (t.button, t.input_border, t.text),
    };
    ui.painter().rect(r, CornerRadius::same(7), fill, Stroke::new(1.0, edge), StrokeKind::Inside);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, color);
    resp
}
