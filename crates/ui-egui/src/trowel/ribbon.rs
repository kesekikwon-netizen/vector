//! The ribbon: one row of tiles in seven groups (design 2.1 §3), Find a Command at the right.
//! Each tile runs a command or picks a tool; what a later job brings is dimmed.

use serde_json::json;

use super::*;

/// Inside the ribbon: its padding, a group's caption and padding, a tile and its icon well.
const PAD: f32 = 8.0;
const CAPTION: f32 = 14.0;
const GROUP_PAD: f32 = 9.0;
const TILE_W: f32 = 52.0;
const GAP: f32 = 6.0;
const WELL: f32 = 44.0;
const FIND: egui::Vec2 = vec2(150.0, 32.0);

/// What a tile does.
#[derive(Clone, Copy)]
enum Act {
    /// Run a menu command.
    Cmd(&'static str),
    /// Pick a tool.
    Tool(&'static str),
    /// Comes with a later job.
    Soon(&'static str),
}

struct Tile {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
    act: Act,
}

/// How a tile looks: at rest, its tool in hand, its view on, Save with changes to save, or off.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Rest,
    On,
    Shown,
    Save,
    Off,
}

fn tile(key: &'static str, icon: &'static str, label: &'static str, act: Act) -> Tile {
    Tile { key, icon, label, act }
}

/// The groups and their tiles, left to right.
fn groups() -> [(&'static str, Vec<Tile>); 7] {
    use Act::*;
    [
        (
            tw!("File"),
            vec![
                tile("open", "tw-open", tw!("Open"), Cmd("file.open")),
                tile("save", "tw-save", tw!("Save"), Cmd("file.save")),
                tile("import", "tw-import", tw!("Import"), Cmd("file.place")),
            ],
        ),
        (
            tw!("New Drawing"),
            vec![
                tile("plan", "tw-plan", tw!("Plan"), Cmd("file.new")),
                tile("section", "tw-section", tw!("Section"), Soon("J004")),
                tile("finds", "tw-finds", tw!("Finds"), Soon("J004")),
                tile("sheet", "tw-sheet", tw!("Plate"), Soon("J004")),
            ],
        ),
        (
            tw!("Underlay"),
            vec![tile("align", "tw-align", tw!("Align"), Soon("J008")), tile("underlay", "tw-underlay", tw!("Underlay"), Cmd("file.place"))],
        ),
        (
            tw!("Draw"),
            vec![
                tile("pen", "tw-pen", tw!("Pen"), Tool("pen")),
                tile("pencil", "tw-pencil", tw!("Pencil"), Tool("pencil")),
                tile("section-line", "tw-section-line", tw!("Section Line"), Soon("J009")),
                tile("spot-height", "tw-spot-height", tw!("Spot Height"), Soon("J009")),
                tile("bucket", "tw-bucket", tw!("Bucket"), Tool("livePaintBucket")),
            ],
        ),
        (
            tw!("Drawing Elements"),
            vec![
                tile("north", "tw-north", tw!("North Arrow"), Soon("J012")),
                tile("scale-bar", "tw-scale-bar", tw!("Scale Bar"), Soon("J012")),
                tile("legend", "tw-legend", tw!("Legend Table"), Soon("J012")),
                tile("title-block", "tw-title-block", tw!("Title Block"), Soon("J012")),
            ],
        ),
        (
            tw!("View"),
            vec![tile("grid", "tw-grid", tw!("Grid"), Cmd("view.grid")), tile("magnet", "tw-magnet", tw!("Snap"), Cmd("view.snapToPoint"))],
        ),
        (
            tw!("Export"),
            vec![
                tile("pdf", "tw-pdf", tw!("PDF"), Cmd("file.export.pdf")),
                tile("dxf", "tw-dxf", tw!("DXF"), Cmd("ui.dxfOptionsDialog")),
                tile("strata", "tw-strata", tw!("To Strata"), Soon("J019")),
            ],
        ),
    ]
}

fn look(app: &VectorcraftApp, act: Act) -> Look {
    match act {
        Act::Soon(_) => Look::Off,
        Act::Tool(id) if app.session.tool_id() == id => Look::On,
        Act::Tool(_) => Look::Rest,
        Act::Cmd(id) if !crate::menus::enabled(app, id) => Look::Off,
        Act::Cmd("file.save") if app.session.active().is_some_and(|d| d.is_dirty()) => Look::Save,
        Act::Cmd(id) if crate::menus::checked(app, id, &json!({})) == Some(true) => Look::Shown,
        Act::Cmd(_) => Look::Rest,
    }
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(RIBBON).exact_size(RIBBON_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let well_top = r.top() + PAD + CAPTION + 5.0;
        let find = Rect::from_center_size(pos2(r.right() - PAD - FIND.x / 2.0, well_top + WELL / 2.0), FIND);
        let mut run = None;
        let mut x = r.left() + PAD;
        let groups = groups();
        let count = groups.len();
        for (g, (caption, tiles)) in groups.into_iter().enumerate() {
            text(ui, pos2(x + GROUP_PAD, r.top() + PAD + CAPTION / 2.0), Align2::LEFT_CENTER, caption, bold(11.0), t.text_dim);
            let mut tx = x + GROUP_PAD;
            for tl in &tiles {
                let rect = Rect::from_min_max(pos2(tx, well_top), pos2(tx + TILE_W, r.bottom() - 6.0));
                if let Some(act) = tile_ui(app, ui, tl, rect, &t) {
                    run = Some(act);
                }
                tx += TILE_W + GAP;
            }
            x = tx - GAP + GROUP_PAD;
            if g + 1 < count {
                ui.painter().vline(x + 0.5, (r.top() + PAD)..=(r.bottom() - 6.0), Stroke::new(1.0, t.border));
                x += 1.0;
            }
        }
        // Find a Command, while it fits beside the tiles.
        if find.left() >= x + PAD {
            let resp = ui.interact(find, wid("find", ()), Sense::click());
            let fill = if resp.hovered() { t.panel } else { t.input };
            field_box(ui, find, fill, t.input_border);
            crate::icons::paint(ui, "search", Rect::from_center_size(find.left_center() + vec2(16.0, 0.0), vec2(13.0, 13.0)), t.text_disabled);
            text(ui, find.left_center() + vec2(29.0, 0.0), Align2::LEFT_CENTER, tw!("Find a command"), font(12.0), t.text_disabled);
            let tip = match crate::menus::shortcut_of("help.commandPalette") {
                Some(sc) => format!("{} ({})", tl!("Search commands and tools"), crate::menus::pretty_shortcut(sc)),
                None => tl!("Search commands and tools").to_string(),
            };
            if resp.on_hover_text(tip).clicked() {
                app.ui.palette_open = true;
                app.ui.palette_query.clear();
            }
        }
        match run {
            Some(Act::Cmd(id)) => crate::menus::invoke(app, id, json!({})),
            Some(Act::Tool(id)) => app.select_tool(id),
            Some(Act::Soon(_)) | None => {}
        }
    });
}

/// One tile in `rect`: its well, icon and label. Returns what it does when clicked.
fn tile_ui(app: &VectorcraftApp, ui: &Ui, tile: &Tile, rect: Rect, t: &Tokens) -> Option<Act> {
    let look = look(app, tile.act);
    let id = wid("tile", tile.key);
    let resp = if look == Look::Off { ui.interact(rect, id, Sense::hover()) } else { ui.interact(rect, id, Sense::click()) };
    let well = Rect::from_min_size(pos2(rect.center().x - WELL / 2.0, rect.top()), vec2(WELL, WELL));
    let fill = match look {
        Look::On => t.tool_active,
        Look::Shown => t.row_selected,
        Look::Save => t.action,
        Look::Off => t.panel_darker,
        Look::Rest if resp.hovered() => t.hover,
        Look::Rest => t.button,
    };
    let edge = if look == Look::Shown { Stroke::new(1.0, t.ring) } else { Stroke::NONE };
    ui.painter().rect(well, CornerRadius::same(12), fill, edge, StrokeKind::Inside);
    let tint = match look {
        Look::On => t.icon_on,
        Look::Save => t.on_action,
        Look::Off => t.icon.gamma_multiply(0.45),
        Look::Rest | Look::Shown => t.icon,
    };
    crate::icons::paint(ui, tile.icon, Rect::from_center_size(well.center(), vec2(24.0, 24.0)), tint);
    let (label_font, label_color) = match look {
        Look::On | Look::Shown | Look::Save => (bold(11.0), t.text),
        Look::Off => (font(11.0), t.text_dim),
        Look::Rest => (font(11.0), t.text),
    };
    let label = pos2(rect.center().x, well.bottom() + 11.0);
    ui.painter().with_clip_rect(rect.expand2(vec2(3.0, 0.0))).text(label, Align2::CENTER_CENTER, tile.label, label_font, label_color);
    let tip = match tile.act {
        Act::Soon(job) => soon(job),
        Act::Tool(id) => vectorcraft_tools::tool_info(id).map(crate::toolbar::tip).unwrap_or_default(),
        Act::Cmd(id) => match crate::menus::shortcut_of(id) {
            Some(sc) => format!("{} ({})", tile.label, crate::menus::pretty_shortcut(sc)),
            None => tile.label.to_string(),
        },
    };
    let resp = resp.on_hover_text(tip);
    (look != Look::Off && resp.clicked()).then_some(tile.act)
}
