//! The ribbon (design 3.0 §5 ③): nine big tiles in four groups for what is done to the drawing as a
//! whole, Find a Command at the right. The drawing tools have their place in the tool column, the
//! drawing elements in the drawing set, grid and snap in the status bar (principle 1: one place for
//! each thing). What a later job brings is dimmed.

use serde_json::json;

use super::*;

/// Inside the ribbon (the mockup's `.a-rib`, `.rg`, `.tile`): its padding, a group's caption and
/// padding, a tile and its icon well, Find a Command.
const PAD_X: f32 = 10.0;
const PAD_TOP: f32 = 6.0;
const CAPTION: f32 = 15.0;
const CAPTION_GAP: f32 = 4.0;
const GROUP_PAD: f32 = 10.0;
const TILE_W: f32 = 58.0;
const GAP: f32 = 4.0;
const WELL: egui::Vec2 = vec2(46.0, 40.0);
const FIND: egui::Vec2 = vec2(210.0, 32.0);

/// What a tile does.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Act {
    /// Run a menu command.
    Cmd(&'static str),
    /// The underlay: show or hide the template layers, or place a scan as one.
    Underlay,
    /// The drawing's scale and coordinate system, in a dialog.
    Scale,
    /// A legend table of the rows in use, near the middle of the view.
    LegendTable,
    /// The plate (report page) and title block dialog.
    Plate,
    /// The Strata survey data browser.
    Strata,
    /// Nothing but open its list (Export).
    OpenList,
    /// Comes with a later job.
    Soon(&'static str),
}

/// A tile's list (its ▾): where it opens it and what is on it.
#[derive(Clone, Copy, PartialEq)]
enum List {
    /// New Drawing: its ▾ opens the list, the rest of it the New dialog.
    New,
    /// Export: the whole tile opens the list.
    Export,
}

struct Tile {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
    act: Act,
    list: Option<List>,
}

/// How a tile looks: at rest, its view on, Save with changes to save, or off.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Rest,
    Shown,
    Save,
    Off,
}

fn tile(key: &'static str, icon: &'static str, label: &'static str, act: Act) -> Tile {
    Tile { key, icon, label, act, list: None }
}

/// The groups and their tiles, left to right.
fn groups() -> [(&'static str, Vec<Tile>); 4] {
    use Act::*;
    [
        (
            tw!("File"),
            vec![
                Tile { list: Some(List::New), ..tile("new", "tw-new", tw!("New Drawing"), Cmd("file.newDialog")) },
                tile("open", "tw-open", tw!("Open"), Cmd("file.open")),
                tile("save", "tw-save", tw!("Save"), Cmd("file.save")),
            ],
        ),
        (tw!("Underlay"), vec![tile("underlay", "tw-underlay", tw!("Place"), Underlay), tile("align", "tw-align", tw!("Align"), Soon("J008"))]),
        (
            tw!("Drawing"),
            vec![
                tile("scale", "tw-scale-bar", tw!("Scale · CRS"), Scale),
                tile("legend", "tw-legend", tw!("Legend Table"), LegendTable),
                tile("plate", "tw-title-block", tw!("Plate"), Plate),
            ],
        ),
        (tw!("Export"), vec![Tile { list: Some(List::Export), ..tile("export", "tw-export", tw!("Export"), OpenList) }]),
    ]
}

/// What a list offers: each item's icon, label and what it does.
fn list_items(list: List) -> Vec<(&'static str, &'static str, Act)> {
    match list {
        List::New => vec![
            ("tw-plan", tw!("Plan"), Act::Cmd("trowel.newPlan")),
            ("tw-section", tw!("Section"), Act::Cmd("trowel.newSection")),
            ("tw-finds", tw!("Finds"), Act::Cmd("trowel.newFinds")),
            ("tw-sheet", tw!("Plate"), Act::Cmd("trowel.newPlate")),
        ],
        List::Export => vec![
            ("tw-pdf", tw!("PDF…"), Act::Cmd("file.export.pdf")),
            ("tw-dxf", tw!("DXF (keeps coordinates)…"), Act::Cmd("ui.dxfOptionsDialog")),
            ("tw-sheet", tw!("SVG…"), Act::Cmd("file.export.svg")),
            ("tw-strata", tw!("Strata Survey Data…"), Act::Strata),
        ],
    }
}

fn look(app: &VectorcraftApp, tile: &Tile) -> Look {
    match tile.act {
        Act::Soon(_) => Look::Off,
        Act::Underlay => match templates(app) {
            (ids, true) if !ids.is_empty() => Look::Shown,
            (ids, _) if !ids.is_empty() => Look::Rest,
            _ if crate::menus::enabled(app, "file.place") => Look::Rest,
            _ => Look::Off,
        },
        Act::Scale | Act::LegendTable | Act::Plate | Act::Strata | Act::OpenList => {
            if app.session.active().is_some() {
                Look::Rest
            } else {
                Look::Off
            }
        }
        Act::Cmd(id) if !crate::menus::enabled(app, id) => Look::Off,
        Act::Cmd("file.save") if app.session.active().is_some_and(|d| d.is_dirty()) => Look::Save,
        Act::Cmd(_) => Look::Rest,
    }
}

/// How wide the groups are, their tiles `tile_w` apart by `gap` (the ribbon's padding at the left
/// included).
fn tiles_width(groups: &[(&str, Vec<Tile>)], tile_w: f32, gap: f32) -> f32 {
    PAD_X
        + groups
            .iter()
            .map(|(_, tiles)| {
                let n = tiles.len() as f32;
                GROUP_PAD * 2.0 + n * tile_w + (n - 1.0).max(0.0) * gap + 1.0
            })
            .sum::<f32>()
}

/// For a narrower window the tiles keep their labels and move closer (design 3.0 §9).
const NARROW_W: f32 = 50.0;
const NARROW_GAP: f32 = 2.0;
/// Find a Command folded to its magnifier, for a narrower window.
const FIND_FOLDED: f32 = 32.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(RIBBON).exact_size(RIBBON_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let groups = groups();
        // The tiles as in the mockup with Find a Command beside them while they fit; then Find a
        // Command folds to its magnifier, then the tiles move closer, then they scroll.
        let room = r.width();
        let full = tiles_width(&groups, TILE_W, GAP);
        let (tile_w, gap, find_w) = if full + PAD_X + FIND.x + PAD_X <= room {
            (TILE_W, GAP, FIND.x)
        } else if full + PAD_X + FIND_FOLDED + PAD_X <= room {
            (TILE_W, GAP, FIND_FOLDED)
        } else {
            (NARROW_W, NARROW_GAP, FIND_FOLDED)
        };
        let well_top = r.top() + PAD_TOP + CAPTION + CAPTION_GAP;
        let find = Rect::from_center_size(pos2(r.right() - PAD_X - find_w / 2.0, well_top + WELL.y / 2.0), vec2(find_w, FIND.y));
        let view = Rect::from_min_max(r.min, pos2(find.left() - PAD_X, r.bottom() - 1.0));
        let width = tiles_width(&groups, tile_w, gap);
        let mut run = None;
        if width <= view.width() {
            run = tiles(app, ui, &groups, (tile_w, gap), r.left(), r.top(), &t);
        } else {
            let mut strip = ui.new_child(egui::UiBuilder::new().max_rect(view).layout(egui::Layout::left_to_right(egui::Align::Min)));
            strip.set_clip_rect(view);
            // A mouse's wheel turns it too.
            strip.style_mut().always_scroll_the_only_direction = true;
            egui::ScrollArea::horizontal().id_salt(wid("ribbon-scroll", ())).auto_shrink([false, false]).show(&mut strip, |ui| {
                let (_, content) = ui.allocate_space(vec2(width, view.height()));
                run = tiles(app, ui, &groups, (tile_w, gap), content.left(), content.top(), &t);
            });
        }
        find_button(app, ui, find, find_w < FIND.x, &t);
        if let Some(act) = run {
            act_on(app, ui.ctx(), act);
        }
    });
}

/// Do what a tile or a list item does.
fn act_on(app: &mut VectorcraftApp, ctx: &egui::Context, act: Act) {
    match act {
        Act::Cmd(id) => crate::menus::invoke(app, id, json!({})),
        Act::Underlay => underlay(app),
        Act::Scale => super::scale_dialog::open(app, ctx),
        Act::LegendTable => legend_table(app, ctx),
        Act::Plate => super::plate::toggle_dialog(ctx),
        Act::Strata => super::gpkg_browser::open_dialog(ctx),
        Act::OpenList | Act::Soon(_) => {}
    }
}

/// A legend table of the rows in use, replacing the one there is, near the middle of the view as
/// the one-click elements go (the Drawing menu's Insert Legend Table does the same).
pub(super) fn legend_table(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let at = place_symbol_at(app, ctx, [0.0, 0.0]);
    if let Err(e) = app.run("style.generateLegend", json!({ "at": at, "filterUnused": true, "replaceExisting": true })) {
        app.status_error(e);
    }
}

/// The drawing's underlays, template layers holding something (a scan placed as a template: locked
/// and dimmed; the plan legend's Underlay row, empty, is not one yet), and whether any of them
/// shows.
fn templates(app: &VectorcraftApp) -> (Vec<u64>, bool) {
    let Some(st) = app.session.active() else { return (vec![], false) };
    let layers: Vec<_> = st.doc.layers.iter().filter(|l| l.is_template() && l.children().is_some_and(|c| !c.is_empty())).collect();
    (layers.iter().map(|l| l.id.0).collect(), layers.iter().any(|l| l.visible))
}

/// The Underlay tile: with underlays ([`templates`]), hide them all or show them all; else place a
/// scan as one (File › Place with Template ticked).
fn underlay(app: &mut VectorcraftApp) {
    let (ids, shown) = templates(app);
    if ids.is_empty() {
        // Under every line, its layer named in the user's words (`file.place`'s `underlay`).
        crate::menus::invoke(app, "file.place", json!({ "template": true, "underlay": tw!("Underlay") }));
    } else if let Err(e) = app.run("layer.setProps", json!({ "ids": ids, "visible": !shown })) {
        app.status_error(e);
    }
}

/// The groups from `left`, their captions under `top`, the tiles `fit` (width, gap) under them, a
/// line after each group but the last. Returns what a clicked tile or list item does.
fn tiles(app: &VectorcraftApp, ui: &Ui, groups: &[(&str, Vec<Tile>)], fit: (f32, f32), left: f32, top: f32, t: &Tokens) -> Option<Act> {
    let (tile_w, gap) = fit;
    let well_top = top + PAD_TOP + CAPTION + CAPTION_GAP;
    let mut run = None;
    let mut x = left + PAD_X;
    let last = groups.len().saturating_sub(1);
    for (i, (caption, tiles)) in groups.iter().enumerate() {
        text(ui, pos2(x + GROUP_PAD + 2.0, top + PAD_TOP + CAPTION / 2.0), Align2::LEFT_CENTER, *caption, font(11.5), t.text_dim);
        let mut tx = x + GROUP_PAD;
        for tl in tiles {
            let rect = Rect::from_min_max(pos2(tx, well_top), pos2(tx + tile_w, top + RIBBON_H - 4.0));
            if let Some(act) = tile_ui(app, ui, tl, rect, t) {
                run = Some(act);
            }
            tx += tile_w + gap;
        }
        x = tx - gap + GROUP_PAD;
        if i < last {
            ui.painter().vline(x + 0.5, (top + PAD_TOP)..=(top + RIBBON_H - 4.0), Stroke::new(1.0, t.border));
        }
        x += 1.0;
    }
    run
}

/// Find a Command at `find` (`folded`: its magnifier alone): it opens the command palette. Its
/// shortcut shows at its right.
fn find_button(app: &mut VectorcraftApp, ui: &Ui, find: Rect, folded: bool, t: &Tokens) {
    let resp = ui.interact(find, wid("find", ()), Sense::click());
    let fill = if resp.hovered() { t.hover } else { t.input };
    field_box(ui, find, fill, t.input_border);
    let shortcut = crate::menus::shortcut_of("help.commandPalette").map(crate::menus::pretty_shortcut);
    if folded {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.center(), vec2(15.0, 15.0)), t.text_dim);
    } else {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.left_center() + vec2(18.0, 0.0), vec2(15.0, 15.0)), t.text_dim);
        let label = text(ui, find.left_center() + vec2(33.0, 0.0), Align2::LEFT_CENTER, tw!("Find a command"), font(12.5), t.text_dim);
        if let Some(sc) = &shortcut {
            let at = find.right_center() - vec2(10.0, 0.0);
            if at.x - width_of(ui, sc, mono(11.0)) > label.right() + 6.0 {
                text(ui, at, Align2::RIGHT_CENTER, sc, mono(11.0), t.text_dim);
            }
        }
    }
    let label = if folded { tw!("Find a command") } else { tl!("Search commands and tools") };
    let tip = match shortcut {
        Some(sc) => format!("{label} ({sc})"),
        None => label.to_string(),
    };
    if resp.on_hover_text(tip).clicked() {
        app.ui.palette_open = true;
        app.ui.palette_query.clear();
    }
}

/// One tile in `rect`: its well, icon and label (with ▾ when it has a list). Returns what it does
/// when clicked, or what was picked from its list.
fn tile_ui(app: &VectorcraftApp, ui: &Ui, tile: &Tile, rect: Rect, t: &Tokens) -> Option<Act> {
    let look = look(app, tile);
    let resp = ui.interact(rect, wid("tile", tile.key), Sense::click());
    let well = Rect::from_min_size(pos2(rect.center().x - WELL.x / 2.0, rect.top()), WELL);
    let fill = match look {
        Look::Shown => Some(t.row_selected),
        Look::Save => Some(t.action),
        Look::Rest | Look::Off if resp.hovered() && look != Look::Off => Some(t.hover),
        Look::Rest | Look::Off => None,
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(well, CornerRadius::same(9), fill);
    }
    let tint = match look {
        Look::Save => t.on_action,
        Look::Off => t.icon.gamma_multiply(0.45),
        Look::Rest | Look::Shown => t.icon,
    };
    crate::icons::paint(ui, tile.icon, Rect::from_center_size(well.center(), vec2(24.0, 24.0)), tint);
    let color = if look == Look::Off { t.text_dim } else { t.text };
    let cy = well.bottom() + 3.0 + 8.0;
    let clip = ui.painter().with_clip_rect(rect.expand2(vec2(3.0, 0.0)));
    let label = if tile.list.is_some() {
        // The label and its ▾ (9 points, dimmed), centred together.
        let w = width_of(ui, tile.label, font(12.0)) + 1.0 + 7.0;
        let x = rect.center().x - w / 2.0;
        let l = clip.text(pos2(x, cy), Align2::LEFT_CENTER, tile.label, font(12.0), color);
        triangle(ui, pos2(l.right() + 1.0 + 3.5, cy + 1.0), 6.0, true, t.text_dim);
        // The ▾ and a little room around it.
        Rect::from_min_max(pos2(l.right() - 1.0, well.bottom()), pos2(l.right() + 11.0, rect.bottom()))
    } else {
        clip.text(pos2(rect.center().x, cy), Align2::CENTER_CENTER, tile.label, font(12.0), color);
        Rect::NOTHING
    };
    let tip = match tile.act {
        Act::Soon(job) => soon(tile.label, job),
        Act::Underlay if templates(app).0.is_empty() => tw!("Place a scan under the drawing").to_string(),
        Act::Underlay => tw!("Show or hide the underlay").to_string(),
        Act::Cmd(id) => match crate::menus::shortcut_of(id) {
            Some(sc) => format!("{} ({})", tile.label, crate::menus::pretty_shortcut(sc)),
            None => tile.label.to_string(),
        },
        _ => tile.label.to_string(),
    };
    let resp = resp.on_hover_text(tip);
    // New Drawing's list opens from its ▾; Export's from the whole tile.
    let opener = match tile.list {
        Some(List::New) => {
            Some(ui.interact(label, wid("tile-more", tile.key), Sense::click()).on_hover_text(tw!("Plan, section, finds or plate at once")))
        }
        Some(List::Export) if look != Look::Off => Some(resp.clone()),
        _ => None,
    };
    let mut picked = None;
    if let (Some(list), Some(opener)) = (tile.list, &opener) {
        egui::Popup::menu(opener).id(wid("tile-list", tile.key)).show(|ui| {
            for (icon, label, act) in list_items(list) {
                let enabled = match act {
                    Act::Cmd(id) => crate::menus::enabled(app, id),
                    _ => app.session.active().is_some(),
                };
                let image = egui::Image::new(crate::icons::source(icon)).tint(t.icon).fit_to_exact_size(vec2(16.0, 16.0));
                let item = ui.add_enabled(enabled, egui::Button::image_and_text(image, label));
                if item.clicked() {
                    picked = Some(act);
                    ui.close();
                }
            }
        });
    }
    if picked.is_some() {
        return picked;
    }
    if opener.as_ref().is_some_and(|o| o.clicked()) {
        return None;
    }
    match tile.act {
        // Off: it says what it waits for, and does nothing.
        Act::Soon(job) if resp.clicked() => {
            note_soon(ui.ctx(), tile.label, job);
            None
        }
        Act::Soon(_) => None,
        _ => (look != Look::Off && resp.clicked()).then_some(tile.act),
    }
}
