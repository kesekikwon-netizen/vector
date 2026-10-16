//! The height box of the tool column's Level Lines and Spot Height (design 3.0 §5 ④): a click on
//! the ground line or on a point asks for the height there, beside the pointer. Enter (or 넣기)
//! draws the level lines (`trowel.levelLines`) or the ▽ spot height (`trowel.spotElevation`);
//! Esc (or 그만두기) leaves it. The box starts with the last height put in, the same one the
//! drawing set's section and spot height elements use ([`datum_elevation_m`]).

use serde_json::{Value, json};

use super::*;

/// What the box is asking for.
#[derive(Clone, Debug, PartialEq)]
pub enum AskFor {
    /// The height at `at` on ground line `profile`.
    Level { profile: u64, at: [f64; 2] },
    /// The height of the point `at`.
    Spot { at: [f64; 2] },
}

/// The box while it is open ([`crate::VectorcraftApp::trowel_ask`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub what: AskFor,
    /// For which drawing (its `uid`): another one put in front closes the box.
    pub doc: u64,
    /// What is typed, the last height until the box first shows.
    pub text: Option<String>,
    /// Where the box sits on screen: beside the pointer as it was when the box opened.
    pub at: Option<Pos2>,
    /// The field is still to take the keyboard: once the click that opened the box is over (egui
    /// takes the focus away from a field on a press or a click elsewhere).
    pub focus: bool,
}

/// The highest and lowest heights taken, in metres: past them a typing slip.
const HIGHEST_M: f64 = 9_000.0;
const LOWEST_M: f64 = -11_000.0;
/// The box's inner width: the field with its unit, the hint under it and the two buttons.
const INNER_W: f32 = 210.0;

/// Open the box for a tool's click (`kind` [`vectorcraft_tools::trowel::ASK_LEVEL`] or
/// [`vectorcraft_tools::trowel::ASK_SPOT`], `p` its `{profileId?, at}`); a click that found no
/// line ([`vectorcraft_tools::trowel::HINT`]) says what to click instead.
pub(crate) fn open(app: &mut VectorcraftApp, kind: &str, p: &Value) {
    use vectorcraft_tools::trowel::{ASK_LEVEL, ASK_SPOT, HINT};
    let at = p.get("at").and_then(|a| Some([a.get(0)?.as_f64()?, a.get(1)?.as_f64()?])).filter(|a| a.iter().all(|v| v.is_finite()));
    let what = match (kind, at, p.get("profileId").and_then(Value::as_u64)) {
        (HINT, _, _) => {
            let hint = match p.get("tool").and_then(Value::as_str) {
                Some("slopeHachure") => tw!("Click the line to hachure along"),
                _ => tw!("Click the ground line"),
            };
            app.status(hint);
            return;
        }
        // A drawing with map coordinates reads its heights from its northings (`trowel.levelLines`
        // refuses it): said before a height is typed.
        (ASK_LEVEL, _, _) if app.session.active().and_then(|st| st.doc.scale.as_ref()).is_some_and(|sc| sc.world_origin.1.abs() >= 1e-6) => {
            app.status_error(tw!("This drawing has map coordinates: draw level lines in a section drawing"));
            return;
        }
        (ASK_LEVEL, Some(at), Some(profile)) => AskFor::Level { profile, at },
        (ASK_SPOT, Some(at), _) => AskFor::Spot { at },
        _ => return,
    };
    let Some(doc) = app.session.active().map(|st| st.uid) else { return };
    app.trowel_ask = Some(Ask { what, doc, text: None, at: None, focus: true });
}

/// A height as typed: `32.5`, `32.5m`, `32,5` (a decimal comma), in metres.
pub(crate) fn parse_height(s: &str) -> Option<f64> {
    let s = s.trim().trim_end_matches(['m', 'M']).trim().replace(',', ".");
    s.parse::<f64>().ok().filter(|v| v.is_finite() && (LOWEST_M..=HIGHEST_M).contains(v))
}

/// A height as the box shows it: one decimal at least, up to three (`93.0`, `32.45`).
fn shown(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0');
    if s.ends_with('.') { format!("{s}0") } else { s.to_string() }
}

/// Draw what the box asked for at height `m`.
fn put(app: &mut VectorcraftApp, what: &AskFor, m: f64) -> Result<Value, String> {
    match what {
        AskFor::Level { profile, at } => app.run("trowel.levelLines", json!({ "profileId": profile, "at": at, "elevationM": m })),
        AskFor::Spot { at } => {
            app.run("trowel.spotElevation", json!({ "at": at, "elevation": m, "marker": "triangleDown", "prefixPlus": false, "format": "auto" }))
        }
    }
}

/// The box, while it is open and its drawing is the one in front.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let Some(mut ask) = app.trowel_ask.clone() else { return };
    if app.session.active().map(|st| st.uid) != Some(ask.doc) {
        app.trowel_ask = None;
        return;
    }
    let t = Tokens::get(ctx);
    let mut text = ask.text.clone().unwrap_or_else(|| shown(datum_elevation_m(ctx)));
    let screen = ctx.content_rect();
    let at = *ask.at.get_or_insert_with(|| ctx.pointer_latest_pos().unwrap_or(screen.center()));
    let title = match ask.what {
        AskFor::Level { .. } => tw!("Height here"),
        AskFor::Spot { .. } => tw!("Height"),
    };
    let mut done = None;
    let mut bad = false;
    egui::Area::new(wid("ask", ())).order(egui::Order::Foreground).fixed_pos(at + vec2(14.0, 14.0)).constrain_to(screen).show(ctx, |ui| {
        egui::Frame::new()
            .fill(t.panel)
            .stroke(Stroke::new(1.0, t.input_border))
            .corner_radius(CornerRadius::same(10))
            .shadow(egui::epaint::Shadow { offset: [0, 2], blur: 10, spread: 0, color: t.text.gamma_multiply(0.14) })
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 8.0);
                // An area is as wide as the screen allows: the box keeps to its own width.
                let inner = width_of(ui, title, font(12.0)).max(INNER_W);
                ui.set_width(inner);
                let (r, _) = ui.allocate_exact_size(vec2(inner, 16.0), Sense::hover());
                text_at(ui, r.left_center(), title, font(12.0), t.text_dim);
                ui.horizontal(|ui| {
                    let field = egui::TextEdit::singleline(&mut text).id(wid("ask-field", ())).desired_width(116.0).font(mono(13.0));
                    let resp = ui.add_sized(vec2(116.0, 26.0), field);
                    // Typing replaces the last height.
                    let pointer = ui.input(|i| i.pointer.any_down() || i.pointer.any_released());
                    if ask.focus && !pointer {
                        crate::widgets::focus_field(ui, resp.id, &text);
                        ask.focus = false;
                    }
                    let (r, _) = ui.allocate_exact_size(vec2(14.0, 26.0), Sense::hover());
                    text_at(ui, r.left_center(), "m", font(12.5), t.text_dim);
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        done = Some(true);
                    }
                });
                bad = parse_height(&text).is_none();
                if bad {
                    text_at(ui, ui.cursor().min + vec2(0.0, 6.0), tw!("Type a height in metres, such as 32.5"), font(10.5), t.error);
                    ui.add_space(14.0);
                }
                let layout = egui::Layout::right_to_left(egui::Align::Center);
                ui.allocate_ui_with_layout(vec2(inner, 28.0), layout, |ui| {
                    if super::scale_dialog::dialog_button(ui, wid("ask", "ok"), tw!("Put in"), true, &t).clicked() {
                        done = Some(true);
                    }
                    if super::scale_dialog::dialog_button(ui, wid("ask", "cancel"), tw!("Cancel"), false, &t).clicked() {
                        done = Some(false);
                    }
                });
            });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        done = Some(false);
    }
    ask.text = Some(text.clone());
    match done {
        Some(true) => {
            let Some(m) = parse_height(&text) else {
                // Wrong: the box stays, saying so, with the field to fix.
                ctx.memory_mut(|mem| mem.request_focus(wid("ask-field", ())));
                app.trowel_ask = Some(ask);
                return;
            };
            app.trowel_ask = None;
            ctx.data_mut(|d| d.insert_temp(datum_elevation_id(), m));
            if let Err(e) = put(app, &ask.what, m) {
                app.status_error(e);
            }
        }
        Some(false) => app.trowel_ask = None,
        None => app.trowel_ask = Some(ask),
    }
}

/// `s` at `at`, left-aligned.
fn text_at(ui: &Ui, at: Pos2, s: &str, f: FontId, color: Color32) {
    text(ui, at, Align2::LEFT_CENTER, s, f, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heights_are_read_as_typed() {
        assert_eq!(parse_height("32.5"), Some(32.5));
        assert_eq!(parse_height(" 32.45m "), Some(32.45));
        assert_eq!(parse_height("32,5"), Some(32.5));
        assert_eq!(parse_height("-1.2 M"), Some(-1.2));
        for bad in ["", "m", "abc", "NaN", "inf", "1e9", "32.5.1"] {
            assert_eq!(parse_height(bad), None, "{bad}");
        }
        assert_eq!((shown(93.0), shown(32.45), shown(32.5), shown(-0.125)), ("93.0".into(), "32.45".into(), "32.5".into(), "-0.125".into()));
    }
}
