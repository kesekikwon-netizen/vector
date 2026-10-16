//! The tool column (design 3.0 §4 `.a-tools`, §5 ④): Illustrator's order in four groups, the
//! archaeology tools under 「고고학」 (keys 1–4) and the view tools last. A slot shows the tool in
//! hand when it holds it, else the one last used from it; a long press or a right click opens its
//! other tools (the classic toolbar's flyout). The fill and stroke are the pen's (§5 ⑧), not here.

use serde_json::json;

use super::*;

/// One slot: its tools (first = the slot's own, with Trowel's icon) and that icon.
type Slot = (&'static [&'static str], &'static str);

/// The groups, top to bottom, a line between each two; the archaeology group has its caption.
const GROUPS: &[(bool, &[Slot])] = &[
    (false, &[(&["selection"], "tw-select"), (&["directSelection", "groupSelection", "lasso", "magicWand"], "tw-direct")]),
    (
        false,
        &[
            (&["pen", "curvature", "addAnchor", "deleteAnchor", "anchorPoint"], "tw-pen"),
            (&["pencil", "smooth", "pathEraser", "join", "paintbrush", "blobBrush"], "tw-pencil"),
            (&["lineSegment", "arc", "spiral"], "tw-line"),
            (&["rectangle", "roundedRectangle", "ellipse", "polygon", "star"], "tw-rect"),
            (&["type", "areaType", "typeOnPath", "verticalType"], "tw-type"),
        ],
    ),
    (
        false,
        &[
            (&["rotate", "reflect"], "tw-rotate"),
            (&["scale", "shear", "reshape", "freeTransform"], "tw-scale"),
            (&["scissors", "knife", "eraser"], "tw-scissors"),
            (&["livePaintBucket", "livePaintSelection", "shapeBuilder"], "tw-bucket"),
            (&["eyedropper", "measure"], "tw-dropper"),
        ],
    ),
    (
        true,
        &[
            (&["sectionCut", "slopeHachure"], "tw-section-line"),
            (&["levelLines"], "tw-level"),
            (&["spotHeight"], "tw-spot-height"),
            (&["strataLine"], "tw-strata-line"),
        ],
    ),
    (false, &[(&["hand", "rotateView"], "tw-hand"), (&["zoom"], "tw-zoom")]),
];

/// A tool's button as in the mockup (`.tl`); a shorter window tightens it.
const BUTTON: egui::Vec2 = vec2(38.0, 32.0);
const BUTTON_TIGHT: f32 = 22.0;
/// The room around the buttons: above the first and under the last (`padding`), between two
/// (`gap`), and either side of the line between groups (`.tsep` margin).
const PAD: f32 = 8.0;
const GAP: f32 = 2.0;
const SEP_MARGIN: f32 = 4.0;
/// From one group's last button to the next one's first: the gaps, the margins and the line.
const SEPARATOR: f32 = 2.0 * GAP + 2.0 * SEP_MARGIN + 1.0;
/// The archaeology group's caption (`.tcap`): above it and its line's height.
const CAPTION_TOP: f32 = 2.0;
const CAPTION_H: f32 = 13.0;
const CAPTION_PT: f32 = 10.0;
/// The icons' size; the Section Line's (its red marks) is 20.
const ICON: f32 = 18.0;
const ICON_SECTION: f32 = 20.0;

/// The archaeology tools' names and what they do (design 3.0 §5 ④), for their tooltips.
fn arch_tip(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "sectionCut" => (tw!("Section Line"), tw!("Click two points: the A–A′ marks, with the next free letter")),
        "slopeHachure" => (tw!("Slope Hachure"), tw!("Click a line: slope hachures along it")),
        "levelLines" => (tw!("Level Lines"), tw!("Click the ground line and type the height there: level lines that stop at it")),
        "spotHeight" => (tw!("Spot Height"), tw!("Click a point and type its height: ▽ and 32.5m")),
        "strataLine" => (tw!("Strata Line"), tw!("Draw a layer boundary: its number and a description line come with it. Enter ends it")),
        _ => return None,
    })
}

/// How tall the tools are with buttons `bh` tall, from the column's top.
fn tools_height(bh: f32) -> f32 {
    let slots: usize = GROUPS.iter().map(|g| g.1.len()).sum();
    let groups = GROUPS.len() as f32;
    let captions = GROUPS.iter().filter(|g| g.0).count() as f32;
    2.0 * PAD + slots as f32 * (bh + GAP) - groups * GAP + (groups - 1.0) * SEPARATOR + captions * (CAPTION_TOP + CAPTION_H + GAP)
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::left(TOOLS)
        .resizable(true)
        .default_size(TOOLS_W)
        .size_range(48.0..=140.0)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(t.panel))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.set_min_width(r.width());
            line_right(ui, r, t.border);
            let ctx = ui.ctx().clone();
            // The slot whose long press opened its flyout, until the next press.
            if ui.input(|i| i.pointer.any_pressed()) {
                ui.data_mut(|d| d.remove::<egui::Id>(held_id()));
            }
            // The tools as in the mockup while they fit; then the buttons tighten; then they scroll.
            let bh = if tools_height(BUTTON.y) <= r.height() { BUTTON.y } else { BUTTON_TIGHT };
            let height = tools_height(bh);
            let chosen = if height <= r.height() {
                tools(app, ui, r, r.top(), bh, &t)
            } else {
                let view = Rect::from_min_max(r.min, pos2(r.right() - 1.0, r.bottom()));
                let mut list = ui.new_child(egui::UiBuilder::new().max_rect(view).layout(egui::Layout::top_down(egui::Align::Min)));
                list.set_clip_rect(view);
                let mut chosen = None;
                egui::ScrollArea::vertical().id_salt(wid("tools-scroll", ())).auto_shrink([false, false]).show(&mut list, |ui| {
                    let (_, content) = ui.allocate_space(vec2(view.width(), height));
                    chosen = tools(app, ui, r, content.top(), bh, &t);
                });
                chosen
            };
            match chosen {
                Some(Pick::Tool(id)) => app.select_tool(&id),
                Some(Pick::Options(id)) => {
                    app.select_tool(&id);
                    app.run("tool.options", json!({ "tool": id })).ok();
                }
                None => {}
            }
            // The classic toolbar's flyouts, open or torn off.
            crate::toolbar::floating(app, &ctx);
            crate::toolbar::flyout(app, &ctx);
        });
}

/// The groups in the column `r`, from `top`, their buttons `bh` tall. Returns what a press picked.
fn tools(app: &mut VectorcraftApp, ui: &Ui, r: Rect, top: f32, bh: f32, t: &Tokens) -> Option<Pick> {
    let active = app.session.tool_id().to_string();
    // Centred left of the column's edge line.
    let mid = r.left() + (r.width() - 1.0) / 2.0;
    let x = mid - BUTTON.x / 2.0;
    let mut y = top + PAD;
    let mut chosen = None;
    for (i, (captioned, slots)) in GROUPS.iter().enumerate() {
        if i > 0 {
            ui.painter().hline((mid - 14.0)..=(mid + 14.0), y + GAP + SEP_MARGIN + 0.5, Stroke::new(1.0, t.border));
            y += SEPARATOR;
        }
        if *captioned {
            y += CAPTION_TOP;
            // 10 points as in the mockup (「고고학」); a longer word in another language smaller, to fit.
            let caption = tw!("Archaeology");
            let room = r.width() - 4.0;
            let wide = width_of(ui, caption, font(CAPTION_PT));
            let size = if wide > room { (CAPTION_PT * room / wide).max(7.0) } else { CAPTION_PT };
            text(ui, pos2(mid, y + CAPTION_H / 2.0), Align2::CENTER_CENTER, caption, font(size), t.text_dim);
            y += CAPTION_H + GAP;
        }
        for (tools, icon) in slots.iter() {
            let key = tools.first().copied().unwrap_or_default();
            // The tool last used from the slot, kept with the classic toolbar's (apart from them).
            let memory = format!("trowel/{key}");
            let on = tools.contains(&active.as_str());
            if on {
                app.ui.slot_tool.insert(memory.clone(), active.clone());
            }
            let shown = if on {
                active.clone()
            } else {
                app.ui.slot_tool.get(&memory).filter(|s| tools.contains(&s.as_str())).cloned().unwrap_or_else(|| key.to_string())
            };
            let rect = Rect::from_min_size(pos2(x, y), vec2(BUTTON.x, bh));
            if let Some(pick) = slot(app, ui, tools, icon, &shown, rect, on, t) {
                chosen = Some(pick);
            }
            y = rect.bottom() + GAP;
        }
        y -= GAP;
    }
    chosen
}

/// Where the slot whose long press opened its flyout is kept.
fn held_id() -> egui::Id {
    wid("tool-held", ())
}

/// What a press on a slot picked.
enum Pick {
    Tool(String),
    /// A double click: the tool and its options.
    Options(String),
}

/// One slot's button at `rect`, showing tool `shown` (the slot's own `icon` when it is the first).
#[allow(clippy::too_many_arguments)]
fn slot(app: &mut VectorcraftApp, ui: &Ui, tools: &[&'static str], icon: &str, shown: &str, rect: Rect, on: bool, t: &Tokens) -> Option<Pick> {
    let key = tools.first().copied().unwrap_or_default();
    let resp = ui.interact(rect, wid("tool", key), Sense::click());
    let fill = if on {
        t.tool_active
    } else if resp.hovered() {
        t.hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
    let info = vectorcraft_tools::tool_info(shown);
    let glyph = if shown == key { icon } else { info.map_or(icon, |i| crate::icons::tool_icon(i.icon)) };
    let size = if glyph == "tw-section-line" { ICON_SECTION } else { ICON };
    let icon_rect = Rect::from_center_size(rect.center(), vec2(size, size));
    crate::icons::paint(ui, glyph, icon_rect, if on { t.icon_on } else { t.icon });
    if glyph == "tw-section-line" {
        section_marks(ui, icon_rect, t);
    }
    if let Some(sc) = crate::shortcut_editor::tool_shortcut(shown) {
        text(ui, rect.right_bottom() - vec2(2.0, 1.0), Align2::RIGHT_BOTTOM, sc, mono(9.0), if on { t.icon_on } else { t.text_disabled });
    }
    // A right press or a long one opens the slot's other tools; a slot torn off into its own strip
    // comes to the front instead.
    let press = if tools.len() > 1 { crate::toolbar::flyout_press(ui, &resp, None) } else { None };
    if let Some(press) = press {
        if crate::toolbar::is_floating(app, key) {
            crate::toolbar::raise_floating(ui.ctx(), key);
        } else {
            crate::toolbar::open_flyout(app, ui.ctx(), tools, rect);
        }
        if press == crate::toolbar::FlyoutPress::Primary {
            ui.data_mut(|d| d.insert_temp(held_id(), resp.id));
        }
    }
    let held = ui.data(|d| d.get_temp::<egui::Id>(held_id())) == Some(resp.id);
    let tip = match arch_tip(shown) {
        Some((name, what)) => {
            let sc = crate::shortcut_editor::tool_shortcut(shown).map(|k| format!(" ({k})")).unwrap_or_default();
            format!("{name}{sc}\n{what}")
        }
        None => info.map(crate::toolbar::tip).unwrap_or_default(),
    };
    // Its other tools, for those who don't know to look.
    let tip = if tools.len() > 1 { format!("{tip}\n{}", tw!("Press and hold for more tools")) } else { tip };
    let resp = resp.on_hover_text(tip);
    if press.is_some() || held && resp.clicked() {
        // Releasing the long press that opened the flyout leaves it open.
        None
    } else if resp.double_clicked() {
        Some(Pick::Options(shown.to_string()))
    } else if resp.clicked() {
        Some(Pick::Tool(shown.to_string()))
    } else {
        None
    }
}

/// The Section Line icon's two marks in the section marks' red (the mockup's `#i-secline`), over
/// the icon drawn at `r` (24 units across).
fn section_marks(ui: &Ui, r: Rect, t: &Tokens) {
    let unit = r.width() / 24.0;
    for left in [2.5, 17.5] {
        let mark = Rect::from_min_size(r.min + vec2(left, 10.0) * unit, vec2(4.0, 4.0) * unit);
        ui.painter().rect_filled(mark, CornerRadius::ZERO, t.section_mark);
    }
}
