//! Weights at Once (design 2.1 §7, figure ③): the card the legend's foot and Drawing › Weights at
//! Once… open beside the right column. It changes the weight of every legend row of the drawing
//! and of the lines that follow them by a ratio, from one weight to another, or one palette step
//! (`legend.weights`), the lines set apart only when asked. Its table shows what changes, as the
//! engine plans it, before Change does it in one undo step; the card stays open after. The whole
//! drawing set comes with J004.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::LegendGroup;
use vectorcraft_engine::cmd::legend::{WeightPlan, plan_weights};

use super::card::{self, GAP_X};
use super::legend::{RATIO, pt};
use super::*;

/// The table's most lines; the rest are counted under it.
const MAX_ROWS: usize = 10;
/// The rows' height.
const ROW_H: f32 = 24.0;
const SEG_H: f32 = 22.0;

/// How the weights change.
#[derive(Clone, Copy, Debug, PartialEq)]
enum How {
    Ratio,
    Value,
    Step,
}

/// The card's settings while it is open: how, the ratio, the weight to change and what it
/// becomes, which way a step goes and whether the lines set apart change too.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rule {
    how: How,
    ratio: f64,
    from: f64,
    to: f64,
    up: bool,
    overrides: bool,
}

impl Default for Rule {
    fn default() -> Self {
        Rule { how: How::Ratio, ratio: RATIO, from: 0.5, to: 0.4, up: false, overrides: false }
    }
}

impl Rule {
    /// `legend.weights`' params.
    fn params(&self) -> Value {
        let mut p = match self.how {
            How::Ratio => json!({ "how": "ratio", "ratio": self.ratio }),
            How::Value => json!({ "how": "value", "from": self.from, "to": self.to }),
            How::Step => json!({ "how": "step", "step": if self.up { 1 } else { -1 } }),
        };
        p["overrides"] = json!(self.overrides);
        p
    }
}

/// What the card asks for.
enum Ask {
    Change,
    Close,
    /// Something typed that can't be used: why.
    Refused(&'static str),
}

fn rule_id() -> egui::Id {
    wid("weights-rule", ())
}

/// Show the card from `at` within `bounds` ([`card::show`]). Returns whether it stays open.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context, at: Pos2, bounds: Rect) -> bool {
    let Some(st) = app.session.active() else { return false };
    let mut rule = ctx.data(|d| d.get_temp::<Rule>(rule_id())).unwrap_or_default();
    let params = rule.params();
    let plan = plan(ctx, st, &params);
    let sub = match super::legend::main_group(&st.doc) {
        Some(LegendGroup::Plan) => tw!("Plan Legend"),
        Some(LegendGroup::Section) => tw!("Section and Elevation Legend"),
        Some(LegendGroup::Finds) => tw!("Finds Legend"),
        None => "",
    };
    let (mut open, asks) = card::show(ctx, wid("weights-card", ()), at, bounds, (tw!("Weights at Once"), sub), |ui, t| body(ui, &mut rule, &plan, t));
    for ask in asks {
        match ask {
            Ask::Change => match app.run("legend.weights", params.clone()) {
                Ok(done) => {
                    let rows = done["rows"].as_array().map_or(0, Vec::len);
                    let lines = done["lines"].as_u64().unwrap_or(0);
                    app.status(crate::i18n::fmt(tw!("Rows changed: {r} · lines: {n}"), &[("r", &rows.to_string()), ("n", &lines.to_string())]));
                }
                Err(e) => app.status(e),
            },
            Ask::Close => open = false,
            Ask::Refused(why) => app.status(why),
        }
    }
    if open {
        ctx.data_mut(|d| d.insert_temp(rule_id(), rule));
    } else {
        // Opened again, it starts over.
        ctx.data_mut(|d| d.remove::<Rule>(rule_id()));
    }
    open
}

/// What `params` would change in the drawing, planned again when the drawing or the params change.
fn plan(ctx: &egui::Context, st: &vectorcraft_engine::DocState, params: &Value) -> Arc<WeightPlan> {
    let key = (st.uid, st.revision, params.to_string());
    let id = wid("weights-plan", ());
    if let Some((was, plan)) = ctx.data(|d| d.get_temp::<((u64, u64, String), Arc<WeightPlan>)>(id))
        && was == key
    {
        return plan;
    }
    // Params the card made are always valid; a refusal would show an empty table.
    let plan = Arc::new(plan_weights(&st.doc, params).unwrap_or_default());
    ctx.data_mut(|d| d.insert_temp(id, (key, plan.clone())));
    plan
}

fn body(ui: &mut Ui, rule: &mut Rule, plan: &WeightPlan, t: &Tokens) -> Vec<Ask> {
    let mut asks = vec![];
    let seg = |mono_font: bool| SegLook { h: SEG_H, pad: 6.0, font: if mono_font { mono(11.0) } else { font(11.0) } };
    // Where: this drawing; the whole set comes with J004.
    let r = card::row(ui, tw!("Where"), SEG_H, t);
    let whole = tw!("Whole Drawing Set");
    let parts = [
        Seg { key: "drawing", label: tw!("This drawing"), on: true, soon: None },
        Seg { key: "set", label: whole, on: false, soon: Some((whole, "J004")) },
    ];
    segments(ui, "weights-where", r.left_center(), &seg(false), &parts, t);
    // How.
    let r = card::row(ui, tw!("How"), SEG_H, t);
    let parts = [
        Seg { key: "ratio", label: tw!("Ratio"), on: rule.how == How::Ratio, soon: None },
        Seg { key: "value", label: tw!("Change Values"), on: rule.how == How::Value, soon: None },
        Seg { key: "step", label: tw!("One Step"), on: rule.how == How::Step, soon: None },
    ];
    match segments(ui, "weights-how", r.left_center(), &seg(false), &parts, t).0 {
        Some("ratio") => rule.how = How::Ratio,
        Some("value") => rule.how = How::Value,
        Some("step") => rule.how = How::Step,
        _ => {}
    }
    match rule.how {
        How::Ratio => {
            let r = card::row(ui, tw!("Ratio"), ROW_H, t);
            let field = Rect::from_min_size(r.min, vec2(70.0, r.height()));
            if let Some(typed) = card::text_field(ui, wid("weights-ratio", ()), field, &pt(rule.ratio), mono(11.0), ("×", ""), t) {
                let typed = typed.trim_start_matches(['×', 'x', '*']).trim();
                match typed.parse::<f64>().ok().filter(|v| (0.1..=10.0).contains(v)) {
                    Some(v) => rule.ratio = v,
                    None => asks.push(Ask::Refused(tw!("The ratio is between 0.1 and 10"))),
                }
            }
        }
        How::Value => {
            let r = card::row(ui, tw!("Values"), ROW_H, t);
            let from = Rect::from_min_size(r.min, vec2(64.0, r.height()));
            let arrow = text(ui, pos2(from.right() + GAP_X, r.center().y), Align2::LEFT_CENTER, "→", font(12.0), t.text_dim);
            let to = Rect::from_min_size(pos2(arrow.right() + GAP_X, r.top()), vec2(64.0, r.height()));
            for (id, field, value) in [("weights-from", from, &mut rule.from), ("weights-to", to, &mut rule.to)] {
                if let Some(typed) = card::text_field(ui, wid(id, ()), field, &pt(*value), mono(11.0), ("", "pt"), t) {
                    match card::points(&typed) {
                        Some(w) => *value = w,
                        None => asks.push(Ask::Refused(tw!("Type a weight in points, such as 0.5"))),
                    }
                }
            }
        }
        How::Step => {
            let r = card::row(ui, tw!("Step"), SEG_H, t);
            let parts = [
                Seg { key: "down", label: tw!("One Step Thinner"), on: !rule.up, soon: None },
                Seg { key: "up", label: tw!("One Step Thicker"), on: rule.up, soon: None },
            ];
            match segments(ui, "weights-step", r.left_center(), &seg(false), &parts, t).0 {
                Some("down") => rule.up = false,
                Some("up") => rule.up = true,
                _ => {}
            }
        }
    }
    // The lines set apart: left as they are unless asked.
    let r = card::row(ui, tw!("Apart"), ROW_H, t);
    let overrides = toggle_rect(ui, r.left_center(), ROW_H, tw!("Lines set apart too"));
    let overrides = toggle(ui, wid("weights-overrides", ()), overrides, tw!("Lines set apart too"), rule.overrides, t);
    if overrides.on_hover_text(tw!("Lines set apart take the change on their own weight")).clicked() {
        rule.overrides = !rule.overrides;
    }
    ui.add_space(4.0);
    table(ui, plan, t);
    let undo = crate::menus::pretty_shortcut("Cmd+Z");
    let n = plan.lines.len().to_string();
    let changes = !plan.rows.is_empty() || !plan.lines.is_empty();
    if changes {
        card::note(ui, &crate::i18n::fmt(tw!("Lines that change: {n}. One {undo} puts them all back."), &[("n", &n), ("undo", &undo)]), t.text_dim);
    } else {
        card::note(ui, tw!("Nothing changes"), t.text_dim);
    }
    if plan.skipped > 0 {
        card::note(ui, &crate::i18n::fmt(tw!("Locked lines left as they are: {n}."), &[("n", &plan.skipped.to_string())]), t.text_dim);
    }
    ui.add_space(4.0);
    let (change, close) = (tw!("Change"), tw!("Close"));
    let places = card::flow(ui, 28.0, &[card::button_w(ui, change), card::button_w(ui, close)]);
    if let Some(r) = places.first()
        && card::primary(ui, wid("weights-apply", ()), *r, change, changes, t).clicked()
    {
        asks.push(Ask::Change);
    }
    if let Some(r) = places.get(1)
        && card::button(ui, wid("weights-close", ()), *r, close, true, t).clicked()
    {
        asks.push(Ask::Close);
    }
    asks
}

/// The rows that change, as planned: those changing alike on one line, their names joined, the
/// heaviest first (the mockup's table: legend row, now, after).
fn table(ui: &mut Ui, plan: &WeightPlan, t: &Tokens) {
    let mut lines: Vec<(String, f64, f64)> = vec![];
    for row in &plan.rows {
        match lines.iter_mut().find(|(_, from, to)| pt(*from) == pt(row.from) && pt(*to) == pt(row.to)) {
            Some((names, _, _)) => *names = format!("{names} · {}", row.name),
            None => lines.push((row.name.clone(), row.from, row.to)),
        }
    }
    lines.sort_by(|a, b| b.1.total_cmp(&a.1).then(b.2.total_cmp(&a.2)));
    let more = lines.len().saturating_sub(MAX_ROWS);
    let (head_h, line_h) = (24.0, 26.0);
    let shown = lines.len().min(MAX_ROWS) + usize::from(more > 0);
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), head_h + line_h * shown as f32), Sense::hover());
    // Its columns: the names, then now and after, right-aligned.
    let after = r.right() - 8.0;
    let now = after - 52.0;
    let names = now - 44.0;
    let p = ui.painter();
    p.hline(r.x_range(), r.top() + 0.5, Stroke::new(1.0, t.text));
    let cy = r.top() + head_h / 2.0;
    text(ui, pos2(r.left() + 8.0, cy), Align2::LEFT_CENTER, tw!("Legend Row"), font(11.0), t.text_dim);
    text(ui, pos2(now, cy), Align2::RIGHT_CENTER, tw!("Now"), font(11.0), t.text_dim);
    text(ui, pos2(after, cy), Align2::RIGHT_CENTER, tw!("After"), font(11.0), t.text_dim);
    p.hline(r.x_range(), r.top() + head_h - 0.5, Stroke::new(1.0, t.border));
    let mut y = r.top() + head_h;
    for (row_names, from, to) in lines.iter().take(MAX_ROWS) {
        let cy = y + line_h / 2.0;
        text_to(ui, pos2(r.left() + 8.0, cy), names, row_names, font(12.0), t.text);
        text(ui, pos2(now, cy), Align2::RIGHT_CENTER, pt(*from), mono(12.0), t.text);
        text(ui, pos2(after, cy), Align2::RIGHT_CENTER, pt(*to), mono(12.0), t.text);
        y += line_h;
        p.hline(r.x_range(), y - 0.5, Stroke::new(1.0, t.pressed));
    }
    if more > 0 {
        let s = crate::i18n::fmt(tw!("…and {n} more"), &[("n", &more.to_string())]);
        text(ui, pos2(r.left() + 8.0, y + line_h / 2.0), Align2::LEFT_CENTER, s, font(12.0), t.text_dim);
    }
    p.hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, t.text));
}
